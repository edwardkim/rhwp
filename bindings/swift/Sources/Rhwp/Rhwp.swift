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
