import Foundation
import Network
import Security

let arguments = CommandLine.arguments
guard arguments.count == 5, let port = NWEndpoint.Port(arguments[2]) else {
    FileHandle.standardError.write("usage: relay-dns.swift <relay-host> <relay-port> <relay-cert.der> <h3|h2>\n".data(using: .utf8)!)
    exit(2)
}
let relayHost = NWEndpoint.Host(arguments[1])
let pinned = try Data(contentsOf: URL(fileURLWithPath: arguments[3]))
let transport = arguments[4]
let queue = DispatchQueue(label: "relay-dns")

let tls = NWProtocolTLS.Options()
sec_protocol_options_set_verify_block(tls.securityProtocolOptions, { _, trust, complete in
    let chain = SecTrustCopyCertificateChain(sec_trust_copy_ref(trust).takeRetainedValue()) as? [SecCertificate]
    complete(chain?.first.map { SecCertificateCopyData($0) as Data == pinned } ?? false)
}, queue)

let relay = NWEndpoint.hostPort(host: relayHost, port: port)
let hop = transport == "h2"
    ? ProxyConfiguration.RelayHop(http2RelayEndpoint: relay, tlsOptions: tls)
    : ProxyConfiguration.RelayHop(http3RelayEndpoint: relay, tlsOptions: tls)
let context = NWParameters.PrivacyContext(description: "minimasque-interop")
context.proxyConfigurations = [ProxyConfiguration(relayHops: [hop])]
let parameters = NWParameters.udp
parameters.setPrivacyContext(context)

func dnsQuery(_ name: String) -> Data {
    var query = Data([0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
    for label in name.split(separator: ".") {
        query.append(UInt8(label.utf8.count))
        query.append(contentsOf: label.utf8)
    }
    query.append(contentsOf: [0x00, 0x00, 0x01, 0x00, 0x01])
    return query
}

let started = Date()
let connection = NWConnection(host: "1.1.1.1", port: 53, using: parameters)
connection.stateUpdateHandler = { state in
    switch state {
    case .ready:
        connection.send(content: dnsQuery("example.com"), completion: .contentProcessed { _ in })
        connection.receiveMessage { data, _, _, error in
            guard let data, data.count >= 12 else {
                print("FAIL no reply \(String(describing: error))")
                exit(1)
            }
            let rcode = data[3] & 0x0f
            let answers = Int(data[6]) << 8 | Int(data[7])
            let millis = Int(Date().timeIntervalSince(started) * 1000)
            print(rcode == 0 ? "OK NOERROR \(answers) answers \(millis) ms" : "FAIL rcode \(rcode)")
            exit(rcode == 0 ? 0 : 1)
        }
    case .failed(let error):
        print("FAIL \(error)")
        exit(1)
    default:
        break
    }
}
connection.start(queue: queue)
queue.asyncAfter(deadline: .now() + 10) {
    print("FAIL timeout in state \(connection.state)")
    exit(1)
}
dispatchMain()
