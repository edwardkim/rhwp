// Issue #7620 후속 — 본문 바닥 경계의 RowBreak 자리 차지 표 재현 HWP 생성기.
//
//   node samples/issue7620/generate-budget.mjs <web pkg 폴더> <출력 폴더>
//
// <web pkg 폴더>는 `scripts/wasm-pack-locked.sh --target web` 산출물(rhwp.js·rhwp_bg.wasm)이다.
// 문서는 앞 문단 N 개 · 표 문단 · (native 생성이 넣는) 빈 문단 · 뒤 문단이다. 표는 R×2,
// 비TAC, 자리 차지, 문단 기준 오프셋 0, 쪽 경계 나눔 RowBreak(2), 바깥 여백 위·아래 2835HU(1cm)다.
// 바깥 여백을 크게 잡아 `표 본체는 본문에 들어가지만 바깥 아래 여백까지는 들어가지 않는` 구간을
// 넓힌다. N·R 만 다르고 나머지 속성은 같다. 다시 열어 쪽별 표 상자와 표 뒤 두 문단의 윗변(px, 96dpi)을
// 출력한다.
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const [pkgDir, outDir] = process.argv.slice(2);
if (!pkgDir || !outDir) {
  console.error("usage: node generate-budget.mjs <web pkg 폴더> <출력 폴더>");
  process.exit(2);
}
const pkg = await import(pathToFileURL(resolve(pkgDir, "rhwp.js")).href);
pkg.initSync({ module: readFileSync(resolve(pkgDir, "rhwp_bg.wasm")) });
const { HwpDocument } = pkg;

const OUTER_HU = 2835;
const AFTER = "표 뒤 문단입니다.";
// 이름 → [앞 문단 수, 행 수]. 기본 A4 본문(높이 876.8px)에서 앞 문단 한 줄은 21.3px, 표 한 행은 17.1px 이다.
const CASES = {
  // 표 본체는 1쪽 본문에 들어가지만 바깥 아래 여백은 본문 바닥을 넘는다 — 첫 행도 아래 여백과 함께 들지 않는다.
  "b-rowbreak-bottom-margin-overflow": [37, 2],
  // 표 문단의 앵커 줄은 1쪽에 들어가지만 바깥 위 여백 + 표 본체가 들어가지 않는다.
  "c-rowbreak-table-moves-next-page": [39, 2],
  // 대조군: 쪽 중간에서 시작해 첫 행들이 아래 여백과 함께 들어가므로 행 경계에서 나뉜다.
  "d-rowbreak-splits-at-row-boundary": [20, 40],
};
const ok = (json, what) => {
  const r = JSON.parse(json);
  if (r.ok === false) throw new Error(`${what}: ${json}`);
  return r;
};

function build(n, rows) {
  const doc = HwpDocument.createEmpty();
  doc.createBlankDocument();
  for (let i = 0; i < n; i++) {
    const s = `앞 문단 ${i + 1}`;
    ok(doc.insertText(0, i, 0, s), "앞 문단");
    ok(doc.splitParagraph(0, i, [...s].length), "문단 분리");
  }
  ok(doc.splitParagraph(0, n, 0), "뒤 문단 분리");
  ok(doc.insertText(0, n + 1, 0, AFTER), "뒤 문단");
  const t = ok(
    doc.createTableEx(
      JSON.stringify({ sectionIdx: 0, paraIdx: n, charOffset: 0, rowCount: rows, colCount: 2, treatAsChar: false }),
    ),
    "createTableEx",
  );
  Array.from({ length: rows * 2 }, (_, c) => `${c % 2 ? "나" : "가"}${(c >> 1) + 1}`).forEach((s, c) => ok(doc.insertTextInCell(0, t.paraIdx, t.controlIdx, c, 0, 0, s), "셀"));
  ok(
    doc.setTableProperties(
      0,
      t.paraIdx,
      t.controlIdx,
      JSON.stringify({ pageBreak: 2, outerTop: OUTER_HU, outerBottom: OUTER_HU }),
    ),
    "RowBreak·바깥 여백",
  );
  return { bytes: doc.exportHwp(), host: t.paraIdx };
}

function geometry(bytes, host) {
  const doc = new HwpDocument(bytes);
  const pages = [];
  for (let p = 0; p < doc.pageCount(); p++) {
    const tree = JSON.parse(doc.getPageRenderTree(p));
    const tables = [];
    const lines = {};
    (function walk(n, inTable) {
      if (n.type === "Table" && n.pi === host && !inTable) {
        tables.push([+n.bbox.y.toFixed(1), +(n.bbox.y + n.bbox.h).toFixed(1)]);
        inTable = true;
      }
      if (!inTable && n.type === "TextLine" && n.pi > host && lines[n.pi] === undefined) lines[n.pi] = +n.bbox.y.toFixed(1);
      (n.children || []).forEach((c) => walk(c, inTable));
    })(tree, false);
    if (tables.length || Object.keys(lines).length) pages.push({ page: p + 1, tables, lines });
  }
  return pages;
}

mkdirSync(outDir, { recursive: true });
for (const [name, [n, rows]] of Object.entries(CASES)) {
  const { bytes, host } = build(n, rows);
  writeFileSync(join(outDir, `${name}.hwp`), bytes);
  console.log(name, JSON.stringify({ host, pages: geometry(bytes, host) }));
}
