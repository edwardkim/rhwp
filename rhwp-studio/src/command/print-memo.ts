import type { MemoInfo } from '@/core/memo-editor';
import type { CursorRect } from '@/core/types';
import { t } from '../i18n/index.ts';
import type { PrintPageInfo } from './print-pages';

export interface PrintMemoOptions { include: boolean; list: boolean }
export interface PrintMemoEntry { memo: MemoInfo; rect: CursorRect | null }
export interface PrintSourcePage { svg: string; info: PrintPageInfo }

const NS = 'http://www.w3.org/2000/svg';
const FONT_SIZE = 13;
const LINE = 18;
const GAP = 8;
const PAPER_INSET = 12;
const TEXT_INSET = 8;
const LIST_TOP = PAPER_INSET + 26;
const REVIEW_PAPER: PrintPageInfo = { width: 794, height: 1123 };
const GRAPHEMES = new Intl.Segmenter(undefined, { granularity: 'grapheme' });

export function createPrintMemoOptions(doc: Document): {
  fieldset: HTMLFieldSetElement;
  value: () => PrintMemoOptions;
} {
  const fieldset = doc.createElement('fieldset');
  fieldset.className = 'print-memo-options';
  const legend = doc.createElement('legend');
  legend.textContent = t('dialog.memo.printOptions');
  fieldset.append(legend);
  const option = (key: 'printInclude' | 'printList', name: string) => {
    const label = doc.createElement('label');
    const input = doc.createElement('input');
    input.type = 'checkbox';
    input.name = name;
    label.append(input, ' ' + t('dialog.memo.' + key));
    fieldset.append(label);
    return input;
  };
  const include = option('printInclude', 'print-memo-include');
  const list = option('printList', 'print-memo-list');
  list.disabled = true;
  include.addEventListener('change', () => {
    list.disabled = !include.checked;
    list.checked = include.checked;
  });
  return { fieldset, value: () => ({ include: include.checked, list: include.checked && list.checked }) };
}

function node(tag: string, attributes: Record<string, string | number>) {
  const element = document.createElementNS(NS, tag);
  for (const [key, value] of Object.entries(attributes)) element.setAttribute(key, String(value));
  return element;
}

function text(svg: Element, x: number, y: number, value: string) {
  const label = node('text', { x, y, 'font-family': 'sans-serif', 'font-size': FONT_SIZE, fill: '#202c20' });
  label.textContent = value;
  svg.append(label);
}

function wrap(value: string, width: number, ctx: CanvasRenderingContext2D): string[] {
  const lines: string[] = [];
  for (const paragraph of value.split(/\r?\n/)) {
    let line = '';
    for (const { segment } of GRAPHEMES.segment(paragraph)) {
      if (line && ctx.measureText(line + segment).width > width) {
        lines.push(line);
        line = '';
      }
      line += segment;
    }
    lines.push(line);
  }
  return lines;
}

interface MemoThread { root: PrintMemoEntry; replies: MemoInfo[] }
interface NoteBlock { lines: string[]; sourcePage: number | null }
interface Sheet { root: SVGSVGElement; info: PrintPageInfo }

function threads(entries: readonly PrintMemoEntry[]): MemoThread[] {
  const groups = new Map<number, MemoThread>();
  for (const entry of entries) {
    if (entry.memo.parentId === null) groups.set(entry.memo.id, { root: entry, replies: [] });
  }
  for (const entry of entries) {
    if (entry.memo.parentId === null) continue;
    const parent = groups.get(entry.memo.parentId);
    if (parent) parent.replies.push(entry.memo);
    else groups.set(entry.memo.id, { root: entry, replies: [] });
  }
  return [...groups.values()];
}

function sheet(info: PrintPageInfo): Sheet {
  return {
    root: node('svg', {
      width: info.width, height: info.height, viewBox: ['0', '0', info.width, info.height].join(' '),
    }) as SVGSVGElement,
    info,
  };
}

function card(root: Element, x: number, y: number, width: number, lines: readonly string[]) {
  const height = lines.length * LINE + 2 * TEXT_INSET;
  root.append(node('rect', { x, y, width, height, fill: '#edf4e7', stroke: '#8fa68a', rx: 3 }));
  lines.forEach((line, i) => text(root, x + TEXT_INSET, y + TEXT_INSET + FONT_SIZE + i * LINE, line));
  return height;
}

function listCapacity(height: number, y: number): number {
  return Math.floor((height - PAPER_INSET - y - 2 * TEXT_INSET) / LINE) - 1;
}

function written(sheet: Sheet): PrintSourcePage {
  return { svg: new XMLSerializer().serializeToString(sheet.root), info: sheet.info };
}

export function composePrintMemos(
  pages: PrintSourcePage[], entries: readonly PrintMemoEntry[], options: PrintMemoOptions,
): PrintSourcePage[] {
  if (!options.include || !entries.length || !pages.length) return pages;
  const ctx = document.createElement('canvas').getContext('2d');
  if (!ctx) throw new Error('메모의 인쇄 너비를 계산할 수 없습니다.');
  ctx.font = String(FONT_SIZE) + 'px sans-serif';
  const byPage = new Map<number, MemoThread[]>();
  const unresolved: MemoThread[] = [];
  const all = threads(entries);
  for (const thread of all) {
    const page = thread.root.rect?.pageIndex;
    if (page === undefined || !pages[page]) {
      unresolved.push(thread);
    } else {
      const siblings = byPage.get(page) ?? [];
      siblings.push(thread);
      byPage.set(page, siblings);
    }
  }

  function lines(thread: MemoThread, width: number): string[] {
    return [thread.root.memo, ...thread.replies].flatMap((memo, index) => {
      const name = memo.author || t('dialog.memo.noAuthor');
      const author = index === 0 ? name : t('dialog.memo.reply') + ' · ' + name;
      return [...wrap(author, width, ctx!), ...wrap(memo.text, width, ctx!)];
    });
  }

  function appendList(out: PrintSourcePage[], blocks: NoteBlock[], info: PrintPageInfo, title: string) {
    if (!blocks.length) return;
    // 출처 쪽 번호와 메모 한 줄이 모두 들어가는 이어쓰기 용지를 먼저 정한다.
    const paper = listCapacity(info.height, LIST_TOP) > 0 ? info : REVIEW_PAPER;
    const firstCapacity = listCapacity(paper.height, LIST_TOP);
    let start = sheet(paper);
    let y = LIST_TOP;
    text(start.root, PAPER_INSET, PAPER_INSET + FONT_SIZE, title);
    for (const { sourcePage, lines } of blocks) {
      const remaining = [...lines];
      const label = sourcePage === null ? t('dialog.memo.noLocation') : t('dialog.memo.page', { p1: sourcePage + 1 });
      while (remaining.length) {
        let capacity = listCapacity(paper.height, y);
        if (capacity < 1) {
          out.push(written(start));
          start = sheet(paper);
          text(start.root, PAPER_INSET, PAPER_INSET + FONT_SIZE, title);
          y = LIST_TOP;
          capacity = firstCapacity;
        }
        const chunk = [label, ...remaining.splice(0, capacity)];
        y += card(start.root, PAPER_INSET, y, paper.width - 2 * PAPER_INSET, chunk) + GAP;
      }
    }
    out.push(written(start));
  }

  const out: PrintSourcePage[] = [];
  for (const [index, page] of pages.entries()) {
    const notes = byPage.get(index);
    if (!notes?.length) { out.push(page); continue; }
    const { width, height } = page.info;
    const rendered = sheet(page.info);
    const scale = 0.72;
    const rail = width * scale + PAPER_INSET;
    const cardWidth = width - rail - PAPER_INSET;
    if (cardWidth < 4 * FONT_SIZE || height < 5 * LINE) {
      out.push(page);
      unresolved.push(...notes);
      continue;
    }
    const sourceDoc = new DOMParser().parseFromString(page.svg, 'image/svg+xml');
    if (sourceDoc.documentElement.localName !== 'svg') throw new Error('인쇄용 문서 SVG를 읽을 수 없습니다.');
    const source = document.importNode(sourceDoc.documentElement, true);
    if (!source.hasAttribute('viewBox')) source.setAttribute('viewBox', ['0', '0', width, height].join(' '));
    source.setAttribute('width', String(width * scale));
    source.setAttribute('height', String(height * scale));
    rendered.root.append(source);

    const overflow: NoteBlock[] = [];
    let next = PAPER_INSET;
    for (const thread of [...notes].sort((a, b) => a.root.rect!.y - b.root.rect!.y)) {
      const rect = thread.root.rect!;
      const fromY = (rect.y + rect.height) * scale;
      const fromX = rect.x * scale;
      const full = lines(thread, Math.max(1, cardWidth - 2 * TEXT_INSET));
      const y = Math.max(next, Math.min(fromY - LINE, height - PAPER_INSET - 2 * TEXT_INSET - 3 * LINE));
      const available = Math.floor((height - PAPER_INSET - y - 2 * TEXT_INSET) / LINE);
      if (available < 2) {
        overflow.push({ lines: full, sourcePage: index });
        continue;
      }
      const truncated = full.length > available;
      const shown = truncated ? [...full.slice(0, available - 1), t('dialog.memo.printContinued')] : full;
      if (truncated) overflow.push({ lines: full.slice(available - 1), sourcePage: index });
      const toY = y + TEXT_INSET;
      rendered.root.append(node('polyline', {
        points: [[fromX, fromY], [width * scale, fromY], [rail, toY]].map(xy => xy.join(',')).join(' '),
        fill: 'none', stroke: '#648458', 'stroke-dasharray': '4 3',
      }));
      next = y + card(rendered.root, rail, y, cardWidth, shown) + GAP;
    }
    out.push(written(rendered));
    appendList(out, overflow, page.info, t('dialog.memo.printContinued'));
  }

  const first = pages[0].info;
  const info = first.width >= 260 && first.height >= 160 ? first : REVIEW_PAPER;
  const listing = options.list ? all : unresolved;
  appendList(out, listing.map(thread => ({
    lines: lines(thread, info.width - 2 * (PAPER_INSET + TEXT_INSET)),
    sourcePage: thread.root.rect?.pageIndex ?? null,
  })), info, t('dialog.memo.printList'));
  return out;
}
