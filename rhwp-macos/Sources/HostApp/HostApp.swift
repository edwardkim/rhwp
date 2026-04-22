import AppKit
import SwiftUI

@main
struct HwpQuickLookApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @StateObject private var viewerStore = DocumentViewerStore()
    @StateObject private var extensionStatus = ExtensionStatusModel()

    var body: some Scene {
        WindowGroup {
            ContentView(store: viewerStore, extensionStatus: extensionStatus)
                .frame(minWidth: 900, minHeight: 620)
                .task {
                    DocumentOpenRouter.bindStore(viewerStore)
                    extensionStatus.refresh()
                    if !DocumentOpenRouter.openPendingURL() {
                        viewerStore.loadSampleIfNeeded()
                    }
                }
        }
        .commands {
            CommandGroup(replacing: .newItem) {
                Button("Open Document...") {
                    viewerStore.openDocument()
                }
                .keyboardShortcut("o", modifiers: [.command])
            }

            CommandMenu("View") {
                Button("Zoom In") {
                    viewerStore.zoomIn()
                }
                .keyboardShortcut("+", modifiers: [.command])

                Button("Zoom Out") {
                    viewerStore.zoomOut()
                }
                .keyboardShortcut("-", modifiers: [.command])

                Button("Actual Size") {
                    viewerStore.resetZoom()
                }
                .keyboardShortcut("0", modifiers: [.command])
            }
        }
    }
}

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    func application(_ sender: NSApplication, openFile filename: String) -> Bool {
        DocumentOpenRouter.requestOpen(URL(fileURLWithPath: filename))
        return true
    }

    func application(_ application: NSApplication, open urls: [URL]) {
        urls.first.map(DocumentOpenRouter.requestOpen)
    }
}
