import XCTest
@testable import Rhwp

final class RhwpTests: XCTestCase {
    func testPageEncoding() {
        XCTAssertEqual(RhwpPage.all.ffiValue, -1)
        XCTAssertEqual(RhwpPage.index(0).ffiValue, 0)
        XCTAssertEqual(RhwpPage.index(7).ffiValue, 7)
    }

    func testResultOutputFiles() {
        let result = RhwpExportResult(
            ok: true,
            pageCount: 1,
            files: ["/tmp/page.txt"],
            imageCount: nil,
            error: nil
        )

        XCTAssertEqual(result.outputFiles, [URL(fileURLWithPath: "/tmp/page.txt")])
    }

    func testReadTextCallsNativeLibraryWithoutExportingTxt() throws {
        let inputFile = repoRoot().appendingPathComponent("samples/KTX.hwp")

        let document = try Rhwp.readText(inputFile: inputFile, page: .index(0))

        XCTAssertTrue(document.ok)
        XCTAssertGreaterThan(document.pageCount ?? 0, 0)
        XCTAssertEqual(document.pages?.count, 1)
        XCTAssertEqual(document.pages?.first?.index, 0)
        XCTAssertFalse(document.text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
    }

    func testExportTextCallsNativeLibrary() throws {
        let inputFile = repoRoot().appendingPathComponent("samples/KTX.hwp")
        let outputDirectory = FileManager.default.temporaryDirectory
            .appendingPathComponent("rhwp-swift-\(UUID().uuidString)")

        let result = try Rhwp.exportText(
            inputFile: inputFile,
            outputDirectory: outputDirectory,
            page: .index(0)
        )

        XCTAssertTrue(result.ok)
        XCTAssertGreaterThan(result.pageCount ?? 0, 0)
        XCTAssertEqual(result.outputFiles.count, 1)
        XCTAssertTrue(FileManager.default.fileExists(atPath: result.outputFiles[0].path))
    }

    func testWriteTextCreatesReloadableHwp() throws {
        let outputFile = FileManager.default.temporaryDirectory
            .appendingPathComponent("rhwp-swift-\(UUID().uuidString)")
            .appendingPathComponent("created.hwp")

        let result = try Rhwp.writeText("한글 English 123", outputFile: outputFile)

        XCTAssertTrue(result.ok)
        XCTAssertEqual(result.outputFile, outputFile)
        XCTAssertEqual(result.pageCount, 1)
        XCTAssertGreaterThan(result.byteCount ?? 0, 512)
        XCTAssertTrue(FileManager.default.fileExists(atPath: outputFile.path))

        let document = try Rhwp.readText(inputFile: outputFile, page: .index(0))
        XCTAssertTrue(document.text.contains("한글"))
        XCTAssertTrue(document.text.contains("English"))
        XCTAssertTrue(document.text.contains("123"))
    }

    private func repoRoot() -> URL {
        URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
    }
}
