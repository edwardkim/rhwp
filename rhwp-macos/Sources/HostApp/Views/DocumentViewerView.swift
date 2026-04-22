import SwiftUI

struct DocumentViewerView: View {
    @ObservedObject var store: DocumentViewerStore

    var body: some View {
        ZStack {
            if store.isLoading {
                ProgressView("Loading...")
            } else if let error = store.errorMessage {
                ErrorStateView(message: error)
            } else if store.hasDocument {
                DocumentPagesView(store: store)
            } else {
                EmptyDocumentView(store: store)
            }
        }
        .safeAreaInset(edge: .bottom) {
            StatusBarView(store: store)
        }
    }
}

private struct DocumentPagesView: View {
    @ObservedObject var store: DocumentViewerStore

    var body: some View {
        ScrollView([.vertical, .horizontal]) {
            LazyVStack(spacing: 18) {
                ForEach(0..<store.pageCount, id: \.self) { page in
                    DocumentPageContainer(store: store, page: page)
                }
            }
            .padding(28)
            .frame(maxWidth: .infinity)
        }
        .background(Color(nsColor: .underPageBackgroundColor))
    }
}

private struct DocumentPageContainer: View {
    @ObservedObject var store: DocumentViewerStore
    let page: Int

    var body: some View {
        let pageSize = store.pageSize(at: page)
        let zoom = CGFloat(store.zoomScale)
        let displaySize = CGSize(width: pageSize.width * zoom, height: pageSize.height * zoom)

        Group {
            if let tree = store.pageTrees[page], let document = store.document {
                DocumentPageView(
                    tree: tree,
                    pageSize: pageSize,
                    zoomScale: zoom,
                    document: document
                )
                .frame(width: displaySize.width, height: displaySize.height)
                .background(Color.white)
                .shadow(color: .black.opacity(0.16), radius: 5, x: 0, y: 2)
            } else {
                ProgressView()
                    .frame(width: max(160, displaySize.width), height: max(120, displaySize.height))
                    .background(Color.white)
                    .shadow(color: .black.opacity(0.10), radius: 4, x: 0, y: 1)
            }
        }
        .id(page)
        .onAppear {
            store.setCurrentPage(page)
            store.loadPage(page)
        }
        .onDisappear {
            store.unloadPage(page)
        }
    }
}

private struct EmptyDocumentView: View {
    @ObservedObject var store: DocumentViewerStore

    var body: some View {
        VStack(spacing: 14) {
            Image(systemName: "doc.richtext")
                .font(.system(size: 48))
                .foregroundStyle(.secondary)
            Text("Open an HWP or HWPX document.")
                .font(.title3)
            HStack {
                Button("Open Document") {
                    store.openDocument()
                }
                Button("Open Sample") {
                    store.loadSample()
                }
            }
        }
    }
}

private struct ErrorStateView: View {
    let message: String

    var body: some View {
        VStack(spacing: 12) {
            Image(systemName: "exclamationmark.triangle")
                .font(.system(size: 40))
                .foregroundStyle(.orange)
            Text(message)
                .multilineTextAlignment(.center)
                .foregroundStyle(.secondary)
        }
        .padding(32)
    }
}

private struct StatusBarView: View {
    @ObservedObject var store: DocumentViewerStore

    var body: some View {
        HStack(spacing: 16) {
            Text(store.filename.isEmpty ? "No document" : store.filename)
                .lineLimit(1)
            Spacer()
            if store.pageCount > 0 {
                Text("Page \(store.currentPage + 1) of \(store.pageCount)")
                Text("\(Int(store.zoomScale * 100))%")
            }
        }
        .font(.caption)
        .foregroundStyle(.secondary)
        .padding(.horizontal, 14)
        .padding(.vertical, 6)
        .background(.bar)
    }
}
