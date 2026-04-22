import CoreGraphics
import Foundation

@MainActor
final class DocumentViewerStore: ObservableObject {
    @Published var document: RhwpDocument?
    @Published var filename: String = ""
    @Published var currentPage: Int = 0
    @Published var errorMessage: String?
    @Published var isLoading = false
    @Published var pageTrees: [Int: RenderNode] = [:]
    @Published var zoomScale: Double = 0.8

    let minimumZoomScale = 0.25
    let maximumZoomScale = 3.0

    var pageCount: Int {
        document?.pageCount ?? 0
    }

    var hasDocument: Bool {
        document != nil && pageCount > 0
    }

    func loadSampleIfNeeded() {
        guard document == nil else {
            return
        }
        loadSample()
    }

    func loadSample() {
        guard let url = Bundle.main.url(forResource: "sample", withExtension: "hwpx") else {
            errorMessage = "샘플 파일을 찾을 수 없습니다."
            return
        }
        loadDocument(from: url)
    }

    func openDocument() {
        guard let url = DocumentOpenPanel.chooseDocumentURL() else {
            return
        }
        loadDocument(from: url)
    }

    func loadDocument(from url: URL) {
        isLoading = true
        errorMessage = nil
        pageTrees.removeAll()
        currentPage = 0

        let didStartSecurityScope = url.startAccessingSecurityScopedResource()
        defer {
            if didStartSecurityScope {
                url.stopAccessingSecurityScopedResource()
            }
        }

        do {
            let data = try Data(contentsOf: url, options: [.mappedIfSafe])
            try loadDocument(data: data, filename: url.lastPathComponent)
        } catch let error as RhwpError {
            errorMessage = error.errorDescription
            document = nil
            filename = ""
        } catch {
            errorMessage = "문서를 열 수 없습니다: \(error.localizedDescription)"
            document = nil
            filename = ""
        }

        isLoading = false
    }

    func pageSize(at page: Int) -> CGSize {
        guard let document else {
            return .zero
        }
        let size = document.pageSize(at: page)
        return CGSize(width: size.width, height: size.height)
    }

    func loadPage(_ page: Int) {
        guard pageTrees[page] == nil, let document else {
            return
        }
        pageTrees[page] = document.renderPageTree(at: page)
    }

    func unloadPage(_ page: Int) {
        pageTrees.removeValue(forKey: page)
    }

    func setCurrentPage(_ page: Int) {
        guard page >= 0, page < pageCount else {
            return
        }
        currentPage = page
    }

    func zoomIn() {
        zoomScale = min(maximumZoomScale, (zoomScale * 1.2).rounded(toPlaces: 2))
    }

    func zoomOut() {
        zoomScale = max(minimumZoomScale, (zoomScale / 1.2).rounded(toPlaces: 2))
    }

    func resetZoom() {
        zoomScale = 1.0
    }

    private func loadDocument(data: Data, filename: String) throws {
        guard !data.isEmpty else {
            throw RhwpError.invalidData
        }

        document = try RhwpDocument(data: data, filename: filename)
        self.filename = filename
        currentPage = 0
        zoomScale = 0.8
    }
}

private extension Double {
    func rounded(toPlaces places: Int) -> Double {
        let divisor = pow(10.0, Double(places))
        return (self * divisor).rounded() / divisor
    }
}
