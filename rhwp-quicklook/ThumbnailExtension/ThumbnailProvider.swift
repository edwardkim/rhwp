import Foundation
import PDFKit
import QuickLookThumbnailing

/// [Task #2267] Finder 아이콘용 썸네일 확장.
///
/// 첫 페이지만 PDF 로 렌더한 뒤 PDFKit 으로 그린다. 전체 문서를 그리면 확장의
/// 메모리·시간 한도를 넘긴다.
final class ThumbnailProvider: QLThumbnailProvider {

    override func provideThumbnail(
        for request: QLFileThumbnailRequest,
        _ handler: @escaping (QLThumbnailReply?, Error?) -> Void
    ) {
        do {
            let pdfData = try RhwpRenderer.renderPDF(
                fileURL: request.fileURL,
                maxPages: RhwpRenderer.thumbnailPageLimit
            )

            guard
                let document = PDFDocument(data: pdfData),
                let page = document.page(at: 0)
            else {
                handler(nil, RhwpRenderer.RenderError.failed("PDF 첫 페이지를 열 수 없습니다."))
                return
            }

            let pageRect = page.bounds(for: .mediaBox)
            guard pageRect.width > 0, pageRect.height > 0 else {
                handler(nil, RhwpRenderer.RenderError.failed("페이지 크기가 유효하지 않습니다."))
                return
            }

            // 요청 크기에 종횡비를 맞춘다 (문서는 세로가 길다).
            let maximumSize = request.maximumSize
            let scale = min(
                maximumSize.width / pageRect.width,
                maximumSize.height / pageRect.height
            )
            let thumbnailSize = CGSize(
                width: max(pageRect.width * scale, 1),
                height: max(pageRect.height * scale, 1)
            )

            let reply = QLThumbnailReply(contextSize: thumbnailSize) { context in
                // 배경을 흰색으로 채운다. PDF 페이지에 배경이 없으면 투명해진다.
                context.setFillColor(.white)
                context.fill(CGRect(origin: .zero, size: thumbnailSize))

                context.saveGState()
                context.scaleBy(x: scale, y: scale)
                context.translateBy(x: -pageRect.origin.x, y: -pageRect.origin.y)
                page.draw(with: .mediaBox, to: context)
                context.restoreGState()

                return true
            }

            handler(reply, nil)
        } catch {
            handler(nil, error)
        }
    }
}
