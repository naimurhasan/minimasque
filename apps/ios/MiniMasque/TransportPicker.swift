import SwiftUI

struct TransportPicker: View {
    @Binding var selection: String

    var body: some View {
        Picker("Transport", selection: $selection) {
            Text("Auto").tag("auto")
            Text("HTTP/3").tag("h3")
            Text("HTTP/2").tag("h2")
        }
        .pickerStyle(.segmented)
    }
}
