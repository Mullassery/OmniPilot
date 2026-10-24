// Copyright 2026 OmniPilot contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use omnipilot_core::{
    Action, Assertion, DeviceId, Platform, RiskLevel, Selector, Target, TaskResult, TaskSpec,
    TaskStatus, PROTOCOL_VERSION,
};
use omnipilot_engine::{Policy, Runtime, RuntimeConfig};
use omnipilot_ios::{IosAdapter, IosAdapterConfig};
use omnipilot_mock::{MockAdapter, DEMO_APP, DEMO_DEVICE_ID};
use omnipilot_protocol::{protocol_schema, DeviceAdapter};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Parser)]
#[command(
    name = "omnipilot",
    version,
    about = "Verification-first mobile automation for Android and iOS"
)]
struct Cli {
    /// Emit machine-readable JSON where supported.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Check the host, Xcode, Android tooling, and the mock adapter.
    Doctor,
    /// List connected devices. The Apple-first build includes the mock device.
    Devices,
    /// List available iOS simulators through simctl.
    IosDevices,
    /// Boot an iOS simulator and wait until it is ready.
    IosBoot(IosDeviceArgs),
    /// Shut down an iOS simulator.
    IosShutdown(IosDeviceArgs),
    /// Run the built-in demo against the XCUITest iOS adapter.
    IosDemo(IosDemoArgs),
    /// Run an explicit, deterministic task plan.
    Run(RunArgs),
    /// Run the built-in mock scenario through the real engine.
    Demo(DemoArgs),
    /// Inspect the protocol and task schemas.
    Protocol {
        #[command(subcommand)]
        command: ProtocolCommand,
    },
    /// Serve OmniPilot over MCP stdio.
    Mcp(McpArgs),
}

#[derive(Debug, Args, Clone)]
struct ReportArgs {
    /// Write a JUnit XML report for this task.
    #[arg(long = "junit", value_name = "PATH")]
    junit: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct RunArgs {
    #[arg(long)]
    goal: String,

    #[arg(long, default_value = "mock")]
    platform: String,

    #[arg(long)]
    device: Option<String>,

    #[arg(long)]
    app: Option<String>,

    /// A JSON-encoded Action, e.g. '{"type":"launch_app","app_id":"com.omnipilot.demo"}'.
    #[arg(long = "step", value_name = "JSON")]
    steps: Vec<String>,

    /// A JSON-encoded Assertion.
    #[arg(long = "assert", value_name = "JSON")]
    assertions: Vec<String>,

    /// Convenience assertion: text must be present on the final screen.
    #[arg(long = "assert-text")]
    assert_text: Vec<String>,

    #[arg(long, default_value_t = 20)]
    max_steps: u32,

    #[arg(long, default_value_t = 120)]
    timeout_secs: u64,

    /// Approve UI mutations even if the local policy asks for approval.
    #[arg(long)]
    approve_ui: bool,

    #[command(flatten)]
    ios: IosArgs,

    #[command(flatten)]
    report: ReportArgs,
}

#[derive(Debug, Args, Clone)]
struct IosArgs {
    /// Path to the generated Xcode project.
    #[arg(long, default_value = "apple/OmniPilot.xcodeproj")]
    ios_project: PathBuf,

    /// Xcode scheme that runs the adapter test.
    #[arg(long, default_value = "OmniPilotAdapter")]
    ios_scheme: String,

    /// Override the xcodebuild destination, e.g. id=<simulator-udid>.
    #[arg(long)]
    ios_destination: Option<String>,

    /// How long to wait for xcodebuild and the adapter ready line.
    #[arg(long, default_value_t = 180)]
    ios_launch_timeout_secs: u64,
}

#[derive(Debug, Args)]
struct McpArgs {
    /// Also launch the XCUITest adapter and expose an iOS simulator.
    #[arg(long)]
    ios: bool,

    /// Simulator UDID for the iOS adapter; implies --ios.
    #[arg(long)]
    ios_device: Option<String>,

    #[command(flatten)]
    ios_options: IosArgs,
}

impl McpArgs {
    fn ios_config(&self) -> Option<IosAdapterConfig> {
        (self.ios || self.ios_device.is_some())
            .then(|| self.ios_options.config(self.ios_device.as_deref()))
    }
}

impl IosArgs {
    fn config(&self, device: Option<&str>) -> IosAdapterConfig {
        let fallback = IosAdapterConfig::default().destination;
        let destination = self
            .ios_destination
            .clone()
            .or_else(|| device.map(|device| format!("id={device}")))
            .unwrap_or(fallback);
        IosAdapterConfig {
            project: self.ios_project.clone(),
            scheme: self.ios_scheme.clone(),
            destination,
            launch_timeout: Duration::from_secs(self.ios_launch_timeout_secs.max(1)),
            ..IosAdapterConfig::default()
        }
    }
}

#[derive(Debug, Args)]
struct IosDeviceArgs {
    /// Simulator UDID from `ios-devices`.
    #[arg(long)]
    device: String,
}

#[derive(Debug, Args)]
struct IosDemoArgs {
    #[arg(long, default_value = "com.omnipilot.demo")]
    app: String,

    #[arg(long)]
    device: Option<String>,

    #[arg(long, default_value_t = 120)]
    timeout_secs: u64,

    #[command(flatten)]
    ios: IosArgs,

    #[command(flatten)]
    report: ReportArgs,
}

#[derive(Debug, Args)]
struct DemoArgs {
    #[arg(long, default_value_t = 120)]
    timeout_secs: u64,

    #[command(flatten)]
    report: ReportArgs,
}

#[derive(Debug, Subcommand)]
enum ProtocolCommand {
    /// Print the versioned JSON Schema bundle.
    Schema,
}

#[derive(Debug, Serialize)]
struct DoctorReport {
    host: String,
    protocol_version: String,
    checks: Vec<DoctorCheck>,
}

#[derive(Debug, Serialize)]
struct DoctorCheck {
    name: String,
    ok: bool,
    detail: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .try_init()
        .ok();

    let cli = Cli::parse();
    let result = match cli.command {
        Command::Doctor => doctor(cli.json).await,
        Command::Devices => devices(cli.json).await,
        Command::IosDevices => ios_devices(cli.json).await,
        Command::IosBoot(args) => ios_boot(args, cli.json),
        Command::IosShutdown(args) => ios_shutdown(args, cli.json),
        Command::IosDemo(args) => ios_demo(args, cli.json).await,
        Command::Run(args) => run(args, cli.json).await,
        Command::Demo(args) => demo(args, cli.json).await,
        Command::Protocol { command } => match command {
            ProtocolCommand::Schema => protocol_schema_command(cli.json),
        },
        Command::Mcp(args) => omnipilot_mcp::run_stdio_with_ios(args.ios_config()).await,
    };

    if let Err(error) = result {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
    Ok(())
}

fn runtime_config() -> RuntimeConfig {
    let evidence_root = std::env::var("OMNIPILOT_TRACES")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("traces"));
    RuntimeConfig {
        evidence_root,
        policy: Policy::default(),
    }
}

fn mock_runtime() -> Arc<Runtime> {
    let adapter = Arc::new(MockAdapter::new());
    Arc::new(Runtime::new(vec![adapter], runtime_config()))
}

async fn ios_runtime(config: IosAdapterConfig) -> Result<(Arc<IosAdapter>, Arc<Runtime>)> {
    let adapter = Arc::new(
        IosAdapter::connect(config)
            .await
            .context("failed to start the iOS adapter")?,
    );
    let runtime = Arc::new(Runtime::new(vec![adapter.clone()], runtime_config()));
    Ok((adapter, runtime))
}

async fn devices(json: bool) -> Result<()> {
    let runtime = mock_runtime();
    let devices = runtime.devices();
    if json {
        println!("{}", serde_json::to_string_pretty(&devices)?);
    } else {
        for device in devices {
            println!(
                "{}\t{}\t{}\t{}",
                device.id,
                device.platform.as_str(),
                device.name,
                device.status_string()
            );
        }
    }
    Ok(())
}

async fn demo(args: DemoArgs, json: bool) -> Result<()> {
    let runtime = mock_runtime();
    let mut spec = MockAdapter::demo_task();
    spec.budgets.max_duration_ms = args.timeout_secs.saturating_mul(1000);
    let result = runtime.run_task(spec).await;
    print_result(&result, json, args.report.junit.as_deref())?;
    exit_for_result(&result)
}

async fn run(args: RunArgs, json: bool) -> Result<()> {
    let platform = parse_platform(&args.platform)?;
    if platform == Platform::Android {
        bail!(
            "the Android adapter is not implemented in this build; Apple-first work is in progress"
        );
    }

    let mut spec = TaskSpec::new(args.goal, platform);
    if platform == Platform::Mock {
        spec.device_id = Some(DeviceId::new(
            args.device
                .clone()
                .unwrap_or_else(|| DEMO_DEVICE_ID.to_string()),
        ));
    }
    spec.app_id = args.app.clone().or_else(|| Some(DEMO_APP.to_string()));

    for (index, raw) in args.steps.iter().enumerate() {
        let action: Action = serde_json::from_str(raw)
            .with_context(|| format!("--step {} is not a valid Action JSON object", index + 1))?;
        spec.steps.push(action);
    }
    for (index, raw) in args.assertions.iter().enumerate() {
        let assertion: Assertion = serde_json::from_str(raw).with_context(|| {
            format!(
                "--assert {} is not a valid Assertion JSON object",
                index + 1
            )
        })?;
        spec.success_criteria.push(assertion);
    }
    for text in args.assert_text {
        spec.success_criteria.push(Assertion::TextPresent { text });
    }

    spec.budgets.max_steps = args.max_steps;
    spec.budgets.max_duration_ms = args.timeout_secs.saturating_mul(1000);
    if args.approve_ui {
        spec.approved_risks.push(RiskLevel::UiMutation);
    }

    let result = match platform {
        Platform::Mock => mock_runtime().run_task(spec).await,
        Platform::Ios => {
            let (adapter, runtime) = ios_runtime(args.ios.config(args.device.as_deref())).await?;
            spec.device_id = Some(adapter.info().id.clone());
            let result = runtime.run_task(spec).await;
            let _ = adapter.shutdown().await;
            result
        }
        Platform::Android => unreachable!("Android is rejected above"),
    };

    print_result(&result, json, args.report.junit.as_deref())?;
    exit_for_result(&result)
}

async fn ios_demo(args: IosDemoArgs, json: bool) -> Result<()> {
    let (adapter, runtime) = ios_runtime(args.ios.config(args.device.as_deref())).await?;
    let mut spec = ios_demo_task(&args.app);
    spec.device_id = Some(adapter.info().id.clone());
    spec.budgets.max_duration_ms = args.timeout_secs.saturating_mul(1000);
    let result = runtime.run_task(spec).await;
    let _ = adapter.shutdown().await;
    print_result(&result, json, args.report.junit.as_deref())?;
    exit_for_result(&result)
}

fn ios_demo_task(app_id: &str) -> TaskSpec {
    let mut spec = TaskSpec::new(
        "Open the demo Settings screen and turn on dark mode",
        Platform::Ios,
    );
    spec.app_id = Some(app_id.to_string());
    spec.steps = vec![
        Action::LaunchApp {
            app_id: app_id.to_string(),
        },
        Action::Tap {
            target: Target::Element {
                selector: Selector::Identifier {
                    value: "home.settings".to_string(),
                },
            },
            times: None,
            duration_ms: None,
        },
        Action::Tap {
            target: Target::Element {
                selector: Selector::Identifier {
                    value: "settings.dark_mode".to_string(),
                },
            },
            times: None,
            duration_ms: None,
        },
    ];
    spec.success_criteria = vec![Assertion::TextPresent {
        text: "Dark mode is On".to_string(),
    }];
    spec.budgets.max_steps = 10;
    spec
}

#[derive(Debug, Deserialize)]
struct SimctlDeviceList {
    devices: BTreeMap<String, Vec<SimctlDevice>>,
}

#[derive(Debug, Deserialize)]
struct SimctlDevice {
    name: String,
    udid: String,
    state: String,
    #[serde(default = "default_true")]
    is_available: bool,
}

#[derive(Debug, Serialize)]
struct IosDeviceRow {
    id: String,
    name: String,
    runtime: String,
    state: String,
    status: String,
}

fn default_true() -> bool {
    true
}

fn runtime_label(key: &str) -> String {
    let short = key.rsplit('.').next().unwrap_or(key);
    match short.strip_prefix("iOS-") {
        Some(version) => format!("iOS {}", version.replace('-', ".")),
        None => short.to_string(),
    }
}

fn simctl_output(args: &[&str]) -> Result<std::process::Output> {
    let mut command = ProcessCommand::new("xcrun");
    command.arg("simctl").args(args);
    if let Some(developer_dir) = developer_dir() {
        command.env("DEVELOPER_DIR", developer_dir);
    }
    command
        .output()
        .context("failed to run xcrun simctl; install Xcode and select it")
}

fn simctl_run(args: &[&str]) -> Result<()> {
    let output = simctl_output(args)?;
    if !output.status.success() {
        bail!(
            "xcrun simctl {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

fn simctl_list_all() -> Result<SimctlDeviceList> {
    let output = simctl_output(&["list", "devices", "--json"])?;
    if !output.status.success() {
        bail!(
            "xcrun simctl failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    serde_json::from_slice(&output.stdout).context("simctl returned invalid JSON")
}

fn simctl_device_state(udid: &str) -> Result<String> {
    let listed = simctl_list_all()?;
    listed
        .devices
        .values()
        .flatten()
        .find(|device| device.udid == udid)
        .map(|device| device.state.clone())
        .ok_or_else(|| anyhow::anyhow!("simulator {udid} was not found; run omnipilot ios-devices"))
}

fn simulator_status(state: &str) -> &'static str {
    match state {
        "Booted" => "ready",
        "Shutdown" => "shutdown",
        _ => "unavailable",
    }
}

fn print_lifecycle(device_id: &str, state: &str, changed: bool, json: bool) -> Result<()> {
    #[derive(Serialize)]
    struct Lifecycle {
        device_id: String,
        state: String,
        changed: bool,
    }

    let row = Lifecycle {
        device_id: device_id.to_string(),
        state: state.to_string(),
        changed,
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&row)?);
    } else {
        println!(
            "{}\t{}\t{}",
            row.device_id,
            row.state,
            if row.changed { "changed" } else { "unchanged" }
        );
    }
    Ok(())
}

fn ios_boot(args: IosDeviceArgs, json: bool) -> Result<()> {
    let state = simctl_device_state(&args.device)?;
    let changed = state != "Booted";
    if changed {
        simctl_run(&["bootstatus", &args.device, "-b"])?;
    }
    print_lifecycle(&args.device, "Booted", changed, json)
}

fn ios_shutdown(args: IosDeviceArgs, json: bool) -> Result<()> {
    let state = simctl_device_state(&args.device)?;
    let changed = state != "Shutdown";
    if changed {
        simctl_run(&["shutdown", &args.device])?;
    }
    print_lifecycle(&args.device, "Shutdown", changed, json)
}

async fn ios_devices(json: bool) -> Result<()> {
    let output = simctl_output(&["list", "devices", "available", "--json"])?;
    if !output.status.success() {
        bail!(
            "xcrun simctl failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let listed: SimctlDeviceList =
        serde_json::from_slice(&output.stdout).context("simctl returned invalid JSON")?;

    let mut rows = Vec::new();
    for (runtime, devices) in listed.devices {
        if !runtime.contains("iOS-") {
            continue;
        }
        let runtime = runtime_label(&runtime);
        for device in devices {
            if !device.is_available {
                continue;
            }
            rows.push(IosDeviceRow {
                id: device.udid,
                name: device.name,
                runtime: runtime.clone(),
                state: device.state.clone(),
                status: simulator_status(&device.state).to_string(),
            });
        }
    }
    rows.sort_by(|left, right| {
        left.runtime
            .cmp(&right.runtime)
            .then(left.name.cmp(&right.name))
    });

    if json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
    } else {
        for row in rows {
            println!(
                "{}\tios\t{} ({})\t{}",
                row.id, row.name, row.runtime, row.status
            );
        }
    }
    Ok(())
}

fn print_result(result: &TaskResult, json: bool, junit: Option<&Path>) -> Result<()> {
    if let Some(path) = junit {
        write_junit(path, result)?;
    }
    if json {
        println!("{}", serde_json::to_string_pretty(result)?);
        return Ok(());
    }

    println!("task:      {}", result.task_id);
    println!("status:    {}", result.status);
    println!("verified:  {}", result.verified);
    println!("summary:   {}", result.summary);
    for step in &result.steps {
        println!(
            "  step {:02}  {:<16} {:<10} {}",
            step.index + 1,
            step.action.name(),
            format!("{:?}", step.status).to_lowercase(),
            step.message
        );
    }
    for assertion in &result.assertions {
        println!(
            "  assert    {:<10} {}",
            if assertion.passed { "PASS" } else { "FAIL" },
            assertion.message
        );
    }
    for limitation in &result.limitations {
        println!("  note      {limitation}");
    }
    println!("evidence:  {} item(s)", result.evidence.len());
    println!("duration:  {} ms", result.metrics.duration_ms);
    Ok(())
}

fn write_junit(path: &Path, result: &TaskResult) -> Result<()> {
    let xml = junit_xml(result);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    std::fs::write(path, xml)
        .with_context(|| format!("failed to write JUnit report to {}", path.display()))
}

fn junit_xml(result: &TaskResult) -> String {
    let duration = result.metrics.duration_ms as f64 / 1000.0;
    let failures = u32::from(!result.verified);
    let status = result.status.to_string();
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(&format!(
        "<testsuites name=\"OmniPilot\" tests=\"1\" failures=\"{failures}\" errors=\"0\" time=\"{duration:.3}\">\n"
    ));
    out.push_str(&format!(
        "  <testsuite name=\"omnipilot\" tests=\"1\" failures=\"{failures}\" errors=\"0\" time=\"{duration:.3}\" timestamp=\"{}\">\n",
        escape_xml(&result.started_at.to_rfc3339())
    ));
    out.push_str(&format!(
        "    <testcase name=\"{}\" classname=\"omnipilot.task\" time=\"{duration:.3}\">\n",
        escape_xml(&result.goal)
    ));
    out.push_str("      <properties>\n");
    out.push_str(&format!(
        "        <property name=\"task_id\" value=\"{}\"/>\n",
        escape_xml(&result.task_id.to_string())
    ));
    out.push_str(&format!(
        "        <property name=\"status\" value=\"{}\"/>\n",
        escape_xml(&status)
    ));
    out.push_str(&format!(
        "        <property name=\"verified\" value=\"{}\"/>\n",
        result.verified
    ));
    out.push_str(&format!(
        "        <property name=\"evidence_count\" value=\"{}\"/>\n",
        result.evidence.len()
    ));
    out.push_str("      </properties>\n");

    if !result.verified {
        let detail = junit_failure_detail(result);
        out.push_str(&format!(
            "      <failure message=\"{}\" type=\"{}\">{}</failure>\n",
            escape_xml(&result.summary),
            escape_xml(&status),
            escape_xml(&detail)
        ));
    }

    out.push_str("      <system-out>\n");
    out.push_str(&format!("status: {}\n", escape_xml(&status)));
    out.push_str(&format!("summary: {}\n", escape_xml(&result.summary)));
    for step in &result.steps {
        out.push_str(&format!(
            "step {} {} {}: {}\n",
            step.index + 1,
            step.action.name(),
            format!("{:?}", step.status).to_lowercase(),
            escape_xml(&step.message)
        ));
    }
    for assertion in &result.assertions {
        out.push_str(&format!(
            "assert {}: {}\n",
            if assertion.passed { "PASS" } else { "FAIL" },
            escape_xml(&assertion.message)
        ));
    }
    for limitation in &result.limitations {
        out.push_str(&format!("note: {}\n", escape_xml(limitation)));
    }
    out.push_str("      </system-out>\n");
    out.push_str("    </testcase>\n");
    out.push_str("  </testsuite>\n");
    out.push_str("</testsuites>\n");
    out
}

fn junit_failure_detail(result: &TaskResult) -> String {
    let mut lines: Vec<String> = result
        .assertions
        .iter()
        .filter(|assertion| !assertion.passed)
        .map(|assertion| assertion.message.clone())
        .collect();
    lines.extend(result.limitations.iter().cloned());
    if let Some(error) = &result.error {
        lines.push(error.message.clone());
    }
    if lines.is_empty() {
        result.summary.clone()
    } else {
        lines.join("\n")
    }
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn exit_for_result(result: &TaskResult) -> Result<()> {
    if result.status == TaskStatus::Completed && result.verified {
        Ok(())
    } else {
        bail!(
            "task finished with status {} (verified={})",
            result.status,
            result.verified
        )
    }
}

fn parse_platform(value: &str) -> Result<Platform> {
    match value {
        "mock" => Ok(Platform::Mock),
        "android" => Ok(Platform::Android),
        "ios" => Ok(Platform::Ios),
        other => bail!("unknown platform {other}; expected mock, android, or ios"),
    }
}

fn protocol_schema_command(json: bool) -> Result<()> {
    let schema = protocol_schema();
    if json {
        println!("{}", serde_json::to_string_pretty(&schema)?);
    } else {
        println!("protocol: {}", schema["protocol_version"]);
        println!("jsonrpc:  {}", schema["jsonrpc"]);
        println!("methods:  {}", schema["methods"]);
    }
    Ok(())
}

async fn doctor(json: bool) -> Result<()> {
    let checks = vec![
        check_mock_adapter(),
        probe("xcodebuild", "xcodebuild", &["-version"]),
        probe("xcrun simctl", "xcrun", &["simctl", "help"]),
        probe("adb", "adb", &["version"]),
        probe("java", "java", &["-version"]),
        check_android_sdk(),
        check_ios_project(),
        check_xcode_select(),
    ];
    let report = DoctorReport {
        host: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
        protocol_version: PROTOCOL_VERSION.to_string(),
        checks,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("host:     {}", report.host);
        println!("protocol: {}", report.protocol_version);
        for check in &report.checks {
            println!(
                "  [{}] {:<16} {}",
                if check.ok { "ok" } else { "--" },
                check.name,
                check.detail
            );
        }
    }
    Ok(())
}

fn check_mock_adapter() -> DoctorCheck {
    let adapter = MockAdapter::new();
    let info = adapter.info();
    DoctorCheck {
        name: "mock adapter".to_string(),
        ok: true,
        detail: format!("{} ({}, {})", info.name, info.id, info.platform.as_str()),
    }
}

fn probe(name: &str, program: &str, args: &[&str]) -> DoctorCheck {
    let mut command = ProcessCommand::new(program);
    command.args(args);
    if let Some(developer_dir) = developer_dir() {
        if program == "xcodebuild" || program == "xcrun" {
            command.env("DEVELOPER_DIR", &developer_dir);
        }
    }
    match command.output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let detail = stdout
                .lines()
                .chain(stderr.lines())
                .next()
                .unwrap_or("available")
                .trim()
                .to_string();
            DoctorCheck {
                name: name.to_string(),
                ok: true,
                detail,
            }
        }
        Ok(output) => DoctorCheck {
            name: name.to_string(),
            ok: false,
            detail: format!("exit {}", output.status),
        },
        Err(error) => DoctorCheck {
            name: name.to_string(),
            ok: false,
            detail: error.to_string(),
        },
    }
}

fn check_ios_project() -> DoctorCheck {
    let project = PathBuf::from("apple/OmniPilot.xcodeproj");
    let scheme = project.join("xcshareddata/xcschemes/OmniPilotAdapter.xcscheme");
    if project.is_dir() && scheme.is_file() {
        DoctorCheck {
            name: "ios adapter".to_string(),
            ok: true,
            detail: format!("{} (OmniPilotAdapter)", project.display()),
        }
    } else {
        DoctorCheck {
            name: "ios adapter".to_string(),
            ok: false,
            detail: format!("missing {} or its adapter scheme", project.display()),
        }
    }
}

fn check_android_sdk() -> DoctorCheck {
    match std::env::var("ANDROID_HOME").or_else(|_| std::env::var("ANDROID_SDK_ROOT")) {
        Ok(path) if Path::new(&path).is_dir() => DoctorCheck {
            name: "android sdk".to_string(),
            ok: true,
            detail: path,
        },
        Ok(path) => DoctorCheck {
            name: "android sdk".to_string(),
            ok: false,
            detail: format!("{path} does not exist"),
        },
        Err(_) => DoctorCheck {
            name: "android sdk".to_string(),
            ok: false,
            detail: "ANDROID_HOME is not set".to_string(),
        },
    }
}

fn check_xcode_select() -> DoctorCheck {
    let selected = ProcessCommand::new("xcode-select").arg("-p").output();
    match selected {
        Ok(output) if output.status.success() => {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if path.contains("Xcode.app") {
                return DoctorCheck {
                    name: "xcode-select".to_string(),
                    ok: true,
                    detail: path,
                };
            }
            match developer_dir() {
                Some(developer_dir) => DoctorCheck {
                    name: "xcode-select".to_string(),
                    ok: true,
                    detail: format!(
                        "{path}; using {} via DEVELOPER_DIR",
                        developer_dir.display()
                    ),
                },
                None => DoctorCheck {
                    name: "xcode-select".to_string(),
                    ok: false,
                    detail: format!("{path} (set Xcode.app or use DEVELOPER_DIR)"),
                },
            }
        }
        Ok(output) => DoctorCheck {
            name: "xcode-select".to_string(),
            ok: false,
            detail: format!("exit {}", output.status),
        },
        Err(error) => DoctorCheck {
            name: "xcode-select".to_string(),
            ok: false,
            detail: error.to_string(),
        },
    }
}

fn developer_dir() -> Option<PathBuf> {
    if let Ok(value) = std::env::var("DEVELOPER_DIR") {
        let path = PathBuf::from(value);
        if path.is_dir() {
            return Some(path);
        }
    }
    let default = PathBuf::from("/Applications/Xcode.app/Contents/Developer");
    default.is_dir().then_some(default)
}

trait DeviceDisplay {
    fn status_string(&self) -> &'static str;
}

impl DeviceDisplay for omnipilot_core::DeviceInfo {
    fn status_string(&self) -> &'static str {
        match self.status {
            omnipilot_core::DeviceStatus::Ready => "ready",
            omnipilot_core::DeviceStatus::Busy => "busy",
            omnipilot_core::DeviceStatus::Unavailable => "unavailable",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulator_states_map_to_statuses() {
        assert_eq!(simulator_status("Booted"), "ready");
        assert_eq!(simulator_status("Shutdown"), "shutdown");
        assert_eq!(simulator_status("Booting"), "unavailable");
    }

    #[test]
    fn escapes_xml_special_characters() {
        assert_eq!(
            escape_xml("<a & \"b\">'"),
            "&lt;a &amp; &quot;b&quot;&gt;&apos;"
        );
    }

    #[tokio::test]
    async fn junit_report_marks_verified_mock_task_as_passing() {
        let runtime = mock_runtime();
        let result = runtime.run_task(MockAdapter::demo_task()).await;
        let xml = junit_xml(&result);
        assert!(xml.contains("failures=\"0\""));
        assert!(xml.contains("<testcase"));
        assert!(!xml.contains("<failure"));
    }

    #[tokio::test]
    async fn junit_report_includes_failed_assertion() {
        let runtime = mock_runtime();
        let mut spec = MockAdapter::demo_task();
        spec.success_criteria = vec![Assertion::TextPresent {
            text: "not on screen".to_string(),
        }];
        let result = runtime.run_task(spec).await;
        let xml = junit_xml(&result);
        assert!(xml.contains("failures=\"1\""));
        assert!(xml.contains("<failure"));
        assert!(xml.contains("not present"));
    }
}
