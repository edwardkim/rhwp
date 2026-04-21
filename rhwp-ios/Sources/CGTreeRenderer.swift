// Core Graphics 렌더러 — 렌더 트리를 CGContext에 직접 그린다.
// 3a단계: 도형(rect, line, ellipse, path) + 이미지 + 표 테두리
// 3b단계: 텍스트(Core Text + 폰트 폴백) — 별도 구현 예정

import CoreGraphics
import CoreText
import Foundation
import ImageIO

@MainActor
class CGTreeRenderer {
    private var imageCache: [UInt16: CGImage] = [:]
    private weak var document: RhwpDocument?

    private var pageHeight: Double = 0

    func render(tree: RenderNode, in context: CGContext, pageHeight: Double, document: RhwpDocument?) {
        self.document = document
        self.pageHeight = pageHeight
        // 호출 측은 좌상단 원점 좌표계로 CGContext를 전달한다.
        // 렌더 트리의 좌표(좌상단 원점)를 그대로 사용할 수 있다.
        // 단, Core Text와 CGImage는 원본 CG 좌표계(좌하단)를 기대하므로
        // 해당 요소에서만 국소적으로 좌표를 조정한다.
        renderNode(tree, in: context)
    }

    func clearCache() {
        imageCache.removeAll()
    }

    // MARK: - 트리 순회

    private func renderNode(_ node: RenderNode, in ctx: CGContext) {
        guard node.visible else { return }

        switch node.nodeType {
        case .page:
            // 페이지 배경 (흰색)
            ctx.setFillColor(CGColor(gray: 1.0, alpha: 1.0))
            ctx.fill(cgRect(node.bbox))
            renderChildren(node, in: ctx)

        case .pageBackground(let bg):
            renderPageBackground(bg, bbox: node.bbox, in: ctx)

        case .body(let body):
            if let clip = body.clipRect {
                ctx.saveGState()
                ctx.clip(to: cgRect(clip))
                renderChildren(node, in: ctx)
                ctx.restoreGState()
            } else {
                renderChildren(node, in: ctx)
            }

        case .tableCell(let cell):
            if cell.clip {
                ctx.saveGState()
                ctx.clip(to: cgRect(node.bbox))
                renderChildren(node, in: ctx)
                ctx.restoreGState()
            } else {
                renderChildren(node, in: ctx)
            }

        case .rectangle(let rect):
            renderRectangle(rect, bbox: node.bbox, in: ctx)

        case .line(let line):
            renderLine(line, in: ctx)

        case .ellipse(let ell):
            renderEllipse(ell, bbox: node.bbox, in: ctx)

        case .path(let path):
            renderPath(path, bbox: node.bbox, in: ctx)

        case .image(let img):
            renderImage(img, bbox: node.bbox, in: ctx)

        case .group:
            renderGroup(node, in: ctx)

        case .textRun(let run):
            renderTextRun(run, bbox: node.bbox, in: ctx)

        case .equation:
            // M3에서 네이티브 수식 렌더링 예정
            break

        case .formObject:
            // 양식 개체는 M3 이후
            break

        case .footnoteMarker(let marker):
            renderFootnoteMarker(marker, bbox: node.bbox, in: ctx)

        default:
            // 구조 노드(header, footer, column 등): 자식만 순회
            renderChildren(node, in: ctx)
        }
    }

    private func renderChildren(_ node: RenderNode, in ctx: CGContext) {
        for child in node.children {
            renderNode(child, in: ctx)
        }
    }

    // MARK: - 사각형

    private func renderRectangle(_ rect: RectangleNode, bbox: BBox, in ctx: CGContext) {
        ctx.saveGState()
        applyTransform(rect.transform, bbox: bbox, in: ctx)

        let r = cgRect(bbox)
        let path: CGPath
        if rect.cornerRadius > 0 {
            path = CGPath(roundedRect: r, cornerWidth: CGFloat(rect.cornerRadius),
                          cornerHeight: CGFloat(rect.cornerRadius), transform: nil)
        } else {
            path = CGPath(rect: r, transform: nil)
        }

        // 그라데이션 채우기
        if let grad = rect.gradient {
            ctx.addPath(path)
            ctx.clip()
            drawGradient(grad, in: r, ctx: ctx)
        } else {
            applyShapeStyleFill(rect.style, path: path, in: ctx)
        }

        applyShapeStyleStroke(rect.style, path: path, in: ctx)
        ctx.restoreGState()
    }

    // MARK: - 직선

    private func renderLine(_ line: LineNode, in ctx: CGContext) {
        ctx.saveGState()
        let style = line.style

        ctx.setStrokeColor(colorRefToCGColor(style.color))
        ctx.setLineWidth(CGFloat(max(style.width, 0.5)))
        applyDash(style.dash, in: ctx)

        ctx.move(to: CGPoint(x: line.x1, y: line.y1))
        ctx.addLine(to: CGPoint(x: line.x2, y: line.y2))
        ctx.strokePath()

        ctx.restoreGState()
    }

    // MARK: - 타원

    private func renderEllipse(_ ell: EllipseNode, bbox: BBox, in ctx: CGContext) {
        ctx.saveGState()
        applyTransform(ell.transform, bbox: bbox, in: ctx)

        let r = cgRect(bbox)
        let path = CGPath(ellipseIn: r, transform: nil)

        if let grad = ell.gradient {
            ctx.addPath(path)
            ctx.clip()
            drawGradient(grad, in: r, ctx: ctx)
        } else {
            applyShapeStyleFill(ell.style, path: path, in: ctx)
        }

        applyShapeStyleStroke(ell.style, path: path, in: ctx)
        ctx.restoreGState()
    }

    // MARK: - 패스

    private func renderPath(_ pathNode: PathNode, bbox: BBox, in ctx: CGContext) {
        ctx.saveGState()
        applyTransform(pathNode.transform, bbox: bbox, in: ctx)

        let cgPath = buildCGPath(pathNode.commands)

        if let grad = pathNode.gradient {
            ctx.addPath(cgPath)
            ctx.clip()
            drawGradient(grad, in: cgRect(bbox), ctx: ctx)
        } else {
            applyShapeStyleFill(pathNode.style, path: cgPath, in: ctx)
        }

        // 패스 노드는 lineStyle이 있으면 그것을 사용
        if let ls = pathNode.lineStyle {
            ctx.addPath(cgPath)
            ctx.setStrokeColor(colorRefToCGColor(ls.color))
            ctx.setLineWidth(CGFloat(max(ls.width, 0.5)))
            applyDash(ls.dash, in: ctx)
            ctx.strokePath()
        } else {
            applyShapeStyleStroke(pathNode.style, path: cgPath, in: ctx)
        }

        ctx.restoreGState()
    }

    private func buildCGPath(_ commands: [PathCommand]) -> CGPath {
        let path = CGMutablePath()
        for cmd in commands {
            switch cmd {
            case .moveTo(let x, let y):
                path.move(to: CGPoint(x: x, y: y))
            case .lineTo(let x, let y):
                path.addLine(to: CGPoint(x: x, y: y))
            case .curveTo(let x1, let y1, let x2, let y2, let x, let y):
                path.addCurve(to: CGPoint(x: x, y: y),
                              control1: CGPoint(x: x1, y: y1),
                              control2: CGPoint(x: x2, y: y2))
            case .arcTo(_, _, _, _, _, let x, let y):
                path.addLine(to: CGPoint(x: x, y: y))
            case .closePath:
                path.closeSubpath()
            }
        }
        return path
    }

    // MARK: - 이미지

    private func renderImage(_ img: ImageNode, bbox: BBox, in ctx: CGContext) {
        guard img.binDataId > 0, let doc = document else { return }

        let cgImage: CGImage
        if let cached = imageCache[img.binDataId] {
            cgImage = cached
        } else {
            guard let data = doc.imageData(binDataId: img.binDataId),
                  let source = CGImageSourceCreateWithData(data as CFData, nil),
                  let cg = CGImageSourceCreateImageAtIndex(source, 0, nil) else { return }
            imageCache[img.binDataId] = cg
            cgImage = cg
        }

        ctx.saveGState()
        applyTransform(img.transform, bbox: bbox, in: ctx)

        let r = cgRect(bbox)
        // CG draw(image:) 는 이미지를 rect에 맞춰 그리지만 상하 반전으로 그린다.
        // 이미지 영역에서만 Y축 반전하여 올바르게 표시한다.
        ctx.saveGState()
        ctx.translateBy(x: r.minX, y: r.minY + r.height)
        ctx.scaleBy(x: 1, y: -1)
        ctx.draw(cgImage, in: CGRect(x: 0, y: 0, width: r.width, height: r.height))
        ctx.restoreGState()

        ctx.restoreGState()
    }

    // MARK: - 그룹

    private func renderGroup(_ node: RenderNode, in ctx: CGContext) {
        ctx.saveGState()
        // 그룹 노드의 transform은 자식에게 적용
        if case .group = node.nodeType {
            // 그룹 자체는 transform 없음 (자식 개별 적용)
        }
        renderChildren(node, in: ctx)
        ctx.restoreGState()
    }

    // MARK: - 페이지 배경

    private func renderPageBackground(_ bg: PageBackgroundNode, bbox: BBox, in ctx: CGContext) {
        let r = cgRect(bbox)
        if let gradient = bg.gradient {
            ctx.saveGState()
            ctx.clip(to: r)
            drawGradient(gradient, in: r, ctx: ctx)
            ctx.restoreGState()
        } else if let bgColor = bg.backgroundColor {
            ctx.setFillColor(colorRefToCGColor(bgColor))
            ctx.fill(r)
        }
        if let borderColor = bg.borderColor, bg.borderWidth > 0 {
            ctx.setStrokeColor(colorRefToCGColor(borderColor))
            ctx.setLineWidth(CGFloat(bg.borderWidth))
            ctx.stroke(r)
        }
    }

    // MARK: - 텍스트 (Core Text)

    private func renderTextRun(_ run: TextRunNode, bbox: BBox, in ctx: CGContext) {
        guard !run.text.isEmpty else { return }

        let style = run.style
        let fontSize = CGFloat(style.fontSize)
        guard fontSize > 0 else { return }

        ctx.saveGState()

        // 음영 (형광펜 배경) — 텍스트 변환 전에 그리기
        if style.shadeColor != 0x00FFFFFF && style.shadeColor != 0 {
            let shadeRect = cgRect(bbox)
            ctx.setFillColor(colorRefToCGColor(style.shadeColor).copy(alpha: 0.3)!)
            ctx.fill(shadeRect)
        }

        // 전체 좌표계가 Y반전(좌상단 원점) 상태이지만,
        // Core Text는 Y축이 위로 증가하는 좌표계를 기대한다.
        // bbox 영역 내에서만 Y축을 다시 반전하여 Core Text가 올바르게 그리도록 한다.
        ctx.saveGState()
        // bbox 영역의 하단으로 이동 → Y반전 → bbox 내부 좌표 (0,0)이 좌하단이 됨
        ctx.translateBy(x: CGFloat(bbox.x), y: CGFloat(bbox.y + bbox.height))
        ctx.scaleBy(x: 1, y: -1)

        // 폰트 생성 (폴백 매핑 적용)
        let iosName = mapHWPFontToIOS(style.fontFamily)
        var font = CTFontCreateWithName(iosName as CFString, fontSize, nil)

        // Bold/Italic traits
        var traits = CTFontSymbolicTraits()
        if style.bold { traits.insert(.boldTrait) }
        if style.italic { traits.insert(.italicTrait) }
        if !traits.isEmpty {
            if let traitFont = CTFontCreateCopyWithSymbolicTraits(font, fontSize, nil, traits, [.boldTrait, .italicTrait]) {
                font = traitFont
            }
        }

        // 장평(ratio) 적용: 가로 스케일링
        if style.ratio != 1.0 && style.ratio > 0 {
            var matrix = CGAffineTransform(scaleX: CGFloat(style.ratio), y: 1.0)
            font = CTFontCreateCopyWithAttributes(font, fontSize, &matrix, nil)
        }

        // 속성 구성
        var attributes: [NSAttributedString.Key: Any] = [
            coreTextFontKey: font,
            coreTextForegroundColorKey: colorRefToCGColor(style.color),
        ]

        // 자간 (letter_spacing)
        if style.letterSpacing != 0 {
            attributes[coreTextKernKey] = CGFloat(style.letterSpacing)
        }

        let attrStr = NSAttributedString(string: run.text, attributes: attributes)
        let line = CTLineCreateWithAttributedString(attrStr)

        // Core Text 좌하단 좌표계에서 베이스라인 위치
        // bbox 내부 좌표: baseline은 bbox.y 상단으로부터의 거리
        // Core Text Y: bbox 하단(0)으로부터 위로 = bbox.height - baseline
        let textY = CGFloat(bbox.height) - CGFloat(run.baseline)
        ctx.textPosition = CGPoint(x: 0, y: textY)
        CTLineDraw(line, ctx)

        ctx.restoreGState()

        // 밑줄 (페이지 좌표계, 전체 Y반전 상태)
        if style.underline != "None" {
            let ulY = CGFloat(bbox.y) + CGFloat(run.baseline) + fontSize * 0.15
            drawTextDecoration(
                in: ctx, x: CGFloat(bbox.x), y: ulY, width: CGFloat(bbox.width),
                shape: style.underlineShape,
                color: style.underlineColor != 0 ? style.underlineColor : style.color
            )
        }

        // 취소선
        if style.strikethrough {
            let stY = CGFloat(bbox.y) + CGFloat(bbox.height) / 2
            drawTextDecoration(
                in: ctx, x: CGFloat(bbox.x), y: stY, width: CGFloat(bbox.width),
                shape: style.strikeShape,
                color: style.strikeColor != 0 ? style.strikeColor : style.color
            )
        }

        ctx.restoreGState()
    }

    /// 각주/미주 마커 (위첨자)
    private func renderFootnoteMarker(_ marker: FootnoteMarkerNode, bbox: BBox, in ctx: CGContext) {
        let fontSize = CGFloat(marker.baseFontSize * 0.55) // 위첨자 55%
        guard fontSize > 0 else { return }

        ctx.saveGState()
        ctx.translateBy(x: CGFloat(bbox.x), y: CGFloat(bbox.y + bbox.height))
        ctx.scaleBy(x: 1, y: -1)

        let iosName = mapHWPFontToIOS(marker.fontFamily)
        let font = CTFontCreateWithName(iosName as CFString, fontSize, nil)
        let attributes: [NSAttributedString.Key: Any] = [
            coreTextFontKey: font,
            coreTextForegroundColorKey: colorRefToCGColor(marker.color),
        ]
        let attrStr = NSAttributedString(string: marker.text, attributes: attributes)
        let line = CTLineCreateWithAttributedString(attrStr)
        ctx.textPosition = CGPoint(x: 0, y: CGFloat(bbox.height) * 0.6)
        CTLineDraw(line, ctx)

        ctx.restoreGState()
    }

    /// 밑줄/취소선 그리기
    private func drawTextDecoration(in ctx: CGContext, x: CGFloat, y: CGFloat, width: CGFloat,
                                     shape: UInt8, color: UInt32) {
        ctx.saveGState()
        ctx.setStrokeColor(colorRefToCGColor(color))
        ctx.setLineWidth(0.5)
        ctx.move(to: CGPoint(x: x, y: y))
        ctx.addLine(to: CGPoint(x: x + width, y: y))
        ctx.strokePath()
        ctx.restoreGState()
    }

    // MARK: - 스타일 적용 헬퍼

    private func applyShapeStyleFill(_ style: ShapeStyle, path: CGPath, in ctx: CGContext) {
        if let pattern = style.pattern {
            // 패턴 채우기 (M3에서 정확한 패턴 구현)
            if let bgColor = UInt32(exactly: pattern.backgroundColor) {
                ctx.addPath(path)
                ctx.setFillColor(colorRefToCGColor(bgColor))
                ctx.fillPath()
            }
        } else if let fillColor = style.fillColor {
            ctx.addPath(path)
            ctx.setAlpha(CGFloat(style.opacity))
            ctx.setFillColor(colorRefToCGColor(fillColor))
            ctx.fillPath()
            ctx.setAlpha(1.0)
        }
    }

    private func applyShapeStyleStroke(_ style: ShapeStyle, path: CGPath, in ctx: CGContext) {
        if let strokeColor = style.strokeColor, style.strokeWidth > 0 {
            ctx.addPath(path)
            ctx.setStrokeColor(colorRefToCGColor(strokeColor))
            ctx.setLineWidth(CGFloat(max(style.strokeWidth, 0.5)))
            applyDash(style.strokeDash, in: ctx)
            ctx.strokePath()
        }
    }

    private func applyDash(_ dash: String, in ctx: CGContext) {
        switch dash {
        case "Dash":
            ctx.setLineDash(phase: 0, lengths: [6, 3])
        case "Dot":
            ctx.setLineDash(phase: 0, lengths: [2, 2])
        case "DashDot":
            ctx.setLineDash(phase: 0, lengths: [6, 3, 2, 3])
        case "DashDotDot":
            ctx.setLineDash(phase: 0, lengths: [6, 3, 2, 3, 2, 3])
        default: // Solid
            ctx.setLineDash(phase: 0, lengths: [])
        }
    }

    private func applyTransform(_ transform: ShapeTransform, bbox: BBox, in ctx: CGContext) {
        guard transform.rotation != 0 || transform.horzFlip || transform.vertFlip else { return }
        let cx = CGFloat(bbox.x + bbox.width / 2)
        let cy = CGFloat(bbox.y + bbox.height / 2)
        ctx.translateBy(x: cx, y: cy)
        if transform.rotation != 0 {
            ctx.rotate(by: CGFloat(transform.rotation * .pi / 180))
        }
        if transform.horzFlip { ctx.scaleBy(x: -1, y: 1) }
        if transform.vertFlip { ctx.scaleBy(x: 1, y: -1) }
        ctx.translateBy(x: -cx, y: -cy)
    }

    // MARK: - 그라데이션

    private func drawGradient(_ info: GradientFillInfo, in rect: CGRect, ctx: CGContext) {
        guard info.colors.count >= 2 else { return }
        let cgColors = info.colors.map { colorRefToCGColor($0) }
        guard let gradient = CGGradient(colorsSpace: CGColorSpaceCreateDeviceRGB(),
                                        colors: cgColors as CFArray,
                                        locations: info.positions.map { CGFloat($0) }) else { return }

        switch info.gradientType {
        case 1: // 선형
            let angle = CGFloat(info.angle) * .pi / 180
            let dx = cos(angle) * rect.width / 2
            let dy = sin(angle) * rect.height / 2
            let center = CGPoint(x: rect.midX, y: rect.midY)
            ctx.drawLinearGradient(gradient,
                start: CGPoint(x: center.x - dx, y: center.y - dy),
                end: CGPoint(x: center.x + dx, y: center.y + dy),
                options: [.drawsBeforeStartLocation, .drawsAfterEndLocation])
        case 2: // 원형
            let center = CGPoint(x: rect.midX, y: rect.midY)
            let radius = max(rect.width, rect.height) / 2
            ctx.drawRadialGradient(gradient,
                startCenter: center, startRadius: 0,
                endCenter: center, endRadius: radius,
                options: [.drawsBeforeStartLocation, .drawsAfterEndLocation])
        default:
            // 원뿔형/사각형 등은 선형으로 근사
            ctx.drawLinearGradient(gradient,
                start: CGPoint(x: rect.minX, y: rect.minY),
                end: CGPoint(x: rect.maxX, y: rect.maxY),
                options: [.drawsBeforeStartLocation, .drawsAfterEndLocation])
        }
    }

    // MARK: - 색상 변환

    /// HWP ColorRef (0x00BBGGRR) → CGColor
    private func colorRefToCGColor(_ ref: UInt32) -> CGColor {
        let r = CGFloat(ref & 0xFF) / 255.0
        let g = CGFloat((ref >> 8) & 0xFF) / 255.0
        let b = CGFloat((ref >> 16) & 0xFF) / 255.0
        return CGColor(red: r, green: g, blue: b, alpha: 1.0)
    }

    private var coreTextForegroundColorKey: NSAttributedString.Key {
        NSAttributedString.Key(kCTForegroundColorAttributeName as String)
    }

    private var coreTextFontKey: NSAttributedString.Key {
        NSAttributedString.Key(kCTFontAttributeName as String)
    }

    private var coreTextKernKey: NSAttributedString.Key {
        NSAttributedString.Key(kCTKernAttributeName as String)
    }

    private func cgRect(_ bbox: BBox) -> CGRect {
        CGRect(x: bbox.x, y: bbox.y, width: bbox.width, height: bbox.height)
    }
}
