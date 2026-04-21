import AppKit
import SwiftUI

@main
struct HwpQuickLookApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
                .frame(minWidth: 520, minHeight: 320)
        }
    }
}

private struct ContentView: View {
    var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            HStack(spacing: 12) {
                Image(systemName: "doc.richtext")
                    .font(.system(size: 34, weight: .semibold))
                    .foregroundStyle(Color.accentColor)

                VStack(alignment: .leading, spacing: 3) {
                    Text("HWP Quick Look")
                        .font(.title2.weight(.semibold))
                    Text("Finder preview extension")
                        .font(.callout)
                        .foregroundStyle(.secondary)
                }
            }

            Divider()

            VStack(alignment: .leading, spacing: 10) {
                Label("Preview extension bundle is included in this app.", systemImage: "checkmark.circle")
                Label("Supported types: HWP and HWPX.", systemImage: "doc.text.magnifyingglass")
                Label("Keep this app in Applications for Finder registration.", systemImage: "folder")
            }
            .font(.body)

            Spacer()

            HStack {
                Button("Open Sample") {
                    SampleFile.open()
                }
                Spacer()
                Text("v0.1.0")
                    .foregroundStyle(.secondary)
            }
        }
        .padding(28)
    }
}

private enum SampleFile {
    static func open() {
        guard let url = Bundle.main.url(forResource: "sample", withExtension: "hwpx") else {
            return
        }
        NSWorkspace.shared.open(url)
    }
}
