import Foundation
import Network

enum DnsProbeError: Error {
    case invalidAddress
    case noReply
}

enum DnsProbe {
    static func run(local: String, name: String) async throws -> String {
    guard let colon = local.lastIndex(of: ":"), let port = NWEndpoint.Port(String(local[local.index(after: colon)...])) else {
        throw DnsProbeError.invalidAddress
    }
    let connection = NWConnection(host: NWEndpoint.Host(String(local[..<colon])), port: port, using: .udp)
    let started = Date()
    let reply: Data = try await withCheckedThrowingContinuation { continuation in
        connection.start(queue: .global())
        connection.send(content: dnsQuery(id: .random(in: 0...UInt16.max), name: name), completion: .contentProcessed { _ in })
        connection.receiveMessage { data, _, _, error in
            if let data { continuation.resume(returning: data) } else { continuation.resume(throwing: error ?? DnsProbeError.noReply) }
        }
    }
    connection.cancel()
    let millis = Int(Date().timeIntervalSince(started) * 1000)
    guard let answer = parseDnsResponse(reply) else { throw DnsProbeError.noReply }
    return "\(name) · \(rcodeName(answer.rcode)) · \(answer.answers) answers · \(millis) ms"
}
}
