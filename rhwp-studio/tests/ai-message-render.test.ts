import test from 'node:test';
import assert from 'node:assert/strict';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { parseInline, parseMarkdown, renderBlocks } from '../src/ai/ui/message-render.ts';

test('굵게·기울임·인라인 코드·링크를 해석한다', () => {
  assert.deepEqual(parseInline('**굵게** 와 *기울임* 과 `코드` [글](https://x.y/z)'), [
    { type: 'strong', children: [{ type: 'text', text: '굵게' }] },
    { type: 'text', text: ' 와 ' },
    { type: 'em', children: [{ type: 'text', text: '기울임' }] },
    { type: 'text', text: ' 과 ' },
    { type: 'code', text: '코드' },
    { type: 'text', text: ' 글 (https://x.y/z)' },
  ]);
  assert.deepEqual(parseInline('2 * 3 * 4'), [{ type: 'text', text: '2 * 3 * 4' }], '공백을 둔 별표는 그대로');
});

test('문단·제목·목록·코드 블록을 나눈다', () => {
  const blocks = parseMarkdown('# 요약\n첫 줄\n둘째 줄\n\n- 가\n- 나\n1. 하나\n```\n<b>코드</b>\n```\n끝');
  assert.deepEqual(blocks.map((b) => b.type), ['heading', 'paragraph', 'list', 'list', 'code', 'paragraph']);
  assert.equal((blocks[1] as { lines: unknown[] }).lines.length, 2);
  assert.equal((blocks[2] as { ordered: boolean }).ordered, false);
  assert.equal((blocks[3] as { ordered: boolean }).ordered, true);
  assert.deepEqual(blocks[4], { type: 'code', text: '<b>코드</b>' });
});

test('닫히지 않은 코드 블록(스트리밍 중)도 끝까지 코드로 본다', () => {
  assert.deepEqual(parseMarkdown('앞\n```\nlet a = 1;'), [
    { type: 'paragraph', lines: [[{ type: 'text', text: '앞' }]] },
    { type: 'code', text: 'let a = 1;' },
  ]);
});

/** 만든 요소 이름과 텍스트만 기록하는 최소 DOM. */
function fakeDocument() {
  const created: string[] = [];
  class Node {
    children: Node[] = [];
    text = '';
    readonly tag: string;
    readonly ownerDocument: unknown;
    constructor(tag: string, ownerDocument: unknown) {
      this.tag = tag;
      this.ownerDocument = ownerDocument;
    }
    append(...nodes: Node[]) { this.children.push(...nodes); }
    replaceChildren() { this.children = []; }
    set textContent(value: string) { this.text = value; }
    get innerText(): string {
      return this.text + this.children.map((c) => c.innerText).join('');
    }
  }
  const doc = {
    createElement: (tag: string) => { created.push(tag); return new Node(tag, doc); },
    createTextNode: (text: string) => { const n = new Node('#text', doc); n.text = text; return n; },
  };
  return { doc, created, root: new Node('div', doc) };
}

test('HTML 처럼 보이는 출력도 정해진 요소와 텍스트로만 만든다', () => {
  const { root, created } = fakeDocument();
  const hostile = '<script>alert(1)</script>\n<img src=x onerror=alert(2)>\n**<iframe src="javascript:1">**\n- <a href="javascript:1">x</a>';
  renderBlocks(root as unknown as HTMLElement, parseMarkdown(hostile));
  const allowed = new Set(['p', 'br', 'strong', 'em', 'code', 'pre', 'ul', 'ol', 'li', 'h3', 'h4', 'h5']);
  assert.ok(created.every((tag) => allowed.has(tag)), `만든 요소: ${created.join(',')}`);
  assert.ok((root as unknown as { innerText: string }).innerText.includes('<script>alert(1)</script>'),
    '태그는 글자로 보인다');
});

test('AI 코드는 HTML 해석 경로를 쓰지 않는다', () => {
  const root = join(dirname(fileURLToPath(import.meta.url)), '..', 'src', 'ai');
  const files: string[] = [];
  const walk = (dir: string) => {
    for (const name of readdirSync(dir)) {
      const path = join(dir, name);
      if (statSync(path).isDirectory()) walk(path);
      else if (path.endsWith('.ts')) files.push(path);
    }
  };
  walk(root);
  assert.ok(files.length > 5);
  for (const file of files) {
    const code = readFileSync(file, 'utf8');
    assert.doesNotMatch(code, /\.(innerHTML|outerHTML)\b|insertAdjacentHTML|createContextualFragment|DOMParser|document\.write/,
      file);
  }
});
