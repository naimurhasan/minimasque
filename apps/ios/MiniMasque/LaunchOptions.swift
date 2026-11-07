import Foundation

struct LaunchOptions {
    var proxy = "192.168.1.50:4433"
    var serverName = "192.168.1.50"
    var certificatePath: String?
    var autoConnect = false
    var autoProbe = false

    static func fromEnvironment(_ environment: [String: String] = ProcessInfo.processInfo.environment) -> LaunchOptions {
        var options = LaunchOptions()
        options.proxy = environment["MINIMASQUE_PROXY"] ?? options.proxy
        options.serverName = environment["MINIMASQUE_NAME"] ?? options.serverName
        options.certificatePath = environment["MINIMASQUE_CA_PATH"]
        options.autoConnect = environment["MINIMASQUE_AUTOCONNECT"] == "1"
        options.autoProbe = environment["MINIMASQUE_AUTOPROBE"] == "1"
        return options
    }
}
