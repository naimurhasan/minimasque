import SwiftUI

struct StatsCard: View {
    let model: ClientModel

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("● Connected · \(model.transportLabel ?? "")").font(.headline)
            Text("Apps can send UDP to \(model.localAddress ?? "")")
            if let stats = model.stats {
                Text("↑ \(formatBytes(stats.bytesUp)) (\(stats.packetsUp) packets)")
                Text("↓ \(formatBytes(stats.bytesDown)) (\(stats.packetsDown) packets)")
                Text("tunnels open \(stats.openTunnels) · dropped \(stats.dropped)")
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding()
        .background(.quaternary, in: RoundedRectangle(cornerRadius: 12))
    }
}
