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

    func testExportTextCallsNativeLibrary() throws {
        let repoRoot = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
        let inputFile = repoRoot.appendingPathComponent("samples/KTX.hwp")
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
}
