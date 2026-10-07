// Issue #7621 재현 HWP 생성기 (native 자리 차지 표만 사용).
//
//   node samples/issue7621/generate.mjs <web pkg 폴더> <출력 폴더>
//
// <web pkg 폴더>는 `scripts/wasm-pack-locked.sh --target web` 산출물(rhwp.js·rhwp_bg.wasm)이다.
// 저장소의 HWP 는 devel d9749c5a1 의 패키지로 만들었다. 문서는 앞 문단 · 표 문단 · (native 생성이
// 넣는) 빈 문단 · 뒤 문단이고, b-text-after-float 는 그 빈 문단에 글자를 넣는다.
// 각 문서를 다시 열어 표 상자와 표 문단 다음 문단의 윗변(px, 96dpi)을 함께 출력한다.
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const [pkgDir, outDir] = process.argv.slice(2);
if (!pkgDir || !outDir) {
  console.error("usage: node generate.mjs <web pkg 폴더> <출력 폴더>");
  process.exit(2);
}
const pkg = await import(pathToFileURL(resolve(pkgDir, "rhwp.js")).href);
pkg.initSync({ module: readFileSync(resolve(pkgDir, "rhwp_bg.wasm")) });
const { HwpDocument } = pkg;

const BEFORE = "표 앞 문단입니다.";
const AFTER = "표 뒤 문단입니다.";
const ok = (json, what) => {
  const r = JSON.parse(json);
  if (r.ok === false) throw new Error(`${what}: ${json}`);
  return r;
};

function build({ textRightAfter }) {
  const doc = HwpDocument.createEmpty();
  doc.createBlankDocument();
  ok(doc.insertText(0, 0, 0, BEFORE), "앞 문단");
  ok(doc.splitParagraph(0, 0, [...BEFORE].length), "표 문단 분리");
  ok(doc.splitParagraph(0, 1, 0), "뒤 문단 분리");
  ok(doc.insertText(0, 2, 0, AFTER), "뒤 문단");
  const t = ok(
    doc.createTableEx(
      JSON.stringify({ sectionIdx: 0, paraIdx: 1, charOffset: 0, rowCount: 2, colCount: 2, treatAsChar: false }),
    ),
    "createTableEx",
  );
  ["가1", "나1", "가2", "나2"].forEach((s, c) => ok(doc.insertTextInCell(0, t.paraIdx, t.controlIdx, c, 0, 0, s), "셀"));
  if (textRightAfter) ok(doc.insertText(0, t.paraIdx + 1, 0, "표 바로 다음 문단입니다."), "다음 문단");
  return { bytes: doc.exportHwp(), table: t };
}

function geometry(bytes, para) {
  const tree = JSON.parse(new HwpDocument(bytes).getPageRenderTree(0));
  let table, next;
  (function walk(n) {
    if (n.type === "Table" && n.pi === para && !table) {
      table = n.bbox;
      return;
    }
    if (n.type === "TextLine" && n.pi === para + 1 && !next) next = n.bbox;
    (n.children || []).forEach(walk);
  })(tree);
  return { tableTop: table.y, tableBottom: +(table.y + table.h).toFixed(1), nextTop: next.y };
}

mkdirSync(outDir, { recursive: true });
for (const [name, options] of [
  ["b-text-after-float", { textRightAfter: true }],
  ["control-float", { textRightAfter: false }],
]) {
  const { bytes, table } = build(options);
  writeFileSync(join(outDir, `${name}.hwp`), bytes);
  console.log(name, JSON.stringify(geometry(bytes, table.paraIdx)));
}
