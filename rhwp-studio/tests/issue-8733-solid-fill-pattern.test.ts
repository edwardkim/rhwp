import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import vm from 'node:vm';

// 실제 다이얼로그의 생성·채우기·확인 메서드를 실행한다. select는 없는 값을 빈 문자열로 만든다.
class Element {
  children: Element[] = [];
  style: Record<string, string> = {};
  dataset: Record<string, string> = {};
  classList = { add() {}, remove() {}, contains() { return false; }, toggle() {} };
  checked = false;
  hidden = false;
  textContent = '';
  private selected = '';
  tag: string;
  constructor(tag = 'div') { this.tag = tag; }
  get value() { return this.selected; }
  set value(value: string) {
    this.selected = this.tag !== 'select' || this.children.some(o => o.value === value) ? value : '';
  }
  appendChild(child: Element) {
    this.children.push(child);
    if (this.tag === 'select' && this.children.length === 1) this.selected = child.value;
    return child;
  }
  append(...children: Element[]) { children.forEach(child => this.appendChild(child)); }
  addEventListener() {}
  setAttribute() {}
}
const noop = () => {};
const input = () => Object.assign(new Element('input'), { value: '0' });
const fields = () => new Proxy({}, { get(target: any, key) { return target[key] ??= input(); } });
const context = vm.createContext({
  ModalDialog: class {}, console, i18nText: (s: string) => s, t: (s: string) => s,
  document: { createElement: (tag: string) => new Element(tag), createTextNode: () => new Element() },
});
function load(file: string, name: string) {
  const text = readFileSync(new URL('../src/ui/' + file + '.ts', import.meta.url), 'utf8')
    .replace(/^import\s[\s\S]*?\sfrom\s+['"][^'"]+['"];\s*$/gm, '');
  const source = stripTypeScriptTypes(text, { mode: 'transform' }).replace(/\bexport\s+/g, '');
  return vm.runInContext('{' + source + ';' + name + '}', context);
}
function setup(file: string, name: string, patternType: number | undefined, fillType = 'solid') {
  const C = load(file, name);
  const target = Object.create(C.prototype);
  const d: any = new Proxy(target, { get(o, key) { return key in o ? o[key] : o[key] = input(); } });
  const props = { fillType, fillColor: '#aabbcc', patternColor: '#112233', patternType, borderFillId: 3 };
  const calls: any[] = [];
  Object.assign(target, {
    tableCtx: { sec: 0, ppi: 0, ci: 0 }, cellIdx: 0, cellProps: { ...props }, tableProps: { ...props },
    mode: 'table', bgTarget: 'cell', borderTarget: 'cell', applyMode: 'each', selectionRange: null,
    services: undefined, activeTabId: 'border', borderEdits: Array.from({ length: 4 }, () => ({ type: 1, width: 0, color: '#000000' })),
    borderPaintSides: [true, false, false, false], borderScopeRadios: [], bgScopeRadios: [], diagScopeRadios: [],
    cellVAlignBtns: [], cellTextDirBtns: [], captionPosBtns: [],
    cellPaddingInputs: fields(), tablePaddingInputs: fields(), marginOuterInputs: fields(),
    buildScopeSection: () => new Element(), updateBorderPreview: noop, normalizeDiagonalExclusive: noop,
    updateDiagonalButtons: noop, updateDiagonalPreview: noop, getSelectedWrap: () => 'TopAndBottom',
    eventBus: { emit: noop },
    wasm: {
      runInBatch: (f: () => void) => f(),
      setCellProperties: (...args: any[]) => calls.push({ target: 'cell', props: structuredClone(args[4]) }),
      setTableProperties: (...args: any[]) => calls.push({ target: 'table', props: structuredClone(args[3]) }),
    },
  });
  C.prototype.buildBackgroundTab.call(d);
  d.bgTarget = 'cell';
  const populate = () => file === 'cell-border-bg-dialog' ? C.prototype.populateFields.call(d) : C.prototype.populateBgFromTarget.call(d);
  populate();
  const confirm = () => { calls.length = 0; C.prototype.onConfirm.call(d); return calls.find(c => c.target === d.bgTarget)?.props ?? calls[0].props; };
  return { d, populate, confirm };
}
for (const [file, name] of [['cell-border-bg-dialog', 'CellBorderBgDialog'], ['table-cell-props-dialog', 'TableCellPropsDialog']]) {
  test(name + ': 새 단색과 무늬 없음은 -1을 보낸다', () => {
    for (const initial of [undefined, -1]) {
      const { d, confirm } = setup(file, name, initial);
      assert.equal(d.bgPatternTypeSelect.value, '-1');
      assert.equal(confirm().patternType, -1);
      assert.equal(d.bgPreviewBox.style.background, '#aabbcc');
    }
  });
  test(name + ': 무늬 0~5는 테두리만 확인해도 보존된다', () => {
    // HWP 무늬 종류는 0~5다 (#8978). 0(가로줄)도 보이는 선택지다.
    assert.deepEqual(
      setup(file, name, -1).d.bgPatternTypeSelect.children.map((o: Element) => o.value),
      ['-1', '0', '1', '2', '3', '4', '5'],
    );
    for (const pattern of [0, 1, 3, 5]) {
      const { d, confirm } = setup(file, name, pattern);
      assert.equal(d.bgPatternTypeSelect.value, String(pattern));
      assert.equal(d.bgPatternTypeSelect.children[0].value, '-1');
      const submitted = confirm();
      assert.equal(submitted.patternType, pattern);
      assert.equal(submitted.patternColor, '#112233');
      assert.ok(Number.isFinite(submitted.patternType));
      if (pattern === 0) assert.match(d.bgPreviewBox.style.background, /repeating-linear-gradient/);
    }
  });
  test(name + ': 원본 무늬를 명시적으로 없애면 -1이다', () => {
    const { d, confirm } = setup(file, name, 0);
    d.bgPatternTypeSelect.value = '-1';
    assert.equal(confirm().patternType, -1);
  });
  test(name + ': 배경 없는 대상 재사용 후 새 단색에 이전 무늬가 남지 않는다', () => {
    const { d, populate, confirm } = setup(file, name, 0);
    d.cellProps = { fillType: 'none' };
    populate();
    d.bgColorRadio.checked = true;
    assert.equal(confirm().patternType, -1);
  });
  if (file === 'table-cell-props-dialog') test(name + ': 표 배경과 셀 배경 전환도 같은 sentinel을 보존한다', () => {
    const { d, populate, confirm } = setup(file, name, 0);
    d.bgTarget = 'table'; d.tableProps = { fillType: 'none' }; populate(); d.bgColorRadio.checked = true;
    assert.equal(confirm().patternType, -1);
    d.bgTarget = 'cell'; populate(); assert.equal(confirm().patternType, 0);
  });
}
