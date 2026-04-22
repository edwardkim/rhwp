import CoreGraphics
import CoreText
import Darwin
import Foundation
import ImageIO

struct RenderCheckError: Error, CustomStringConvertible {
    let description: String
}

struct TextStats {
    var textRunCount = 0
    var hangulRunCount = 0
    var hangulScalarCount = 0
    var missingGlyphCount = 0
}

@main
struct Stage3RenderCheck {
    @MainActor
    static func main() throws {
        var args = Array(CommandLine.arguments.dropFirst())
        guard args.count >= 2 else {
            throw RenderCheckError(description: "usage: stage3_render_check <output-dir> <hwp-or-hwpx> [...]")
        }

        let outputDir = URL(fileURLWithPath: args.removeFirst(), isDirectory: true)
        try FileManager.default.createDirectory(at: outputDir, withIntermediateDirectories: true)

        var failed = false
        for input in args {
            do {
                let summary = try render(inputPath: input, outputDir: outputDir)
                print(summary)
            } catch {
                failed = true
                print("FAIL \(input): \(error)", to: &standardError)
            }
        }

        if failed {
            print("FAIL: one or more render checks failed", to: &standardError)
            exit(1)
        }
    }

    @MainActor
    private static func render(inputPath: String, outputDir: URL) throws -> String {
        let inputURL = URL(fileURLWithPath: inputPath)
        let data = try Data(contentsOf: inputURL)
        let doc = try RhwpDocument(data: data, filename: inputURL.lastPathComponent)
        guard doc.pageCount > 0 else {
            throw RenderCheckError(description: "page count is zero")
        }

        let pageIndex = 0
        guard let tree = doc.renderPageTree(at: pageIndex) else {
            throw RenderCheckError(description: "render tree is nil")
        }

        var stats = TextStats()
        collectTextStats(tree, into: &stats)
        guard stats.textRunCount > 0 else {
            throw RenderCheckError(description: "render tree has no text runs")
        }
        guard stats.hangulRunCount > 0, stats.hangulScalarCount > 0 else {
            throw RenderCheckError(description: "render tree has no Hangul text runs")
        }
        guard stats.missingGlyphCount == 0 else {
            throw RenderCheckError(description: "missing Hangul glyphs: \(stats.missingGlyphCount)")
        }

        let pageSize = doc.pageSize(at: pageIndex)
        guard pageSize.width > 0, pageSize.height > 0 else {
            throw RenderCheckError(description: "invalid page size \(pageSize.width)x\(pageSize.height)")
        }

        let scale = 1.0
        let pixelWidth = max(1, Int(ceil(pageSize.width * scale)))
        let pixelHeight = max(1, Int(ceil(pageSize.height * scale)))
        let bytesPerPixel = 4
        let bytesPerRow = pixelWidth * bytesPerPixel
        var pixels = [UInt8](repeating: 255, count: pixelHeight * bytesPerRow)

        let colorSpace = CGColorSpaceCreateDeviceRGB()
        let bitmapInfo = CGImageAlphaInfo.premultipliedLast.rawValue
        guard let ctx = CGContext(
            data: &pixels,
            width: pixelWidth,
            height: pixelHeight,
            bitsPerComponent: 8,
            bytesPerRow: bytesPerRow,
            space: colorSpace,
            bitmapInfo: bitmapInfo
        ) else {
            throw RenderCheckError(description: "failed to create bitmap context")
        }

        ctx.setFillColor(CGColor(gray: 1, alpha: 1))
        ctx.fill(CGRect(x: 0, y: 0, width: pixelWidth, height: pixelHeight))
        ctx.translateBy(x: 0, y: CGFloat(pixelHeight))
        ctx.scaleBy(x: CGFloat(scale), y: -CGFloat(scale))

        let renderer = CGTreeRenderer()
        renderer.render(tree: tree, in: ctx, pageHeight: pageSize.height, document: doc)

        guard let image = ctx.makeImage() else {
            throw RenderCheckError(description: "failed to create CGImage")
        }

        let basename = inputURL.deletingPathExtension().lastPathComponent
        let outputURL = outputDir.appendingPathComponent("\(basename)-page1.png")
        try writePNG(image: image, to: outputURL)

        let nonWhitePixels = countNonWhitePixels(pixels, bytesPerRow: bytesPerRow, width: pixelWidth, height: pixelHeight)
        guard nonWhitePixels > 0 else {
            throw RenderCheckError(description: "rendered bitmap is blank")
        }

        return "OK \(inputURL.lastPathComponent): page=1 size=\(pixelWidth)x\(pixelHeight) textRuns=\(stats.textRunCount) hangulRuns=\(stats.hangulRunCount) hangulScalars=\(stats.hangulScalarCount) nonWhitePixels=\(nonWhitePixels) png=\(outputURL.path)"
    }

    private static func collectTextStats(_ node: RenderNode, into stats: inout TextStats) {
        if case .textRun(let run) = node.nodeType {
            stats.textRunCount += 1
            if containsHangul(run.text) {
                stats.hangulRunCount += 1
                stats.hangulScalarCount += hangulScalarCount(run.text)
                stats.missingGlyphCount += missingHangulGlyphCount(in: run)
            }
        }

        for child in node.children {
            collectTextStats(child, into: &stats)
        }
    }

    private static func containsHangul(_ text: String) -> Bool {
        hangulScalarCount(text) > 0
    }

    private static func hangulScalarCount(_ text: String) -> Int {
        text.unicodeScalars.filter { scalar in
            (0xAC00...0xD7AF).contains(Int(scalar.value)) ||
            (0x1100...0x11FF).contains(Int(scalar.value)) ||
            (0x3130...0x318F).contains(Int(scalar.value))
        }.count
    }

    private static func missingHangulGlyphCount(in run: TextRunNode) -> Int {
        let fontName = mapHWPFontToIOS(run.style.fontFamily)
        let baseFont = CTFontCreateWithName(fontName as CFString, CGFloat(run.style.fontSize), nil)

        var missing = 0
        for scalar in run.text.unicodeScalars {
            guard (0xAC00...0xD7AF).contains(Int(scalar.value)) ||
                  (0x1100...0x11FF).contains(Int(scalar.value)) ||
                  (0x3130...0x318F).contains(Int(scalar.value)) else {
                continue
            }
            let text = String(scalar)
            let font = CTFontCreateForString(baseFont, text as CFString, CFRange(location: 0, length: text.utf16.count))
            var character = UniChar(scalar.value)
            var glyph = CGGlyph()
            if !CTFontGetGlyphsForCharacters(font, &character, &glyph, 1) || glyph == 0 {
                missing += 1
            }
        }
        return missing
    }

    private static func countNonWhitePixels(_ pixels: [UInt8], bytesPerRow: Int, width: Int, height: Int) -> Int {
        var count = 0
        for y in 0..<height {
            let row = y * bytesPerRow
            for x in 0..<width {
                let i = row + x * 4
                let r = pixels[i]
                let g = pixels[i + 1]
                let b = pixels[i + 2]
                let a = pixels[i + 3]
                if a > 0 && (r < 245 || g < 245 || b < 245) {
                    count += 1
                }
            }
        }
        return count
    }

    private static func writePNG(image: CGImage, to url: URL) throws {
        guard let destination = CGImageDestinationCreateWithURL(url as CFURL, "public.png" as CFString, 1, nil) else {
            throw RenderCheckError(description: "failed to create PNG destination: \(url.path)")
        }
        CGImageDestinationAddImage(destination, image, nil)
        guard CGImageDestinationFinalize(destination) else {
            throw RenderCheckError(description: "failed to write PNG: \(url.path)")
        }
    }
}

struct StandardError: TextOutputStream {
    mutating func write(_ string: String) {
        FileHandle.standardError.write(Data(string.utf8))
    }
}

var standardError = StandardError()
