import SwiftUI

struct InspectorLog: View {
    let lines: [String]

    private func color(for line: String) -> Color {
        switch line.first {
        case "▶": return .yellow
        case "◀": return .green
        case "●": return .teal
        case "◆": return .pink
        default: return .secondary
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text("Protocol inspector").font(.headline)
            ForEach(Array(lines.enumerated()), id: \.offset) { _, line in
                Text(line).font(.caption.monospaced()).foregroundStyle(color(for: line))
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding()
        .background(.quaternary, in: RoundedRectangle(cornerRadius: 12))
    }
}
