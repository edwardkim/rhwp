/** Rust의 공통 겹침 배치를 소비한다. 위치·크기·장평을 다시 정하지 않는다. */
import type { Canvas, CanvasKit } from 'canvaskit-wasm';
import type { LayerCharOverlapOp, LayerTextRunOp } from '../core/types';

export function replayComposedCharOverlap(
  api: CanvasKit,
  canvas: Canvas,
  op: LayerCharOverlapOp,
  draw: (glyph: LayerTextRunOp) => void,
): boolean {
  const plan = op.composition;
  if (!plan || !Array.isArray(plan.interior) || plan.interior.length > 9
    || typeof plan.reversed !== 'boolean' || !Number.isFinite(op.rotation)) return false;
  const glyphs = [...(plan.border ? [plan.border] : []), ...plan.interior];
  if (glyphs.some(glyph => !glyph || glyph.type !== 'textRun' || glyph.charOverlap
    || typeof glyph.text !== 'string' || [...glyph.text].length !== 1 || !glyph.bbox
    || ![glyph.bbox.x, glyph.bbox.y, glyph.bbox.width, glyph.bbox.height, glyph.baseline].every(Number.isFinite)
    || glyph.bbox.width < 0 || glyph.bbox.height <= 0)) return false;
  canvas.save();
  try {
    if (op.rotation !== 0) {
      canvas.rotate(op.rotation, op.bbox.x + op.bbox.width / 2, op.bbox.y + op.bbox.height / 2);
    }
    if (plan.reversed) {
      canvas.saveLayer();
      try {
        if (plan.border) draw(plan.border);
        const erase = new api.Paint();
        try {
          erase.setBlendMode(api.BlendMode.DstOut);
          canvas.saveLayer(erase);
          try { for (const glyph of plan.interior) draw(glyph); }
          finally { canvas.restore(); }
        } finally { erase.delete(); }
      } finally { canvas.restore(); }
    } else {
      for (const glyph of glyphs) draw(glyph);
    }
    return true;
  } finally { canvas.restore(); }
}
