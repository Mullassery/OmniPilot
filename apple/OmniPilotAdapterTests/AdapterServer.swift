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
import Network

final class AdapterServer {
    let token = UUID().uuidString
    private(set) var lastError: String?

    private let queue = DispatchQueue(label: "omnipilot.adapter.server")
    private let device = AdapterDevice()
    private var listener: NWListener?
    private var connection: AdapterConnection?
    private var initialized = false
    private var finished = false

    var onReady: (() -> Void)?
    var isFinished: Bool { finished }

    func start() throws {
        let parameters = NWParameters.tcp
        parameters.requiredLocalEndpoint = .hostPort(host: .ipv4(.loopback), port: .any)
        let listener = try NWListener(using: parameters)
        self.listener = listener

        listener.stateUpdateHandler = { [weak self] state in
            guard let self else { return }
            switch state {
            case .ready:
                guard let port = listener.port?.rawValue else {
                    self.fail("listener became ready without a port")
                    return
                }
                let line = "OMNIPILOT_ADAPTER_READY port=\(port) token=\(self.token)\n"
                FileHandle.standardOutput.write(Data(line.utf8))
                DispatchQueue.main.async { self.onReady?() }
            case .failed(let error):
                self.fail("listener failed: \(error)")
            default:
                break
            }
        }

        listener.newConnectionHandler = { [weak self] connection in
            guard let self else {
                connection.cancel()
                return
            }
            DispatchQueue.main.async { self.accept(connection) }
        }

        listener.start(queue: queue)
    }

    func stop() {
        finish()
    }

    private func accept(_ networkConnection: NWConnection) {
        guard connection == nil else {
            networkConnection.cancel()
            return
        }
        let connection = AdapterConnection(connection: networkConnection, queue: queue)
        connection.onLine = { [weak self] line in
            DispatchQueue.main.async { self?.handle(line, connection: connection) }
        }
        connection.onClose = { [weak self] in
            DispatchQueue.main.async { self?.finish() }
        }
        self.connection = connection
        connection.start()
    }

    private func handle(_ line: String, connection: AdapterConnection) {
        guard let data = line.data(using: .utf8), let object = AdapterJSON.decode(data) else {
            connection.send(
                failureResponse(
                    id: 0,
                    failure: AdapterFailure.invalidParams("request is not valid JSON")
                )
            )
            return
        }
        guard let idNumber = object["id"] as? NSNumber else {
            return
        }
        let id = idNumber.uint64Value
        guard object["jsonrpc"] as? String == AdapterProtocol.jsonrpc else {
            connection.send(
                failureResponse(
                    id: id,
                    failure: AdapterFailure(AdapterProtocol.invalidRequest, "jsonrpc must be 2.0")
                )
            )
            return
        }
        guard let method = object["method"] as? String else {
            connection.send(
                failureResponse(
                    id: id,
                    failure: AdapterFailure(AdapterProtocol.invalidRequest, "method is required")
                )
            )
            return
        }
        let params = object["params"] as? [String: Any] ?? [:]

        do {
            if method == "shutdown" {
                guard initialized else {
                    throw AdapterFailure(
                        AdapterProtocol.unauthorized,
                        "initialize must succeed before other methods"
                    )
                }
                connection.send(
                    successResponse(id: id, result: ["ok": true, "message": "ok"]),
                    closeAfter: true
                )
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) { [weak self] in
                    self?.finish()
                }
                return
            }

            let result = try dispatch(method: method, params: params)
            connection.send(successResponse(id: id, result: result))
        } catch let failure as AdapterFailure {
            connection.send(failureResponse(id: id, failure: failure))
        } catch {
            connection.send(
                failureResponse(
                    id: id,
                    failure: AdapterFailure(AdapterProtocol.internalError, "\(error)")
                )
            )
        }
    }

    private func dispatch(method: String, params: [String: Any]) throws -> [String: Any] {
        if method != "initialize" && !initialized {
            throw AdapterFailure(
                AdapterProtocol.unauthorized,
                "initialize must succeed before other methods"
            )
        }

        switch method {
        case "initialize":
            if initialized {
                throw AdapterFailure(
                    AdapterProtocol.invalidRequest,
                    "session is already initialized"
                )
            }
            guard params["protocol_version"] as? String == AdapterProtocol.version else {
                throw AdapterFailure.invalidParams(
                    "unsupported protocol version; expected \(AdapterProtocol.version)"
                )
            }
            guard params["token"] as? String == token else {
                throw AdapterFailure(AdapterProtocol.unauthorized, "invalid session token")
            }
            initialized = true
            return initializeResult()

        case "observe":
            return device.observe(params)

        case "act":
            return try device.act(params)

        case "logs":
            return device.logEntries(params)

        case "cancel":
            device.cancel(params)
            return ["ok": true, "message": "cancellation recorded"]

        default:
            throw AdapterFailure(
                AdapterProtocol.methodNotFound,
                "method not found: \(method)"
            )
        }
    }

    private func initializeResult() -> [String: Any] {
        [
            "protocol_version": AdapterProtocol.version,
            "adapter": [
                "name": "omnipilot-ios-xcuitest",
                "version": "0.1.0",
                "platform": "ios"
            ],
            "device": device.info(),
            "capabilities": device.capabilities(),
            "limits": [
                "max_observation_bytes": 4 * 1024 * 1024,
                "max_log_entries": 500,
                "supports_cancellation": true,
                "supports_screenshot": true,
                "supports_hierarchy": true
            ]
        ]
    }

    private func successResponse(id: UInt64, result: [String: Any]) -> String {
        encode([
            "jsonrpc": AdapterProtocol.jsonrpc,
            "id": id,
            "result": result
        ])
    }

    private func failureResponse(id: UInt64, failure: AdapterFailure) -> String {
        var error: [String: Any] = [
            "code": failure.code,
            "message": failure.message
        ]
        if let data = failure.data {
            error["data"] = data
        }
        return encode([
            "jsonrpc": AdapterProtocol.jsonrpc,
            "id": id,
            "error": error
        ])
    }

    private func encode(_ object: [String: Any]) -> String {
        guard let data = AdapterJSON.encode(object),
              let string = String(data: data, encoding: .utf8)
        else {
            return "{\"jsonrpc\":\"2.0\",\"id\":0,\"error\":{\"code\":-32603,\"message\":\"encoding failed\"}}"
        }
        return string
    }

    private func fail(_ message: String) {
        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            self.lastError = message
            FileHandle.standardError.write(Data("OMNIPILOT_ADAPTER_ERROR \(message)\n".utf8))
            self.finish()
        }
    }

    private func finish() {
        guard !finished else { return }
        finished = true
        connection?.cancel()
        connection = nil
        listener?.cancel()
        listener = nil
    }
}

private final class AdapterConnection {
    private let connection: NWConnection
    private let queue: DispatchQueue
    private var buffer = Data()

    var onLine: ((String) -> Void)?
    var onClose: (() -> Void)?

    init(connection: NWConnection, queue: DispatchQueue) {
        self.connection = connection
        self.queue = queue
    }

    func start() {
        connection.stateUpdateHandler = { [weak self] state in
            guard let self else { return }
            switch state {
            case .ready:
                self.receive()
            case .failed, .cancelled:
                self.onClose?()
            default:
                break
            }
        }
        connection.start(queue: queue)
    }

    func send(_ text: String, closeAfter: Bool = false) {
        connection.send(
            content: Data((text + "\n").utf8),
            completion: .contentProcessed { [weak self] _ in
                if closeAfter {
                    self?.connection.cancel()
                }
            }
        )
    }

    func cancel() {
        connection.cancel()
    }

    private func receive() {
        connection.receive(
            minimumIncompleteLength: 1,
            maximumLength: 64 * 1024
        ) { [weak self] data, _, isComplete, error in
            guard let self else { return }
            if let data {
                self.buffer.append(data)
            }
            while let newline = self.buffer.firstIndex(of: 0x0A) {
                let lineData = Data(self.buffer[self.buffer.startIndex..<newline])
                self.buffer.removeSubrange(self.buffer.startIndex...newline)
                if let line = String(data: lineData, encoding: .utf8) {
                    let trimmed = line.trimmingCharacters(in: .whitespacesAndNewlines)
                    if !trimmed.isEmpty {
                        self.onLine?(trimmed)
                    }
                }
            }
            if isComplete || error != nil {
                self.onClose?()
                return
            }
            self.receive()
        }
    }
}
