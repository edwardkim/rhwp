// 실제 Chrome·FontFace·공개 Noto 글꼴로 PNG exporter의 글꼴 연결을 검사한다.
// HWP 엔진 대신 1쪽 Canvas adapter를 사용해 브라우저 글꼴 공급 계약을 격리한다.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawn} from 'node:child_process';
import {mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {tmpdir} from 'node:os';
import {dirname, join, resolve} from 'node:path';
import test from 'node:test';
import {fileURLToPath} from 'node:url';
import {findChrome} from '../rasterize-svg-webfonts.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const require = createRequire(join(root, 'rhwp-studio/package.json'));
const sampleText = 'Wm ilil @@@ 12345 총투자';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
function run(args) {
  return new Promise((accept, reject) => {
    const child = spawn(process.execPath, args, {cwd: root});
    let output = '';
    child.stdout.on('data', chunk => { output += chunk; });
    child.stderr.on('data', chunk => { output += chunk; });
    child.on('error', reject);
    child.on('close', code => code === 0 ? accept(output) : reject(Error(output)));
  });
}

test('공급한 실제 font program을 Canvas PNG가 대체 글꼴 없이 사용한다', async t => {
  const folder = mkdtempSync(join(tmpdir(), 'rhwp-sweep-font-binding-'));
  t.after(() => rmSync(folder, {recursive: true, force: true}));
  const pkg = join(folder, 'pkg');
  mkdirSync(pkg);
  writeFileSync(join(pkg, 'package.json'), '{"type":"module"}');
  writeFileSync(join(pkg, 'rhwp_bg.wasm'), Buffer.from([0, 97, 115, 109, 1, 0, 0, 0]));
  writeFileSync(join(pkg, 'rhwp.js'), `
    export default async function(){}
    export const version=()=> 'canvas-font-contract-adapter';
    export class HwpDocument {
      pageCount(){return 1;} setHangul2024Compat(){} free(){}
      getPageLayerTreeWithProfile(){return '{"fontResources":{"faces":[],"blobs":[]}}';}
      getPageRenderTree(){return '{"type":"Page","bbox":{"x":0,"y":0,"w":500,"h":70}}';}
      renderPageToCanvasFilteredWithProfile(_page,canvas){
        canvas.width=500;canvas.height=70;const c=canvas.getContext('2d');
        c.fillStyle='white';c.fillRect(0,0,500,70);c.fillStyle='black';
        c.font='20px SweepFontProbe';c.fillText(${JSON.stringify(sampleText)},10,35);
      }
    }
  `);
  const input = join(folder, 'input');
  writeFileSync(input, 'Canvas adapter input; no HWP document');
  const font = readFileSync(join(root, 'assets/fonts/NotoSansKR-Bold.woff2'));
  const fontPath = join(folder, 'reference.woff2');
  writeFileSync(fontPath, font);
  const fonts = join(folder, 'fonts.json'), options = join(folder, 'options.json');
  writeFileSync(fonts, JSON.stringify({faces: [{path: fontPath, sha256: sha(font),
    sourceSha256: sha(font), source: fontPath, families: ['SweepFontProbe'],
    weight: 400, style: 'normal', faceIndex: 0, repairs: []}]}));
  writeFileSync(options, JSON.stringify({compat: '2022', scale: 1, workers: 1, pages: [1], fontEnvironment: null}));
  const output = join(folder, 'output');
  await run([join(root, 'scripts/export-wasm-png-for-sweep.mjs'), '--pkg', pkg,
    '--input', input, '--out', output, '--fonts', fonts, '--options', options]);
  const browser = await require('puppeteer-core').launch({
    executablePath: findChrome(process.env.VISUAL_SWEEP_CHROME), headless: true,
  });
  t.after(() => browser.close());
  const page = await browser.newPage();
  const expected = await page.evaluate(async ({base64, text}) => {
    const bytes = Uint8Array.from(atob(base64), char => char.charCodeAt(0));
    const face = new FontFace('IndependentFontReference', bytes);
    await face.load();document.fonts.add(face);
    const canvas = document.createElement('canvas');canvas.width=500;canvas.height=70;
    const c=canvas.getContext('2d');c.fillStyle='white';c.fillRect(0,0,500,70);
    c.fillStyle='black';c.font='20px IndependentFontReference';c.fillText(text,10,35);
    return canvas.toDataURL('image/png').split(',')[1];
  }, {base64: font.toString('base64'), text: sampleText});
  const actual = readFileSync(join(output, 'rhwp_png/rhwp_001.png'));
  assert.deepEqual(actual, Buffer.from(expected, 'base64'),
    '적재 성공 표기만으로 통과하지 않고, 독립 등록한 동일 program의 실제 PNG와 비교한다');
});
