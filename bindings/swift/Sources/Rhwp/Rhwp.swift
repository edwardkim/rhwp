import CRhwpNative
import Foundation

public enum RhwpPage: Sendable, Equatable {
    case all
    case index(Int32)

    var ffiValue: Int32 {
        switch self {
        case .all:
            return -1
        case let .index(page):
            return page
        }
    }
}

public struct RhwpExportResult: Codable, Sendable, Equatable {
    public let ok: Bool
    public let pageCount: Int?
    public let files: [String]?
    public let imageCount: Int?
    public let error: String?

    public var outputFiles: [URL] {
        (files ?? []).map(URL.init(fileURLWithPath:))
    }
}

public struct RhwpTextPage: Codable, Sendable, Equatable, Identifiable {
    public let index: Int
    public let text: String

    public var id: Int {
        index
    }
}

public struct RhwpDocumentText: Codable, Sendable, Equatable {
    public let ok: Bool
    public let pageCount: Int?
    public let pages: [RhwpTextPage]?
    public let error: String?

    public var text: String {
        (pages ?? []).map(\.text).joined(separator: "\n")
    }
}

public enum RhwpError: Error, LocalizedError, Equatable {
    case nativeReturnedNull
    case invalidUTF8
    case invalidJSON(String)
    case exportFailed(String)

    public var errorDescription: String? {
        switch self {
        case .nativeReturnedNull:
            return "Native rhwp call returned a null result pointer."
        case .invalidUTF8:
            return "Native rhwp call returned invalid UTF-8."
        case let .invalidJSON(payload):
            return "Native rhwp call returned invalid JSON: \(payload)"
        case let .exportFailed(message):
            return message
        }
    }
}

public enum Rhwp {
    /// 문서의 페이지 수.
    public static func pageCount(inputFile: URL) throws -> Int {
        let count = inputFile.path.withCString { rhwp_page_count($0) }
        guard count >= 0 else {
            throw RhwpError.exportFailed("페이지 수를 읽을 수 없습니다: \(inputFile.path)")
        }
        return Int(count)
    }

    /// 문서를 PDF 로 렌더링한다.
    ///
    /// [Task #2267] Quick Look 확장이 쓰는 진입점.
    ///
    /// - Parameters:
    ///   - firstPage: 0-based 시작 페이지
    ///   - maxPages: 렌더할 최대 페이지 수. `nil` 이면 문서 끝까지.
    ///     **확장은 메모리·시간 한도가 있으므로 반드시 제한을 건다** (썸네일 1, 미리보기 소수).
    ///   - fontDirectory: 폰트 탐색 절대경로. 코어의 기본 폰트 탐색은 작업디렉터리
    ///     상대경로라 샌드박스된 확장에서는 잡히지 않는다. 번들 Resources 경로를 넘긴다.
    ///   - embedText: `false` 면 글리프를 path 로 변환한다. 메모리를 크게 줄이는 대신
    ///     PDF 의 텍스트 선택·검색을 잃는다.
    public static func renderPDF(
        inputFile: URL,
        firstPage: UInt32 = 0,
        maxPages: Int32? = nil,
        fontDirectory: URL? = nil,
        embedText: Bool = true
    ) throws -> Data {
        let buffer: RhwpBuffer = inputFile.path.withCString { input in
            let render: (UnsafePointer<CChar>?) -> RhwpBuffer = { fontDir in
                rhwp_render_pdf(input, firstPage, maxPages ?? 0, fontDir, embedText ? 1 : 0)
            }
            if let fontDirectory {
                return fontDirectory.path.withCString { render($0) }
            }
            return render(nil)
        }

        defer { rhwp_buffer_free(buffer) }

        if let error = buffer.error {
            throw RhwpError.exportFailed(String(cString: error))
        }
        guard let data = buffer.data, buffer.len > 0 else {
            throw RhwpError.nativeReturnedNull
        }
        return Data(bytes: data, count: buffer.len)
    }

    public static func readText(
        inputFile: URL,
        page: RhwpPage = .all
    ) throws -> RhwpDocumentText {
        try callNativeText(
            inputFile: inputFile,
            page: page,
            function: rhwp_read_text
        )
    }

    public static func exportText(
        inputFile: URL,
        outputDirectory: URL,
        page: RhwpPage = .all
    ) throws -> RhwpExportResult {
        try callNative(
            inputFile: inputFile,
            outputDirectory: outputDirectory,
            page: page,
            function: rhwp_export_text
        )
    }

    public static func exportMarkdown(
        inputFile: URL,
        outputDirectory: URL,
        page: RhwpPage = .all
    ) throws -> RhwpExportResult {
        try callNative(
            inputFile: inputFile,
            outputDirectory: outputDirectory,
            page: page,
            function: rhwp_export_markdown
        )
    }

    private static func callNative(
        inputFile: URL,
        outputDirectory: URL,
        page: RhwpPage,
        function: (UnsafePointer<CChar>?, UnsafePointer<CChar>?, Int32) -> UnsafeMutablePointer<CChar>?
    ) throws -> RhwpExportResult {
        let pointer = inputFile.path.withCString { inputPath in
            outputDirectory.path.withCString { outputPath in
                function(inputPath, outputPath, page.ffiValue)
            }
        }

        guard let pointer else {
            throw RhwpError.nativeReturnedNull
        }

        defer {
            rhwp_string_free(pointer)
        }

        guard let payload = String(validatingUTF8: pointer) else {
            throw RhwpError.invalidUTF8
        }

        let data = Data(payload.utf8)
        let result: RhwpExportResult
        do {
            result = try JSONDecoder().decode(RhwpExportResult.self, from: data)
        } catch {
            throw RhwpError.invalidJSON(payload)
        }

        if result.ok {
            return result
        }

        throw RhwpError.exportFailed(result.error ?? "rhwp export failed.")
    }

    private static func callNativeText(
        inputFile: URL,
        page: RhwpPage,
        function: (UnsafePointer<CChar>?, Int32) -> UnsafeMutablePointer<CChar>?
    ) throws -> RhwpDocumentText {
        let pointer = inputFile.path.withCString { inputPath in
            function(inputPath, page.ffiValue)
        }

        guard let pointer else {
            throw RhwpError.nativeReturnedNull
        }

        defer {
            rhwp_string_free(pointer)
        }

        guard let payload = String(validatingUTF8: pointer) else {
            throw RhwpError.invalidUTF8
        }

        let data = Data(payload.utf8)
        let result: RhwpDocumentText
        do {
            result = try JSONDecoder().decode(RhwpDocumentText.self, from: data)
        } catch {
            throw RhwpError.invalidJSON(payload)
        }

        if result.ok {
            return result
        }

        throw RhwpError.exportFailed(result.error ?? "rhwp read failed.")
    }
}
