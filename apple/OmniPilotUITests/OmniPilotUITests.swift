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

import XCTest

final class OmniPilotUITests: XCTestCase {
    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    func testDarkModeFlow() throws {
        let app = XCUIApplication()
        app.launch()

        XCTAssertTrue(app.staticTexts["home.title"].waitForExistence(timeout: 10))
        app.buttons["home.settings"].tap()

        let toggle = app.switches["settings.dark_mode"]
        XCTAssertTrue(toggle.waitForExistence(timeout: 10))

        let state = app.staticTexts["settings.dark_mode.state"]
        XCTAssertTrue(state.waitForExistence(timeout: 10))
        XCTAssertEqual(state.label, "Dark mode is Off")

        toggle.coordinate(withNormalizedOffset: CGVector(dx: 0.9, dy: 0.5)).tap()
        let isOn = NSPredicate(format: "label == 'Dark mode is On'")
        expectation(for: isOn, evaluatedWith: state)
        waitForExpectations(timeout: 5)

        let attachment = XCTAttachment(screenshot: XCUIScreen.main.screenshot())
        attachment.name = "dark-mode-on"
        attachment.lifetime = .keepAlways
        add(attachment)
    }

    func testAboutAndBack() throws {
        let app = XCUIApplication()
        app.launch()

        XCTAssertTrue(app.buttons["home.about"].waitForExistence(timeout: 10))
        app.buttons["home.about"].tap()
        XCTAssertTrue(app.staticTexts["about.title"].waitForExistence(timeout: 10))

        app.buttons["about.back"].tap()
        XCTAssertTrue(app.staticTexts["home.title"].waitForExistence(timeout: 10))
    }
}
