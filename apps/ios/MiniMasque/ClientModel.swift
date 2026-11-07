import Foundation
import Observation

private let logCapacity = 200
private let demoName = "example.com"

@MainActor
@Observable
final class ClientModel {
    var proxy: String
    var serverName: String
    var target = "1.1.1.1:53"
    var transport = "auto"
    var certificateName: String?
    var connecting = false
    var transportLabel: String?
    var localAddress: String?
    var error: String?
    var stats: StatsRecord?
    var log: [String] = []
    var probeResult: String?

    private let options: LaunchOptions
    private var certificate: Data?
    private var forwarder: Forwarder?
    private var poller: Task<Void, Never>?

    var connected: Bool { forwarder != nil }

    init(options: LaunchOptions) {
        self.options = options
        proxy = options.proxy
        serverName = options.serverName
    }

    func start() async {
        if let path = options.certificatePath, let data = FileManager.default.contents(atPath: path) {
            setCertificate(name: URL(fileURLWithPath: path).lastPathComponent, data: data)
        }
        guard options.autoConnect else { return }
        await connect()
        if options.autoProbe { await probe() }
    }

    func setCertificate(name: String, data: Data) {
        certificate = data
        certificateName = name
    }

    func connect() async {
        guard let certificate else {
            error = "Pick the proxy certificate first"
            return
        }
        let config = ForwarderConfig(
            proxy: proxy, serverName: serverName, caDer: certificate,
            target: target, local: "127.0.0.1:0", transport: transport, template: nil
        )
        connecting = true
        error = nil
        do {
            let started = try await Task.detached { try Forwarder.start(config: config) }.value
            forwarder = started
            transportLabel = started.transport()
            localAddress = started.localAddress()
            poll(started)
        } catch {
            self.error = "\(error)"
        }
        connecting = false
    }

    func disconnect() {
        poller?.cancel()
        forwarder?.stop()
        forwarder = nil
        transportLabel = nil
        localAddress = nil
    }

    func probe() async {
        guard let localAddress else { return }
        do {
            probeResult = try await DnsProbe.run(local: localAddress, name: demoName)
        } catch {
            probeResult = "failed: \(error)"
        }
        refresh()
    }

    private func poll(_ forwarder: Forwarder) {
        poller = Task { [weak self] in
            while !Task.isCancelled {
                self?.refresh()
                try? await Task.sleep(for: .milliseconds(500))
            }
        }
    }

    private func refresh() {
        guard let forwarder else { return }
        stats = forwarder.stats()
        log = Array((log + forwarder.drainEvents()).suffix(logCapacity))
    }
}
