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

import SwiftUI

enum DemoScreen {
    case home
    case settings
    case about
}

final class DemoModel: ObservableObject {
    @Published var screen: DemoScreen = .home
    @Published var darkMode = false
    @Published var searchText = ""
}

struct DemoRootView: View {
    @StateObject private var model = DemoModel()

    var body: some View {
        Group {
            switch model.screen {
            case .home:
                HomeView(model: model)
            case .settings:
                SettingsView(model: model)
            case .about:
                AboutView(model: model)
            }
        }
        .preferredColorScheme(model.darkMode ? .dark : .light)
    }
}

private struct HomeView: View {
    @ObservedObject var model: DemoModel

    var body: some View {
        VStack(spacing: 20) {
            Text("OmniPilot Demo")
                .font(.largeTitle.bold())
                .accessibilityIdentifier("home.title")

            Text("A tiny app used to exercise the iOS automation adapter.")
                .multilineTextAlignment(.center)
                .foregroundStyle(.secondary)
                .padding(.horizontal)

            Button("Settings") {
                model.screen = .settings
            }
            .buttonStyle(.borderedProminent)
            .accessibilityIdentifier("home.settings")

            Button("About") {
                model.screen = .about
            }
            .buttonStyle(.bordered)
            .accessibilityIdentifier("home.about")
        }
        .padding()
    }
}

private struct SettingsView: View {
    @ObservedObject var model: DemoModel

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            HStack {
                Button("Back") {
                    model.screen = .home
                }
                .accessibilityIdentifier("settings.back")
                Spacer()
            }

            Text("Settings")
                .font(.largeTitle.bold())
                .accessibilityIdentifier("settings.title")

            Toggle("Dark mode", isOn: $model.darkMode)
                .accessibilityIdentifier("settings.dark_mode")

            Text(model.darkMode ? "Dark mode is On" : "Dark mode is Off")
                .foregroundStyle(.secondary)
                .accessibilityIdentifier("settings.dark_mode.state")

            TextField("Search settings", text: $model.searchText)
                .textFieldStyle(.roundedBorder)
                .accessibilityIdentifier("settings.search")

            Spacer()
        }
        .padding()
    }
}

private struct AboutView: View {
    @ObservedObject var model: DemoModel

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            HStack {
                Button("Back") {
                    model.screen = .home
                }
                .accessibilityIdentifier("about.back")
                Spacer()
            }

            Text("About OmniPilot")
                .font(.largeTitle.bold())
                .accessibilityIdentifier("about.title")

            Text("A verification-first mobile automation platform.")
                .accessibilityIdentifier("about.body")

            Spacer()
        }
        .padding()
    }
}
