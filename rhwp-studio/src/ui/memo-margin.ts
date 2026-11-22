import type { EventBus } from '@/core/event-bus';
import type { MemoInfo } from '@/core/memo-editor';
import { t } from '../i18n/index.ts';

interface LocatedMemo {
  memo: MemoInfo;
  x: number;
  y: number;
  edge: number;
}

const CARD_HEIGHT = 72;
const CARD_PITCH = 80;
const SVG_NS = 'http://www.w3.org/2000/svg';

export class MemoMargin {
  private panel = document.createElement('aside');
  private cards = document.createElement('div');
  private empty = document.createElement('p');
  private guides = document.createElementNS(SVG_NS, 'svg');
  private shown = false;
  private connected = false;
  private entries = new Map<number, HTMLButtonElement>();

  constructor(
    private container: HTMLElement, bus: EventBus, private open: (id: number) => void, schedule: () => void,
  ) {
    const title = document.createElement('h2');
    title.textContent = t('dialog.memo.margin');
    this.panel.className = 'memo-margin';
    this.panel.setAttribute('aria-label', title.textContent);
    this.panel.hidden = true;
    this.cards.className = 'memo-margin-cards';
    this.empty.textContent = t('dialog.memo.marginEmpty');
    this.cards.append(this.empty);
    this.panel.append(title, this.cards);
    this.guides.classList.add('memo-margin-guides');
    this.guides.setAttribute('aria-hidden', 'true');
    container.after(this.panel, this.guides);
    this.cards.addEventListener('scroll', schedule, { passive: true });
    bus.on('memo-view-toggle', option => {
      if (option === 'margin') this.shown = !this.shown;
      if (option === 'guides') this.connected = !this.connected;
      this.panel.hidden = !this.shown;
      for (const [name, value] of [['margin', this.shown], ['guides', this.connected]] as const) {
        for (const item of document.querySelectorAll('[data-cmd="view:memo-' + name + '"]')) {
          item.classList.toggle('active', value);
          item.setAttribute('aria-pressed', String(value));
        }
      }
      schedule();
    });
  }

  update(located: readonly LocatedMemo[]): void {
    this.guides.replaceChildren();
    if (!this.shown) return;
    const content = this.container.querySelector('#scroll-content')!;
    const editorRect = this.container.parentElement!.getBoundingClientRect();
    const contentRect = content.getBoundingClientRect();
    const viewRect = this.container.getBoundingClientRect();
    const railRect = this.cards.getBoundingClientRect();
    const visible = located.filter(mark => contentRect.top + mark.y >= viewRect.top
      && contentRect.top + mark.y <= viewRect.bottom).sort((a, b) => a.y - b.y);
    const ids = new Set(visible.map(({ memo }) => memo.id));
    for (const [id, card] of this.entries) {
      if (ids.has(id)) continue;
      card.remove();
      this.entries.delete(id);
    }
    this.empty.hidden = visible.length !== 0;
    let next = 4;
    for (const { memo, x, y, edge } of visible) {
      let card = this.entries.get(memo.id);
      if (!card) {
        card = document.createElement('button');
        card.type = 'button';
        card.dataset.memoId = String(memo.id);
        card.addEventListener('click', () => this.open(memo.id));
        card.append(document.createElement('strong'), document.createElement('span'));
        this.cards.append(card);
        this.entries.set(memo.id, card);
      }
      card.children[0].textContent = memo.author || t('dialog.memo.noAuthor');
      card.children[1].textContent = memo.text;
      card.title = memo.text;
      const fromY = contentRect.top + y;
      const preferred = fromY - railRect.top - CARD_HEIGHT / 2;
      const top = Math.max(next, Math.min(preferred, railRect.height - CARD_HEIGHT - 4));
      card.style.top = String(top) + 'px';
      next = top + CARD_PITCH;

      const toY = railRect.top + top + CARD_HEIGHT / 2 - this.cards.scrollTop;
      const fromX = contentRect.left + x;
      if (!this.connected || toY < railRect.top || toY > railRect.bottom
        || fromX < viewRect.left || fromX > viewRect.right) continue;
      const elbow = Math.max(fromX, Math.min(viewRect.right, contentRect.left + edge));
      const point = (px: number, py: number) => (px - editorRect.left) + ',' + (py - editorRect.top);
      const guide = document.createElementNS(SVG_NS, 'polyline');
      guide.setAttribute('points', [point(fromX, fromY), point(elbow, fromY),
        point(railRect.left, toY)].join(' '));
      this.guides.append(guide);
    }
  }
}
