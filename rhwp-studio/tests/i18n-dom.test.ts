import test from 'node:test';
import assert from 'node:assert/strict';

import { registerCatalog, resetCatalogs, setLocale } from '../src/i18n/core.ts';
import { applyI18nToElement } from '../src/i18n/dom.ts';

const TEXT_NODE = 3;
const ELEMENT_NODE = 1;

interface FakeNode {
  nodeType: number;
  nodeValue: string | null;
  tagName?: string;
}

function textNode(value: string): FakeNode {
  return { nodeType: TEXT_NODE, nodeValue: value };
}

function elementNode(tagName: string): FakeNode {
  return { nodeType: ELEMENT_NODE, nodeValue: null, tagName };
}

/** dom.ts 가 실제로 쓰는 것만 흉내 낸다 — 저장소의 기존 테스트가 쓰는 방식. */
function fakeElement(attrs: Record<string, string>, childNodes: FakeNode[] = []) {
  const el = {
    attrs: { ...attrs },
    childNodes,
    written: {} as Record<string, string>,
    ownerDocument: {
      createElement: (tag: string) => elementNode(tag),
      createTextNode: (value: string) => textNode(value),
    },
    get textContent(): string {
      return el.childNodes.map((n) => n.nodeValue ?? '').join('');
    },
    set textContent(value: string) {
      el.childNodes = value === '' ? [] : [textNode(value)];
    },
    getAttribute: (name: string) => el.attrs[name] ?? null,
    setAttribute: (name: string, value: string) => {
      el.written[name] = value;
    },
    appendChild: (node: FakeNode) => {
      el.childNodes.push(node);
      return node;
    },
  };
  return el;
}

function withCatalogs(ko: Record<string, string>, en: Record<string, string>, fn: () => void) {
  resetCatalogs();
  registerCatalog('ko', ko);
  registerCatalog('en', en);
  setLocale('en');
  try {
    fn();
  } finally {
    resetCatalogs();
  }
}

test('자식 요소가 없으면 텍스트 전체를 바꾼다', () => {
  withCatalogs({ 'menu.file.open': '열기' }, { 'menu.file.open': 'Open' }, () => {
    const el = fakeElement({ 'data-i18n': 'menu.file.open' }, [textNode('열기')]);
    applyI18nToElement(el as never);
    assert.equal(el.textContent, 'Open');
  });
});

test('아이콘 자식이 있어도 직계 텍스트만 바꾼다 — 아이콘이 사라지면 안 된다', () => {
  withCatalogs({ 'command.edit.find.text': '찾기(F) ' }, { 'command.edit.find.text': 'Find ' }, () => {
    const icon = elementNode('span');
    const el = fakeElement({ 'data-i18n': 'command.edit.find.text' }, [textNode('찾기(F) '), icon]);
    applyI18nToElement(el as never);
    assert.equal(el.childNodes.length, 2);
    assert.equal(el.childNodes[0].nodeValue, 'Find ');
    assert.equal(el.childNodes[1], icon);
  });
});

test('텍스트가 요소 뒤에 오는 경우에도 그 텍스트만 바뀐다', () => {
  withCatalogs({ 'ui.charfxMenu.label': '양각' }, { 'ui.charfxMenu.label': 'Emboss' }, () => {
    const sample = elementNode('span');
    const el = fakeElement({ 'data-i18n': 'ui.charfxMenu.label' }, [sample, textNode('양각')]);
    applyI18nToElement(el as never);
    assert.equal(el.childNodes[0], sample);
    assert.equal(el.childNodes[1].nodeValue, 'Emboss');
  });
});

test('여러 줄 라벨은 <br> 로 재구성한다 — innerHTML 을 쓰지 않는다', () => {
  withCatalogs(
    { 'command.edit.cut.toolbarLabel': '오려\n두기' },
    { 'command.edit.cut.toolbarLabel': 'Cut' },
    () => {
      const el = fakeElement({ 'data-i18n-lines': 'command.edit.cut.toolbarLabel' }, [
        textNode('오려'),
        elementNode('br'),
        textNode('두기'),
      ]);
      applyI18nToElement(el as never);
      assert.deepEqual(
        el.childNodes.map((n) => (n.nodeType === TEXT_NODE ? n.nodeValue : `<${n.tagName}>`)),
        ['Cut'],
      );
    },
  );
});

test('여러 줄 값은 줄 수만큼 <br> 을 만든다', () => {
  withCatalogs({ 'x.y': '가\n나' }, { 'x.y': 'One\nTwo' }, () => {
    const el = fakeElement({ 'data-i18n-lines': 'x.y' }, [textNode('가'), elementNode('br'), textNode('나')]);
    applyI18nToElement(el as never);
    assert.deepEqual(
      el.childNodes.map((n) => (n.nodeType === TEXT_NODE ? n.nodeValue : `<${n.tagName}>`)),
      ['One', '<br>', 'Two'],
    );
  });
});

test('title·aria-label·placeholder·value 속성을 바꾼다', () => {
  withCatalogs(
    { 'a.tooltip': '찾기 (Ctrl+F)', 'a.ariaLabel': '주 메뉴' },
    { 'a.tooltip': 'Find (Ctrl+F)', 'a.ariaLabel': 'Main menu' },
    () => {
      const el = fakeElement({
        'data-i18n-title': 'a.tooltip',
        'data-i18n-aria-label': 'a.ariaLabel',
      });
      applyI18nToElement(el as never);
      assert.equal(el.written['title'], 'Find (Ctrl+F)');
      assert.equal(el.written['aria-label'], 'Main menu');
    },
  );
});

test('번역이 없으면 원문이 그대로 들어간다', () => {
  withCatalogs({ 'menu.file.open': '열기' }, {}, () => {
    const el = fakeElement({ 'data-i18n': 'menu.file.open' }, [textNode('열기')]);
    applyI18nToElement(el as never);
    assert.equal(el.textContent, '열기');
  });
});

// ── 실행 중에 바뀐 표시는 덮어쓰지 않는다 (이슈 #5852 검토 ①) ──────────────────────────
// 정적 마크업의 원문은 '초기 문구' 일 뿐이다. 앱이 그 자리에 현재 상태(구역 2/3, 수정 모드)를
// 써 넣은 뒤 로케일 적용이 다시 돌면, 원문을 번역한 초기 문구가 현재 상태를 지운다.
// 그래서 **요소가 아직 원문을 보이고 있을 때만** 번역으로 바꾼다.

test('앱이 바꿔 둔 상태 표시는 그대로 둔다 — 구역: 2 / 3 이 Section: 1 / 1 로 돌아가면 안 된다', () => {
  withCatalogs({ 'ui.sbSection.label': '구역: 1 / 1' }, { 'ui.sbSection.label': 'Section: 1 / 1' }, () => {
    const el = fakeElement({ 'data-i18n': 'ui.sbSection.label' }, [textNode('구역: 2 / 3')]);
    applyI18nToElement(el as never);
    assert.equal(el.textContent, '구역: 2 / 3');
  });
});

test('수정 모드 표시가 초기 문구(삽입)의 번역으로 덮이지 않는다', () => {
  withCatalogs({ 'ui.sbMode.label': '삽입' }, { 'ui.sbMode.label': 'Insert' }, () => {
    const el = fakeElement({ 'data-i18n': 'ui.sbMode.label' }, [textNode('수정')]);
    applyI18nToElement(el as never);
    assert.equal(el.textContent, '수정');
  });
});

test('마크업 들여쓰기 공백이 있어도 원문이면 바꾼다', () => {
  withCatalogs({ 'menu.file': '파일' }, { 'menu.file': 'File' }, () => {
    const el = fakeElement({ 'data-i18n': 'menu.file' }, [textNode('\n      파일\n    ')]);
    applyI18nToElement(el as never);
    assert.equal(el.textContent, 'File');
  });
});

test('두 번 적용해도 같다(멱등) — 이미 번역된 글자는 그대로 둔다', () => {
  withCatalogs({ 'menu.file': '파일' }, { 'menu.file': 'File' }, () => {
    const el = fakeElement({ 'data-i18n': 'menu.file' }, [textNode('파일')]);
    applyI18nToElement(el as never);
    applyI18nToElement(el as never);
    assert.equal(el.textContent, 'File');
  });
});

test('앱이 바꿔 둔 속성(title)은 그대로 둔다', () => {
  withCatalogs({ 'a.tooltip': '찾기' }, { 'a.tooltip': 'Find' }, () => {
    const el = fakeElement({ 'data-i18n-title': 'a.tooltip', title: '찾기 — 3건 일치' });
    applyI18nToElement(el as never);
    assert.equal(el.written['title'], undefined);
  });
});

test('앱이 바꿔 둔 여러 줄 라벨도 그대로 둔다', () => {
  withCatalogs({ 'x.lines': '오려\n두기' }, { 'x.lines': 'Cut' }, () => {
    const el = fakeElement({ 'data-i18n-lines': 'x.lines' }, [textNode('붙여'), elementNode('br'), textNode('넣기')]);
    applyI18nToElement(el as never);
    assert.deepEqual(el.childNodes.map((n) => n.nodeValue ?? `<${n.tagName}>`), ['붙여', '<br>', '넣기']);
  });
});


test('직계 텍스트 없이 아이콘만 있는 요소도 자식을 보존한다', () => {
  withCatalogs({ 'menu.file.open': '열기' }, { 'menu.file.open': 'Open' }, () => {
    const icon = elementNode('span');
    const el = fakeElement({ 'data-i18n': 'menu.file.open' }, [textNode('  '), icon]);
    applyI18nToElement(el as never);
    assert.equal(el.childNodes[1], icon);
    assert.equal(el.textContent.trim(), 'Open');
    applyI18nToElement(el as never);
    assert.equal(el.childNodes[1], icon);
    assert.equal(el.textContent.trim(), 'Open');
  });
});
