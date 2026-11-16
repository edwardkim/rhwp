// Issue #7620 후속 — 쪽 바닥에 걸린 여러 줄 행의 CellBreak/RowBreak 재현 HWP 생성기.
//
//   node samples/issue7620/generate-cellbreak.mjs <web pkg 폴더> <출력 폴더> [앞 문단 수]
//
// <web pkg 폴더>는 `scripts/wasm-pack-locked.sh --target web` 산출물(rhwp.js·rhwp_bg.wasm)이다.
// 문서는 앞 문단 N 개 · 표 문단 · (native 생성이 넣는) 빈 문단 · 뒤 문단이다. 표는 ROWS×2, 비TAC,
// 자리 차지, 문단 기준 오프셋 0, 바깥 여백 기본값이다. 각 셀에는 LINES 개 문단(한 줄씩)을 넣어
// 행 하나가 여러 줄 높이를 갖게 한다. 두 문서는 쪽 경계 나눔만 다르다:
//   e — 셀 단위로 나눔(CellBreak, 1): 쪽 바닥에 걸린 행 전체가 다음 쪽으로 간다고 본다.
//   f — 나눔(RowBreak, 2): 걸린 행의 글줄부터 다음 쪽이 이어받는다고 본다(대조군).
// 다시 열어 쪽별 표 상자, 셀 글줄의 쪽 소속, 표 뒤 문단의 윗변(px, 96dpi)을 출력한다.
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const [pkgDir, outDir, nArg] = process.argv.slice(2);
if (!pkgDir || !outDir) {
  console.error("usage: node generate-cellbreak.mjs <web pkg 폴더> <출력 폴더> [앞 문단 수]");
  process.exit(2);
}
const pkg = await import(pathToFileURL(resolve(pkgDir, "rhwp.js")).href);
pkg.initSync({ module: readFileSync(resolve(pkgDir, "rhwp_bg.wasm")) });
const { HwpDocument } = pkg;

const N = nArg === undefined ? 30 : Number(nArg);
const ROWS = 3;
const LINES = 6;
const AFTER = "표 뒤 문단입니다.";
const CASES = {
  "e-cellbreak-row-moves-next-page": 1,
  "f-rowbreak-row-splits-at-line": 2,
};
const ok = (json, what) => {
  const r = JSON.parse(json);
  if (r.ok === false) throw new Error(`${what}: ${json}`);
  return r;
};

function build(pageBreak) {
  const doc = HwpDocument.createEmpty();
  doc.createBlankDocument();
  for (let i = 0; i < N; i++) {
    const s = `앞 문단 ${i + 1}`;
    ok(doc.insertText(0, i, 0, s), "앞 문단");
    ok(doc.splitParagraph(0, i, [...s].length), "문단 분리");
  }
  ok(doc.splitParagraph(0, N, 0), "뒤 문단 분리");
  ok(doc.insertText(0, N + 1, 0, AFTER), "뒤 문단");
  const t = ok(
    doc.createTableEx(
      JSON.stringify({ sectionIdx: 0, paraIdx: N, charOffset: 0, rowCount: ROWS, colCount: 2, treatAsChar: false }),
    ),
    "createTableEx",
  );
  for (let c = 0; c < ROWS * 2; c++) {
    const tag = `${c % 2 ? "나" : "가"}${(c >> 1) + 1}`;
    for (let l = 0; l < LINES; l++) {
      const s = `${tag}-${l + 1}`;
      ok(doc.insertTextInCell(0, t.paraIdx, t.controlIdx, c, l, 0, s), "셀 글");
      if (l + 1 < LINES) ok(doc.splitParagraphInCell(0, t.paraIdx, t.controlIdx, c, l, [...s].length), "셀 문단 분리");
    }
  }
  ok(doc.setTableProperties(0, t.paraIdx, t.controlIdx, JSON.stringify({ pageBreak })), "쪽 경계 나눔");
  return { bytes: doc.exportHwp(), host: t.paraIdx };
}

function geometry(bytes, host) {
  const doc = new HwpDocument(bytes);
  const pages = [];
  for (let p = 0; p < doc.pageCount(); p++) {
    const tree = JSON.parse(doc.getPageRenderTree(p));
    const tables = [];
    const cellText = [];
    const lines = {};
    (function walk(n, inTable) {
      if (n.type === "Table" && n.pi === host && !inTable) {
        tables.push([+n.bbox.y.toFixed(1), +(n.bbox.y + n.bbox.h).toFixed(1)]);
        inTable = true;
      }
      if (inTable && n.type === "TextRun" && n.text) cellText.push(n.text);
      if (!inTable && n.type === "TextLine" && n.pi > host && lines[n.pi] === undefined) lines[n.pi] = +n.bbox.y.toFixed(1);
      (n.children || []).forEach((c) => walk(c, inTable));
    })(tree, false);
    if (tables.length || Object.keys(lines).length)
      pages.push({ page: p + 1, tables, cells: cellText.join(" "), lines });
  }
  return { pageCount: doc.pageCount(), pages };
}

mkdirSync(outDir, { recursive: true });
for (const [name, pageBreak] of Object.entries(CASES)) {
  const { bytes, host } = build(pageBreak);
  writeFileSync(join(outDir, `${name}.hwp`), bytes);
  console.log(name, JSON.stringify({ n: N, host, ...geometry(bytes, host) }));
}
