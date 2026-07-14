import Foundation
import QuickLookUI

/// [Task #2267] 데이터 기반 Quick Look 미리보기 확장.
///
/// 뷰를 직접 그리지 않고 **PDF 로 변환해 시스템에 넘긴다**(`QLPreviewReply`).
/// 확대·스크롤·다중 페이지 UI 를 Quick Look 이 제공하므로 뷰 코드가 필요 없고,
/// PDF 는 macOS 의 1급 시민이라 렌더 품질도 시스템이 책임진다.
final class PreviewProvider: QLPreviewProvider, QLPreviewingController {

    func providePreview(for request: QLFilePreviewRequest) async throws -> QLPreviewReply {
        // 렌더는 동기·CPU 바운드다. 파일 URL 만 캡처해 백그라운드에서 수행한다.
        let fileURL = request.fileURL

        let pdf = try await Task.detached(priority: .userInitiated) {
            try RhwpRenderer.renderPDF(
                fileURL: fileURL,
                maxPages: RhwpRenderer.previewPageLimit
            )
        }.value

        return QLPreviewReply(
            dataOfContentType: .pdf,
            contentSize: .zero  // PDF 자체가 페이지 크기를 갖는다.
        ) { _ in
            pdf
        }
    }
}
