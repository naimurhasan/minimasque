import Foundation

private let units = ["B", "KiB", "MiB", "GiB"]

func formatBytes(_ value: UInt64) -> String {
    var scaled = Double(value)
    var unit = 0
    while scaled >= 1024 && unit < units.count - 1 {
        scaled /= 1024
        unit += 1
    }
    return unit == 0 ? "\(value) B" : String(format: "%.1f %@", scaled, units[unit])
}
