import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const files = [
  'auto-backend-exam-corrected.json',
  'auto-backend-exam-stage3.json',
  'canvas2d-document-switch-corrected.json',
  'canvas2d-document-switch-stage3.json',
  'canvas2d-edit-undo-redo-corrected.json',
  'canvas2d-edit-undo-redo-stage3.json',
  'canvas2d-exam-auto-zoom-sequence-corrected.json',
  'canvas2d-exam-auto-zoom-sequence-stage3.json',
  'canvas2d-exam-horizontal-corrected.json',
  'canvas2d-exam-horizontal-stage3.json',
  'canvas2d-exam-rapid-reverse-corrected.json',
  'canvas2d-exam-rapid-reverse-stage3.json',
  'canvas2d-four-page-double-zoom-corrected.json',
  'canvas2d-four-page-double-zoom-stage3.json',
  'canvas2d-four-page-facing-corrected.json',
  'canvas2d-four-page-facing-stage3.json',
  'canvas2d-four-page-single-zoom-corrected.json',
  'canvas2d-four-page-single-zoom-stage3.json',
  'canvas2d-ktx-4-layer-zoom-corrected-repeat2.json',
  'canvas2d-ktx-4-layer-zoom-corrected.json',
  'canvas2d-ktx-4-layer-zoom-stage3-repeat2.json',
  'canvas2d-ktx-4-layer-zoom-stage3.json',
  'canvas2d-prosecutor-21p-last-row-corrected.json',
  'canvas2d-prosecutor-21p-last-row-stage3.json',
  'canvaskit-ktx-4-layer-zoom-corrected.json',
  'canvaskit-ktx-4-layer-zoom-stage3.json',
];

function compact(value) {
  if (Array.isArray(value)) return value.map(compact);
  if (!value || typeof value !== 'object') return value;
  const output = {};
  const isSnapshot = Number.isInteger(value.pageCount) && Array.isArray(value.visible);
  for (const [key, child] of Object.entries(value)) {
    if (isSnapshot && ['browser', 'longTasks', 'note', 'traces'].includes(key)) continue;
    output[key] = compact(child);
  }
  return output;
}

for (const name of files) {
  const file = path.join(here, name);
  const value = JSON.parse(fs.readFileSync(file, 'utf8'));
  fs.writeFileSync(file, `${JSON.stringify(compact(value), null, 2)}\n`);
}

console.log(`compacted ${files.length} functional evidence files`);
