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

import Foundation
import UIKit
import XCTest

final class AdapterDevice {
    private var app: XCUIApplication?
    private var appId: String?
    private var logs: [[String: Any]] = []
    private var cancelled = false

    private let maxElements = 300
    private let maxLogs = 500

    func info() -> [String: Any] {
        let environment = ProcessInfo.processInfo.environment
        let device = UIDevice.current
        let identifier = environment["SIMULATOR_UDID"]
            ?? device.identifierForVendor?.uuidString
            ?? "ios-simulator"
        let name = environment["SIMULATOR_DEVICE_NAME"] ?? device.name
        let model = environment["SIMULATOR_MODEL_IDENTIFIER"] ?? device.model
        let osVersion = environment["SIMULATOR_RUNTIME_VERSION"] ?? device.systemVersion
        let size = screenSize()

        return [
            "id": identifier,
            "platform": "ios",
            "name": name,
            "model": model,
            "os_version": osVersion,
            "screen": ["width": size.width, "height": size.height],
            "status": "ready"
        ]
    }

    func capabilities() -> [[String: Any]] {
        [
            ["name": "launch_app", "support": "supported"],
            ["name": "terminate_app", "support": "supported"],
            ["name": "tap", "support": "supported"],
            ["name": "long_press", "support": "supported"],
            ["name": "swipe", "support": "supported"],
            ["name": "input_text", "support": "supported"],
            ["name": "press_key", "support": "supported"],
            ["name": "wait", "support": "supported"],
            ["name": "screenshot", "support": "supported"],
            ["name": "observe", "support": "supported"],
            ["name": "logs", "support": "supported"]
        ]
    }

    func observe(_ params: [String: Any]) -> [String: Any] {
        let includeScreenshot = params["include_screenshot"] as? Bool ?? true
        let includeHierarchy = params["include_hierarchy"] as? Bool ?? true
        let size = screenSize()
        let capturedAt = AdapterJSON.timestamp()

        var result: [String: Any] = [
            "id": UUID().uuidString,
            "captured_at": capturedAt,
            "platform": "ios",
            "screen": ["width": size.width, "height": size.height],
            "elements": [],
            "hierarchy_truncated": false
        ]

        if let app {
            result["app"] = [
                "app_id": appId ?? "unknown",
                "foreground": app.state == .runningForeground
            ]

            if includeHierarchy {
                let all = app.descendants(matching: .any).allElementsBoundByIndex
                let truncated = all.count > maxElements
                var used = Set<String>()
                var elements: [[String: Any]] = []

                for (index, element) in all.prefix(maxElements).enumerated() {
                    let role = role(for: element.elementType)
                    let identifier = element.identifier.isEmpty ? nil : element.identifier
                    let label = element.label.isEmpty ? nil : element.label
                    let value = stringValue(element.value)
                    let text = label ?? value
                    var id = identifier ?? "\(role)-\(index)"
                    if used.contains(id) {
                        id = "\(id)-\(index)"
                    }
                    used.insert(id)

                    var entry: [String: Any] = [
                        "id": id,
                        "role": role,
                        "bounds": bounds(element.frame),
                        "enabled": element.isEnabled,
                        "focused": false,
                        "clickable": element.isHittable
                    ]
                    if let label {
                        entry["label"] = label
                    }
                    if let value {
                        entry["value"] = value
                    }
                    if let identifier {
                        entry["identifier"] = identifier
                    }
                    if let text {
                        entry["text"] = text
                    }
                    elements.append(entry)
                }

                result["elements"] = elements
                result["hierarchy_truncated"] = truncated
            }
        }

        if includeScreenshot {
            result["screenshot"] = screenshot()
        }

        return result
    }

    func act(_ params: [String: Any]) throws -> [String: Any] {
        guard let action = params["action"] as? [String: Any],
              let type = action["type"] as? String
        else {
            throw AdapterFailure.invalidParams("action must be an object with a type")
        }
        guard !cancelled else {
            throw AdapterFailure(
                AdapterProtocol.application,
                "adapter session was cancelled",
                retryable: false,
                kind: "cancelled"
            )
        }

        let actionId = params["action_id"] as? String ?? UUID().uuidString
        let message: String
        var observation: [String: Any]?

        switch type {
        case "launch_app":
            guard let appId = action["app_id"] as? String, !appId.isEmpty else {
                throw AdapterFailure.invalidParams("launch_app requires app_id")
            }
            let app = XCUIApplication(bundleIdentifier: appId)
            app.launch()
            guard app.wait(for: .runningForeground, timeout: 15) else {
                throw AdapterFailure.appNotFound("app \(appId) did not reach the foreground")
            }
            self.app = app
            self.appId = appId
            message = "launched \(appId)"

        case "terminate_app":
            guard let requested = action["app_id"] as? String, !requested.isEmpty else {
                throw AdapterFailure.invalidParams("terminate_app requires app_id")
            }
            if let app, appId == requested {
                app.terminate()
                self.app = nil
                self.appId = nil
                message = "terminated \(requested)"
            } else {
                message = "\(requested) was not running"
            }

        case "tap":
            guard let target = action["target"] as? [String: Any] else {
                throw AdapterFailure.invalidParams("tap requires a target")
            }
            let times = intValue(action["times"]) ?? 1
            let duration = intValue(action["duration_ms"]) ?? 0
            if target["kind"] as? String == "point" {
                try tapPoint(target)
            } else {
                let element = try resolveTarget(target)
                guard element.waitForExistence(timeout: 10) else {
                    throw AdapterFailure.elementNotFound("tap target was not found")
                }
                if duration > 0 {
                    element.press(forDuration: Double(duration) / 1000.0)
                } else if element.elementType == .switch || element.elementType == .toggle {
                    let coordinate = element.coordinate(
                        withNormalizedOffset: CGVector(dx: 0.9, dy: 0.5)
                    )
                    for _ in 0..<max(1, times) {
                        coordinate.tap()
                    }
                } else {
                    for _ in 0..<max(1, times) {
                        element.tap()
                    }
                }
            }
            message = "tapped \(describe(target))"

        case "long_press":
            guard let target = action["target"] as? [String: Any] else {
                throw AdapterFailure.invalidParams("long_press requires a target")
            }
            let element = try resolveTarget(target)
            guard element.waitForExistence(timeout: 10) else {
                throw AdapterFailure.elementNotFound("long-press target was not found")
            }
            let duration = intValue(action["duration_ms"]) ?? 1000
            element.press(forDuration: Double(duration) / 1000.0)
            message = "long-pressed \(describe(target))"

        case "swipe":
            guard let app else {
                throw AdapterFailure.appNotFound("no app is running")
            }
            guard let direction = action["direction"] as? String else {
                throw AdapterFailure.invalidParams("swipe requires a direction")
            }
            let distance = min(max(doubleValue(action["distance"]) ?? 0.5, 0.05), 0.9)
            let duration = Double(intValue(action["duration_ms"]) ?? 50) / 1000.0
            try swipe(app, direction: direction, distance: distance, duration: duration)
            message = "swiped \(direction)"

        case "input_text":
            guard let text = action["text"] as? String else {
                throw AdapterFailure.invalidParams("input_text requires text")
            }
            let clear = action["clear_existing"] as? Bool ?? false
            if let target = action["target"] as? [String: Any] {
                let element = try resolveTarget(target)
                guard element.waitForExistence(timeout: 10) else {
                    throw AdapterFailure.elementNotFound("text target was not found")
                }
                element.tap()
                if clear, let existing = stringValue(element.value), !existing.isEmpty {
                    element.typeText(String(repeating: XCUIKeyboardKey.delete.rawValue, count: existing.count))
                }
                element.typeText(text)
            } else {
                guard let app else {
                    throw AdapterFailure.appNotFound("no app is running")
                }
                app.typeText(text)
            }
            message = "entered \(text.count) character(s)"

        case "press_key":
            try pressKey(action["key"])
            message = "pressed key"

        case "wait":
            let duration = intValue(action["duration_ms"]) ?? 0
            Thread.sleep(forTimeInterval: Double(duration) / 1000.0)
            message = "waited \(duration)ms"

        case "screenshot":
            observation = observe([
                "include_screenshot": true,
                "include_hierarchy": false
            ])
            message = "captured screenshot"

        default:
            throw AdapterFailure.unsupported("unsupported action: \(type)")
        }

        addLog("info", "action", message)
        var result: [String: Any] = [
            "action_id": actionId,
            "status": "executed",
            "message": message
        ]
        if let observation {
            result["observation"] = observation
        }
        return result
    }

    func logEntries(_ params: [String: Any]) -> [String: Any] {
        let requested = intValue(params["limit"]) ?? 100
        let limit = min(max(0, requested), maxLogs)
        return ["entries": Array(logs.suffix(limit))]
    }

    func cancel(_ params: [String: Any]) {
        cancelled = true
        let reason = params["reason"] as? String ?? "client cancelled"
        addLog("warn", "session", "cancel requested: \(reason)")
    }

    private func screenSize() -> (width: UInt32, height: UInt32) {
        let size: CGSize
        if let app, app.state == .runningForeground {
            size = app.frame.size
        } else {
            size = XCUIScreen.main.screenshot().image.size
        }
        let width = UInt32(clamping: Int(max(1, size.width.rounded())))
        let height = UInt32(clamping: Int(max(1, size.height.rounded())))
        return (width, height)
    }

    private func screenshot() -> [String: Any] {
        let screenshot = XCUIScreen.main.screenshot()
        let image = screenshot.image
        let size = image.size
        return [
            "mime": "image/png",
            "base64": screenshot.pngRepresentation.base64EncodedString(),
            "width": UInt32(clamping: Int(max(1, size.width.rounded()))),
            "height": UInt32(clamping: Int(max(1, size.height.rounded()))),
            "redacted": false
        ]
    }

    private func resolveTarget(_ target: [String: Any]) throws -> XCUIElement {
        guard let app else {
            throw AdapterFailure.appNotFound("no app is running")
        }
        guard let selector = target["selector"] as? [String: Any] else {
            throw AdapterFailure.invalidParams("element target requires a selector")
        }
        return try resolve(selector, in: app)
    }

    private func resolve(_ selector: [String: Any], in app: XCUIApplication) throws -> XCUIElement {
        guard let kind = selector["kind"] as? String else {
            throw AdapterFailure.invalidParams("selector requires a kind")
        }
        let query = app.descendants(matching: .any)

        switch kind {
        case "identifier":
            guard let value = selector["value"] as? String else {
                throw AdapterFailure.invalidParams("identifier selector requires a value")
            }
            return query.matching(identifier: value).firstMatch

        case "text":
            guard let value = selector["value"] as? String else {
                throw AdapterFailure.invalidParams("text selector requires a value")
            }
            return query.matching(
                NSPredicate(format: "label == %@ OR value == %@", value, value)
            ).firstMatch

        case "text_contains":
            guard let value = selector["value"] as? String else {
                throw AdapterFailure.invalidParams("text_contains selector requires a value")
            }
            return query.matching(
                NSPredicate(format: "label CONTAINS[c] %@ OR value CONTAINS[c] %@", value, value)
            ).firstMatch

        case "label":
            guard let value = selector["value"] as? String else {
                throw AdapterFailure.invalidParams("label selector requires a value")
            }
            return query.matching(NSPredicate(format: "label == %@", value)).firstMatch

        case "index":
            guard let value = intValue(selector["value"]) else {
                throw AdapterFailure.invalidParams("index selector requires a value")
            }
            return query.element(boundBy: value)

        case "role_and_text":
            guard let role = selector["role"] as? String,
                  let text = selector["text"] as? String
            else {
                throw AdapterFailure.invalidParams("role_and_text requires role and text")
            }
            return app.descendants(matching: elementType(for: role)).matching(
                NSPredicate(format: "label == %@ OR value == %@", text, text)
            ).firstMatch

        default:
            throw AdapterFailure.invalidParams("unknown selector kind \(kind)")
        }
    }

    private func swipe(
        _ app: XCUIApplication,
        direction: String,
        distance: CGFloat,
        duration: TimeInterval
    ) throws {
        let half = distance / 2
        let center = 0.5
        let start: CGVector
        let end: CGVector
        switch direction {
        case "up":
            start = CGVector(dx: center, dy: center + half)
            end = CGVector(dx: center, dy: center - half)
        case "down":
            start = CGVector(dx: center, dy: center - half)
            end = CGVector(dx: center, dy: center + half)
        case "left":
            start = CGVector(dx: center + half, dy: center)
            end = CGVector(dx: center - half, dy: center)
        case "right":
            start = CGVector(dx: center - half, dy: center)
            end = CGVector(dx: center + half, dy: center)
        default:
            throw AdapterFailure.invalidParams("unknown swipe direction \(direction)")
        }
        let origin = app.coordinate(withNormalizedOffset: start)
        let destination = app.coordinate(withNormalizedOffset: end)
        origin.press(forDuration: max(0.05, duration), thenDragTo: destination)
    }

    private func tapPoint(_ target: [String: Any]) throws {
        guard let app else {
            throw AdapterFailure.appNotFound("no app is running")
        }
        guard let point = target["point"] as? [String: Any],
              let x = doubleValue(point["x"]),
              let y = doubleValue(point["y"])
        else {
            throw AdapterFailure.invalidParams("point target requires x and y")
        }
        let coordinate = app.coordinate(
            withNormalizedOffset: CGVector(dx: x / 1000.0, dy: y / 1000.0)
        )
        coordinate.tap()
    }

    private func pressKey(_ value: Any?) throws {
        if let key = value as? String {
            switch key {
            case "home":
                XCUIDevice.shared.press(.home)
            case "enter":
                guard let app else {
                    throw AdapterFailure.appNotFound("no app is running")
                }
                app.typeText("\n")
            case "delete":
                guard let app else {
                    throw AdapterFailure.appNotFound("no app is running")
                }
                app.typeText(XCUIKeyboardKey.delete.rawValue)
            default:
                throw AdapterFailure.unsupported("unsupported key: \(key)")
            }
            return
        }
        if let custom = value as? [String: Any], let text = custom["custom"] as? String {
            guard let app else {
                throw AdapterFailure.appNotFound("no app is running")
            }
            app.typeText(text)
            return
        }
        throw AdapterFailure.invalidParams("press_key requires a key")
    }

    private func describe(_ target: [String: Any]) -> String {
        if let selector = target["selector"] as? [String: Any] {
            let kind = (selector["kind"] as? String) ?? "element"
            if let value = selector["value"] as? String {
                return "\(kind)=\(value)"
            }
            if let role = selector["role"] as? String,
               let text = selector["text"] as? String {
                return "\(kind)=\(role):\(text)"
            }
            return kind
        }
        return "point"
    }

    private func bounds(_ frame: CGRect) -> [String: Any] {
        let safe = frame.isNull || frame.isInfinite ? .zero : frame
        return [
            "x": Int32(clamping: Int(safe.origin.x.rounded())),
            "y": Int32(clamping: Int(safe.origin.y.rounded())),
            "width": UInt32(clamping: Int(max(0, safe.size.width.rounded()))),
            "height": UInt32(clamping: Int(max(0, safe.size.height.rounded())))
        ]
    }

    private func stringValue(_ value: Any?) -> String? {
        guard let value else { return nil }
        if value is NSNull { return nil }
        if let string = value as? String {
            return string.isEmpty ? nil : string
        }
        if let number = value as? NSNumber {
            return number.stringValue
        }
        let string = String(describing: value)
        return string.isEmpty ? nil : string
    }

    private func intValue(_ value: Any?) -> Int? {
        (value as? NSNumber)?.intValue
    }

    private func doubleValue(_ value: Any?) -> CGFloat? {
        guard let number = value as? NSNumber else { return nil }
        return CGFloat(number.doubleValue)
    }

    private func addLog(_ level: String, _ tag: String, _ message: String) {
        logs.append([
            "timestamp": AdapterJSON.timestamp(),
            "level": level,
            "tag": tag,
            "message": message
        ])
        if logs.count > maxLogs {
            logs.removeFirst(logs.count - maxLogs)
        }
    }

    private func role(for type: XCUIElement.ElementType) -> String {
        switch type {
        case .application: return "application"
        case .window: return "window"
        case .button: return "button"
        case .staticText: return "text"
        case .textField: return "text_field"
        case .secureTextField: return "secure_text_field"
        case .textView: return "text_view"
        case .switch, .toggle: return "switch"
        case .slider: return "slider"
        case .image: return "image"
        case .cell: return "cell"
        case .table: return "table"
        case .collectionView: return "collection"
        case .scrollView: return "scroll"
        case .navigationBar: return "navigation_bar"
        case .tabBar: return "tab_bar"
        case .tabGroup: return "tab_group"
        case .toolbar: return "toolbar"
        case .menu: return "menu"
        case .menuItem: return "menu_item"
        case .link: return "link"
        case .searchField: return "search_field"
        case .picker: return "picker"
        case .pickerWheel: return "picker_wheel"
        case .segmentedControl: return "segmented_control"
        case .progressIndicator: return "progress"
        case .activityIndicator: return "activity"
        case .alert: return "alert"
        case .sheet: return "sheet"
        case .group: return "group"
        default: return String(describing: type)
        }
    }

    private func elementType(for role: String) -> XCUIElement.ElementType {
        switch role {
        case "button": return .button
        case "text": return .staticText
        case "text_field": return .textField
        case "secure_text_field": return .secureTextField
        case "text_view": return .textView
        case "switch": return .switch
        case "image": return .image
        case "cell": return .cell
        case "scroll": return .scrollView
        case "navigation_bar": return .navigationBar
        case "tab_bar": return .tabBar
        case "link": return .link
        case "search_field": return .searchField
        default: return .any
        }
    }
}
