import CanvasKitInit from 'canvaskit-wasm';
import type { CanvasKit, Font, Image, Paint, Shader, Surface, Typeface, TypefaceFontProvider } from 'canvaskit-wasm';
import canvaskitWasmUrl from 'canvaskit-wasm/bin/canvaskit.wasm?url';

import { fontFamilyWithFallback, resolveFont } from '@/core/font-substitution';
import type { CanvasKitRenderMode } from '@/view/render-backend';
import type {
  LayerBounds,
  LayerClipNode,
  LayerEllipseOp,
  LayerEquationLayoutBox,
  LayerFootnoteMarkerOp,
  LayerFormObjectOp,
  LayerGradient,
  LayerImageOp,
  LayerLeafNode,
  LayerLineOp,
  LayerLineStyle,
  LayerNode,
  LayerPageBackgroundOp,
  LayerPaintOp,
  LayerPathCommand,
  LayerPathOp,
  LayerPatternFill,
  LayerRectangleOp,
  LayerShapeShadow,
  LayerTabLeader,
  LayerTextRunOp,
  PageLayerTree,
} from '@/core/types';

const FONT_SANS_REGULAR_URL = new URL('../../../web/fonts/NotoSansKR-Regular.woff2', import.meta.url).href;
const FONT_SANS_BOLD_URL = new URL('../../../web/fonts/NotoSansKR-Bold.woff2', import.meta.url).href;
const FONT_SERIF_REGULAR_URL = new URL('../../../web/fonts/NotoSerifKR-Regular.woff2', import.meta.url).href;
const FONT_SERIF_BOLD_URL = new URL('../../../web/fonts/NotoSerifKR-Bold.woff2', import.meta.url).href;
const FONT_MONO_REGULAR_URL = new URL('../../../web/fonts/D2Coding-Regular.woff2', import.meta.url).href;
const FONT_HAMCHOROM_DOTUM_URL = new URL('../../../web/fonts/NotoSansKR-Regular.woff2', import.meta.url).href;
const FONT_HAMCHOROM_DOTUM_BOLD_URL = new URL('../../../web/fonts/NotoSansKR-Bold.woff2', import.meta.url).href;
const FONT_HAMCHOROM_BATANG_URL = new URL('../../../web/fonts/NotoSerifKR-Regular.woff2', import.meta.url).href;
const FONT_HAMCHOROM_BATANG_BOLD_URL = new URL('../../../web/fonts/NotoSerifKR-Bold.woff2', import.meta.url).href;

const HAMCHOROM_DOTUM_FAMILY = 'HCR Dotum';
const HAMCHOROM_BATANG_FAMILY = 'HCR Batang';
const HAMCHOROM_DOTUM_ALIASES = new Set([
  '함초롬돋움',
  '함초롱돋움',
  '한컴돋움',
  '새돋움',
  HAMCHOROM_DOTUM_FAMILY,
]);
const HAMCHOROM_BATANG_ALIASES = new Set([
  '함초롬바탕',
  '함초롱바탕',
  '한컴바탕',
  '새바탕',
  HAMCHOROM_BATANG_FAMILY,
]);

const SANS_ALIASES = [
  'Noto Sans KR',
  'Noto Sans CJK KR',
  'NanumGothic',
  '나눔고딕',
  '맑은 고딕',
  'Malgun Gothic',
  'Apple SD Gothic Neo',
  'Pretendard',
  '돋움',
  '돋움체',
  '굴림',
  '새굴림',
  'HY중고딕',
  'HY그래픽',
  'HY그래픽M',
  'HYHeadLine M',
  'HYHeadLine Medium',
  'HY헤드라인M',
  'SpoqaHanSans',
];

const SERIF_ALIASES = [
  'Noto Serif KR',
  'Noto Serif CJK KR',
  'NanumMyeongjo',
  '나눔명조',
  '바탕',
  'AppleMyungjo',
  '궁서',
  '새궁서',
  'HY신명조',
  'HY견명조',
  'Batang',
];

const MONO_ALIASES = [
  'D2Coding',
  'NanumGothicCoding',
  '나눔고딕코딩',
  '굴림체',
  'GulimChe',
  '바탕체',
  'Noto Sans Mono',
];

export class CanvasKitLayerRenderer {
  private readonly imageCache = new Map<string, Image>();
  private readonly patternImageCache = new Map<string, Image | null>();
  private readonly fontAliases = new Set<string>();

  private constructor(
    private readonly canvasKit: CanvasKit,
    private readonly fontProvider: TypefaceFontProvider,
    private readonly renderMode: CanvasKitRenderMode,
  ) {}

  static async create(renderMode: CanvasKitRenderMode = 'compat'): Promise<CanvasKitLayerRenderer> {
    const canvasKit = await CanvasKitInit({
      locateFile: (file) => file === 'canvaskit.wasm' ? canvaskitWasmUrl : file,
    });
    const fontProvider = canvasKit.TypefaceFontProvider.Make();
    const renderer = new CanvasKitLayerRenderer(canvasKit, fontProvider, renderMode);
    await renderer.registerFonts();
    return renderer;
  }

  renderPage(
    tree: PageLayerTree,
    targetCanvas: HTMLCanvasElement,
    scale: number,
  ): void {
    const surface = this.canvasKit.MakeSWCanvasSurface(targetCanvas);
    if (!surface) {
      throw new Error('CanvasKit surface 생성 실패');
    }

    try {
      const canvas = surface.getCanvas();
      canvas.clear(this.canvasKit.TRANSPARENT);
      canvas.save();
      canvas.scale(scale, scale);
      this.renderNode(canvas, tree.root);
      canvas.restore();
      surface.flush();
      this.renderFallbackOverlays(tree.root, targetCanvas, scale);
    } finally {
      surface.delete();
    }
  }

  private async registerFonts(): Promise<void> {
    const fontFiles = new Map<string, Uint8Array>();

    const loadFontFile = async (url: string): Promise<Uint8Array> => {
      const cached = fontFiles.get(url);
      if (cached) return cached;
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`CanvasKit font fetch failed: ${response.status} ${url}`);
      }
      const bytes = new Uint8Array(await response.arrayBuffer());
      fontFiles.set(url, bytes);
      return bytes;
    };

    const registerAliases = async (aliases: string[], regularUrl: string, boldUrl?: string): Promise<void> => {
      const regularBytes = await loadFontFile(regularUrl);
      const boldBytes = boldUrl ? await loadFontFile(boldUrl) : null;

      for (const alias of aliases) {
        this.fontProvider.registerFont(regularBytes, alias);
        this.fontAliases.add(alias);
        if (boldBytes) {
          this.fontProvider.registerFont(boldBytes, alias);
        }
      }
    };

    await registerAliases([HAMCHOROM_DOTUM_FAMILY], FONT_HAMCHOROM_DOTUM_URL, FONT_HAMCHOROM_DOTUM_BOLD_URL);
    await registerAliases([HAMCHOROM_BATANG_FAMILY], FONT_HAMCHOROM_BATANG_URL, FONT_HAMCHOROM_BATANG_BOLD_URL);
    await registerAliases(SANS_ALIASES, FONT_SANS_REGULAR_URL, FONT_SANS_BOLD_URL);
    await registerAliases(SERIF_ALIASES, FONT_SERIF_REGULAR_URL, FONT_SERIF_BOLD_URL);
    await registerAliases(MONO_ALIASES, FONT_MONO_REGULAR_URL);
  }

  private renderNode(
    canvas: ReturnType<Surface['getCanvas']>,
    node: LayerNode,
  ): void {
    switch (node.kind) {
      case 'group':
        for (const child of node.children) {
          this.renderNode(canvas, child);
        }
        break;
      case 'clipRect':
        this.renderClipNode(canvas, node);
        break;
      case 'leaf':
        this.renderLeafNode(canvas, node);
        break;
    }
  }

  private renderClipNode(
    canvas: ReturnType<Surface['getCanvas']>,
    node: LayerClipNode,
  ): void {
    canvas.save();
    canvas.clipRect(
      this.canvasKit.XYWHRect(node.clip.x, node.clip.y, node.clip.width, node.clip.height),
      this.canvasKit.ClipOp.Intersect,
      true,
    );
    this.renderNode(canvas, node.child);
    canvas.restore();
  }

  private renderLeafNode(
    canvas: ReturnType<Surface['getCanvas']>,
    node: LayerLeafNode,
  ): void {
    for (const op of node.ops) {
      this.renderOp(canvas, op);
    }
  }

  private renderOp(
    canvas: ReturnType<Surface['getCanvas']>,
    op: LayerPaintOp,
  ): void {
    switch (op.type) {
      case 'pageBackground':
        this.renderPageBackground(canvas, op);
        return;
      case 'textRun':
        if (this.shouldOverlayTextRun(op)) {
          return;
        }
        this.renderTextRun(canvas, op);
        return;
      case 'footnoteMarker':
        if (this.shouldOverlayFootnoteMarker(op)) {
          return;
        }
        this.renderFootnoteMarker(canvas, op);
        return;
      case 'line':
        this.renderLine(canvas, op);
        return;
      case 'rectangle':
        this.renderRectangle(canvas, op);
        return;
      case 'ellipse':
        this.renderEllipse(canvas, op);
        return;
      case 'path':
        this.renderPath(canvas, op);
        return;
      case 'image':
        this.renderImage(canvas, op);
        return;
      case 'equation':
        return;
      case 'formObject':
        this.renderFormObject(canvas, op);
        return;
    }
  }

  private shouldOverlayTextRun(op: LayerTextRunOp): boolean {
    return true;
  }

  private shouldOverlayFootnoteMarker(op: LayerFootnoteMarkerOp): boolean {
    return true;
  }

  private renderPageBackground(canvas: ReturnType<Surface['getCanvas']>, op: LayerPageBackgroundOp): void {
    const fill = this.makeShapeFillPaint(op.bbox, op.backgroundColor ?? null, 1, op.gradient);
    if (fill) {
      canvas.drawRect(this.toRect(op.bbox), fill.paint);
      fill.shader?.delete();
      fill.paint.delete();
    }

    if (op.image?.base64) {
      this.drawEncodedImage(canvas, op.image.base64, op.bbox, op.image.fillMode);
    }

    if (op.borderColor && op.borderWidth > 0) {
      const paint = this.makePaint(op.borderColor, 'stroke');
      paint.setStrokeWidth(op.borderWidth);
      canvas.drawRect(this.toRect(op.bbox), paint);
      paint.delete();
    }
  }

  private renderTextRun(canvas: ReturnType<Surface['getCanvas']>, op: LayerTextRunOp): void {
    const ratio = typeof op.style.ratio === 'number' && op.style.ratio > 0 ? op.style.ratio : 1;
    const outlineType = op.style.outlineType ?? 0;
    const shadowType = op.style.shadowType ?? 0;
    const shadowColor = typeof op.style.shadowColor === 'string' ? op.style.shadowColor : op.style.color;
    const shadowOffsetX = typeof op.style.shadowOffsetX === 'number' ? op.style.shadowOffsetX : 0;
    const shadowOffsetY = typeof op.style.shadowOffsetY === 'number' ? op.style.shadowOffsetY : 0;
    const emboss = !!op.style.emboss;
    const engrave = !!op.style.engrave;
    const emphasisDot = op.style.emphasisDot ?? 0;
    const shadeColor = (typeof op.style.shadeColor === 'string' ? op.style.shadeColor : '#ffffff').toLowerCase();
    const primaryObjects = this.makeTextObjects(
      op.style.fontFamily,
      op.style.fontSize,
      op.style.bold,
      op.style.italic,
      op.style.color,
      ratio,
    );
    const clusters = splitIntoClusters(op.text);
    const textObjectsByFamily = new Map<string, { typeface: Typeface; font: Font; paint: Paint }>();
    textObjectsByFamily.set(op.style.fontFamily, primaryObjects);
    const fallbackFamilies = [
      op.style.fontFamily,
      'Noto Sans KR',
      'Noto Sans CJK KR',
      'NanumGothic',
      'D2Coding',
      'NanumGothicCoding',
      'Noto Serif KR',
      'Noto Serif CJK KR',
    ].filter((family, index, all) => all.indexOf(family) === index);
    const clusterFonts: Font[] = [];
    for (const cluster of clusters) {
      let selectedFont = primaryObjects.font;
      const primaryGlyphs = primaryObjects.font.getGlyphIDs(cluster.text);
      if (primaryGlyphs?.some((glyphId) => glyphId === 0)) {
        for (const family of fallbackFamilies) {
          let candidate = textObjectsByFamily.get(family);
          if (!candidate) {
            candidate = this.makeTextObjects(
              family,
              op.style.fontSize,
              op.style.bold,
              op.style.italic,
              op.style.color,
              ratio,
            );
            textObjectsByFamily.set(family, candidate);
          }
          const candidateGlyphs = candidate.font.getGlyphIDs(cluster.text);
          if (candidateGlyphs && candidateGlyphs.every((glyphId) => glyphId !== 0)) {
            selectedFont = candidate.font;
            break;
          }
        }
      }
      clusterFonts.push(selectedFont);
    }
    const drawClusters = (originX: number, originY: number) => {
      const textWidth = op.positions.at(-1) ?? 0;
      if (textWidth > 0 && shadeColor !== '#ffffff') {
        const shadePaint = this.makePaint(shadeColor, 'fill');
        canvas.drawRect(
          this.canvasKit.XYWHRect(originX, originY - op.style.fontSize, textWidth, op.style.fontSize * 1.2),
          shadePaint,
        );
        shadePaint.delete();
      }

      const drawPass = (dx: number, dy: number, fillPaint: Paint, strokePaint?: Paint) => {
        for (const [index, cluster] of clusters.entries()) {
          if (cluster.text === ' ' || cluster.text === '\t' || cluster.text === '\u2007') {
            continue;
          }
          const x = originX + op.positions[cluster.start] + dx;
          const y = originY + dy;
          canvas.drawText(cluster.text, x, y, fillPaint, clusterFonts[index]);
          if (strokePaint) {
            canvas.drawText(cluster.text, x, y, strokePaint, clusterFonts[index]);
          }
        }
      };

      if (emboss || engrave) {
        const offset = Math.max(op.style.fontSize / 20, 1);
        const firstPaint = this.makePaint(emboss ? '#ffffff' : '#808080', 'fill');
        const secondPaint = this.makePaint(emboss ? '#808080' : '#ffffff', 'fill');
        drawPass(-offset, -offset, firstPaint);
        drawPass(offset, offset, secondPaint);
        drawPass(0, 0, primaryObjects.paint);
        firstPaint.delete();
        secondPaint.delete();
      } else {
        if (shadowType > 0) {
          const shadowPaint = this.makePaint(shadowColor, 'fill');
          drawPass(shadowOffsetX, shadowOffsetY, shadowPaint);
          shadowPaint.delete();
        }

        if (outlineType > 0) {
          const fillPaint = this.makePaint('#ffffff', 'fill');
          const strokePaint = this.makePaint(op.style.color, 'stroke');
          strokePaint.setStrokeWidth(Math.max(op.style.fontSize / 25, 0.5));
          drawPass(0, 0, fillPaint, strokePaint);
          fillPaint.delete();
          strokePaint.delete();
        } else {
          drawPass(0, 0, primaryObjects.paint);
        }
      }

      if (emphasisDot > 0) {
        const dotChar =
          emphasisDot === 1 ? '●'
            : emphasisDot === 2 ? '○'
              : emphasisDot === 3 ? 'ˇ'
                : emphasisDot === 4 ? '˜'
                  : emphasisDot === 5 ? '･'
                    : emphasisDot === 6 ? '˸'
                      : '';
        if (dotChar) {
          const dotSize = op.style.fontSize * 0.3;
          const dotY = originY - op.style.fontSize * 1.05;
          const dotObjects = this.makeTextObjects('Noto Sans KR', dotSize, false, false, op.style.color);
          for (const position of op.positions.slice(0, -1)) {
            const dotX = originX + position + (op.style.fontSize * ratio * 0.5);
            canvas.drawText(dotChar, dotX, dotY, dotObjects.paint, dotObjects.font);
          }
          dotObjects.paint.delete();
          dotObjects.font.delete();
          dotObjects.typeface.delete();
        }
      }

      if (op.tabLeaders?.length) {
        this.drawTabLeaders(canvas, op.tabLeaders, originX, originY, op.style.color);
      }

      if (op.style.underline !== 'none') {
        const underlinePaint = this.makePaint(op.style.underlineColor || op.style.color, 'stroke');
        underlinePaint.setStrokeWidth(1);
        const y = op.style.underline === 'top' ? originY - op.style.fontSize + 1 : originY + 2;
        canvas.drawLine(originX, y, originX + textWidth, y, underlinePaint);
        underlinePaint.delete();
      }
      if (op.style.strikethrough) {
        const strikePaint = this.makePaint(op.style.strikeColor || op.style.color, 'stroke');
        strikePaint.setStrokeWidth(1);
        const y = originY - op.style.fontSize * 0.3;
        canvas.drawLine(originX, y, originX + textWidth, y, strikePaint);
        strikePaint.delete();
      }
    };

    if (op.rotation !== 0) {
      const cx = op.bbox.x + op.bbox.width / 2;
      const cy = op.bbox.y + op.bbox.height / 2;
      canvas.save();
      canvas.translate(cx, cy);
      canvas.rotate(op.rotation, 0, 0);
      drawClusters(-op.bbox.width / 2, -op.bbox.height / 2 + op.baseline);
      canvas.restore();
    } else {
      drawClusters(op.bbox.x, op.bbox.y + op.baseline);
    }

    for (const { paint, font, typeface } of textObjectsByFamily.values()) {
      paint.delete();
      font.delete();
      typeface.delete();
    }
  }

  private renderFootnoteMarker(canvas: ReturnType<Surface['getCanvas']>, op: Extract<LayerPaintOp, { type: 'footnoteMarker' }>): void {
    const { font, paint, typeface } = this.makeTextObjects(op.fontFamily, op.fontSize, false, false, op.color);
    canvas.drawText(op.text, op.bbox.x, op.bbox.y + op.bbox.height * 0.4, paint, font);
    paint.delete();
    font.delete();
    typeface.delete();
  }

  private renderLine(canvas: ReturnType<Surface['getCanvas']>, op: LayerLineOp): void {
    this.withTransform(canvas, op.bbox, op.transform, () => {
      const width = Math.max(op.style.width, 0.5);
      const dx = op.x2 - op.x1;
      const dy = op.y2 - op.y1;
      const lineLength = Math.hypot(dx, dy);
      let lineX1 = op.x1;
      let lineY1 = op.y1;
      let lineX2 = op.x2;
      let lineY2 = op.y2;

      if (lineLength > 0) {
        const unitX = dx / lineLength;
        const unitY = dy / lineLength;
        if (op.style.startArrow !== 'none') {
          const [arrowWidth, arrowHeight] = calculateArrowDimensions(width, lineLength, op.style.startArrowSize);
          drawArrowHead(
            this.canvasKit,
            canvas,
            op.x1,
            op.y1,
            -unitX,
            -unitY,
            arrowWidth,
            arrowHeight,
            op.style.startArrow,
            op.style.color,
            width,
          );
          lineX1 += unitX * arrowWidth;
          lineY1 += unitY * arrowWidth;
        }
        if (op.style.endArrow !== 'none') {
          const [arrowWidth, arrowHeight] = calculateArrowDimensions(width, lineLength, op.style.endArrowSize);
          drawArrowHead(
            this.canvasKit,
            canvas,
            op.x2,
            op.y2,
            unitX,
            unitY,
            arrowWidth,
            arrowHeight,
            op.style.endArrow,
            op.style.color,
            width,
          );
          lineX2 -= unitX * arrowWidth;
          lineY2 -= unitY * arrowWidth;
        }
      }

      const drawSegment = (strokeWidth: number, offsetRatio: number) => {
        const paint = this.makeLinePaint(op.style.color, strokeWidth, op.style.dash);
        if (strokeWidth < 0.5) {
          paint.setStrokeWidth(strokeWidth);
        }
        let offsetX = 0;
        let offsetY = 0;
        if (lineLength > 0 && offsetRatio !== 0) {
          const normalX = -dy / lineLength;
          const normalY = dx / lineLength;
          offsetX = normalX * width * offsetRatio;
          offsetY = normalY * width * offsetRatio;
        }
        if (op.style.shadow) {
          this.drawShadow(
            canvas,
            op.style.shadow,
            'stroke',
            op.style.shadow.color,
            strokeWidth,
            (shadowPaint) => canvas.drawLine(lineX1 + offsetX, lineY1 + offsetY, lineX2 + offsetX, lineY2 + offsetY, shadowPaint),
          );
        }
        canvas.drawLine(lineX1 + offsetX, lineY1 + offsetY, lineX2 + offsetX, lineY2 + offsetY, paint);
        paint.delete();
      };

      switch (op.style.lineType) {
        case 'double':
          drawSegment(width * 0.3, -0.35);
          drawSegment(width * 0.3, 0.35);
          break;
        case 'thickThinDouble':
          drawSegment(width * 0.4, -0.30);
          drawSegment(width * 0.2, 0.40);
          break;
        case 'thinThickDouble':
          drawSegment(width * 0.2, -0.40);
          drawSegment(width * 0.4, 0.30);
          break;
        case 'thinThickThinTriple':
          drawSegment(width * 0.15, -0.425);
          drawSegment(width * 0.30, 0);
          drawSegment(width * 0.15, 0.425);
          break;
        default:
          drawSegment(width, 0);
      }
    });
  }

  private renderRectangle(canvas: ReturnType<Surface['getCanvas']>, op: LayerRectangleOp): void {
    this.withTransform(canvas, op.bbox, op.transform, () => {
      const fill = this.makeShapeFillPaint(op.bbox, op.style.fillColor, op.style.opacity, op.gradient, op.style.pattern);
      const strokePaint = op.style.strokeColor ? this.makeLinePaint(op.style.strokeColor, op.style.strokeWidth, op.style.strokeDash, op.style.opacity) : null;
      const rect = this.toRect(op.bbox);
      const drawRect = (paint: Paint) => {
        if (op.cornerRadius > 0) {
          canvas.drawRRect(this.canvasKit.RRectXY(rect, op.cornerRadius, op.cornerRadius), paint);
          return;
        }
        canvas.drawRect(rect, paint);
      };

      if (op.style.shadow) {
        this.drawShadow(
          canvas,
          op.style.shadow,
          fill ? 'fill' : 'stroke',
          op.style.shadow.color,
          op.style.strokeWidth,
          drawRect,
        );
      }

      if (fill) {
        drawRect(fill.paint);
        fill.shader?.delete();
        fill.paint.delete();
      }
      if (strokePaint) {
        drawRect(strokePaint);
        strokePaint.delete();
      }
    });
  }

  private renderEllipse(canvas: ReturnType<Surface['getCanvas']>, op: LayerEllipseOp): void {
    this.withTransform(canvas, op.bbox, op.transform, () => {
      const fill = this.makeShapeFillPaint(op.bbox, op.style.fillColor, op.style.opacity, op.gradient, op.style.pattern);
      const strokePaint = op.style.strokeColor ? this.makeLinePaint(op.style.strokeColor, op.style.strokeWidth, op.style.strokeDash, op.style.opacity) : null;
      const oval = this.toRect(op.bbox);
      const drawOval = (paint: Paint) => canvas.drawOval(oval, paint);

      if (op.style.shadow) {
        this.drawShadow(
          canvas,
          op.style.shadow,
          fill ? 'fill' : 'stroke',
          op.style.shadow.color,
          op.style.strokeWidth,
          drawOval,
        );
      }

      if (fill) {
        drawOval(fill.paint);
        fill.shader?.delete();
        fill.paint.delete();
      }
      if (strokePaint) {
        drawOval(strokePaint);
        strokePaint.delete();
      }
    });
  }

  private renderPath(canvas: ReturnType<Surface['getCanvas']>, op: LayerPathOp): void {
    this.withTransform(canvas, op.bbox, op.transform, () => {
      const path = this.makePath(op.commands);
      const pathBounds = computePathPaintBounds(op.commands, op.bbox);
      const fill = this.makeShapeFillPaint(pathBounds, op.style.fillColor, op.style.opacity, op.gradient, op.style.pattern);
      const strokePaint = op.style.strokeColor ? this.makeLinePaint(op.style.strokeColor, op.style.strokeWidth, op.style.strokeDash, op.style.opacity) : null;
      const drawPath = (paint: Paint) => canvas.drawPath(path, paint);

      if (op.style.shadow) {
        this.drawShadow(
          canvas,
          op.style.shadow,
          fill ? 'fill' : 'stroke',
          op.style.shadow.color,
          op.style.strokeWidth,
          drawPath,
        );
      }

      if (fill) {
        drawPath(fill.paint);
        fill.shader?.delete();
        fill.paint.delete();
      }
      if (strokePaint) {
        drawPath(strokePaint);
        strokePaint.delete();
      }
      if (op.lineStyle && op.connectorEndpoints) {
        const { x1, y1, x2, y2 } = op.connectorEndpoints;
        const connectorLength = Math.max(Math.hypot(x2 - x1, y2 - y1), 1);

        if (op.lineStyle.startArrow !== 'none') {
          let directionX = x1 - x2;
          let directionY = y1 - y2;
          for (const command of op.commands.slice(1)) {
            if (command.type === 'lineTo') {
              if (Math.abs(x1 - command.x) > 0.5 || Math.abs(y1 - command.y) > 0.5) {
                directionX = x1 - command.x;
                directionY = y1 - command.y;
                break;
              }
              continue;
            }
            if (command.type === 'curveTo') {
              if (Math.abs(x1 - command.x1) > 0.5 || Math.abs(y1 - command.y1) > 0.5) {
                directionX = x1 - command.x1;
                directionY = y1 - command.y1;
                break;
              }
            }
          }
          const directionLength = Math.max(Math.hypot(directionX, directionY), 0.001);
          const [arrowWidth, arrowHeight] = calculateArrowDimensions(op.lineStyle.width, connectorLength, op.lineStyle.startArrowSize);
          drawArrowHead(
            this.canvasKit,
            canvas,
            x1,
            y1,
            directionX / directionLength,
            directionY / directionLength,
            arrowWidth,
            arrowHeight,
            op.lineStyle.startArrow,
            op.lineStyle.color,
            op.lineStyle.width,
          );
        }

        if (op.lineStyle.endArrow !== 'none') {
          const points: Array<[number, number]> = [];
          for (const command of op.commands) {
            if (command.type === 'moveTo' || command.type === 'lineTo') {
              points.push([command.x, command.y]);
              continue;
            }
            if (command.type === 'curveTo') {
              points.push([command.x2, command.y2]);
              points.push([command.x3, command.y3]);
            }
          }
          let directionX = x2 - x1;
          let directionY = y2 - y1;
          for (let index = points.length - 1; index >= 0; index -= 1) {
            const [pointX, pointY] = points[index];
            const candidateX = x2 - pointX;
            const candidateY = y2 - pointY;
            if (Math.abs(candidateX) > 0.5 || Math.abs(candidateY) > 0.5) {
              directionX = candidateX;
              directionY = candidateY;
              break;
            }
          }
          const directionLength = Math.max(Math.hypot(directionX, directionY), 0.001);
          const [arrowWidth, arrowHeight] = calculateArrowDimensions(op.lineStyle.width, connectorLength, op.lineStyle.endArrowSize);
          drawArrowHead(
            this.canvasKit,
            canvas,
            x2,
            y2,
            directionX / directionLength,
            directionY / directionLength,
            arrowWidth,
            arrowHeight,
            op.lineStyle.endArrow,
            op.lineStyle.color,
            op.lineStyle.width,
          );
        }
      }
      path.delete();
    });
  }

  private renderImage(canvas: ReturnType<Surface['getCanvas']>, op: LayerImageOp): void {
    this.withTransform(canvas, op.bbox, op.transform, () => {
      if (!op.base64) return;
      this.drawEncodedImage(canvas, op.base64, op.bbox, op.fillMode, op.originalSize, op.crop);
    });
  }

  private renderFormObject(
    canvas: ReturnType<Surface['getCanvas']>,
    op: LayerFormObjectOp,
  ): void {
    const { x, y, width: w, height: h } = op.bbox;

    switch (op.formType) {
      case 'pushButton': {
        const fillPaint = this.makePaint('#d0d0d0', 'fill');
        const strokePaint = this.makeLinePaint('#a0a0a0', 0.5, 'solid');
        canvas.drawRect(this.toRect(op.bbox), fillPaint);
        canvas.drawRect(this.toRect(op.bbox), strokePaint);
        fillPaint.delete();
        strokePaint.delete();

        if (op.caption) {
          const fontSize = Math.min(Math.max(h * 0.55, 7), 12);
          const family = this.resolveCanvasKitFontFamily('Noto Sans KR');
          const { font, paint, typeface } = this.makeTextObjects(family, fontSize, false, false, '#808080');
          const cssFont = `${fontSize}px "${family}"`;
          const textWidth = (globalThis as any).measureTextWidth?.(cssFont, op.caption) ?? op.caption.length * fontSize * 0.55;
          canvas.drawText(op.caption, x + w / 2 - textWidth / 2, y + h / 2 + fontSize * 0.35, paint, font);
          paint.delete();
          font.delete();
          typeface.delete();
        }
        return;
      }
      case 'checkBox': {
        const boxSize = Math.min(h * 0.7, 13);
        const boxY = y + (h - boxSize) / 2;
        const boxX = x + 2;
        const fillPaint = this.makePaint('#ffffff', 'fill');
        const strokePaint = this.makeLinePaint('#606060', 0.8, 'solid');
        canvas.drawRect(this.canvasKit.XYWHRect(boxX, boxY, boxSize, boxSize), fillPaint);
        canvas.drawRect(this.canvasKit.XYWHRect(boxX, boxY, boxSize, boxSize), strokePaint);
        fillPaint.delete();
        strokePaint.delete();

        if (op.value !== 0) {
          const path = new this.canvasKit.PathBuilder();
          path.moveTo(boxX + boxSize * 0.2, boxY + boxSize * 0.55);
          path.lineTo(boxX + boxSize * 0.45, boxY + boxSize * 0.8);
          path.lineTo(boxX + boxSize * 0.85, boxY + boxSize * 0.2);
          const markPaint = this.makeLinePaint('#000000', 1.5, 'solid');
          const checkPath = path.detach();
          canvas.drawPath(checkPath, markPaint);
          markPaint.delete();
          checkPath.delete();
          path.delete();
        }

        if (op.caption) {
          const fontSize = Math.min(Math.max(h * 0.55, 7), 12);
          const { font, paint, typeface } = this.makeTextObjects('Noto Sans KR', fontSize, false, false, op.foreColor);
          canvas.drawText(op.caption, boxX + boxSize + 3, y + h / 2 + fontSize * 0.35, paint, font);
          paint.delete();
          font.delete();
          typeface.delete();
        }
        return;
      }
      case 'radioButton': {
        const r = Math.min(h * 0.3, 6.5);
        const cx = x + 2 + r;
        const cy = y + h / 2;
        const fillPaint = this.makePaint('#ffffff', 'fill');
        const strokePaint = this.makeLinePaint('#606060', 0.8, 'solid');
        canvas.drawCircle(cx, cy, r, fillPaint);
        canvas.drawCircle(cx, cy, r, strokePaint);
        fillPaint.delete();
        strokePaint.delete();

        if (op.value !== 0) {
          const dotPaint = this.makePaint('#000000', 'fill');
          canvas.drawCircle(cx, cy, r * 0.5, dotPaint);
          dotPaint.delete();
        }

        if (op.caption) {
          const fontSize = Math.min(Math.max(h * 0.55, 7), 12);
          const { font, paint, typeface } = this.makeTextObjects('Noto Sans KR', fontSize, false, false, op.foreColor);
          canvas.drawText(op.caption, cx + r + 3, y + h / 2 + fontSize * 0.35, paint, font);
          paint.delete();
          font.delete();
          typeface.delete();
        }
        return;
      }
      case 'comboBox': {
        const btnW = Math.min(h * 0.8, 16);
        const fillPaint = this.makePaint('#ffffff', 'fill');
        const strokePaint = this.makeLinePaint('#a0a0a0', 0.8, 'solid');
        canvas.drawRect(this.toRect(op.bbox), fillPaint);
        canvas.drawRect(this.toRect(op.bbox), strokePaint);
        fillPaint.delete();
        strokePaint.delete();

        const buttonRect = this.canvasKit.XYWHRect(x + w - btnW, y, btnW, h);
        const buttonFill = this.makePaint('#e0e0e0', 'fill');
        const buttonStroke = this.makeLinePaint('#a0a0a0', 0.5, 'solid');
        canvas.drawRect(buttonRect, buttonFill);
        canvas.drawRect(buttonRect, buttonStroke);
        buttonFill.delete();
        buttonStroke.delete();

        const arrowCx = x + w - btnW / 2;
        const arrowCy = y + h / 2;
        const arrowSize = Math.min(h * 0.2, 4);
        const arrowPath = new this.canvasKit.PathBuilder();
        arrowPath.moveTo(arrowCx - arrowSize, arrowCy - arrowSize * 0.5);
        arrowPath.lineTo(arrowCx + arrowSize, arrowCy - arrowSize * 0.5);
        arrowPath.lineTo(arrowCx, arrowCy + arrowSize * 0.5);
        arrowPath.close();
        const arrowPaint = this.makePaint('#404040', 'fill');
        const arrowShape = arrowPath.detach();
        canvas.drawPath(arrowShape, arrowPaint);
        arrowPaint.delete();
        arrowShape.delete();
        arrowPath.delete();

        if (op.text) {
          const fontSize = Math.min(Math.max(h * 0.55, 7), 12);
          const { font, paint, typeface } = this.makeTextObjects('Noto Sans KR', fontSize, false, false, op.foreColor);
          canvas.drawText(op.text, x + 3, y + h / 2 + fontSize * 0.35, paint, font);
          paint.delete();
          font.delete();
          typeface.delete();
        }
        return;
      }
      case 'edit': {
        const fillPaint = this.makePaint('#ffffff', 'fill');
        const strokePaint = this.makeLinePaint('#a0a0a0', 0.8, 'solid');
        canvas.drawRect(this.toRect(op.bbox), fillPaint);
        canvas.drawRect(this.toRect(op.bbox), strokePaint);
        fillPaint.delete();
        strokePaint.delete();

        if (op.text) {
          const fontSize = Math.min(Math.max(h * 0.55, 7), 12);
          const { font, paint, typeface } = this.makeTextObjects('Noto Sans KR', fontSize, false, false, op.foreColor);
          canvas.drawText(op.text, x + 3, y + h / 2 + fontSize * 0.35, paint, font);
          paint.delete();
          font.delete();
          typeface.delete();
        }
      }
    }
  }

  private renderFallbackOverlays(node: LayerNode, targetCanvas: HTMLCanvasElement, scale: number): void {
    const ctx = targetCanvas.getContext('2d');
    if (!ctx) {
      return;
    }
    ctx.save();
    ctx.setTransform(scale, 0, 0, scale, 0, 0);
    this.renderFallbackOverlayNode(ctx, node);
    ctx.restore();
  }

  private renderFallbackOverlayNode(ctx: CanvasRenderingContext2D, node: LayerNode): void {
    if (node.kind === 'group') {
      for (const child of node.children) {
        this.renderFallbackOverlayNode(ctx, child);
      }
      return;
    }
    if (node.kind === 'clipRect') {
      ctx.save();
      ctx.beginPath();
      ctx.rect(node.clip.x, node.clip.y, node.clip.width, node.clip.height);
      ctx.clip();
      this.renderFallbackOverlayNode(ctx, node.child);
      ctx.restore();
      return;
    }
    for (const op of node.ops) {
      if (op.type === 'equation') {
        renderEquationLayoutBox(ctx, op.layoutBox, op.bbox.x, op.bbox.y, op.color, op.fontSize, false, false);
        continue;
      }
      if (op.type === 'textRun' && this.shouldOverlayTextRun(op)) {
        this.renderTextRunOverlay(ctx, op);
        continue;
      }
      if (op.type === 'footnoteMarker' && this.shouldOverlayFootnoteMarker(op)) {
        this.renderFootnoteMarkerOverlay(ctx, op);
      }
    }
  }

  private renderTextRunOverlay(ctx: CanvasRenderingContext2D, op: LayerTextRunOp): void {
    const ratio = typeof op.style.ratio === 'number' && op.style.ratio > 0 ? op.style.ratio : 1;
    const hasRatio = Math.abs(ratio - 1) > 0.01;
    const outlineType = op.style.outlineType ?? 0;
    const shadowType = op.style.shadowType ?? 0;
    const shadowColor = typeof op.style.shadowColor === 'string' ? op.style.shadowColor : op.style.color;
    const shadowOffsetX = typeof op.style.shadowOffsetX === 'number' ? op.style.shadowOffsetX : 0;
    const shadowOffsetY = typeof op.style.shadowOffsetY === 'number' ? op.style.shadowOffsetY : 0;
    const emboss = !!op.style.emboss;
    const engrave = !!op.style.engrave;
    const emphasisDot = op.style.emphasisDot ?? 0;
    const shadeColor = (typeof op.style.shadeColor === 'string' ? op.style.shadeColor : '#ffffff').toLowerCase();
    const fontSize = op.style.fontSize || 12;
    const clusters = splitIntoClusters(op.text);
    const drawClusters = (originX: number, originY: number) => {
      const textWidth = op.positions.at(-1) ?? 0;
      if (textWidth > 0 && shadeColor !== '#ffffff') {
        ctx.save();
        ctx.fillStyle = shadeColor;
        ctx.fillRect(originX, originY - fontSize, textWidth, fontSize * 1.2);
        ctx.restore();
      }

      const drawPass = (dx: number, dy: number, fillColor: string, strokeColor?: string, lineWidth = 0) => {
        ctx.save();
        ctx.fillStyle = fillColor;
        if (strokeColor) {
          ctx.strokeStyle = strokeColor;
          ctx.lineWidth = lineWidth;
          ctx.lineJoin = 'round';
        }
        for (const cluster of clusters) {
          if (cluster.text === ' ' || cluster.text === '\t' || cluster.text === '\u2007') {
            continue;
          }
          if (startsWithInvalidControl(cluster.text)) {
            continue;
          }
          const x = originX + op.positions[cluster.start] + dx;
          const y = originY + dy;
          if (isHalfwidthScaledCluster(cluster.text) && !hasRatio) {
            ctx.save();
            ctx.translate(x, y);
            ctx.scale(0.5, 1);
            ctx.fillText(cluster.text, 0, 0);
            if (strokeColor) {
              ctx.strokeText(cluster.text, 0, 0);
            }
            ctx.restore();
            continue;
          }
          if (hasRatio) {
            ctx.save();
            ctx.translate(x, y);
            ctx.scale(ratio, 1);
            ctx.fillText(cluster.text, 0, 0);
            if (strokeColor) {
              ctx.strokeText(cluster.text, 0, 0);
            }
            ctx.restore();
            continue;
          }
          ctx.fillText(cluster.text, x, y);
          if (strokeColor) {
            ctx.strokeText(cluster.text, x, y);
          }
        }
        ctx.restore();
      };

      if (emboss || engrave) {
        const offset = Math.max(fontSize / 20, 1);
        drawPass(-offset, -offset, emboss ? '#ffffff' : '#808080');
        drawPass(offset, offset, emboss ? '#808080' : '#ffffff');
        drawPass(0, 0, op.style.color);
      } else {
        if (shadowType > 0) {
          drawPass(shadowOffsetX, shadowOffsetY, shadowColor);
        }
        if (outlineType > 0) {
          drawPass(0, 0, '#ffffff', op.style.color, Math.max(fontSize / 25, 0.5));
        } else {
          drawPass(0, 0, op.style.color);
        }
      }

      if (emphasisDot > 0) {
        const dotChar =
          emphasisDot === 1 ? '●'
            : emphasisDot === 2 ? '○'
              : emphasisDot === 3 ? 'ˇ'
                : emphasisDot === 4 ? '˜'
                  : emphasisDot === 5 ? '･'
                    : emphasisDot === 6 ? '˸'
                      : '';
        if (dotChar) {
          ctx.save();
          this.setCanvasTextFont(ctx, 'Noto Sans KR', fontSize * 0.3, false, false);
          ctx.fillStyle = op.style.color;
          const dotY = originY - fontSize * 1.05;
          for (const position of op.positions.slice(0, -1)) {
            const dotX = originX + position + (fontSize * ratio * 0.5);
            ctx.fillText(dotChar, dotX, dotY);
          }
          ctx.restore();
        }
      }

      if (op.tabLeaders?.length) {
        this.drawTabLeadersOverlay(ctx, op.tabLeaders, originX, originY, op.style.color);
      }

      if (op.style.underline !== 'none') {
        ctx.save();
        ctx.strokeStyle = op.style.underlineColor || op.style.color;
        ctx.lineWidth = 1;
        const y = op.style.underline === 'top' ? originY - fontSize + 1 : originY + 2;
        ctx.beginPath();
        ctx.moveTo(originX, y);
        ctx.lineTo(originX + textWidth, y);
        ctx.stroke();
        ctx.restore();
      }

      if (op.style.strikethrough) {
        ctx.save();
        ctx.strokeStyle = op.style.strikeColor || op.style.color;
        ctx.lineWidth = 1;
        const y = originY - fontSize * 0.3;
        ctx.beginPath();
        ctx.moveTo(originX, y);
        ctx.lineTo(originX + textWidth, y);
        ctx.stroke();
        ctx.restore();
      }
    };

    ctx.save();
    this.setCanvasTextFont(ctx, op.style.fontFamily, fontSize, op.style.bold, op.style.italic);
    ctx.textBaseline = 'alphabetic';
    if (op.rotation !== 0) {
      const cx = op.bbox.x + op.bbox.width / 2;
      const cy = op.bbox.y + op.bbox.height / 2;
      ctx.translate(cx, cy);
      ctx.rotate((op.rotation * Math.PI) / 180);
      drawClusters(-op.bbox.width / 2, -op.bbox.height / 2 + op.baseline);
    } else {
      drawClusters(op.bbox.x, op.bbox.y + op.baseline);
    }
    ctx.restore();
  }

  private renderFootnoteMarkerOverlay(ctx: CanvasRenderingContext2D, op: LayerFootnoteMarkerOp): void {
    ctx.save();
    this.setCanvasTextFont(ctx, op.fontFamily, op.fontSize, false, false);
    ctx.textBaseline = 'alphabetic';
    ctx.fillStyle = op.color;
    ctx.fillText(op.text, op.bbox.x, op.bbox.y + op.bbox.height * 0.4);
    ctx.restore();
  }

  private drawTabLeadersOverlay(ctx: CanvasRenderingContext2D, leaders: LayerTabLeader[], originX: number, baselineY: number, color: string): void {
    for (const leader of leaders) {
      ctx.save();
      ctx.strokeStyle = color;
      ctx.lineWidth = 1;
      ctx.setLineDash(
        leader.fillType === 2 ? [4, 2]
          : leader.fillType === 3 ? [1.5, 2.5]
            : [],
      );
      const y = baselineY + 1;
      ctx.beginPath();
      ctx.moveTo(originX + leader.startX, y);
      ctx.lineTo(originX + leader.endX, y);
      ctx.stroke();
      ctx.restore();
    }
  }

  private setCanvasTextFont(
    ctx: CanvasRenderingContext2D,
    fontFamily: string,
    fontSize: number,
    bold: boolean,
    italic: boolean,
  ): void {
    ctx.font = buildCanvasTextFont(fontFamily, fontSize, bold, italic);
  }

  private drawEncodedImage(
    canvas: ReturnType<Surface['getCanvas']>,
    base64: string,
    bbox: LayerBounds,
    fillMode = 'fitToSize',
    originalSize?: { width: number; height: number },
    crop?: { left: number; top: number; right: number; bottom: number },
  ): void {
    const image = this.getImage(base64);
    if (!image) return;
    const drawImageRect = (srcRect: ReturnType<CanvasKit['XYWHRect']>, dstRect: ReturnType<CanvasKit['XYWHRect']>) => {
      const paint = new this.canvasKit.Paint();
      canvas.drawImageRectOptions(
        image,
        srcRect,
        dstRect,
        this.canvasKit.FilterMode.Linear,
        this.canvasKit.MipmapMode.None,
        paint,
      );
      paint.delete();
    };

    if (fillMode === 'fitToSize' || fillMode === 'none') {
      if (crop) {
        const imgW = image.width();
        const imgH = image.height();
        const scaleX = crop.right / imgW;
        const srcX = crop.left / scaleX;
        const srcY = crop.top / scaleX;
        const srcW = (crop.right - crop.left) / scaleX;
        const srcH = (crop.bottom - crop.top) / scaleX;
        const isCropped = srcX > 0.5 || srcY > 0.5 || Math.abs(srcW - imgW) > 1 || Math.abs(srcH - imgH) > 1;
        if (isCropped) {
          drawImageRect(
            this.canvasKit.XYWHRect(srcX, srcY, srcW, srcH),
            this.toRect(bbox),
          );
          return;
        }
      }
      drawImageRect(
        this.canvasKit.XYWHRect(0, 0, image.width(), image.height()),
        this.toRect(bbox),
      );
      return;
    }

    const imageWidth = originalSize?.width ?? image.width();
    const imageHeight = originalSize?.height ?? image.height();
    const { x, y } = this.resolveImagePlacement(fillMode, bbox, imageWidth, imageHeight);

    canvas.save();
    canvas.clipRect(this.toRect(bbox), this.canvasKit.ClipOp.Intersect, true);

    if (fillMode === 'tileAll' || fillMode === 'tileHorzTop' || fillMode === 'tileHorzBottom' || fillMode === 'tileVertLeft' || fillMode === 'tileVertRight') {
      if (fillMode === 'tileAll') {
        for (let ty = bbox.y; ty < bbox.y + bbox.height; ty += imageHeight) {
          for (let tx = bbox.x; tx < bbox.x + bbox.width; tx += imageWidth) {
            drawImageRect(
              this.canvasKit.XYWHRect(0, 0, image.width(), image.height()),
              this.canvasKit.XYWHRect(tx, ty, imageWidth, imageHeight),
            );
          }
        }
      } else if (fillMode === 'tileHorzTop' || fillMode === 'tileHorzBottom') {
        const ty = fillMode === 'tileHorzTop' ? bbox.y : bbox.y + bbox.height - imageHeight;
        for (let tx = bbox.x; tx < bbox.x + bbox.width; tx += imageWidth) {
          drawImageRect(
            this.canvasKit.XYWHRect(0, 0, image.width(), image.height()),
            this.canvasKit.XYWHRect(tx, ty, imageWidth, imageHeight),
          );
        }
      } else {
        const tx = fillMode === 'tileVertLeft' ? bbox.x : bbox.x + bbox.width - imageWidth;
        for (let ty = bbox.y; ty < bbox.y + bbox.height; ty += imageHeight) {
          drawImageRect(
            this.canvasKit.XYWHRect(0, 0, image.width(), image.height()),
            this.canvasKit.XYWHRect(tx, ty, imageWidth, imageHeight),
          );
        }
      }
    } else {
      drawImageRect(
        this.canvasKit.XYWHRect(0, 0, image.width(), image.height()),
        this.canvasKit.XYWHRect(x, y, imageWidth, imageHeight),
      );
    }

    canvas.restore();
  }

  private resolveImagePlacement(fillMode: string, bbox: LayerBounds, imageWidth: number, imageHeight: number): { x: number; y: number } {
    switch (fillMode) {
      case 'leftTop':
        return { x: bbox.x, y: bbox.y };
      case 'centerTop':
        return { x: bbox.x + (bbox.width - imageWidth) / 2, y: bbox.y };
      case 'rightTop':
        return { x: bbox.x + bbox.width - imageWidth, y: bbox.y };
      case 'leftCenter':
        return { x: bbox.x, y: bbox.y + (bbox.height - imageHeight) / 2 };
      case 'center':
        return { x: bbox.x + (bbox.width - imageWidth) / 2, y: bbox.y + (bbox.height - imageHeight) / 2 };
      case 'rightCenter':
        return { x: bbox.x + bbox.width - imageWidth, y: bbox.y + (bbox.height - imageHeight) / 2 };
      case 'leftBottom':
        return { x: bbox.x, y: bbox.y + bbox.height - imageHeight };
      case 'centerBottom':
        return { x: bbox.x + (bbox.width - imageWidth) / 2, y: bbox.y + bbox.height - imageHeight };
      case 'rightBottom':
        return { x: bbox.x + bbox.width - imageWidth, y: bbox.y + bbox.height - imageHeight };
      default:
        return { x: bbox.x, y: bbox.y };
    }
  }

  private drawTabLeaders(canvas: ReturnType<Surface['getCanvas']>, leaders: LayerTabLeader[], originX: number, baselineY: number, color: string): void {
    for (const leader of leaders) {
      const dash = leader.fillType === 2 ? 'dash' : leader.fillType === 3 ? 'dot' : 'solid';
      const paint = this.makeLinePaint(color, 1, dash);
      const y = baselineY + 1;
      canvas.drawLine(originX + leader.startX, y, originX + leader.endX, y, paint);
      paint.delete();
    }
  }

  private makePath(commands: LayerPathCommand[]) {
    const builder = new this.canvasKit.PathBuilder();
    for (const command of commands) {
      switch (command.type) {
        case 'moveTo':
          builder.moveTo(command.x, command.y);
          break;
        case 'lineTo':
          builder.lineTo(command.x, command.y);
          break;
        case 'curveTo':
          builder.cubicTo(command.x1, command.y1, command.x2, command.y2, command.x3, command.y3);
          break;
        case 'arcTo':
          builder.arcToRotated(command.rx, command.ry, command.rotation, !command.largeArc, !command.sweep, command.x, command.y);
          break;
        case 'closePath':
          builder.close();
          break;
      }
    }
    const path = builder.detach();
    builder.delete();
    return path;
  }

  private makeTextObjects(fontFamily: string, fontSize: number, bold: boolean, italic: boolean, color: string, scaleX = 1): { typeface: Typeface; font: Font; paint: Paint } {
    const family = this.resolveCanvasKitFontFamily(fontFamily);
    const typeface = this.fontProvider.matchFamilyStyle(family, {
      weight: this.canvasKit.FontWeight.Normal,
      slant: this.canvasKit.FontSlant.Upright,
    });
    const font = new this.canvasKit.Font(typeface, fontSize || 12);
    font.setEmbolden(bold);
    font.setScaleX(scaleX > 0 ? scaleX : 1);
    font.setSkewX(italic ? -0.25 : 0);
    if (this.renderMode === 'compat') {
      font.setSubpixel(true);
      if (fontSize >= 48 && bold && !italic) {
        font.setEdging(this.canvasKit.FontEdging.SubpixelAntiAlias);
        font.setHinting(this.canvasKit.FontHinting.Slight);
      }
    }
    const paint = this.makePaint(color, 'fill');
    return { typeface, font, paint };
  }

  private resolveCanvasKitFontFamily(fontFamily: string): string {
    const resolved = resolveFont(fontFamily, 0, 0);
    if (HAMCHOROM_DOTUM_ALIASES.has(resolved) || HAMCHOROM_DOTUM_ALIASES.has(fontFamily)) {
      return HAMCHOROM_DOTUM_FAMILY;
    }
    if (HAMCHOROM_BATANG_ALIASES.has(resolved) || HAMCHOROM_BATANG_ALIASES.has(fontFamily)) {
      return HAMCHOROM_BATANG_FAMILY;
    }
    if (this.fontAliases.has(resolved)) return resolved;
    if (this.fontAliases.has(fontFamily)) return fontFamily;

    const lower = resolved.toLowerCase();
    if (/gulimche|batangche|coding|courier/.test(lower) || /굴림체|바탕체/.test(resolved)) {
      return 'D2Coding';
    }
    if (/batang|gungsuh|serif|times/.test(lower) || /바탕|명조|궁서/.test(resolved)) {
      return 'Noto Serif KR';
    }
    return 'Noto Sans KR';
  }

  private makePaint(color: string, style: 'fill' | 'stroke', opacity = 1): Paint {
    const paint = new this.canvasKit.Paint();
    paint.setAntiAlias(true);
    paint.setStyle(style === 'fill' ? this.canvasKit.PaintStyle.Fill : this.canvasKit.PaintStyle.Stroke);
    const rgba = [...this.canvasKit.parseColorString(color)] as number[];
    rgba[3] = (rgba[3] ?? 1) * opacity;
    paint.setColor(rgba as any);
    return paint;
  }

  private makeLinePaint(color: string, width: number, dash: string, opacity = 1): Paint {
    const paint = this.makePaint(color, 'stroke', opacity);
    const strokeWidth = Math.max(width, 0.5);
    paint.setStrokeWidth(strokeWidth);

    if (dash !== 'solid') {
      const stroke = Math.max(width, 0.5);
      const intervals =
        dash === 'dash' ? [stroke * 4, stroke * 2]
          : dash === 'dot' ? [stroke * 1.5, stroke * 2.5]
            : dash === 'dashDot' ? [stroke * 4, stroke * 2, stroke * 1.5, stroke * 2]
              : [stroke * 4, stroke * 2, stroke * 1.5, stroke * 2, stroke * 1.5, stroke * 2];
      const effect = this.canvasKit.PathEffect.MakeDash(intervals, 0);
      paint.setPathEffect(effect);
      effect.delete();
    }

    return paint;
  }

  private makeShapeFillPaint(
    bounds: LayerBounds,
    fillColor: string | null | undefined,
    opacity: number,
    gradient?: LayerGradient,
    pattern?: LayerPatternFill,
  ): { paint: Paint; shader: Shader | null } | null {
    const shader = gradient ? this.makeGradientShader(gradient, bounds) : pattern ? this.makePatternShader(pattern) : null;
    if (!shader && !fillColor) {
      return null;
    }

    const paint = this.makePaint(fillColor ?? '#ffffff', 'fill', opacity);
    if (shader) {
      paint.setShader(shader);
      paint.setAlphaf(opacity);
    }
    return { paint, shader };
  }

  private makeGradientShader(gradient: LayerGradient, bounds: LayerBounds): Shader | null {
    if (gradient.colors.length < 2) {
      return null;
    }

    const colors = gradient.colors.map((color) => this.canvasKit.parseColorString(color));
    const positions = gradient.positions.length > 0 ? gradient.positions : null;
    if (gradient.gradientType === 2 || gradient.gradientType === 3 || gradient.gradientType === 4) {
      const cx = bounds.x + bounds.width * (gradient.centerX / 100);
      const cy = bounds.y + bounds.height * (gradient.centerY / 100);
      const radius = Math.max(bounds.width, bounds.height) / 2;
      return this.canvasKit.Shader.MakeRadialGradient(
        [cx, cy],
        radius,
        colors,
        positions,
        this.canvasKit.TileMode.Clamp,
      );
    }

    const [x0, y0, x1, y1] = angleToCanvasCoords(gradient.angle, bounds.x, bounds.y, bounds.width, bounds.height);
    return this.canvasKit.Shader.MakeLinearGradient(
      [x0, y0],
      [x1, y1],
      colors,
      positions,
      this.canvasKit.TileMode.Clamp,
    );
  }

  private makePatternShader(pattern: LayerPatternFill): Shader | null {
    const image = this.getPatternImage(pattern);
    return image
      ? image.makeShaderOptions(
        this.canvasKit.TileMode.Repeat,
        this.canvasKit.TileMode.Repeat,
        this.canvasKit.FilterMode.Nearest,
        this.canvasKit.MipmapMode.None,
      )
      : null;
  }

  private getPatternImage(pattern: LayerPatternFill): Image | null {
    const cacheKey = `${pattern.patternType}:${pattern.patternColor}:${pattern.backgroundColor}`;
    if (this.patternImageCache.has(cacheKey)) {
      return this.patternImageCache.get(cacheKey) ?? null;
    }

    const bytes = rasterizePatternTileToPngBytes(pattern);
    const image = bytes ? this.canvasKit.MakeImageFromEncoded(bytes) : null;
    this.patternImageCache.set(cacheKey, image);
    return image;
  }

  private drawShadow(
    canvas: ReturnType<Surface['getCanvas']>,
    shadow: LayerShapeShadow | undefined,
    style: 'fill' | 'stroke',
    color: string,
    strokeWidth: number,
    draw: (paint: Paint) => void,
  ): void {
    if (!shadow) {
      return;
    }

    const opacity = shadow.alpha > 0 ? 1 - (shadow.alpha / 255) : 1;
    const paint = this.makePaint(color, style, opacity);
    if (style === 'stroke') {
      paint.setStrokeWidth(Math.max(strokeWidth, 0.5));
    }
    const blur = this.canvasKit.MaskFilter.MakeBlur(this.canvasKit.BlurStyle.Normal, 1, false);
    paint.setMaskFilter(blur);
    blur.delete();

    canvas.save();
    canvas.translate(shadow.offsetX, shadow.offsetY);
    draw(paint);
    canvas.restore();
    paint.delete();
  }

  private withTransform(
    canvas: ReturnType<Surface['getCanvas']>,
    bbox: LayerBounds,
    transform: { rotation: number; horzFlip: boolean; vertFlip: boolean },
    draw: () => void,
  ): void {
    if (!transform.rotation && !transform.horzFlip && !transform.vertFlip) {
      draw();
      return;
    }

    const cx = bbox.x + bbox.width / 2;
    const cy = bbox.y + bbox.height / 2;

    canvas.save();
    if (transform.horzFlip) {
      canvas.translate(cx * 2, 0);
      canvas.scale(-1, 1);
    }
    if (transform.vertFlip) {
      canvas.translate(0, cy * 2);
      canvas.scale(1, -1);
    }
    if (transform.rotation) {
      canvas.rotate(transform.rotation, cx, cy);
    }
    draw();
    canvas.restore();
  }

  private drawCanvasKitImage(
    canvas: ReturnType<Surface['getCanvas']>,
    image: Image,
    bbox: LayerBounds,
  ): void {
    const paint = new this.canvasKit.Paint();
    canvas.drawImageRectOptions(
      image,
      this.canvasKit.XYWHRect(0, 0, image.width(), image.height()),
      this.toRect(bbox),
      this.canvasKit.FilterMode.Linear,
      this.canvasKit.MipmapMode.None,
      paint,
    );
    paint.delete();
  }

  private getImage(base64: string): Image | null {
    const cached = this.imageCache.get(base64);
    if (cached) return cached;

    const bytes = decodeBase64(base64);
    const image = this.canvasKit.MakeImageFromEncoded(bytes);
    if (!image) return null;
    this.imageCache.set(base64, image);
    return image;
  }

  private toRect(bounds: LayerBounds) {
    return this.canvasKit.XYWHRect(bounds.x, bounds.y, bounds.width, bounds.height);
  }
}

function decodeBase64(base64: string): Uint8Array {
  const binary = window.atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let idx = 0; idx < binary.length; idx += 1) {
    bytes[idx] = binary.charCodeAt(idx);
  }
  return bytes;
}

function buildCanvasTextFont(fontFamily: string, fontSize: number, bold: boolean, italic: boolean): string {
  const resolved = resolveFont(fontFamily, 0, 0);
  const baseFamily = resolved || fontFamily;
  return `${italic ? 'italic ' : ''}${bold ? 'bold ' : ''}${(fontSize || 12).toFixed(3)}px ${fontFamilyWithFallback(baseFamily)}`;
}

function startsWithInvalidControl(text: string): boolean {
  if (!text) {
    return false;
  }
  const code = text.codePointAt(0) ?? 0;
  return code < 0x20 && code !== 0x09 && code !== 0x0a && code !== 0x0d;
}

function isHalfwidthScaledCluster(text: string): boolean {
  const code = text.codePointAt(0) ?? 0;
  return (code >= 0x2018 && code <= 0x2027) || code === 0x00b7;
}

function angleToCanvasCoords(angle: number, x: number, y: number, width: number, height: number): [number, number, number, number] {
  const normalized = ((angle % 360) + 360) % 360;
  switch (normalized) {
    case 0:
      return [x, y, x, y + height];
    case 45:
      return [x, y, x + width, y + height];
    case 90:
      return [x, y, x + width, y];
    case 135:
      return [x, y + height, x + width, y];
    case 180:
      return [x, y + height, x, y];
    case 225:
      return [x + width, y + height, x, y];
    case 270:
      return [x + width, y, x, y];
    case 315:
      return [x + width, y, x, y + height];
    default: {
      const radians = normalized * (Math.PI / 180);
      const sin = Math.sin(radians);
      const cos = Math.cos(radians);
      const centerX = x + width / 2;
      const centerY = y + height / 2;
      return [
        centerX - sin * width / 2,
        centerY - cos * height / 2,
        centerX + sin * width / 2,
        centerY + cos * height / 2,
      ];
    }
  }
}

function rasterizePatternTileToPngBytes(pattern: LayerPatternFill): Uint8Array | null {
  const canvas = document.createElement('canvas');
  canvas.width = 6;
  canvas.height = 6;
  const ctx = canvas.getContext('2d');
  if (!ctx) {
    return null;
  }

  ctx.fillStyle = pattern.backgroundColor;
  ctx.fillRect(0, 0, 6, 6);
  ctx.strokeStyle = pattern.patternColor;
  ctx.lineWidth = 1;

  switch (pattern.patternType) {
    case 0:
      ctx.beginPath();
      ctx.moveTo(0, 3);
      ctx.lineTo(6, 3);
      ctx.stroke();
      break;
    case 1:
      ctx.beginPath();
      ctx.moveTo(3, 0);
      ctx.lineTo(3, 6);
      ctx.stroke();
      break;
    case 2:
      ctx.beginPath();
      ctx.moveTo(6, 0);
      ctx.lineTo(0, 6);
      ctx.stroke();
      break;
    case 3:
      ctx.beginPath();
      ctx.moveTo(0, 0);
      ctx.lineTo(6, 6);
      ctx.stroke();
      break;
    case 4:
      ctx.beginPath();
      ctx.moveTo(3, 0);
      ctx.lineTo(3, 6);
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(0, 3);
      ctx.lineTo(6, 3);
      ctx.stroke();
      break;
    case 5:
      ctx.beginPath();
      ctx.moveTo(0, 0);
      ctx.lineTo(6, 6);
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(6, 0);
      ctx.lineTo(0, 6);
      ctx.stroke();
      break;
    default:
      break;
  }

  const dataUrl = canvas.toDataURL('image/png');
  const [, encoded = ''] = dataUrl.split(',');
  return decodeBase64(encoded);
}

function computePathPaintBounds(commands: LayerPathCommand[], fallback: LayerBounds): LayerBounds {
  let minX = Number.POSITIVE_INFINITY;
  let minY = Number.POSITIVE_INFINITY;
  let maxX = Number.NEGATIVE_INFINITY;
  let maxY = Number.NEGATIVE_INFINITY;

  const record = (x: number, y: number) => {
    minX = Math.min(minX, x);
    minY = Math.min(minY, y);
    maxX = Math.max(maxX, x);
    maxY = Math.max(maxY, y);
  };

  for (const command of commands) {
    switch (command.type) {
      case 'moveTo':
      case 'lineTo':
        record(command.x, command.y);
        break;
      case 'curveTo':
        record(command.x1, command.y1);
        record(command.x2, command.y2);
        record(command.x3, command.y3);
        break;
      case 'arcTo':
        record(command.x, command.y);
        break;
      case 'closePath':
        break;
    }
  }

  if (!Number.isFinite(minX) || !Number.isFinite(minY) || !Number.isFinite(maxX) || !Number.isFinite(maxY)) {
    return fallback;
  }

  return {
    x: minX,
    y: minY,
    width: Math.max(maxX - minX, 1),
    height: Math.max(maxY - minY, 1),
  };
}

function calculateArrowDimensions(strokeWidth: number, lineLength: number, arrowSize: number): [number, number] {
  const widthLevel = Math.floor(arrowSize / 3);
  const lengthLevel = arrowSize % 3;
  const widthMultiplier = widthLevel === 0 ? 1.5 : widthLevel === 1 ? 2.5 : 3.5;
  const lengthMultiplier = lengthLevel === 0 ? 1 : lengthLevel === 1 ? 1.5 : 2;
  const arrowHeight = Math.max(strokeWidth * widthMultiplier, 3);
  const arrowWidth = Math.min(arrowHeight * lengthMultiplier, lineLength * 0.3);
  return [arrowWidth, arrowHeight];
}

function drawArrowHead(
  canvasKit: CanvasKit,
  canvas: ReturnType<Surface['getCanvas']>,
  tipX: number,
  tipY: number,
  directionX: number,
  directionY: number,
  arrowWidth: number,
  arrowHeight: number,
  arrowStyle: string,
  color: string,
  strokeWidth: number,
): void {
  if (arrowStyle === 'none') {
    return;
  }

  const alongX = -directionX;
  const alongY = -directionY;
  const perpX = directionY;
  const perpY = -directionX;
  const halfHeight = arrowHeight / 2;
  const toWorld = (along: number, perp: number): [number, number] => [
    tipX + along * alongX + perp * perpX,
    tipY + along * alongY + perp * perpY,
  ];

  const builder = new canvasKit.PathBuilder();
  const fillPaint = new canvasKit.Paint();
  fillPaint.setAntiAlias(true);
  fillPaint.setStyle(canvasKit.PaintStyle.Fill);
  fillPaint.setColor(canvasKit.parseColorString(color));

  const strokePaint = new canvasKit.Paint();
  strokePaint.setAntiAlias(true);
  strokePaint.setStyle(canvasKit.PaintStyle.Stroke);
  strokePaint.setColor(canvasKit.parseColorString(color));
  strokePaint.setStrokeWidth(Math.max(strokeWidth * 0.3, 0.5));

  if (arrowStyle === 'arrow' || arrowStyle === 'concaveArrow') {
    const [baseX1, baseY1] = toWorld(arrowWidth, -halfHeight);
    const [baseX2, baseY2] = toWorld(arrowWidth, halfHeight);
    builder.moveTo(tipX, tipY);
    builder.lineTo(baseX1, baseY1);
    if (arrowStyle === 'concaveArrow') {
      const [centerX, centerY] = toWorld(arrowWidth - arrowWidth * 0.3, 0);
      builder.lineTo(centerX, centerY);
    }
    builder.lineTo(baseX2, baseY2);
    builder.close();
    const path = builder.detach();
    canvas.drawPath(path, fillPaint);
    path.delete();
    builder.delete();
    fillPaint.delete();
    strokePaint.delete();
    return;
  }

  if (arrowStyle === 'diamond' || arrowStyle === 'openDiamond') {
    const halfWidth = arrowWidth / 2;
    const [point1X, point1Y] = toWorld(0, 0);
    const [point2X, point2Y] = toWorld(halfWidth, -halfHeight);
    const [point3X, point3Y] = toWorld(arrowWidth, 0);
    const [point4X, point4Y] = toWorld(halfWidth, halfHeight);
    builder.moveTo(point1X, point1Y);
    builder.lineTo(point2X, point2Y);
    builder.lineTo(point3X, point3Y);
    builder.lineTo(point4X, point4Y);
    builder.close();
    const path = builder.detach();
    if (arrowStyle === 'diamond') {
      canvas.drawPath(path, fillPaint);
    } else {
      const whiteFill = new canvasKit.Paint();
      whiteFill.setAntiAlias(true);
      whiteFill.setStyle(canvasKit.PaintStyle.Fill);
      whiteFill.setColor(canvasKit.parseColorString('white'));
      canvas.drawPath(path, whiteFill);
      canvas.drawPath(path, strokePaint);
      whiteFill.delete();
    }
    path.delete();
    builder.delete();
    fillPaint.delete();
    strokePaint.delete();
    return;
  }

  if (arrowStyle === 'circle' || arrowStyle === 'openCircle') {
    const halfWidth = arrowWidth / 2;
    const [centerX, centerY] = toWorld(halfWidth, 0);
    const radiusX = halfWidth * 0.8;
    const radiusY = halfHeight * 0.8;
    if (arrowStyle === 'circle') {
      canvas.drawOval(canvasKit.LTRBRect(centerX - radiusX, centerY - radiusY, centerX + radiusX, centerY + radiusY), fillPaint);
    } else {
      const whiteFill = new canvasKit.Paint();
      whiteFill.setAntiAlias(true);
      whiteFill.setStyle(canvasKit.PaintStyle.Fill);
      whiteFill.setColor(canvasKit.parseColorString('white'));
      const oval = canvasKit.LTRBRect(centerX - radiusX, centerY - radiusY, centerX + radiusX, centerY + radiusY);
      canvas.drawOval(oval, whiteFill);
      canvas.drawOval(oval, strokePaint);
      whiteFill.delete();
    }
    builder.delete();
    fillPaint.delete();
    strokePaint.delete();
    return;
  }

  if (arrowStyle === 'square' || arrowStyle === 'openSquare') {
    const [point1X, point1Y] = toWorld(0, -halfHeight);
    const [point2X, point2Y] = toWorld(arrowWidth, -halfHeight);
    const [point3X, point3Y] = toWorld(arrowWidth, halfHeight);
    const [point4X, point4Y] = toWorld(0, halfHeight);
    builder.moveTo(point1X, point1Y);
    builder.lineTo(point2X, point2Y);
    builder.lineTo(point3X, point3Y);
    builder.lineTo(point4X, point4Y);
    builder.close();
    const path = builder.detach();
    if (arrowStyle === 'square') {
      canvas.drawPath(path, fillPaint);
    } else {
      const whiteFill = new canvasKit.Paint();
      whiteFill.setAntiAlias(true);
      whiteFill.setStyle(canvasKit.PaintStyle.Fill);
      whiteFill.setColor(canvasKit.parseColorString('white'));
      canvas.drawPath(path, whiteFill);
      canvas.drawPath(path, strokePaint);
      whiteFill.delete();
    }
    path.delete();
  }

  builder.delete();
  fillPaint.delete();
  strokePaint.delete();
}

const EQUATION_SCRIPT_SCALE = 0.7;
const EQUATION_BIG_OP_SCALE = 1.5;

function renderEquationLayoutBox(
  ctx: CanvasRenderingContext2D,
  layout: LayerEquationLayoutBox,
  parentX: number,
  parentY: number,
  color: string,
  fontSize: number,
  italic: boolean,
  bold: boolean,
): void {
  const x = parentX + layout.x;
  const y = parentY + layout.y;

  switch (layout.kind.type) {
    case 'row':
      for (const child of layout.kind.children) {
        renderEquationLayoutBox(ctx, child, x, y, color, fontSize, italic, bold);
      }
      return;
    case 'text': {
      const size = equationFontSizeFromBox(layout, fontSize);
      setEquationFont(ctx, size, true, bold);
      ctx.fillStyle = color;
      ctx.fillText(layout.kind.text, x, y + layout.baseline);
      return;
    }
    case 'number': {
      const size = equationFontSizeFromBox(layout, fontSize);
      setEquationFont(ctx, size, false, bold);
      ctx.fillStyle = color;
      ctx.fillText(layout.kind.text, x, y + layout.baseline);
      return;
    }
    case 'symbol': {
      const size = equationFontSizeFromBox(layout, fontSize);
      setEquationFont(ctx, size, false, false);
      ctx.fillStyle = color;
      ctx.save();
      ctx.textAlign = 'center';
      ctx.fillText(layout.kind.text, x + layout.width / 2, y + layout.baseline);
      ctx.restore();
      return;
    }
    case 'mathSymbol': {
      const size = equationFontSizeFromBox(layout, fontSize);
      setEquationFont(ctx, size, false, false);
      ctx.fillStyle = color;
      ctx.fillText(layout.kind.text, x, y + layout.baseline);
      return;
    }
    case 'function': {
      const size = equationFontSizeFromBox(layout, fontSize);
      setEquationFont(ctx, size, false, false);
      ctx.fillStyle = color;
      ctx.fillText(layout.kind.name, x, y + layout.baseline);
      return;
    }
    case 'fraction':
      renderEquationLayoutBox(ctx, layout.kind.numer, x, y, color, fontSize, italic, bold);
      ctx.strokeStyle = color;
      ctx.lineWidth = fontSize * 0.04;
      ctx.beginPath();
      ctx.moveTo(x + fontSize * 0.05, y + layout.baseline);
      ctx.lineTo(x + layout.width - fontSize * 0.05, y + layout.baseline);
      ctx.stroke();
      renderEquationLayoutBox(ctx, layout.kind.denom, x, y, color, fontSize, italic, bold);
      return;
    case 'sqrt': {
      const bodyLeft = x + layout.kind.body.x - fontSize * 0.1;
      const signHeight = layout.height;
      const midX = bodyLeft - fontSize * 0.15;
      const midY = y + signHeight;
      const startX = midX - fontSize * 0.3;
      const startY = y + signHeight * 0.6;
      const tickX = startX - fontSize * 0.1;
      const tickY = startY - fontSize * 0.05;

      ctx.strokeStyle = color;
      ctx.lineWidth = fontSize * 0.04;
      ctx.beginPath();
      ctx.moveTo(tickX, tickY);
      ctx.lineTo(startX, startY);
      ctx.lineTo(midX, midY);
      ctx.lineTo(bodyLeft, y);
      ctx.lineTo(x + layout.width, y);
      ctx.stroke();

      if (layout.kind.index) {
        renderEquationLayoutBox(
          ctx,
          layout.kind.index,
          x,
          y,
          color,
          fontSize * EQUATION_SCRIPT_SCALE,
          false,
          false,
        );
      }
      renderEquationLayoutBox(ctx, layout.kind.body, x, y, color, fontSize, italic, bold);
      return;
    }
    case 'superscript':
      renderEquationLayoutBox(ctx, layout.kind.base, x, y, color, fontSize, italic, bold);
      renderEquationLayoutBox(
        ctx,
        layout.kind.sup,
        x,
        y,
        color,
        fontSize * EQUATION_SCRIPT_SCALE,
        italic,
        bold,
      );
      return;
    case 'subscript':
      renderEquationLayoutBox(ctx, layout.kind.base, x, y, color, fontSize, italic, bold);
      renderEquationLayoutBox(
        ctx,
        layout.kind.sub,
        x,
        y,
        color,
        fontSize * EQUATION_SCRIPT_SCALE,
        italic,
        bold,
      );
      return;
    case 'subSup':
      renderEquationLayoutBox(ctx, layout.kind.base, x, y, color, fontSize, italic, bold);
      renderEquationLayoutBox(
        ctx,
        layout.kind.sub,
        x,
        y,
        color,
        fontSize * EQUATION_SCRIPT_SCALE,
        italic,
        bold,
      );
      renderEquationLayoutBox(
        ctx,
        layout.kind.sup,
        x,
        y,
        color,
        fontSize * EQUATION_SCRIPT_SCALE,
        italic,
        bold,
      );
      return;
    case 'bigOp': {
      const opFontSize = fontSize * EQUATION_BIG_OP_SCALE;
      const supHeight = layout.kind.sup ? layout.kind.sup.height + fontSize * 0.05 : 0;
      const opX = x + (layout.width - estimateEquationOperatorWidth(layout.kind.symbol, opFontSize)) / 2;
      const opY = y + supHeight + opFontSize * 0.8;
      setEquationFont(ctx, opFontSize, false, false);
      ctx.fillStyle = color;
      ctx.fillText(layout.kind.symbol, opX, opY);
      if (layout.kind.sup) {
        renderEquationLayoutBox(
          ctx,
          layout.kind.sup,
          x,
          y,
          color,
          fontSize * EQUATION_SCRIPT_SCALE,
          false,
          false,
        );
      }
      if (layout.kind.sub) {
        renderEquationLayoutBox(
          ctx,
          layout.kind.sub,
          x,
          y,
          color,
          fontSize * EQUATION_SCRIPT_SCALE,
          false,
          false,
        );
      }
      return;
    }
    case 'limit': {
      const name = layout.kind.isUpper ? 'Lim' : 'lim';
      const size = equationFontSizeFromBox(layout, fontSize);
      setEquationFont(ctx, size, false, false);
      ctx.fillStyle = color;
      ctx.fillText(name, x, y + size * 0.8);
      if (layout.kind.sub) {
        renderEquationLayoutBox(
          ctx,
          layout.kind.sub,
          x,
          y,
          color,
          fontSize * EQUATION_SCRIPT_SCALE,
          false,
          false,
        );
      }
      return;
    }
    case 'matrix': {
      const brackets = layout.kind.style === 'paren' ? ['(', ')']
        : layout.kind.style === 'bracket' ? ['[', ']']
          : layout.kind.style === 'vert' ? ['|', '|']
            : ['', ''];
      if (brackets[0]) {
        drawEquationStretchBracket(ctx, brackets[0], x, y, fontSize * 0.3, layout.height, color, fontSize);
        drawEquationStretchBracket(ctx, brackets[1], x + layout.width - fontSize * 0.3, y, fontSize * 0.3, layout.height, color, fontSize);
      }
      for (const row of layout.kind.cells) {
        for (const cell of row) {
          renderEquationLayoutBox(ctx, cell, x, y, color, fontSize, italic, bold);
        }
      }
      return;
    }
    case 'rel':
      renderEquationLayoutBox(ctx, layout.kind.over, x, y, color, fontSize, italic, bold);
      renderEquationLayoutBox(ctx, layout.kind.arrow, x, y, color, fontSize, italic, bold);
      if (layout.kind.under) {
        renderEquationLayoutBox(ctx, layout.kind.under, x, y, color, fontSize, italic, bold);
      }
      return;
    case 'eqAlign':
      for (const row of layout.kind.rows) {
        renderEquationLayoutBox(ctx, row.left, x, y, color, fontSize, italic, bold);
        renderEquationLayoutBox(ctx, row.right, x, y, color, fontSize, italic, bold);
      }
      return;
    case 'paren':
      if (layout.kind.left) {
        drawEquationStretchBracket(ctx, layout.kind.left, x, y, fontSize * 0.3, layout.height, color, fontSize);
      }
      renderEquationLayoutBox(ctx, layout.kind.body, x, y, color, fontSize, italic, bold);
      if (layout.kind.right) {
        drawEquationStretchBracket(
          ctx,
          layout.kind.right,
          x + layout.width - fontSize * 0.3,
          y,
          fontSize * 0.3,
          layout.height,
          color,
          fontSize,
        );
      }
      return;
    case 'decoration':
      renderEquationLayoutBox(ctx, layout.kind.body, x, y, color, fontSize, italic, bold);
      drawEquationDecoration(
        ctx,
        layout.kind.decoration,
        x + layout.kind.body.x + layout.kind.body.width / 2,
        y + fontSize * 0.05,
        layout.kind.body.width,
        color,
        fontSize,
      );
      return;
    case 'fontStyle': {
      const nextItalic = layout.kind.fontStyle === 'roman' ? false : layout.kind.fontStyle === 'italic' ? true : italic;
      const nextBold = layout.kind.fontStyle === 'roman' ? false : layout.kind.fontStyle === 'bold' ? true : bold;
      renderEquationLayoutBox(ctx, layout.kind.body, x, y, color, fontSize, nextItalic, nextBold);
      return;
    }
    case 'space':
    case 'newline':
    case 'empty':
      return;
  }
}

function equationFontSizeFromBox(layout: LayerEquationLayoutBox, baseFontSize: number): number {
  return layout.height > 0 ? layout.height : baseFontSize;
}

function estimateEquationOperatorWidth(text: string, fontSize: number): number {
  return Array.from(text).length * fontSize * 0.6;
}

function setEquationFont(
  ctx: CanvasRenderingContext2D,
  size: number,
  italic: boolean,
  bold: boolean,
): void {
  const style = italic ? 'italic ' : '';
  const weight = bold ? 'bold ' : '';
  ctx.font = `${style}${weight}${size.toFixed(1)}px 'Latin Modern Math', 'STIX Two Math', 'Cambria Math', 'Pretendard', serif`;
}

function drawEquationStretchBracket(
  ctx: CanvasRenderingContext2D,
  bracket: string,
  x: number,
  y: number,
  width: number,
  height: number,
  color: string,
  fontSize: number,
): void {
  const midX = x + width / 2;
  ctx.strokeStyle = color;
  ctx.lineWidth = fontSize * 0.04;

  switch (bracket) {
    case '(':
      ctx.beginPath();
      ctx.moveTo(midX + width * 0.2, y);
      ctx.quadraticCurveTo(x, y + height / 2, midX + width * 0.2, y + height);
      ctx.stroke();
      return;
    case ')':
      ctx.beginPath();
      ctx.moveTo(midX - width * 0.2, y);
      ctx.quadraticCurveTo(x + width, y + height / 2, midX - width * 0.2, y + height);
      ctx.stroke();
      return;
    case '[':
      ctx.beginPath();
      ctx.moveTo(midX + width * 0.2, y);
      ctx.lineTo(midX - width * 0.2, y);
      ctx.lineTo(midX - width * 0.2, y + height);
      ctx.lineTo(midX + width * 0.2, y + height);
      ctx.stroke();
      return;
    case ']':
      ctx.beginPath();
      ctx.moveTo(midX - width * 0.2, y);
      ctx.lineTo(midX + width * 0.2, y);
      ctx.lineTo(midX + width * 0.2, y + height);
      ctx.lineTo(midX - width * 0.2, y + height);
      ctx.stroke();
      return;
    case '{': {
      const quarterHeight = height / 4;
      ctx.beginPath();
      ctx.moveTo(midX + width * 0.2, y);
      ctx.quadraticCurveTo(midX - width * 0.1, y, midX - width * 0.1, y + quarterHeight);
      ctx.quadraticCurveTo(midX - width * 0.1, y + quarterHeight * 2, midX - width * 0.3, y + quarterHeight * 2);
      ctx.quadraticCurveTo(midX - width * 0.1, y + quarterHeight * 2, midX - width * 0.1, y + quarterHeight * 3);
      ctx.quadraticCurveTo(midX - width * 0.1, y + height, midX + width * 0.2, y + height);
      ctx.stroke();
      return;
    }
    case '}': {
      const quarterHeight = height / 4;
      ctx.beginPath();
      ctx.moveTo(midX - width * 0.2, y);
      ctx.quadraticCurveTo(midX + width * 0.1, y, midX + width * 0.1, y + quarterHeight);
      ctx.quadraticCurveTo(midX + width * 0.1, y + quarterHeight * 2, midX + width * 0.3, y + quarterHeight * 2);
      ctx.quadraticCurveTo(midX + width * 0.1, y + quarterHeight * 2, midX + width * 0.1, y + quarterHeight * 3);
      ctx.quadraticCurveTo(midX + width * 0.1, y + height, midX - width * 0.2, y + height);
      ctx.stroke();
      return;
    }
    case '|':
      ctx.beginPath();
      ctx.moveTo(midX, y);
      ctx.lineTo(midX, y + height);
      ctx.stroke();
      return;
    default:
      setEquationFont(ctx, height, false, false);
      ctx.fillStyle = color;
      ctx.save();
      ctx.textAlign = 'center';
      ctx.fillText(bracket, midX, y + height * 0.7);
      ctx.restore();
  }
}

function drawEquationDecoration(
  ctx: CanvasRenderingContext2D,
  decoration: string,
  midX: number,
  y: number,
  width: number,
  color: string,
  fontSize: number,
): void {
  const strokeWidth = fontSize * 0.03;
  const halfWidth = width / 2;
  ctx.strokeStyle = color;
  ctx.lineWidth = strokeWidth;

  switch (decoration) {
    case 'hat':
      ctx.beginPath();
      ctx.moveTo(midX - halfWidth * 0.6, y + fontSize * 0.15);
      ctx.lineTo(midX, y);
      ctx.lineTo(midX + halfWidth * 0.6, y + fontSize * 0.15);
      ctx.stroke();
      return;
    case 'bar':
    case 'overline':
      ctx.beginPath();
      ctx.moveTo(midX - halfWidth, y + fontSize * 0.05);
      ctx.lineTo(midX + halfWidth, y + fontSize * 0.05);
      ctx.stroke();
      return;
    case 'vec': {
      const arrowY = y + fontSize * 0.05;
      ctx.beginPath();
      ctx.moveTo(midX - halfWidth, arrowY);
      ctx.lineTo(midX + halfWidth, arrowY);
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(midX + halfWidth - fontSize * 0.1, arrowY - fontSize * 0.06);
      ctx.lineTo(midX + halfWidth, arrowY);
      ctx.lineTo(midX + halfWidth - fontSize * 0.1, arrowY + fontSize * 0.06);
      ctx.stroke();
      return;
    }
    case 'tilde': {
      const tildeY = y + fontSize * 0.08;
      ctx.beginPath();
      ctx.moveTo(midX - halfWidth * 0.6, tildeY);
      ctx.quadraticCurveTo(midX - halfWidth * 0.2, tildeY - fontSize * 0.08, midX, tildeY);
      ctx.quadraticCurveTo(midX + halfWidth * 0.2, tildeY + fontSize * 0.08, midX + halfWidth * 0.6, tildeY);
      ctx.stroke();
      return;
    }
    case 'dot':
      ctx.fillStyle = color;
      ctx.beginPath();
      ctx.arc(midX, y + fontSize * 0.06, fontSize * 0.03, 0, Math.PI * 2);
      ctx.fill();
      return;
    case 'dDot': {
      const gap = fontSize * 0.1;
      ctx.fillStyle = color;
      ctx.beginPath();
      ctx.arc(midX - gap, y + fontSize * 0.06, fontSize * 0.03, 0, Math.PI * 2);
      ctx.fill();
      ctx.beginPath();
      ctx.arc(midX + gap, y + fontSize * 0.06, fontSize * 0.03, 0, Math.PI * 2);
      ctx.fill();
      return;
    }
    case 'underline':
    case 'under': {
      const underlineY = y + fontSize * 1.1;
      ctx.beginPath();
      ctx.moveTo(midX - halfWidth, underlineY);
      ctx.lineTo(midX + halfWidth, underlineY);
      ctx.stroke();
      return;
    }
    default:
      ctx.beginPath();
      ctx.moveTo(midX - halfWidth * 0.5, y + fontSize * 0.1);
      ctx.lineTo(midX + halfWidth * 0.5, y + fontSize * 0.1);
      ctx.stroke();
  }
}

function splitIntoClusters(text: string): Array<{ start: number; text: string }> {
  const chars = Array.from(text);
  const clusters: Array<{ start: number; text: string }> = [];

  let idx = 0;
  while (idx < chars.length) {
    if (isHangulChoseong(chars[idx])) {
      const start = idx;
      let cluster = chars[idx];
      idx += 1;
      if (idx < chars.length && isHangulJungseong(chars[idx])) {
        cluster += chars[idx];
        idx += 1;
        if (idx < chars.length && isHangulJongseong(chars[idx])) {
          cluster += chars[idx];
          idx += 1;
        }
      }
      clusters.push({ start, text: cluster });
      continue;
    }

    clusters.push({ start: idx, text: chars[idx] });
    idx += 1;
  }

  return clusters;
}

function isHangulChoseong(char: string): boolean {
  const code = char.codePointAt(0) ?? 0;
  return (code >= 0x1100 && code <= 0x115f) || (code >= 0xa960 && code <= 0xa97f);
}

function isHangulJungseong(char: string): boolean {
  const code = char.codePointAt(0) ?? 0;
  return (code >= 0x1160 && code <= 0x11a7) || (code >= 0xd7b0 && code <= 0xd7c6);
}

function isHangulJongseong(char: string): boolean {
  const code = char.codePointAt(0) ?? 0;
  return (code >= 0x11a8 && code <= 0x11ff) || (code >= 0xd7cb && code <= 0xd7fb);
}
