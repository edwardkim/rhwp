/**
 * ui/yangsik-parts-dialog.ts — 양식.hwpx 부품 즉시 삽입 다이얼로그
 *
 * vite plugin endpoint 에서 manifest.json + fragment XML 을 fetch 하고,
 * 카드 클릭 시 wasm `pasteHwpxFragmentInDocument` 로 byte-preserving paste 한다.
 *
 * absorb_repo 의 `personal-yangsik-parts.ts` 를 본체에 흡수하면서 personal layer
 * (atoms/labels/registry) 의존성을 제거하고 manifest entry 만으로 카드를 합성.
 */

import { ModalDialog } from './dialog';

const CATEGORY_ORDER = [
  '제목박스',
  '글상자',
  '내용박스',
  '참고박스',
  '붙임박스',
  '결재란',
  '단계표',
  '사례박스',
  '대외주의',
  '그림틀',
  '표',
];

interface FragmentSourceDefinitions {
  char_prs?: string;
  para_prs?: string;
  styles?: string;
  border_fills?: string;
}

export interface FragmentManifestEntry {
  part_name: string;
  category?: string;
  label_extra?: string;
  kind?: string;
  byte_length?: number;
  fragment_file: string;
  preview_text?: string;
  source_definitions?: FragmentSourceDefinitions;
}

interface FragmentManifest {
  fragments?: FragmentManifestEntry[];
}

export type FragmentInserter = (entry: FragmentManifestEntry) => Promise<boolean>;

export class YangsikPartsDialog extends ModalDialog {
  private fragments: FragmentManifestEntry[];
  private filtered: FragmentManifestEntry[];
  private activeCategory: string;
  private searchQuery = '';
  private listEl?: HTMLDivElement;
  private statusEl?: HTMLDivElement;
  private categoryButtons = new Map<string, HTMLButtonElement>();

  constructor(
    fragments: FragmentManifestEntry[],
    private readonly insert: FragmentInserter,
  ) {
    super('양식 부품', 760);
    this.fragments = fragments;
    const present = CATEGORY_ORDER.find((t) => fragments.some((f) => f.category === t));
    this.activeCategory = present ?? (fragments[0]?.category ?? '');
    this.filtered = this.computeFiltered();
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.style.padding = '12px 16px';
    body.style.maxHeight = '70vh';
    body.style.display = 'flex';
    body.style.flexDirection = 'column';
    body.style.gap = '10px';

    body.appendChild(this.buildToolbar());

    const listWrap = document.createElement('div');
    listWrap.style.overflowY = 'auto';
    listWrap.style.maxHeight = '55vh';
    listWrap.style.border = '1px solid #e0e0e0';
    listWrap.style.borderRadius = '4px';
    listWrap.style.padding = '4px';
    listWrap.style.background = '#fafafa';
    this.listEl = listWrap;
    body.appendChild(listWrap);

    const status = document.createElement('div');
    status.style.fontSize = '12px';
    status.style.color = '#666';
    status.style.minHeight = '16px';
    this.statusEl = status;
    body.appendChild(status);

    this.renderList();
    return body;
  }

  protected onConfirm(): boolean {
    return false;
  }

  private buildToolbar(): HTMLElement {
    const bar = document.createElement('div');
    bar.style.display = 'flex';
    bar.style.flexDirection = 'column';
    bar.style.gap = '6px';

    const row = document.createElement('div');
    row.style.display = 'flex';
    row.style.flexWrap = 'wrap';
    row.style.gap = '4px';

    const counts = new Map<string, number>();
    for (const f of this.fragments) {
      const cat = f.category ?? '';
      counts.set(cat, (counts.get(cat) ?? 0) + 1);
    }
    const present = CATEGORY_ORDER.filter((t) => counts.has(t));

    for (const t of present) {
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.textContent = `${t} (${counts.get(t)})`;
      btn.style.padding = '4px 10px';
      btn.style.border = '1px solid #c0c0c0';
      btn.style.borderRadius = '14px';
      btn.style.background = t === this.activeCategory ? '#4285f4' : '#fff';
      btn.style.color = t === this.activeCategory ? '#fff' : '#333';
      btn.style.cursor = 'pointer';
      btn.style.fontSize = '12px';
      btn.addEventListener('click', () => {
        this.activeCategory = t;
        this.refreshChipStyles();
        this.filtered = this.computeFiltered();
        this.renderList();
      });
      this.categoryButtons.set(t, btn);
      row.appendChild(btn);
    }
    bar.appendChild(row);

    const search = document.createElement('input');
    search.type = 'search';
    search.placeholder = '검색 (이름/미리보기)';
    search.style.width = '100%';
    search.style.padding = '6px 8px';
    search.style.border = '1px solid #c0c0c0';
    search.style.borderRadius = '4px';
    search.style.fontSize = '13px';
    search.addEventListener('input', () => {
      this.searchQuery = search.value.trim();
      this.filtered = this.computeFiltered();
      this.renderList();
    });
    bar.appendChild(search);

    const hint = document.createElement('div');
    hint.style.fontSize = '11px';
    hint.style.color = '#888';
    hint.textContent = '카드를 누르면 현재 커서 위치에 박스/표 스타일 그대로 삽입됩니다.';
    bar.appendChild(hint);

    return bar;
  }

  private refreshChipStyles(): void {
    for (const [t, btn] of this.categoryButtons) {
      const active = t === this.activeCategory;
      btn.style.background = active ? '#4285f4' : '#fff';
      btn.style.color = active ? '#fff' : '#333';
    }
  }

  private computeFiltered(): FragmentManifestEntry[] {
    const q = this.searchQuery.toLowerCase();
    return this.fragments.filter((f) => {
      if (f.category !== this.activeCategory) return false;
      if (!q) return true;
      const hay = `${f.part_name} ${f.preview_text ?? ''}`.toLowerCase();
      return hay.includes(q);
    });
  }

  private renderList(): void {
    if (!this.listEl) return;
    this.listEl.innerHTML = '';
    const max = 200;
    const items = this.filtered.slice(0, max);
    if (items.length === 0) {
      const empty = document.createElement('div');
      empty.style.padding = '20px';
      empty.style.color = '#888';
      empty.style.textAlign = 'center';
      empty.textContent = '결과 없음';
      this.listEl.appendChild(empty);
      return;
    }
    for (const f of items) {
      this.listEl.appendChild(this.buildCard(f));
    }
    if (this.statusEl) {
      this.statusEl.textContent =
        this.filtered.length > max
          ? `${this.filtered.length}건 중 ${max}건 표시 (검색으로 좁혀주세요)`
          : `${this.filtered.length}건`;
    }
  }

  private buildCard(entry: FragmentManifestEntry): HTMLElement {
    const card = document.createElement('button');
    card.type = 'button';
    card.style.display = 'flex';
    card.style.justifyContent = 'space-between';
    card.style.alignItems = 'flex-start';
    card.style.gap = '10px';
    card.style.padding = '8px 10px';
    card.style.margin = '3px 0';
    card.style.background = '#fff';
    card.style.border = '1px solid #d8d8d8';
    card.style.borderRadius = '4px';
    card.style.cursor = 'pointer';
    card.style.textAlign = 'left';
    card.style.width = '100%';

    const left = document.createElement('div');
    left.style.flex = '1';
    left.style.minWidth = '0';

    const title = document.createElement('div');
    title.textContent = entry.part_name;
    title.style.fontSize = '13px';
    title.style.color = '#222';
    title.style.fontWeight = '500';
    left.appendChild(title);

    if (entry.preview_text) {
      const preview = document.createElement('div');
      preview.textContent = entry.preview_text;
      preview.style.fontSize = '12px';
      preview.style.color = '#555';
      preview.style.marginTop = '3px';
      preview.style.wordBreak = 'break-word';
      preview.style.whiteSpace = 'pre-wrap';
      preview.style.maxHeight = '3.5em';
      preview.style.overflow = 'hidden';
      left.appendChild(preview);
    }

    const meta = document.createElement('div');
    meta.style.fontSize = '11px';
    meta.style.color = '#888';
    meta.style.marginTop = '4px';
    const labelExtra = entry.label_extra ? ` · ${entry.label_extra}` : '';
    const kindLabel = entry.kind === 'table' ? '표' : '문단';
    meta.textContent = `${kindLabel}${labelExtra}`;
    left.appendChild(meta);

    const badge = document.createElement('span');
    badge.textContent = '삽입';
    badge.style.flexShrink = '0';
    badge.style.padding = '4px 12px';
    badge.style.border = '1px solid #c0c0c0';
    badge.style.borderRadius = '4px';
    badge.style.background = '#f5f5f5';
    badge.style.fontSize = '12px';
    badge.style.alignSelf = 'center';

    card.appendChild(left);
    card.appendChild(badge);

    card.addEventListener('click', async (e) => {
      e.stopPropagation();
      badge.textContent = '삽입중…';
      const ok = await this.insert(entry);
      if (ok) {
        badge.textContent = '삽입됨';
        badge.style.background = '#4caf50';
        badge.style.color = '#fff';
        badge.style.borderColor = '#4caf50';
        setTimeout(() => {
          badge.textContent = '삽입';
          badge.style.background = '#f5f5f5';
          badge.style.color = '#333';
          badge.style.borderColor = '#c0c0c0';
        }, 900);
      } else {
        badge.textContent = '삽입';
        if (this.statusEl) {
          this.statusEl.textContent =
            '삽입 실패: 본문 편집 위치가 없습니다 (커서를 본문에 둔 뒤 다시 시도해 주세요).';
        }
      }
    });

    return card;
  }
}

/**
 * vite base 가 `/rhwp/` 같이 prefix 인 환경에서도 동작하도록 BASE_URL 을 사용한다.
 * BASE_URL 은 항상 trailing slash 포함 (예: `/`, `/rhwp/`).
 */
function apiUrl(tail: string): string {
  const base = (import.meta as any).env?.BASE_URL ?? '/';
  return `${base}api/personal-templates/yangsik-fragments/${tail}`;
}

/**
 * vite plugin endpoint 에서 manifest 를 fetch 한다.
 */
export async function fetchYangsikFragmentManifest(): Promise<FragmentManifestEntry[]> {
  const resp = await fetch(apiUrl('manifest'));
  if (!resp.ok) return [];
  const data = (await resp.json()) as FragmentManifest;
  return data.fragments ?? [];
}

/**
 * vite plugin endpoint 에서 단일 fragment XML 을 fetch 한다.
 */
export async function fetchYangsikFragmentXml(fragmentFile: string): Promise<string> {
  const resp = await fetch(apiUrl(encodeURIComponent(fragmentFile)));
  if (!resp.ok) throw new Error(`fragment fetch HTTP ${resp.status}`);
  return await resp.text();
}
