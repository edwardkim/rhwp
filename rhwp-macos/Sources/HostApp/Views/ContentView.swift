import SwiftUI

struct ContentView: View {
    @ObservedObject var store: DocumentViewerStore
    @ObservedObject var extensionStatus: ExtensionStatusModel

    var body: some View {
        HStack(spacing: 0) {
            SidebarView(store: store, extensionStatus: extensionStatus)
                .frame(minWidth: 250, idealWidth: 280, maxWidth: 340)

            Divider()

            DocumentViewerView(store: store)
        }
        .toolbar {
            ToolbarItemGroup {
                Button {
                    store.openDocument()
                } label: {
                    Label("Open Document", systemImage: "folder")
                }

                Button {
                    store.loadSample()
                } label: {
                    Label("Open Sample", systemImage: "doc")
                }
            }

            ToolbarItemGroup {
                Button {
                    store.zoomOut()
                } label: {
                    Label("Zoom Out", systemImage: "minus.magnifyingglass")
                }
                .disabled(!store.hasDocument)

                Slider(
                    value: $store.zoomScale,
                    in: store.minimumZoomScale...store.maximumZoomScale
                )
                .frame(width: 130)
                .disabled(!store.hasDocument)

                Button {
                    store.zoomIn()
                } label: {
                    Label("Zoom In", systemImage: "plus.magnifyingglass")
                }
                .disabled(!store.hasDocument)

                Button {
                    store.resetZoom()
                } label: {
                    Label("Actual Size", systemImage: "arrow.up.left.and.down.right.magnifyingglass")
                }
                .disabled(!store.hasDocument)
            }
        }
    }
}

private struct SidebarView: View {
    @ObservedObject var store: DocumentViewerStore
    @ObservedObject var extensionStatus: ExtensionStatusModel

    var body: some View {
        List {
            Section("Document") {
                Label(store.filename.isEmpty ? "No document" : store.filename, systemImage: "doc.richtext")
                if store.pageCount > 0 {
                    Label("\(store.currentPage + 1) / \(store.pageCount) pages", systemImage: "number")
                    Label("\(Int(store.zoomScale * 100))%", systemImage: "magnifyingglass")
                }
            }

            Section("Extensions") {
                ForEach(ExtensionStatus.allCases, id: \.self) { status in
                    ExtensionStatusRow(
                        title: status.title,
                        bundleIdentifier: status.bundleIdentifier,
                        state: extensionStatus.state(for: status)
                    )
                }

                Button {
                    extensionStatus.refresh()
                } label: {
                    Label("Refresh Status", systemImage: "arrow.clockwise")
                }
            }

            Section("Build") {
                Label(BuildInfo.displayVersion, systemImage: "info.circle")
            }
        }
        .listStyle(.sidebar)
    }
}

private struct ExtensionStatusRow: View {
    let title: String
    let bundleIdentifier: String
    let state: ExtensionRegistrationState

    var body: some View {
        HStack(alignment: .top, spacing: 10) {
            Image(systemName: state.symbolName)
                .foregroundStyle(state.color)
                .frame(width: 16)

            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .lineLimit(1)
                Text("\(state.label) · \(bundleIdentifier)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
        }
    }
}
