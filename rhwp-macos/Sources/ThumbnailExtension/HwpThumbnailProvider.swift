import CoreGraphics
import QuickLookThumbnailing

final class HwpThumbnailProvider: QLThumbnailProvider {
    override func provideThumbnail(
        for request: QLFileThumbnailRequest,
        _ handler: @escaping (QLThumbnailReply?, Error?) -> Void
    ) {
        Task { @MainActor in
            do {
                let renderedPage = try HwpPageImageRenderer.renderFirstPage(fileURL: request.fileURL)
                let contextSize = Self.aspectFit(renderedPage.size, within: request.maximumSize)
                let image = renderedPage.image
                let reply = QLThumbnailReply(contextSize: contextSize) { context in
                    Self.drawPageImage(image, in: context, size: contextSize)
                    return true
                }
                reply.extensionBadge = request.fileURL.pathExtension.uppercased()
                handler(reply, nil)
            } catch HwpRenderError.fileTooLarge {
                let reply = QLThumbnailReply(contextSize: request.maximumSize) { context in
                    Self.drawFallback(in: context, size: request.maximumSize)
                    return true
                }
                reply.extensionBadge = request.fileURL.pathExtension.uppercased()
                handler(reply, nil)
            } catch {
                handler(nil, error)
            }
        }
    }

    private static func aspectFit(_ source: CGSize, within maximumSize: CGSize) -> CGSize {
        guard source.width > 0, source.height > 0, maximumSize.width > 0, maximumSize.height > 0 else {
            return CGSize(width: 128, height: 128)
        }
        let scale = min(maximumSize.width / source.width, maximumSize.height / source.height)
        return CGSize(width: max(1, source.width * scale), height: max(1, source.height * scale))
    }

    private static func drawPageImage(_ image: CGImage, in context: CGContext, size: CGSize) {
        let rect = drawingBounds(in: context, fallbackSize: size)
        context.saveGState()
        context.setFillColor(CGColor(gray: 1, alpha: 1))
        context.fill(rect)
        context.draw(image, in: rect)
        context.restoreGState()
    }

    private static func drawFallback(in context: CGContext, size: CGSize) {
        let rect = drawingBounds(in: context, fallbackSize: size)
        context.saveGState()
        context.setFillColor(CGColor(gray: 0.93, alpha: 1))
        context.fill(rect)
        context.setStrokeColor(CGColor(gray: 0.55, alpha: 1))
        context.setLineWidth(max(1, min(size.width, size.height) * 0.04))
        context.stroke(rect.insetBy(dx: 2, dy: 2))
        context.restoreGState()
    }

    private static func drawingBounds(in context: CGContext, fallbackSize: CGSize) -> CGRect {
        let clipBounds = context.boundingBoxOfClipPath
        guard
            !clipBounds.isNull,
            !clipBounds.isInfinite,
            clipBounds.width.isFinite,
            clipBounds.height.isFinite,
            clipBounds.width > 0,
            clipBounds.height > 0
        else {
            return CGRect(origin: .zero, size: fallbackSize)
        }
        return clipBounds
    }
}
