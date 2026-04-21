import CoreGraphics
import Foundation
import ImageIO
import QuickLookUI
import UniformTypeIdentifiers

private let maxPreviewFileSize = 50 * 1024 * 1024

final class HwpPreviewProvider: QLPreviewProvider, QLPreviewingController {
    func providePreview(for request: QLFilePreviewRequest) async throws -> QLPreviewReply {
        try await MainActor.run {
            try Self.createPreview(for: request)
        }
    }

    @MainActor
    private static func createPreview(for request: QLFilePreviewRequest) throws -> QLPreviewReply {
        do {
            let result = try PreviewRenderer.render(fileURL: request.fileURL)
            return QLPreviewReply(
                dataOfContentType: .png,
                contentSize: result.size
            ) { reply in
                reply.title = request.fileURL.lastPathComponent
                return result.pngData
            }
        } catch PreviewError.fileTooLarge {
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

private enum PreviewError: Error {
    case fileTooLarge
    case emptyDocument
    case renderTreeUnavailable
    case invalidPageSize
    case bitmapContextUnavailable
    case imageUnavailable
    case pngEncodingFailed
}

@MainActor
private enum PreviewRenderer {
    static func render(fileURL: URL) throws -> (pngData: Data, size: CGSize) {
        let values = try fileURL.resourceValues(forKeys: [.fileSizeKey])
        if let fileSize = values.fileSize, fileSize > maxPreviewFileSize {
            throw PreviewError.fileTooLarge
        }

        let data = try Data(contentsOf: fileURL, options: [.mappedIfSafe])
        let document = try RhwpDocument(data: data, filename: fileURL.lastPathComponent)
        guard document.pageCount > 0 else {
            throw PreviewError.emptyDocument
        }
        guard let tree = document.renderPageTree(at: 0) else {
            throw PreviewError.renderTreeUnavailable
        }

        let pageSize = document.pageSize(at: 0)
        guard pageSize.width > 0, pageSize.height > 0 else {
            throw PreviewError.invalidPageSize
        }

        let width = max(1, Int(ceil(pageSize.width)))
        let height = max(1, Int(ceil(pageSize.height)))
        let bytesPerRow = width * 4
        var pixels = [UInt8](repeating: 255, count: height * bytesPerRow)
        let colorSpace = CGColorSpaceCreateDeviceRGB()
        guard let context = CGContext(
            data: &pixels,
            width: width,
            height: height,
            bitsPerComponent: 8,
            bytesPerRow: bytesPerRow,
            space: colorSpace,
            bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
        ) else {
            throw PreviewError.bitmapContextUnavailable
        }

        context.setFillColor(CGColor(gray: 1, alpha: 1))
        context.fill(CGRect(x: 0, y: 0, width: width, height: height))
        context.translateBy(x: 0, y: CGFloat(height))
        context.scaleBy(x: 1, y: -1)

        let renderer = CGTreeRenderer()
        renderer.render(tree: tree, in: context, pageHeight: pageSize.height, document: document)

        guard let image = context.makeImage() else {
            throw PreviewError.imageUnavailable
        }
        guard let pngData = encodePNG(image) else {
            throw PreviewError.pngEncodingFailed
        }

        return (pngData, CGSize(width: width, height: height))
    }

    private static func encodePNG(_ image: CGImage) -> Data? {
        let data = NSMutableData()
        guard let destination = CGImageDestinationCreateWithData(data, UTType.png.identifier as CFString, 1, nil) else {
            return nil
        }
        CGImageDestinationAddImage(destination, image, nil)
        guard CGImageDestinationFinalize(destination) else {
            return nil
        }
        return data as Data
    }
}
