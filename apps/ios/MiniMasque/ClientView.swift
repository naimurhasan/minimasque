import SwiftUI

struct ClientView: View {
    @Bindable var model: ClientModel

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                Text("MiniMASQUE").font(.largeTitle.bold())
                Text("CONNECT-UDP over HTTP/3, with HTTP/2 fallback").foregroundStyle(.secondary)
                ConnectionForm(model: model)
                if model.connected {
                    StatsCard(model: model)
                    if model.canMigrate {
                        MigrateButton { Task { await model.migrate() } }
                    }
                    ProbeCard(result: model.probeResult) { Task { await model.probe() } }
                }
                InspectorLog(lines: model.log)
            }
            .padding()
        }
        .defaultScrollAnchor(.bottom)
    }
}
