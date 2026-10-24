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

@MainActor
final class OmniPilotAdapterTests: XCTestCase {
    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    func testServeAdapter() throws {
        let server = AdapterServer()
        try server.start()

        let deadline = Date().addingTimeInterval(60)
        while !server.isFinished && Date() < deadline {
            RunLoop.main.run(until: Date().addingTimeInterval(0.1))
        }

        XCTAssertTrue(
            server.isFinished,
            server.lastError ?? "adapter server did not finish within 60 seconds"
        )
    }
}
