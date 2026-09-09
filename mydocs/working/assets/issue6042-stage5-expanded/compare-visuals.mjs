import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const require = createRequire(path.resolve(here, '../../../../rhwp-studio/package.json'));
const { PNG } = require('pngjs');
const pixelmatchModule = require('pixelmatch');
const pixelmatch = pixelmatchModule.default ?? pixelmatchModule;

function decodeJpeg(source, label) {
  const target = path.join(here, `.compare-${label}.png`);
  const result = spawnSync('/usr/bin/sips', ['-s', 'format', 'png', source, '--out', target], {
    encoding: 'utf8',
  });
  if (result.status !== 0) throw new Error(`sips failed: ${result.stderr}`);
  const decoded = PNG.sync.read(fs.readFileSync(target));
  fs.unlinkSync(target);
  return decoded;
}

function compare({ name, beforeName, afterName, note }) {
  const beforePath = path.join(here, beforeName);
  const afterPath = path.join(here, afterName);
  const diffName = `${name}-diff.png`;
  const before = decodeJpeg(beforePath, `${name}-before`);
  const after = decodeJpeg(afterPath, `${name}-after`);
  if (before.width !== after.width || before.height !== after.height) {
    throw new Error(`dimension mismatch for ${name}: ${before.width}x${before.height} != ${after.width}x${after.height}`);
  }
  const diff = new PNG({ width: before.width, height: before.height });
  const differingPixels = pixelmatch(before.data, after.data, diff.data, before.width, before.height, {
    threshold: 0.1,
    includeAA: false,
  });
  fs.writeFileSync(path.join(here, diffName), PNG.sync.write(diff));
  const totalPixels = before.width * before.height;
  return {
    before: beforeName,
    after: afterName,
    diff: diffName,
    width: before.width,
    height: before.height,
    threshold: 0.1,
    includeAntialiasing: false,
    differingPixels,
    differingPercent: Number(((differingPixels / totalPixels) * 100).toFixed(4)),
    note,
  };
}

const result = {
  schemaVersion: 1,
  comparisons: {
    canvas2dAuto34: compare({
      name: 'canvas2d-exam-auto-34',
      beforeName: 'canvas2d-exam-auto-34-stage3.jpg',
      afterName: 'canvas2d-exam-auto-34-corrected.jpg',
      note: '동일 1280x720·DPR 2·자동 보기·34%의 browser JPEG compositor 비교다. footer 시간과 probe 상태 문구를 포함한다.',
    }),
    canvasKitKtx100: compare({
      name: 'canvaskit-ktx-100',
      beforeName: 'canvaskit-ktx-100-stage3.jpg',
      afterName: 'canvaskit-ktx-100-corrected.jpg',
      note: '동일 1280x720·DPR 2·CanvasKit·KTX 4-layer·100%의 browser JPEG compositor 비교다. footer 시간과 probe 상태 문구를 포함한다.',
    }),
  },
};

fs.writeFileSync(path.join(here, 'visual-comparison.json'), `${JSON.stringify(result, null, 2)}\n`);
console.log(JSON.stringify(result, null, 2));
