import SwiftUI

@main
struct MiniMasqueApp: App {
    @State private var model = ClientModel(options: .fromEnvironment())

    var body: some Scene {
        WindowGroup {
            ClientView(model: model)
                .preferredColorScheme(.dark)
                .task { await model.start() }
        }
    }
}
