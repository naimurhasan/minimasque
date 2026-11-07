import SwiftUI

struct MigrateButton: View {
    let action: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Button("Migrate path", action: action)
                .buttonStyle(.bordered)
                .frame(maxWidth: .infinity)
            Text("Moves QUIC to a new UDP socket, like switching Wi-Fi to mobile data. The tunnel stays open.")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
    }
}
