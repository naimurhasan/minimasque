import SwiftUI
import UniformTypeIdentifiers

struct ConnectionForm: View {
    @Bindable var model: ClientModel
    @State private var importing = false

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            field("Proxy (ip:port)", text: $model.proxy)
            field("Certificate name", text: $model.serverName)
            field("UDP target", text: $model.target)
            TransportPicker(selection: $model.transport).disabled(model.connected)
            Button(model.certificateName.map { "Certificate: \($0)" } ?? "Pick proxy certificate (.der)") {
                importing = true
            }
            .buttonStyle(.bordered)
            .disabled(model.connected)
            if model.connected {
                Button("Disconnect") { model.disconnect() }.buttonStyle(.borderedProminent)
            } else {
                Button(model.connecting ? "Connecting…" : "Connect") { Task { await model.connect() } }
                    .buttonStyle(.borderedProminent)
                    .disabled(model.connecting)
            }
            if let error = model.error {
                Text(error).foregroundStyle(.red)
            }
        }
        .fileImporter(isPresented: $importing, allowedContentTypes: [.data]) { result in
            guard let url = try? result.get(), url.startAccessingSecurityScopedResource() else { return }
            defer { url.stopAccessingSecurityScopedResource() }
            if let data = try? Data(contentsOf: url) {
                model.setCertificate(name: url.lastPathComponent, data: data)
            }
        }
    }

    private func field(_ title: String, text: Binding<String>) -> some View {
        TextField(title, text: text)
            .textFieldStyle(.roundedBorder)
            .textInputAutocapitalization(.never)
            .autocorrectionDisabled()
            .disabled(model.connected)
    }
}
