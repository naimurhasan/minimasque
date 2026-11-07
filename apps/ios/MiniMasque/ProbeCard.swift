import SwiftUI

struct ProbeCard: View {
    let result: String?
    let onProbe: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Button("Resolve example.com through the tunnel", action: onProbe).buttonStyle(.bordered)
            if let result {
                Text(result)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding()
        .background(.quaternary, in: RoundedRectangle(cornerRadius: 12))
    }
}
