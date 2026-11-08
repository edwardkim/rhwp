// #8271 합성 입력 생성기 — 편집 API 로 만들고 HWPX 로 저장한다.
//
// 사용: node make_inputs.mjs <web pkg 디렉터리> <출력 디렉터리>
//   예: node mydocs/report/issue8271_hwpx_reopen_tac_picture_float/make_inputs.mjs pkg samples/issue8271
//
// HWPX 저장은 편집한 host 문단의 linesegarray 를 뺀다. 그래서 다시 여는 쪽(rhwp·한/글)이 그 문단을
// 저장 줄 없이 재조판한다 — 이 입력이 시험하는 경로다. 저장한 파일을 열어 host 문단에
// linesegarray 가 없는지 확인한다.
//
// 1. tac_picture_topandbottom_textbox: #8168 재현과 같은 편집(「앞 글 뒤 글」 둘째 글자 자리에
//    글자처럼 그림, 같은 문단 끝에 종이 기준 세로 15000HU 위아래 글상자).
// 2. tac_picture_behindtext_textbox: 1 과 같고 글상자 배치만 글 뒤(BehindText).
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const [pkgDir, outDir] = process.argv.slice(2);
if (!pkgDir || !outDir) {
  console.error('usage: node make_inputs.mjs <web pkg dir> <out dir>');
  process.exit(2);
}
const root = resolve(new URL('../../..', import.meta.url).pathname);
const mod = await import(pathToFileURL(join(resolve(pkgDir), 'rhwp.js')).href);
mod.initSync({ module: readFileSync(join(resolve(pkgDir), 'rhwp_bg.wasm')) });
const jpg = readFileSync(join(root, 'rhwp-studio/public/samples/images/moogung.jpg'));
mkdirSync(outDir, { recursive: true });

for (const [name, textWrap] of [
  ['tac_picture_topandbottom_textbox.hwpx', 'TopAndBottom'],
  ['tac_picture_behindtext_textbox.hwpx', 'BehindText'],
]) {
  const d = mod.HwpDocument.createEmpty();
  d.createBlankDocument();
  d.insertText(0, 0, 0, '앞 글 뒤 글');
  d.splitParagraph(0, 0, 7);
  d.insertText(0, 1, 0, '다음 문단');
  const p = JSON.parse(d.insertPicture(0, 0, 2, '', jpg, 8504, 4447, 1000, 523, 'jpg', '그림'));
  d.setPictureProperties(0, p.paraIdx, p.controlIdx, JSON.stringify({ treatAsChar: true }));
  const pd = JSON.parse(d.getPageDef(0));
  const s = JSON.parse(d.createShapeControl(JSON.stringify({
    sectionIdx: 0, paraIdx: 0, charOffset: 7, width: 17008, height: 5669,
    horzOffset: pd.marginLeft, vertOffset: 15000, treatAsChar: false, textWrap, shapeType: 'textbox',
  })));
  d.insertTextInCell(0, s.paraIdx, s.controlIdx, 0, 0, 0, '중요');
  const bytes = d.exportHwpx();
  writeFileSync(join(outDir, name), bytes);
  console.log(`${name}\t${bytes.length} bytes`);
}
