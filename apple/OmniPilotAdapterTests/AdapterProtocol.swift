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

enum AdapterProtocol {
    static let version = "omnipilot.adapter/1"
    static let jsonrpc = "2.0"

    static let application = -32000
    static let unauthorized = -32001
    static let unsupported = -32002
    static let invalidRequest = -32600
    static let methodNotFound = -32601
    static let invalidParams = -32602
    static let internalError = -32603
}

struct AdapterFailure: Error {
    let code: Int
    let message: String
    let retryable: Bool
    let kind: String?

    init(_ code: Int, _ message: String, retryable: Bool = false, kind: String? = nil) {
        self.code = code
        self.message = message
        self.retryable = retryable
        self.kind = kind
    }

    var data: [String: Any]? {
        var value: [String: Any] = [:]
        if retryable {
            value["retryable"] = true
        }
        if let kind {
            value["kind"] = kind
        }
        return value.isEmpty ? nil : value
    }

    static func invalidParams(_ message: String) -> AdapterFailure {
        AdapterFailure(AdapterProtocol.invalidParams, message)
    }

    static func elementNotFound(_ message: String) -> AdapterFailure {
        AdapterFailure(
            AdapterProtocol.application,
            message,
            retryable: true,
            kind: "element_not_found"
        )
    }

    static func appNotFound(_ message: String) -> AdapterFailure {
        AdapterFailure(
            AdapterProtocol.application,
            message,
            retryable: false,
            kind: "app_not_found"
        )
    }

    static func unsupported(_ message: String) -> AdapterFailure {
        AdapterFailure(AdapterProtocol.unsupported, message)
    }
}

enum AdapterJSON {
    static func encode(_ object: [String: Any]) -> Data? {
        try? JSONSerialization.data(withJSONObject: object, options: [])
    }

    static func decode(_ data: Data) -> [String: Any]? {
        try? JSONSerialization.jsonObject(with: data, options: []) as? [String: Any]
    }

    static func timestamp(_ date: Date = Date()) -> String {
        formatter.string(from: date)
    }

    private static let formatter: ISO8601DateFormatter = {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return formatter
    }()
}
