export interface FlowImageBbox {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface FlowImageCrop {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

export interface FlowImagePaintOp {
  bbox: FlowImageBbox;
  mime: string;
  base64: string;
  crop: FlowImageCrop | null;
  originalSizeHu: [number, number] | null;
  rotation: number;
  horzFlip: boolean;
  vertFlip: boolean;
  // 그림 효과(회색조/흑백/밝기/명암)를 DOM <img> 에 적용할 CSS filter. 없으면 null.
  // web_canvas.rs render_image 의 compose_image_filter 와 동일 산출.
  filter: string | null;
  // DOM 정적 그림도 원래 PageLayerTree의 clip 계보를 유지해야 한다.
  clip: FlowImageBbox | null;
}

type LayerNodeLike = {
  kind?: unknown;
  layer?: unknown;
  clip?: unknown;
  ops?: unknown;
  children?: unknown;
  child?: unknown;
};

type LayerPaintOpLike = {
  type?: unknown;
  bbox?: unknown;
  mime?: unknown;
  base64?: unknown;
  crop?: unknown;
  originalSizeHu?: unknown;
  effect?: unknown;
  brightness?: unknown;
  contrast?: unknown;
  bakedWatermark?: unknown;
  transform?: {
    rotation?: unknown;
    horzFlip?: unknown;
    vertFlip?: unknown;
  };
};

/**
 * 그림 효과(ImageEffect)+밝기/명암을 DOM <img> 용 CSS filter 문자열로 변환한다.
 * `src/renderer/web_canvas.rs::compose_image_filter` 와 동일한 산출을 유지해
 * WASM canvas 경로와 DOM flow-image 경로의 렌더 결과가 일치하도록 한다.
 * 워터마크로 baked 된 픽셀(bakedWatermark)은 효과가 이미 적용돼 있으므로 제외한다.
 */
export function composeImageFilter(op: LayerPaintOpLike): string | null {
  if (op.bakedWatermark === true) return null;
  const parts: string[] = [];
  const effect = typeof op.effect === 'string' ? op.effect : 'realPic';
  if (effect === 'grayScale' || effect === 'pattern8x8') {
    parts.push('grayscale(100%)');
  } else if (effect === 'blackWhite') {
    parts.push('grayscale(100%)');
    parts.push('contrast(1000%)');
  }
  const brightness = finiteNumber(op.brightness);
  const contrast = finiteNumber(op.contrast);
  if (brightness !== 0) parts.push(`brightness(${((100 + brightness) / 100).toFixed(4)})`);
  if (contrast !== 0) parts.push(`contrast(${((100 + contrast) / 100).toFixed(4)})`);
  return parts.length > 0 ? parts.join(' ') : null;
}

export function collectFlowImagePaintOps(
  root: unknown,
  isFlowImage: (op: LayerPaintOpLike, layer: unknown) => boolean,
): FlowImagePaintOp[] {
  const images: FlowImagePaintOp[] = [];

  const visit = (
    value: unknown,
    inheritedLayer: unknown,
    inheritedClip: FlowImageBbox | undefined | null,
  ): void => {
    if (!isLayerNode(value) || inheritedClip === null) return;

    const activeLayer = value.layer ?? inheritedLayer;
    const clip = value.kind === 'clipRect' && isFiniteBbox(value.clip)
      ? intersectBboxes(inheritedClip, value.clip)
      : inheritedClip;
    if (clip === null) return;

    if (Array.isArray(value.ops)) {
      for (const op of value.ops) {
        if (!isLayerPaintOp(op) || !isFlowImage(op, activeLayer)) continue;
        if (
          typeof op.mime !== 'string' ||
          typeof op.base64 !== 'string' ||
          !isFiniteBbox(op.bbox)
        ) {
          continue;
        }
        images.push({
          bbox: op.bbox,
          mime: op.mime,
          base64: op.base64,
          crop: isFiniteCrop(op.crop) ? op.crop : null,
          originalSizeHu: isPositiveSizeTuple(op.originalSizeHu) ? op.originalSizeHu : null,
          rotation: finiteNumber(op.transform?.rotation),
          horzFlip: op.transform?.horzFlip === true,
          vertFlip: op.transform?.vertFlip === true,
          filter: composeImageFilter(op),
          clip: clip ?? null,
        });
      }
    }

    if (Array.isArray(value.children)) {
      for (const child of value.children) {
        visit(child, activeLayer, clip);
      }
    }
    if (value.child !== undefined) {
      visit(value.child, activeLayer, clip);
    }
  };

  visit(root, null, undefined);
  return images;
}

export function visibleFlowImageBbox(image: FlowImagePaintOp): FlowImageBbox | null {
  return image.clip === null ? image.bbox : intersectBboxes(image.bbox, image.clip);
}

function isLayerNode(value: unknown): value is LayerNodeLike {
  return value !== null && typeof value === 'object';
}

function isLayerPaintOp(value: unknown): value is LayerPaintOpLike {
  return value !== null && typeof value === 'object';
}

function isPositiveSizeTuple(value: unknown): value is [number, number] {
  return Array.isArray(value)
    && value.length === 2
    && value.every((entry) => typeof entry === 'number' && Number.isFinite(entry) && entry > 0);
}

function intersectBboxes(
  first: FlowImageBbox | undefined,
  second: FlowImageBbox,
): FlowImageBbox | null {
  if (first === undefined) return second;
  const left = Math.max(first.x, second.x);
  const top = Math.max(first.y, second.y);
  const right = Math.min(first.x + first.width, second.x + second.width);
  const bottom = Math.min(first.y + first.height, second.y + second.height);
  if (right <= left || bottom <= top) return null;
  return { x: left, y: top, width: right - left, height: bottom - top };
}

function isFiniteBbox(value: unknown): value is FlowImageBbox {
  if (value === null || typeof value !== 'object') return false;
  const bbox = value as Partial<FlowImageBbox>;
  return (
    Number.isFinite(bbox.x) &&
    Number.isFinite(bbox.y) &&
    Number.isFinite(bbox.width) &&
    Number.isFinite(bbox.height) &&
    (bbox.width ?? 0) > 0 &&
    (bbox.height ?? 0) > 0
  );
}

function isFiniteCrop(value: unknown): value is FlowImageCrop {
  if (value === null || typeof value !== 'object') return false;
  const crop = value as Partial<FlowImageCrop>;
  return (
    Number.isFinite(crop.left) &&
    Number.isFinite(crop.top) &&
    Number.isFinite(crop.right) &&
    Number.isFinite(crop.bottom) &&
    (crop.right ?? 0) > (crop.left ?? 0) &&
    (crop.bottom ?? 0) > (crop.top ?? 0)
  );
}

function finiteNumber(value: unknown): number {
  const number = Number(value);
  return Number.isFinite(number) ? number : 0;
}
