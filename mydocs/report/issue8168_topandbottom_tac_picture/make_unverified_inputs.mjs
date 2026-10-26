// #8168 미검증 경우의 합성 입력 생성기 — 편집 API 로 만들고 HWP 로 저장한다.
//
// 사용: node make_unverified_inputs.mjs <web pkg 디렉터리> <출력 디렉터리>
//   예: node mydocs/report/issue8168_topandbottom_tac_picture/make_unverified_inputs.mjs pkg samples/issue8168
//
// HWP 로 저장한다. HWPX 저장은 편집한 문단의 linesegarray 를 빼므로(한/글이 다시 계산하게 하는 설계)
// rhwp 가 다시 열면 저장 줄 정보 없이 재조판한다 — 이 입력이 시험할 저장 줄 경로(`para_line_top_hu`)를
// 거치지 않는다. 이슈 당시 한/글이 거부하던 HWP 저장본은 #8172(글상자 저장 비트)로 열린다.
// 저장 LineSeg 는 rhwp 재조판 결과다 — 한/글 Print PDF 가 독립 기준이다.
//
// 1. page_relative: 이슈 재현과 같고, 위아래 글상자만 쪽 기준(VertRelTo::Page) 세로 15000HU.
//    #8206 은 종이 기준과 같은 갈래(그림을 밀지 않음)로 처리한다.
// 2. multiline_para_offset_above: 세 줄 문단의 둘째 줄에 글자처럼 그림, 같은 문단에 문단 기준
//    위아래 글상자(세로 300HU — 0 보다 크고 그림 줄 위끝보다 위). #8206 은 그림을 글상자 높이만큼 민다.
// 3. multiline_para_offset_below: 2 와 같고 글상자 세로 오프셋만 그림 줄 아래(9000HU — 문단 끝보다도
//    아래). #8206 은 그림을 밀지 않는다(비교용). 실제 줄 위끝은 저장본의 LineSeg 로 기록한다.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const [pkgDir, outDir] = process.argv.slice(2);
if (!pkgDir || !outDir) {
  console.error('usage: node make_unverified_inputs.mjs <web pkg dir> <out dir>');
  process.exit(2);
}
const root = resolve(new URL('../../..', import.meta.url).pathname);
const mod = await import(pathToFileURL(join(resolve(pkgDir), 'rhwp.js')).href);
mod.initSync({ module: readFileSync(join(resolve(pkgDir), 'rhwp_bg.wasm')) });
const jpg = readFileSync(join(root, 'rhwp-studio/public/samples/images/moogung.jpg'));
mkdirSync(outDir, { recursive: true });

const SHORT = '앞 글 뒤 글';
// 본문 폭(A4, 기본 여백)에서 세 줄로 나뉘는 길이. 그림은 둘째 줄 가운데쯤에 든다.
const LONG = '이 문단은 여러 줄로 나뉘도록 길게 쓴 문장입니다. 첫째 줄은 글만 있고, '
  + '둘째 줄 가운데에 글자처럼 취급한 그림이 들어갑니다. 셋째 줄도 글만 이어지며 '
  + '같은 문단에 위아래로 배치한 글상자가 함께 앵커되어 있습니다. 끝.';
const PIC_AT_LONG = 60;

function base(text) {
  const d = mod.HwpDocument.createEmpty();
  d.createBlankDocument();
  d.insertText(0, 0, 0, text);
  d.splitParagraph(0, 0, [...text].length);
  d.insertText(0, 1, 0, '다음 문단');
  return d;
}

function addTacPicture(d, at) {
  const p = JSON.parse(d.insertPicture(0, 0, at, '', jpg, 8504, 4447, 1000, 523, 'jpg', '그림'));
  d.setPictureProperties(0, p.paraIdx, p.controlIdx, JSON.stringify({ treatAsChar: true }));
}

function addFloat(d, at, vertOffset, vertRelTo) {
  const pd = JSON.parse(d.getPageDef(0));
  const s = JSON.parse(d.createShapeControl(JSON.stringify({
    sectionIdx: 0, paraIdx: 0, charOffset: at, width: 17008, height: 5669,
    horzOffset: pd.marginLeft, vertOffset, treatAsChar: false,
    textWrap: 'TopAndBottom', shapeType: 'textbox',
  })));
  d.setShapeProperties(0, s.paraIdx, s.controlIdx, JSON.stringify({ vertRelTo, vertOffset }));
  d.insertTextInCell(0, s.paraIdx, s.controlIdx, 0, 0, 0, '중요');
  return s;
}

function save(d, name) {
  const bytes = d.exportHwp();
  writeFileSync(join(outDir, name), bytes);
  console.log(`${name}\t${bytes.length} bytes`);
}

{
  const d = base(SHORT);
  addTacPicture(d, 2);
  addFloat(d, 7, 15000, 'Page');
  save(d, 'tac_picture_with_page_relative_topandbottom_textbox.hwp');
}

for (const [name, offset] of [
  ['multiline_tac_picture_para_topandbottom_offset_above.hwp', 300],
  ['multiline_tac_picture_para_topandbottom_offset_below.hwp', 9000],
]) {
  const d = base(LONG);
  addTacPicture(d, PIC_AT_LONG);
  const line = JSON.parse(d.getLineInfo(0, 0, PIC_AT_LONG));
  if (line.lineCount !== 3 || line.lineIndex !== 1) throw new Error(`그림이 세 줄 문단의 둘째 줄에 있어야 한다: ${JSON.stringify(line)}`);
  addFloat(d, [...LONG].length, offset, 'Para');
  save(d, name);
}
