import Foundation

struct DnsAnswer: Equatable {
    let rcode: Int
    let answers: Int
}

private let rcodes = ["NOERROR", "FORMERR", "SERVFAIL", "NXDOMAIN", "NOTIMP", "REFUSED"]

func dnsQuery(id: UInt16, name: String) -> Data {
    var query = Data([UInt8(id >> 8), UInt8(id & 0xff), 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
    for label in name.split(separator: ".") {
        query.append(UInt8(label.utf8.count))
        query.append(contentsOf: label.utf8)
    }
    query.append(contentsOf: [0x00, 0x00, 0x01, 0x00, 0x01])
    return query
}

func parseDnsResponse(_ data: Data) -> DnsAnswer? {
    guard data.count >= 12 else { return nil }
    let bytes = [UInt8](data)
    return DnsAnswer(rcode: Int(bytes[3] & 0x0f), answers: Int(bytes[6]) << 8 | Int(bytes[7]))
}

func rcodeName(_ rcode: Int) -> String {
    rcodes.indices.contains(rcode) ? rcodes[rcode] : "RCODE \(rcode)"
}
