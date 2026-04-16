import CanvasKitInit from 'canvaskit-wasm';
import type { CanvasKit, Font, Image, Paint, Surface, Typeface, TypefaceFontProvider } from 'canvaskit-wasm';
import canvaskitWasmUrl from 'canvaskit-wasm/bin/canvaskit.wasm?url';

import { resolveFont } from '@/core/font-substitution';
import type { CanvasKitRenderMode } from '@/view/render-backend';
import type {
  LayerBounds,
  LayerClipNode,
  LayerEllipseOp,
  LayerImageOp,
  LayerLeafNode,
  LayerLineOp,
  LayerNode,
  LayerPageBackgroundOp,
  LayerPaintOp,
  LayerPathCommand,
  LayerPathOp,
  LayerRectangleOp,
  LayerTabLeader,
  LayerTextRunOp,
  PageLayerTree,
} from '@/core/types';

const FONT_SANS_REGULAR_URL = new URL('../../../web/fonts/NotoSansKR-Regular.woff2', import.meta.url).href;
const FONT_SANS_BOLD_URL = new URL('../../../web/fonts/NotoSansKR-Bold.woff2', import.meta.url).href;
const FONT_SERIF_REGULAR_URL = new URL('../../../web/fonts/NotoSerifKR-Regular.woff2', import.meta.url).href;
const FONT_SERIF_BOLD_URL = new URL('../../../web/fonts/NotoSerifKR-Bold.woff2', import.meta.url).href;
const FONT_MONO_REGULAR_URL = new URL('../../../web/fonts/D2Coding-Regular.woff2', import.meta.url).href;

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
  '새돋움',
  '새굴림',
  '한컴돋움',
  '함초롬돋움',
  '함초롱돋움',
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
  '새바탕',
  '한컴바탕',
  '함초롬바탕',
  '함초롱바탕',
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

  renderPage(tree: PageLayerTree, targetCanvas: HTMLCanvasElement, scale: number): void {
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

    await registerAliases(SANS_ALIASES, FONT_SANS_REGULAR_URL, FONT_SANS_BOLD_URL);
    await registerAliases(SERIF_ALIASES, FONT_SERIF_REGULAR_URL, FONT_SERIF_BOLD_URL);
    await registerAliases(MONO_ALIASES, FONT_MONO_REGULAR_URL);
  }

  private renderNode(canvas: ReturnType<Surface['getCanvas']>, node: LayerNode): void {
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

  private renderClipNode(canvas: ReturnType<Surface['getCanvas']>, node: LayerClipNode): void {
    canvas.save();
    canvas.clipRect(
      this.canvasKit.XYWHRect(node.clip.x, node.clip.y, node.clip.width, node.clip.height),
      this.canvasKit.ClipOp.Intersect,
      true,
    );
    this.renderNode(canvas, node.child);
    canvas.restore();
  }

  private renderLeafNode(canvas: ReturnType<Surface['getCanvas']>, node: LayerLeafNode): void {
    for (const op of node.ops) {
      this.renderOp(canvas, op);
    }
  }

  private renderOp(canvas: ReturnType<Surface['getCanvas']>, op: LayerPaintOp): void {
    switch (op.type) {
      case 'pageBackground':
        this.renderPageBackground(canvas, op);
        return;
      case 'textRun':
        this.renderTextRun(canvas, op);
        return;
      case 'footnoteMarker':
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
        this.renderPlaceholderRect(canvas, op.bbox, '#dcdcdc');
        return;
      case 'formObject':
        this.renderPlaceholderRect(canvas, op.bbox, '#f0f0f0');
    }
  }

  private renderPageBackground(canvas: ReturnType<Surface['getCanvas']>, op: LayerPageBackgroundOp): void {
    if (op.backgroundColor) {
      const paint = this.makePaint(op.backgroundColor, 'fill');
      canvas.drawRect(this.toRect(op.bbox), paint);
      paint.delete();
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
    const primaryObjects = this.makeTextObjects(
      op.style.fontFamily,
      op.style.fontSize,
      op.style.bold,
      op.style.italic,
      op.style.color,
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
      if (op.style.shadowType > 0) {
        const shadowPaint = this.makePaint(op.style.shadowColor, 'fill');
        for (const [index, cluster] of clusters.entries()) {
          const x = originX + op.positions[cluster.start];
          canvas.drawText(
            cluster.text,
            x + op.style.shadowOffsetX,
            originY + op.style.shadowOffsetY,
            shadowPaint,
            clusterFonts[index],
          );
        }
        shadowPaint.delete();
      }

      for (const [index, cluster] of clusters.entries()) {
        canvas.drawText(
          cluster.text,
          originX + op.positions[cluster.start],
          originY,
          primaryObjects.paint,
          clusterFonts[index],
        );
      }

      if (op.tabLeaders?.length) {
        this.drawTabLeaders(canvas, op.tabLeaders, originX, originY, op.style.color);
      }

      const textWidth = op.positions.at(-1) ?? 0;
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
      const paint = this.makeLinePaint(op.style.color, op.style.width, op.style.dash);
      canvas.drawLine(op.x1, op.y1, op.x2, op.y2, paint);
      paint.delete();
    });
  }

  private renderRectangle(canvas: ReturnType<Surface['getCanvas']>, op: LayerRectangleOp): void {
    this.withTransform(canvas, op.bbox, op.transform, () => {
      const fillPaint = op.style.fillColor ? this.makePaint(op.style.fillColor, 'fill', op.style.opacity) : null;
      const strokePaint = op.style.strokeColor ? this.makeLinePaint(op.style.strokeColor, op.style.strokeWidth, op.style.strokeDash, op.style.opacity) : null;
      const rect = this.toRect(op.bbox);

      if (fillPaint) {
        if (op.cornerRadius > 0) {
          canvas.drawRRect(this.canvasKit.RRectXY(rect, op.cornerRadius, op.cornerRadius), fillPaint);
        } else {
          canvas.drawRect(rect, fillPaint);
        }
        fillPaint.delete();
      }

      if (strokePaint) {
        if (op.cornerRadius > 0) {
          canvas.drawRRect(this.canvasKit.RRectXY(rect, op.cornerRadius, op.cornerRadius), strokePaint);
        } else {
          canvas.drawRect(rect, strokePaint);
        }
        strokePaint.delete();
      }
    });
  }

  private renderEllipse(canvas: ReturnType<Surface['getCanvas']>, op: LayerEllipseOp): void {
    this.withTransform(canvas, op.bbox, op.transform, () => {
      const fillPaint = op.style.fillColor ? this.makePaint(op.style.fillColor, 'fill', op.style.opacity) : null;
      const strokePaint = op.style.strokeColor ? this.makeLinePaint(op.style.strokeColor, op.style.strokeWidth, op.style.strokeDash, op.style.opacity) : null;
      const oval = this.toRect(op.bbox);

      if (fillPaint) {
        canvas.drawOval(oval, fillPaint);
        fillPaint.delete();
      }
      if (strokePaint) {
        canvas.drawOval(oval, strokePaint);
        strokePaint.delete();
      }
    });
  }

  private renderPath(canvas: ReturnType<Surface['getCanvas']>, op: LayerPathOp): void {
    this.withTransform(canvas, op.bbox, op.transform, () => {
      const path = this.makePath(op.commands);
      const fillPaint = op.style.fillColor ? this.makePaint(op.style.fillColor, 'fill', op.style.opacity) : null;
      const strokePaint = op.style.strokeColor ? this.makeLinePaint(op.style.strokeColor, op.style.strokeWidth, op.style.strokeDash, op.style.opacity) : null;

      if (fillPaint) {
        canvas.drawPath(path, fillPaint);
        fillPaint.delete();
      }
      if (strokePaint) {
        canvas.drawPath(path, strokePaint);
        strokePaint.delete();
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

  private renderPlaceholderRect(canvas: ReturnType<Surface['getCanvas']>, bbox: LayerBounds, color: string): void {
    const paint = this.makePaint(color, 'fill');
    canvas.drawRect(this.toRect(bbox), paint);
    paint.delete();
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
      if (this.renderMode === 'compat') {
        canvas.drawImageRectOptions(
          image,
          srcRect,
          dstRect,
          this.canvasKit.FilterMode.Linear,
          this.canvasKit.MipmapMode.None,
          paint,
        );
      } else {
        canvas.drawImageRect(image, srcRect, dstRect, paint, false);
      }
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

  private makeTextObjects(fontFamily: string, fontSize: number, bold: boolean, italic: boolean, color: string): { typeface: Typeface; font: Font; paint: Paint } {
    const family = this.resolveCanvasKitFontFamily(fontFamily);
    const typeface = this.fontProvider.matchFamilyStyle(family, {
      weight: bold ? this.canvasKit.FontWeight.Bold : this.canvasKit.FontWeight.Normal,
      slant: italic ? this.canvasKit.FontSlant.Italic : this.canvasKit.FontSlant.Upright,
    });
    const font = new this.canvasKit.Font(typeface, fontSize || 12);
    font.setEmbolden(bold);
    if (this.renderMode === 'compat') {
      font.setSubpixel(true);
    }
    const paint = this.makePaint(color, 'fill');
    return { typeface, font, paint };
  }

  private resolveCanvasKitFontFamily(fontFamily: string): string {
    const resolved = resolveFont(fontFamily, 0, 0);
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
    paint.setStrokeWidth(Math.max(width, 1));

    if (dash !== 'solid') {
      const stroke = Math.max(width, 1);
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
