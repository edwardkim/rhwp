import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL, fileURLToPath } from 'node:url';

const root = path.dirname(fileURLToPath(import.meta.url));
const repository = path.resolve(process.cwd());
const fixtures = JSON.parse(fs.readFileSync(path.join(root, 'fixture-map.json'), 'utf8'));
const pkg = path.resolve(process.argv[2]);
const output = path.resolve(process.argv[3]);
const sourceSha = process.argv[4];
assert.match(sourceSha ?? '', /^[0-9a-f]{40}$/);
const wasmSha256 = createHash('sha256').update(fs.readFileSync(path.join(pkg,'rhwp_bg.wasm'))).digest('hex');
const { default: init, HwpDocument } = await import(pathToFileURL(path.join(pkg, 'rhwp.js')));
await init({ module_or_path: fs.readFileSync(path.join(pkg, 'rhwp_bg.wasm')) });
const json = value => typeof value === 'string' ? JSON.parse(value) : value;
const empty = () => { const doc = HwpDocument.createEmpty(); doc.createBlankDocument(); return doc; };
const load = (pr, name) => {
  const item = fixtures[`${pr}/${name}`];
  assert.ok(item, '등록한 실제 입력');
  const bytes = fs.readFileSync(path.join(repository, item.path));
  assert.equal(createHash('sha256').update(bytes).digest('hex'), item.sha256, '입력 SHA');
  return new HwpDocument(bytes);
};
for (const pr of [8526, 8528, 8541, 8558, 8564, 8570, 8571, 8572, 8586, 8602, 8616, 8629]) {
  const folder = path.join(output, `pr${pr}`);
  fs.mkdirSync(folder, { recursive: true });
  const observed = {};
  const result = { pr, sourceSha, wasmSha256, runtime: '지정한 제품 SHA의 실제 WASM 공개 API', observations: observed };
  const save = (doc, name = 'api-result') => {
    for (const [ext, bytes] of [['hwp', doc.exportHwp()], ['hwpx', doc.exportHwpx()]]) {
      const file = path.join(folder, `${name}.${ext}`);
      fs.writeFileSync(file, bytes);
      observed[`${name}_${ext}`] = file;
      assert.ok(new HwpDocument(bytes).pageCount() > 0, `${ext} 저장본 재열기`);
    }
  };
  try {
    if (pr === 8526) {
      const doc = empty();
      observed.before = json(doc.getStyleList());
      observed.index = doc.createStyle('{"name":"끼운 스타일","type":0,"nextStyleId":1,"index":1}');
      assert.equal(observed.index, 1, '선택한 스타일 뒤에 삽입');
      observed.after = json(doc.getStyleList());
      save(doc);
    }
    if (pr === 8528) {
      const doc = load(pr, 'follow-two-sizes.hwp');
      const id = doc.createNumbering('{"levelFormats":["^1."],"numberFormats":[0],"startNumber":1}');
      observed.charShape = doc.setNumberingCharShape(id, 0, '{"fontSize":0,"italic":true}');
      assert.ok(Number.isInteger(observed.charShape));
      for (const para of [0, 1]) doc.applyParaFormat(0, para, JSON.stringify({headType:'Number',paraLevel:0,numberingId:id}));
      observed.layout = json(doc.getPageTextLayout(0));
      save(doc, 'follow-body-size');
    }
    if (pr === 8541) {
      const doc = empty();
      doc.insertText(0, 0, 0, '앞뒤');
      observed.split = json(doc.splitSection(0, 0, 1));
      assert.equal(doc.getSectionCount(), 2);
      save(doc, 'split');
      observed.merge = json(doc.mergeSection(1));
      assert.equal(doc.getSectionCount(), 1);
      observed.mergedParagraphs = Array.from({length:doc.getParagraphCount(0)}, (_,para) => doc.getTextRange(0,para,0,doc.getParagraphLength(0,para)));
      assert.deepEqual(observed.mergedParagraphs, ['앞','뒤'], '구역 경계만 합치고 문단 소속·내용 순서를 보존');
      save(doc, 'merged');
    }
    if (pr === 8558) {
      for (const image of [false, true]) {
        const doc = empty(); doc.insertText(0, 0, 0, '사용자 정의 글머리표');
        const png = image ? fs.readFileSync(path.resolve('assets/logo/logo-16.png')) : undefined;
        const id = doc.createBullet('{"char":"❏","align":1,"autoIndent":false,"widthAdjust":1000,"textOffset":70,"charShape":{"fontSize":1000,"italic":true}}', png, image ? 'png' : undefined);
        assert.ok(id > 0);
        doc.applyParaFormat(0, 0, JSON.stringify({headType:'Bullet',paraLevel:0,numberingId:id}));
        save(doc, image ? 'picture-bullet' : 'character-bullet');
      }
    }
    if (pr === 8564) {
      const doc = load(pr, 'hancom-date-path-codes.hwpx');
      observed.before = json(doc.getFieldList());
      assert.equal(observed.before.length, 2, '원본의 날짜·경로 코드 두 개');
      assert.deepEqual(observed.before.map(x=>x.value), ['2026년 11월 4일','date-codes.hwpx']);
      for (const [para, offset, text] of [[0,13,'뒤'],[0,1,'앞'],[1,15,'뒤'],[1,0,'앞']]) doc.insertText(0, para, offset, text);
      observed.after = json(doc.getFieldList());
      assert.deepEqual(observed.after.map(x=>x.value), observed.before.map(x=>x.value), '코드 경계의 입력은 코드 밖에 남음');
      save(doc, 'code-boundary');
    }
    if (pr === 8570 || pr === 8571) {
      const doc = empty();
      let parent = 0, control;
      if (pr === 8570) control = json(doc.createTable(0,0,0,1,1)).controlIdx;
      else {
        doc.createHeaderFooter(0,true,0);
        observed.header = json(doc.getHeaderFooter(0,true,0));
        parent = observed.header.paraIndex; control = observed.header.controlIndex;
      }
      const cellPath = [{controlIndex:control,cellIndex:0,cellParaIndex:0}];
      observed.creation = json(doc.createTableInCellByPath(0,parent,JSON.stringify(cellPath),0,'{"rowCount":1,"colCount":2,"treatAsChar":false}'));
      assert.ok(Array.isArray(observed.creation.cellPath));
      save(doc, pr === 8570 ? 'non-inline-cell-table' : 'non-inline-header-table');
    }
    if (pr === 8572) {
      const doc = load(pr, 'footnote-01.hwp');
      const layout = json(doc.getPageControlLayout(0));
      const find = node => {
        if (!node || typeof node !== 'object') return undefined;
        if ((node.type === 'Table' || node.controlType === 'table') && (node.paraIndex === 0 || node.parentParaIndex === 0)) return node.controlIndex ?? node.controlIdx;
        for (const value of Object.values(node)) { const index = find(value); if (index !== undefined) return index; }
      };
      let control = find(layout);
      if (control === undefined) {
        const tree = json(doc.getPageRenderTree(0));
        const tables = [];
        const walk = node => { if (node?.type === 'Table') tables.push(node); for (const child of node?.children ?? []) walk(child); };
        walk(tree.root ?? tree);
        control = tables.find(node => node.pi === 0)?.ci;
      }
      assert.ok(Number.isInteger(control), '원본 첫 문단 제목 표 주소');
      observed.controlIndex = control;
      observed.edit = json(doc.insertTextInCell(0,0,control,0,1,0,'표안한글'));
      assert.equal(doc.pageCount(), 7, '독립 편집 Print의 7쪽과 일치');
      save(doc, 'cell-typed');
    }
    if (pr === 8586) {
      const doc = load(pr, 'hancom-page-number-mid.hwpx');
      observed.before = json(doc.getPageNumberPosition(0));
      observed.set = json(doc.setPageNumberPosition(0,5,0,true));
      observed.after = json(doc.getPageNumberPosition(0));
      save(doc, 'page-number-updated');
    }
    if (pr === 8602) {
      const doc = empty(); doc.insertText(0,0,0,'앞 문단'); doc.splitParagraph(0,0,4,undefined);
      const table = json(doc.createTable(0,1,0,2,2));
      observed.deleted = json(doc.deleteTableControl(0,1,table.controlIdx));
      save(doc, 'empty-deleted');
    }
    if (pr === 8616) {
      const doc = empty(); doc.insertText(0,0,0,'가나');
      observed.insert = json(doc.insertRuby(0,0,1,'{"mainText":"漢字","rubyText":"한자"}'));
      const index = observed.insert.controlIdx;
      doc.setComposeControl(0,0,index,'{"mainText":"漢字","rubyText":"한자","below":true,"styleId":1}');
      observed.value = json(doc.getComposeControl(0,0,index));
      assert.equal(observed.value.below,true);
      save(doc, 'ruby-edited');
      doc.deleteComposeControl(0,0,index,true);
      assert.equal(doc.getTextRange(0,0,0,100),'가漢字나');
      save(doc, 'ruby-deleted');
    }
    if (pr === 8629) {
      const doc = load(pr,'hancom-mac-200-b5.hwp');
      observed.section = json(doc.getSectionDef(0));
      assert.equal(observed.section.wonggojiFormat,true);
      assert.ok(Number.isInteger(observed.section.outlineNumberingId), '새 base의 개요 ID와 원고지 속성을 함께 조회');
      const retained = {
        outlineNumberingId: observed.section.outlineNumberingId,
        lineGrid: observed.section.lineGrid,
        charGrid: observed.section.charGrid,
        wonggojiFormat: observed.section.wonggojiFormat,
      };
      doc.setSectionDef(0, JSON.stringify(retained));
      observed.reappliedSection = json(doc.getSectionDef(0));
      for (const [key, value] of Object.entries(retained)) assert.equal(observed.reappliedSection[key],value,key);
      assert.equal(doc.pageCount(),1);
      save(doc, 'manuscript-roundtrip');
    }
    result.status = 'passed';
  } catch (error) { result.status = 'failed'; result.error = String(error); }
  fs.writeFileSync(path.join(folder,'claim-probe.json'),JSON.stringify(result,null,2));
  console.log(JSON.stringify({pr,status:result.status,error:result.error}));
  if (result.status !== 'passed') process.exitCode = 1;
}
