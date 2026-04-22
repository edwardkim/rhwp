import Foundation

@MainActor
enum DocumentOpenRouter {
    private static weak var store: DocumentViewerStore?
    private static var pendingURL: URL?

    static func bindStore(_ store: DocumentViewerStore) {
        self.store = store
    }

    @discardableResult
    static func openPendingURL() -> Bool {
        guard let url = pendingURL else {
            return false
        }
        pendingURL = nil
        store?.loadDocument(from: url)
        return true
    }

    static func requestOpen(_ url: URL) {
        if let store {
            store.loadDocument(from: url)
        } else {
            pendingURL = url
        }
    }
}
