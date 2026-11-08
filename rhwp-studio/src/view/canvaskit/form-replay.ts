import type { LayerFormObjectOp } from '@/core/types';

export const CANVASKIT_MAX_FORM_TEXT_LENGTH = 4096;
const MAX_FLOAT32 = 3.4028234663852886e38;

function scalar(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value) && Math.abs(value) <= MAX_FLOAT32;
}

function validRect(value: unknown): boolean {
  if (!value || typeof value !== 'object') return false;
  const { x, y, width, height } = value as Record<string, unknown>;
  return scalar(x) && scalar(y) && scalar(width) && scalar(height)
    && width >= 0 && height >= 0 && scalar(x + width) && scalar(y + height);
}

export type FormDrawingReplayRejection = 'invalidGeometry' | 'missingProducerDrawing'
  | 'formPrimitiveBudgetExceeded' | 'formTextBudgetExceeded' | 'formColorBudgetExceeded';

/** Validate the serialized producer appearance, without deriving layout from raw form fields. */
export function formDrawingReplayRejection(op: LayerFormObjectOp): FormDrawingReplayRejection | null {
  if (!validRect(op.bbox) || op.bbox.width <= 0 || op.bbox.height <= 0) return 'invalidGeometry';
  const drawing = op.drawing;
  if (!drawing) return 'missingProducerDrawing';
  if (!Array.isArray(drawing.primitives)) return 'invalidGeometry';
  if (drawing.primitives.length > 512) return 'formPrimitiveBudgetExceeded';
  let points = 0;
  for (const primitive of drawing.primitives) {
    if (!primitive || typeof primitive !== 'object') return 'invalidGeometry';
    switch (primitive.kind) {
      case 'rect':
        if (!validRect(primitive.bbox)) return 'invalidGeometry';
        break;
      case 'circle': {
        const { x, y, radius } = primitive;
        if (!scalar(x) || !scalar(y) || !scalar(radius) || radius < 0
          || ![x - radius, x + radius, y - radius, y + radius].every(scalar)) return 'invalidGeometry';
        break;
      }
      case 'polyline':
        if (!Array.isArray(primitive.points) || !scalar(primitive.width) || primitive.width < 0
          || typeof primitive.closed !== 'boolean') return 'invalidGeometry';
        points += primitive.points.length;
        if (points > 4096) return 'formPrimitiveBudgetExceeded';
        if (!primitive.points.every(point => Array.isArray(point) && point.length === 2 && point.every(scalar))) {
          return 'invalidGeometry';
        }
        break;
      default: return 'invalidGeometry';
    }
    if (typeof primitive.color !== 'string' || primitive.color.length > 128) return 'formColorBudgetExceeded';
  }
  const label = drawing.label;
  if (label !== undefined && label !== null) {
    if (typeof label !== 'object') return 'invalidGeometry';
    if ([label.text, label.fontFamily].some(value => typeof value !== 'string'
      || value.length > CANVASKIT_MAX_FORM_TEXT_LENGTH)) return 'formTextBudgetExceeded';
    if (typeof label.color !== 'string' || label.color.length > 128) return 'formColorBudgetExceeded';
    if (!scalar(label.x) || !scalar(label.baseline) || !scalar(label.fontSize)
      || label.fontSize <= 0 || label.fontSize > 4096 || !scalar(label.baseline - label.fontSize)
      || typeof label.bold !== 'boolean' || typeof label.italic !== 'boolean') return 'invalidGeometry';
  }
  return null;
}
