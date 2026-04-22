import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers

let hwpQuickLookMaxFileSize = 50 * 1024 * 1024

struct HwpRenderedPage {
    let image: CGImage
    let size: CGSize
}

enum HwpRenderError: Error {
    case fileTooLarge
    case emptyDocument
    case renderTreeUnavailable
    case invalidPageSize
    case bitmapContextUnavailable
    case imageUnavailable
    case pngEncodingFailed
}

@MainActor
enum HwpPageImageRenderer {
    static func renderFirstPage(fileURL: URL) throws -> HwpRenderedPage {
        let values = try fileURL.resourceValues(forKeys: [.fileSizeKey])
        if let fileSize = values.fileSize, fileSize > hwpQuickLookMaxFileSize {
            throw HwpRenderError.fileTooLarge
        }

        let data = try Data(contentsOf: fileURL, options: [.mappedIfSafe])
        let document = try RhwpDocument(data: data, filename: fileURL.lastPathComponent)
        guard document.pageCount > 0 else {
            throw HwpRenderError.emptyDocument
        }
        guard let tree = document.renderPageTree(at: 0) else {
            throw HwpRenderError.renderTreeUnavailable
        }

        let pageSize = document.pageSize(at: 0)
        guard pageSize.width > 0, pageSize.height > 0 else {
            throw HwpRenderError.invalidPageSize
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
            throw HwpRenderError.bitmapContextUnavailable
        }

        context.setFillColor(CGColor(gray: 1, alpha: 1))
        context.fill(CGRect(x: 0, y: 0, width: width, height: height))
        context.translateBy(x: 0, y: CGFloat(height))
        context.scaleBy(x: 1, y: -1)

        let renderer = CGTreeRenderer()
        renderer.render(tree: tree, in: context, pageHeight: pageSize.height, document: document)

        guard let image = context.makeImage() else {
            throw HwpRenderError.imageUnavailable
        }

        return HwpRenderedPage(image: image, size: CGSize(width: width, height: height))
    }

    static func encodePNG(_ image: CGImage) throws -> Data {
        let data = NSMutableData()
        guard let destination = CGImageDestinationCreateWithData(data, UTType.png.identifier as CFString, 1, nil) else {
            throw HwpRenderError.pngEncodingFailed
        }
        CGImageDestinationAddImage(destination, image, nil)
        guard CGImageDestinationFinalize(destination) else {
            throw HwpRenderError.pngEncodingFailed
        }
        return data as Data
    }
}
