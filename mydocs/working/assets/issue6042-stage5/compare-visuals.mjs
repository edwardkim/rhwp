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

const beforePath = path.join(here, 'exam-4col-34-stage3-before.jpg');
const afterPath = path.join(here, 'exam-4col-34-stage4-after.jpg');
const diffPath = path.join(here, 'exam-4col-34-diff.png');
const resultPath = path.join(here, 'exam-4col-34-visual-comparison.json');

function decodeJpegWithSips(source, label) {
  const target = path.join(here, `.compare-${label}.png`);
  const result = spawnSync('/usr/bin/sips', ['-s', 'format', 'png', source, '--out', target], {
    encoding: 'utf8',
  });
  if (result.status !== 0) {
    throw new Error(`sips failed for ${path.basename(source)}: ${result.stderr}`);
  }
  const decoded = PNG.sync.read(fs.readFileSync(target));
  fs.unlinkSync(target);
  return decoded;
}

const before = decodeJpegWithSips(beforePath, 'before');
const after = decodeJpegWithSips(afterPath, 'after');
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
  note: '브라우저 제어기가 반환한 JPEG compositor screenshot 전체 비교다. footer의 문서 로드 시간 숫자 차이도 포함하며 화질 판정에는 쓰지 않는다.',
};

fs.writeFileSync(diffPath, PNG.sync.write(diff));
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`);
console.log(JSON.stringify(result, null, 2));
