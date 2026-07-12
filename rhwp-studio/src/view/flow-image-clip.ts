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
  rotation: number;
  horzFlip: boolean;
  vertFlip: boolean;
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
  transform?: {
    rotation?: unknown;
    horzFlip?: unknown;
    vertFlip?: unknown;
  };
};

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
          rotation: finiteNumber(op.transform?.rotation),
          horzFlip: op.transform?.horzFlip === true,
          vertFlip: op.transform?.vertFlip === true,
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
