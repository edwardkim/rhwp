// #8282 합성 입력 생성기 — 편집 API 로 만들고 HWPX 로 저장한다.
//
// 사용: node make_inputs.mjs <web pkg 디렉터리> <출력 디렉터리>
//   예: node mydocs/report/issue8282_hwpx_reopen_tac_picture_square/make_inputs.mjs pkg samples/issue8282
//
// HWPX 저장은 편집한 host 문단의 linesegarray 를 뺀다. 그래서 다시 여는 쪽(rhwp·한/글)이 그 문단을
// 저장 줄 없이 재조판한다 — 이 입력이 시험하는 경로다.
//
// 1. tac_picture_square_textbox: #8271 입력과 같은 편집에서 글상자 배치만 어울림(Square).
//    글상자는 종이 기준 세로 15000HU 로 host 줄 아래에 있어 줄 폭과 겹치지 않는다(이슈 재현).
// 2. tac_picture_square_textbox_overlap: 세 줄로 나뉘는 긴 host 문단, 어울림 글상자를 종이 기준
//    가로 34016HU·세로 9000HU 에 두어 그림 줄(첫 줄)과 둘째 줄의 오른쪽을 덮는다.
// 3. tac_picture_square_picture_overlap: 2 와 같고 부동 개체만 어울림 그림(같은 크기·위치) — 비교용.
// 4. tac_picture_square_textbox_left_overlap (#8399 변형 D): 180자 host 문단, 어울림 글상자를 종이 기준
//    가로 8504HU(본문 왼쪽)·세로 13000HU 에 두어 host 줄 0–3 의 왼쪽을 덮는다.
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

const SHORT = '앞 글 뒤 글';
const LONG = '앞 글 뒤에 이어지는 이 문단은 어울림 글상자 옆으로 여러 줄에 걸쳐 흐르도록 길게 쓴 문장입니다. '
  + '글상자가 줄과 겹치는 세로 범위에서는 줄 폭이 줄어듭니다. 끝.';
const UNIT = '어울림 글상자가 본문 왼쪽에서 여러 줄과 겹치면 한글은 글상자 오른쪽 남은 폭으로 줄을 다시 나눈다. ';
const LEFT = [...UNIT.repeat(4)].slice(0, 179).join('') + '끝';

function build(text, float, horzOffset, vertOffset) {
  const d = mod.HwpDocument.createEmpty();
  d.createBlankDocument();
  const end = [...text].length;
  d.insertText(0, 0, 0, text);
  d.splitParagraph(0, 0, end);
  d.insertText(0, 1, 0, '다음 문단');
  const p = JSON.parse(d.insertPicture(0, 0, 2, '', jpg, 8504, 4447, 1000, 523, 'jpg', '그림'));
  d.setPictureProperties(0, p.paraIdx, p.controlIdx, JSON.stringify({ treatAsChar: true }));
  if (float === 'textbox') {
    const s = JSON.parse(d.createShapeControl(JSON.stringify({
      sectionIdx: 0, paraIdx: 0, charOffset: end + 1, width: 17008, height: 5669,
      horzOffset, vertOffset, treatAsChar: false, textWrap: 'Square', shapeType: 'textbox',
    })));
    d.insertTextInCell(0, s.paraIdx, s.controlIdx, 0, 0, 0, '중요');
  } else {
    const f = JSON.parse(d.insertPicture(0, 0, end + 1, '', jpg, 17008, 5669, 1000, 523, 'jpg', '부동'));
    d.setPictureProperties(0, f.paraIdx, f.controlIdx, JSON.stringify({
      treatAsChar: false, textWrap: 'Square', vertRelTo: 'Paper', horzRelTo: 'Paper', vertOffset, horzOffset,
    }));
  }
  return d;
}

function save(d, name) {
  const bytes = d.exportHwpx();
  writeFileSync(join(outDir, name), bytes);
  console.log(`${name}\t${bytes.length} bytes`);
}

const pd = JSON.parse(build(SHORT, 'textbox', 0, 0).getPageDef(0));
save(build(SHORT, 'textbox', pd.marginLeft, 15000), 'tac_picture_square_textbox.hwpx');
save(build(LONG, 'textbox', 34016, 9000), 'tac_picture_square_textbox_overlap.hwpx');
save(build(LONG, 'picture', 34016, 9000), 'tac_picture_square_picture_overlap.hwpx');
save(build(LEFT, 'textbox', 8504, 13000), 'tac_picture_square_textbox_left_overlap.hwpx');
