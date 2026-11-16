// #8199 제보 재현 스크립트를 로컬 fresh WASM 패키지로 그대로 실행한다.
// 사용: node generate.mjs <pkg 디렉터리> <입력 F05.hwp> <출력 .hwp>
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const [pkgDir, input, output] = process.argv.slice(2);
const rhwp = await import(pathToFileURL(path.join(pkgDir, 'rhwp.js')).href);
rhwp.initSync({ module: fs.readFileSync(path.join(pkgDir, 'rhwp_bg.wasm')) });

const doc = new rhwp.HwpDocument(new Uint8Array(fs.readFileSync(input)));
const pagesBefore = doc.pageCount();
const png = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGP4//8/AAX+Av4N70a4AAAAAElFTkSuQmCC',
  'base64',
); // 1x1
const r = JSON.parse(
  doc.insertPicture(0, 0, 0, '', new Uint8Array(png), 2835, 2835, 1, 1, 'png', 'repro', 0, 0),
);
const props = doc.setPictureProperties(0, r.paraIdx, r.controlIdx, JSON.stringify({ textWrap: 'InFrontOfText' }));
const bytes = doc.exportHwp();
fs.writeFileSync(output, bytes);

const reopened = new rhwp.HwpDocument(new Uint8Array(fs.readFileSync(output)));
console.log(JSON.stringify({ insert: r, props: JSON.parse(props), pagesBefore, pagesAfterEdit: doc.pageCount(), pagesReopened: reopened.pageCount(), bytes: bytes.length }));
