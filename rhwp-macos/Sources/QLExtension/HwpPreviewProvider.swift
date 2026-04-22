import Foundation
import QuickLookUI
import UniformTypeIdentifiers

final class HwpPreviewProvider: QLPreviewProvider, QLPreviewingController {
    func providePreview(for request: QLFilePreviewRequest) async throws -> QLPreviewReply {
        try await MainActor.run {
            try Self.createPreview(for: request)
        }
    }

    @MainActor
    private static func createPreview(for request: QLFilePreviewRequest) throws -> QLPreviewReply {
        do {
            let result = try HwpPageImageRenderer.renderFirstPage(fileURL: request.fileURL)
            let pngData = try HwpPageImageRenderer.encodePNG(result.image)
            return QLPreviewReply(
                dataOfContentType: .png,
                contentSize: result.size
            ) { reply in
                reply.title = request.fileURL.lastPathComponent
                return pngData
            }
        } catch HwpRenderError.fileTooLarge {
            return Self.textReply("The file is larger than 50 MB.")
        } catch {
            throw error
        }
    }

    private static func textReply(_ text: String) -> QLPreviewReply {
        QLPreviewReply(
            dataOfContentType: .plainText,
            contentSize: CGSize(width: 520, height: 120)
        ) { _ in
            Data(text.utf8)
        }
    }
}
