/**
 * 모델 응답의 안전한 표시.
 *
 * 모델 출력은 신뢰할 수 없는 문자열이다(문서에 숨은 지시가 섞여 나올 수 있다). 그래서
 * HTML 로 해석하지 않는다 — Markdown 일부만 직접 해석해 요소를 만들고, 글자는 모두
 * `textContent` 로 넣는다. `innerHTML` 계열은 이 모듈과 AI 패널 어디에서도 쓰지 않는다
 * (테스트가 소스로 지킨다).
 *
 * 지원: 문단, 제목(#), 목록(-, *, 1.), 코드 블록(```), 굵게(**), 기울임(*·_), 인라인 코드(`).
 * 링크 `[글](주소)` 는 누를 수 있게 만들지 않고 「글 (주소)」 텍스트로 보인다.
 */

export type Inline =
  | { type: 'text'; text: string }
  | { type: 'strong'; children: Inline[] }
  | { type: 'em'; children: Inline[] }
  | { type: 'code'; text: string };

export type Block =
  | { type: 'paragraph'; lines: Inline[][] }
  | { type: 'heading'; level: 1 | 2 | 3; children: Inline[] }
  | { type: 'list'; ordered: boolean; items: Inline[][] }
  | { type: 'code'; text: string };

const FENCE = /^\s*```/;
const HEADING = /^(#{1,6})\s+(.*)$/;
const BULLET = /^\s*[-*+]\s+(.*)$/;
const ORDERED = /^\s*\d{1,3}[.)]\s+(.*)$/;

export function parseInline(text: string): Inline[] {
  const out: Inline[] = [];
  let buffer = '';
  const flush = () => {
    if (buffer) out.push({ type: 'text', text: buffer });
    buffer = '';
  };
  let i = 0;
  while (i < text.length) {
    const rest = text.slice(i);
    let match: RegExpMatchArray | null;
    if ((match = rest.match(/^`([^`\n]+)`/))) {
      flush();
      out.push({ type: 'code', text: match[1] });
      i += match[0].length;
    } else if ((match = rest.match(/^\*\*(?=\S)([\s\S]*?\S)\*\*/))) {
      flush();
      out.push({ type: 'strong', children: parseInline(match[1]) });
      i += match[0].length;
    } else if ((match = rest.match(/^([*_])(?=\S)([^*_\n]*?\S)\1(?![*_\w])/))) {
      flush();
      out.push({ type: 'em', children: parseInline(match[2]) });
      i += match[0].length;
    } else if ((match = rest.match(/^\[([^\]\n]+)\]\(([^)\s]+)\)/))) {
      buffer += `${match[1]} (${match[2]})`;
      i += match[0].length;
    } else {
      buffer += text[i];
      i += 1;
    }
  }
  flush();
  return out;
}

export function parseMarkdown(source: string): Block[] {
  const lines = source.replace(/\r\n?/g, '\n').split('\n');
  const blocks: Block[] = [];
  let paragraph: Inline[][] = [];
  let list: { ordered: boolean; items: Inline[][] } | null = null;

  const closeParagraph = () => {
    if (paragraph.length > 0) blocks.push({ type: 'paragraph', lines: paragraph });
    paragraph = [];
  };
  const closeList = () => {
    if (list) blocks.push({ type: 'list', ordered: list.ordered, items: list.items });
    list = null;
  };

  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i];
    if (FENCE.test(line)) {
      closeParagraph();
      closeList();
      const code: string[] = [];
      i += 1;
      // 닫는 울타리가 아직 안 온(스트리밍 중) 코드 블록도 끝까지 코드로 보인다.
      while (i < lines.length && !FENCE.test(lines[i])) {
        code.push(lines[i]);
        i += 1;
      }
      blocks.push({ type: 'code', text: code.join('\n') });
      continue;
    }
    if (line.trim() === '') {
      closeParagraph();
      closeList();
      continue;
    }
    const heading = line.match(HEADING);
    if (heading) {
      closeParagraph();
      closeList();
      const level = Math.min(3, heading[1].length) as 1 | 2 | 3;
      blocks.push({ type: 'heading', level, children: parseInline(heading[2]) });
      continue;
    }
    const bullet = line.match(BULLET);
    const ordered = bullet ? null : line.match(ORDERED);
    if (bullet || ordered) {
      closeParagraph();
      const isOrdered = ordered !== null;
      if (list && list.ordered !== isOrdered) closeList();
      if (!list) list = { ordered: isOrdered, items: [] };
      list.items.push(parseInline((bullet ?? ordered)![1]));
      continue;
    }
    closeList();
    paragraph.push(parseInline(line));
  }
  closeParagraph();
  closeList();
  return blocks;
}

function appendInlines(parent: HTMLElement, inlines: readonly Inline[]): void {
  const doc = parent.ownerDocument;
  for (const inline of inlines) {
    switch (inline.type) {
      case 'text':
        parent.append(doc.createTextNode(inline.text));
        break;
      case 'code': {
        const el = doc.createElement('code');
        el.textContent = inline.text;
        parent.append(el);
        break;
      }
      case 'strong':
      case 'em': {
        const el = doc.createElement(inline.type);
        appendInlines(el, inline.children);
        parent.append(el);
        break;
      }
    }
  }
}

/** 블록을 요소로 만든다. 글자는 모두 텍스트 노드다. */
export function renderBlocks(container: HTMLElement, blocks: readonly Block[]): void {
  const doc = container.ownerDocument;
  container.replaceChildren();
  for (const block of blocks) {
    switch (block.type) {
      case 'paragraph': {
        const p = doc.createElement('p');
        block.lines.forEach((line, index) => {
          if (index > 0) p.append(doc.createElement('br'));
          appendInlines(p, line);
        });
        container.append(p);
        break;
      }
      case 'heading': {
        const h = doc.createElement(`h${block.level + 2}`);
        appendInlines(h, block.children);
        container.append(h);
        break;
      }
      case 'list': {
        const list = doc.createElement(block.ordered ? 'ol' : 'ul');
        for (const item of block.items) {
          const li = doc.createElement('li');
          appendInlines(li, item);
          list.append(li);
        }
        container.append(list);
        break;
      }
      case 'code': {
        const pre = doc.createElement('pre');
        const code = doc.createElement('code');
        code.textContent = block.text;
        pre.append(code);
        container.append(pre);
        break;
      }
    }
  }
}

export function renderMarkdown(container: HTMLElement, source: string): void {
  renderBlocks(container, parseMarkdown(source));
}
