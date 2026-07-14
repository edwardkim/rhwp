import Foundation

/// [Task #2267] HWP/HWPX → PDF 렌더러. Quick Look 확장 두 개가 공유한다.
///
/// 확장 프로세스에는 하드 제약이 있다: 약 **80MB 경고 / 120MB 강제 종료 / 30초 타임아웃**.
/// 아래 상수와 옵션은 전부 그 한도에서 역산한 실측 기반이다.
enum RhwpRenderer {

    /// 미리보기에서 렌더할 최대 페이지 수.
    ///
    /// 실측 (1페이지 렌더 최대 RSS, `embed_text=0`):
    ///
    /// | 문서 | 1쪽 | 3쪽 | 5쪽 | 10쪽 |
    /// |---|---|---|---|---|
    /// | 이미지 다수 8쪽 | 92 | 94 | **179** | 193 |
    /// | 비트맵 다수 20쪽 | 100 | 102 | 104 | 143 |
    /// | 이미지 중간 74쪽 | 41 | 48 | 50 | 81 |
    ///
    /// **5페이지는 120MB 강제 종료선을 넘는다.** 3페이지가 실측 최대 102MB 로 안전한
    /// 상한이다. 늘리려면 반드시 재측정한다.
    static let previewPageLimit: Int32 = 3

    /// 썸네일은 첫 페이지만 그린다.
    static let thumbnailPageLimit: Int32 = 1

    /// 번들에 담긴 한글 폰트 디렉터리.
    ///
    /// 코어의 기본 폰트 탐색 경로는 **작업디렉터리 상대경로**(`ttfs/` 등)라, 샌드박스된
    /// 확장 프로세스에서는 절대 잡히지 않는다. 번들 Resources 의 절대경로를 넘겨야
    /// 한글이 깨지지 않는다.
    static var fontDirectory: URL? {
        Bundle.main.resourceURL?.appendingPathComponent("opensource", isDirectory: true)
    }

    enum RenderError: LocalizedError {
        case failed(String)

        var errorDescription: String? {
            switch self {
            case let .failed(message):
                return message
            }
        }
    }

    /// 문서를 PDF 로 렌더링한다.
    ///
    /// `embedText` 는 기본 `false` 다. 텍스트를 PDF 폰트로 임베드하면 폰트 서브셋
    /// 과정에서 한글 폰트 전체가 메모리에 올라와 RSS 가 100MB 이상 뛴다 (#2264).
    /// 미리보기는 시각 표현만 필요하므로 글리프를 path 로 그린다.
    static func renderPDF(
        fileURL: URL,
        maxPages: Int32,
        embedText: Bool = false
    ) throws -> Data {
        let buffer: RhwpBuffer = fileURL.path.withCString { input in
            let render: (UnsafePointer<CChar>?) -> RhwpBuffer = { fontDir in
                rhwp_render_pdf(input, 0, maxPages, fontDir, embedText ? 1 : 0)
            }
            if let fontDirectory {
                return fontDirectory.path.withCString { render($0) }
            }
            return render(nil)
        }

        defer { rhwp_buffer_free(buffer) }

        if let error = buffer.error {
            throw RenderError.failed(String(cString: error))
        }
        guard let data = buffer.data, buffer.len > 0 else {
            throw RenderError.failed("렌더 결과가 비어 있습니다.")
        }
        return Data(bytes: data, count: buffer.len)
    }
}
