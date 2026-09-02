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
const beforePath = path.join(here, 'exam-4col-34-stage3.jpg');
const afterPath = path.join(here, 'exam-4col-34-corrected.jpg');
const diffPath = path.join(here, 'exam-4col-34-diff.png');
const resultPath = path.join(here, 'exam-4col-34-visual-comparison.json');
const snapshotPath = path.join(here, 'exam-4col-34-visual-snapshots.json');

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

const before = decodeJpeg(beforePath, 'before');
const after = decodeJpeg(afterPath, 'after');
const snapshots = JSON.parse(fs.readFileSync(snapshotPath, 'utf8'));

function comparableSnapshot(snapshot) {
  return {
    dpr: snapshot.dpr,
    viewport: snapshot.viewport,
    zoom: snapshot.zoom,
    columns: snapshot.columns,
    visible: snapshot.visible,
    retained: snapshot.retained,
    scroll: snapshot.scroll,
    activePixels: snapshot.activePixels,
    pages: snapshot.pages
      .map(page => ({
        page: page.page,
        dpr: page.dpr,
        scale: page.scale,
        pixels: page.pixels,
        box: page.box,
        surfaces: page.surfaces,
      }))
      .sort((left, right) => left.page - right.page),
  };
}

if (before.width !== after.width || before.height !== after.height) {
  throw new Error(`dimension mismatch: ${before.width}x${before.height} != ${after.width}x${after.height}`);
}

const diff = new PNG({ width: before.width, height: before.height });
const differingPixels = pixelmatch(before.data, after.data, diff.data, before.width, before.height, {
  threshold: 0.1,
  includeAA: false,
});
const totalPixels = before.width * before.height;
const result = {
  schemaVersion: 1,
  before: path.basename(beforePath),
  after: path.basename(afterPath),
  diff: path.basename(diffPath),
  width: before.width,
  height: before.height,
  threshold: 0.1,
  includeAntialiasing: false,
  differingPixels,
  differingPercent: Number(((differingPixels / totalPixels) * 100).toFixed(4)),
  coordinatesAndSurfacesEqual: JSON.stringify(comparableSnapshot(snapshots.baseline))
    === JSON.stringify(comparableSnapshot(snapshots.corrected)),
  note: '동일 viewport·4열·34%·scroll의 browser JPEG compositor 화면 비교다. footer load time과 probe 상태 문구 차이를 포함한다.',
};

fs.writeFileSync(diffPath, PNG.sync.write(diff));
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`);
console.log(JSON.stringify(result, null, 2));
