import type { CommandServices } from '@/command/types';
import { formatDiffLocationCombined } from '@/compare/diff-location-label';
import { compareDocuments } from '@/compare/diff-engine';
import type { CompareSessionStore } from '@/compare/session';
import type { CompareOptions, DiffItem, DiffKind } from '@/compare/types';
import { CompareResultWindow } from './compare-result-window';

const DEFAULT_KINDS: DiffKind[] = ['text', 'table', 'shape', 'image', 'chart'];
const DEFAULT_COMPARE_OPTS: CompareOptions = {
  caseSensitive: true,
  ignoreWhitespace: true,
  kinds: DEFAULT_KINDS,
  // 외부 문서 비교는 stable_id 공유를 가정하지 않고 정렬 기반으로 고정.
  strategy: 'alignment',
  anchorTuning: {
    // 문서 비교 전용 프리셋: 앵커 품질을 높여 오정렬 연쇄를 줄인다.
    minTextLen: 22,
    minUniqueChars: 7,
    maxWhitespaceRatio: 0.58,
    minEntropy: 2.05,
  },
  performanceTuning: {
    // UI 프리징 방지: 타임버짓 이후 greedy/fallback 비중을 높인다.
    maxComputeMs: 2200,
    hardSegmentCells: 160000,
  },
};

type CompareFile = {
  bytes: Uint8Array;
  fileName: string;
};

export class CompareDialog {
  private readonly services: CommandServices;
  private readonly compareSessionStore: CompareSessionStore;
  private _open = false;
  private running = false;

  private wrap!: HTMLDivElement;
  private leftFileNameEl!: HTMLSpanElement;
  private rightFileNameEl!: HTMLSpanElement;
  private runBtn!: HTMLButtonElement;
  private openTwoPaneBtn!: HTMLButtonElement;
  private caseSensitiveCheck!: HTMLInputElement;
  private resultMetaEl!: HTMLSpanElement;
  private resultListEl!: HTMLUListElement;

  private leftFile: CompareFile | null = null;
  private rightFile: CompareFile | null = null;
  private resultWindow: CompareResultWindow | null = null;

  constructor(services: CommandServices, compareSessionStore: CompareSessionStore) {
    this.services = services;
    this.compareSessionStore = compareSessionStore;
  }

  isOpen(): boolean {
    return this._open;
  }

  show(): void {
    if (this._open) return;
    this._open = true;
    this.build();
    document.body.appendChild(this.wrap);
  }

  hide(): void {
    this._open = false;
    this.wrap?.remove();
  }

  private build(): void {
    this.wrap = document.createElement('div');
    this.wrap.className = 'compare-dialog doc-compare-dialog';

    const title = document.createElement('div');
    title.className = 'compare-dialog-title';
    title.innerHTML = '<span>문서 비교</span>';
    const close = document.createElement('button');
    close.className = 'dialog-close';
    close.textContent = '\u00D7';
    close.addEventListener('click', () => this.hide());
    title.appendChild(close);
    this.wrap.appendChild(title);

    const body = document.createElement('div');
    body.className = 'compare-dialog-body';

    const hint = document.createElement('p');
    hint.className = 'history-hint';
    hint.textContent =
      '두 문서를 업로드해 차이를 계산합니다. 결과를 클릭하면 좌/우 상세창에서 변경된 부분이 하이라이트되며 변경 구간 중심으로 표시됩니다.';
    body.appendChild(hint);

    const leftRow = document.createElement('div');
    leftRow.className = 'compare-row';
    const leftLabel = document.createElement('span');
    leftLabel.className = 'compare-label';
    leftLabel.textContent = '왼쪽 문서';
    const leftBtn = document.createElement('button');
    leftBtn.className = 'dialog-btn';
    leftBtn.textContent = '파일 선택';
    this.leftFileNameEl = document.createElement('span');
    this.leftFileNameEl.className = 'compare-file';
    this.leftFileNameEl.textContent = '(선택 안 됨)';
    leftBtn.addEventListener('click', () => void this.pickFile('left'));
    leftRow.append(leftLabel, leftBtn, this.leftFileNameEl);
    body.appendChild(leftRow);

    const rightRow = document.createElement('div');
    rightRow.className = 'compare-row';
    const rightLabel = document.createElement('span');
    rightLabel.className = 'compare-label';
    rightLabel.textContent = '오른쪽 문서';
    const rightBtn = document.createElement('button');
    rightBtn.className = 'dialog-btn';
    rightBtn.textContent = '파일 선택';
    this.rightFileNameEl = document.createElement('span');
    this.rightFileNameEl.className = 'compare-file';
    this.rightFileNameEl.textContent = '(선택 안 됨)';
    rightBtn.addEventListener('click', () => void this.pickFile('right'));
    rightRow.append(rightLabel, rightBtn, this.rightFileNameEl);
    body.appendChild(rightRow);

    const optWrap = document.createElement('div');
    optWrap.className = 'compare-options';
    const optRow = document.createElement('div');
    optRow.className = 'compare-strategy-row';
    const caseLabel = document.createElement('label');
    caseLabel.className = 'compare-checkbox';
    this.caseSensitiveCheck = document.createElement('input');
    this.caseSensitiveCheck.type = 'checkbox';
    this.caseSensitiveCheck.id = 'doc-compare-case-sensitive';
    this.caseSensitiveCheck.checked = DEFAULT_COMPARE_OPTS.caseSensitive;
    const caseSpan = document.createElement('span');
    caseSpan.textContent = '영문 대소문자 구분';
    caseLabel.append(this.caseSensitiveCheck, caseSpan);
    optRow.appendChild(caseLabel);
    optWrap.appendChild(optRow);
    body.appendChild(optWrap);

    const actions = document.createElement('div');
    actions.className = 'compare-actions';
    this.runBtn = document.createElement('button');
    this.runBtn.className = 'dialog-btn';
    this.runBtn.textContent = '문서 비교 실행';
    this.runBtn.addEventListener('click', () => void this.onRunCompare());
    this.openTwoPaneBtn = document.createElement('button');
    this.openTwoPaneBtn.className = 'dialog-btn';
    this.openTwoPaneBtn.textContent = '2개 창 띄우기';
    this.openTwoPaneBtn.disabled = true;
    this.openTwoPaneBtn.addEventListener('click', () => this.openResultWindow());
    actions.append(this.runBtn, this.openTwoPaneBtn);
    body.appendChild(actions);

    const resultsWrap = document.createElement('div');
    resultsWrap.className = 'compare-results';

    const resultTitle = document.createElement('div');
    resultTitle.className = 'compare-kinds-title';
    resultTitle.textContent = '비교 결과';

    this.resultMetaEl = document.createElement('span');
    this.resultMetaEl.className = 'compare-result-meta';
    this.resultMetaEl.textContent = '비교 실행 전';
    this.resultListEl = document.createElement('ul');
    this.resultListEl.className = 'compare-result-list';

    resultsWrap.append(resultTitle, this.resultMetaEl, this.resultListEl);
    body.appendChild(resultsWrap);

    this.wrap.appendChild(body);
    this.prefillRightFromCurrentDocument();
  }

  /** 에디터에 열린 문서를 오른쪽 비교 대상으로 기본 설정한다. */
  private prefillRightFromCurrentDocument(): void {
    if (this.rightFile) return;
    const wasm = this.services.wasm;
    if (!wasm.hasLoadedDocument()) return;
    try {
      const fmt = wasm.getSourceFormat();
      const raw = fmt === 'hwpx' ? wasm.exportHwpx() : wasm.exportHwp();
      const bytes = new Uint8Array(raw);
      this.rightFile = { bytes, fileName: wasm.fileName };
      this.rightFileNameEl.textContent = wasm.fileName;
    } catch {
      /* 미로드·보내기 실패 시 무시 */
    }
  }

  private async pickFile(side: 'left' | 'right'): Promise<void> {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.hwp,.hwpx';
    input.style.display = 'none';
    document.body.appendChild(input);
    input.click();
    const selected = await new Promise<File | null>((resolve) => {
      input.onchange = () => resolve(input.files?.[0] ?? null);
    });
    input.remove();
    if (!selected) return;
    const name = selected.name.toLowerCase();
    if (!name.endsWith('.hwp') && !name.endsWith('.hwpx')) {
      this.resultMetaEl.textContent = 'HWP/HWPX 파일만 선택할 수 있습니다.';
      return;
    }
    const bytes = new Uint8Array(await selected.arrayBuffer());
    const picked: CompareFile = { bytes, fileName: selected.name };
    if (side === 'left') {
      this.leftFile = picked;
      this.leftFileNameEl.textContent = selected.name;
    } else {
      this.rightFile = picked;
      this.rightFileNameEl.textContent = selected.name;
    }
  }

  private async onRunCompare(): Promise<void> {
    if (this.running) return;
    if (!this.leftFile || !this.rightFile) {
      this.resultMetaEl.textContent = '왼쪽/오른쪽 문서를 모두 선택하세요.';
      return;
    }

    const ctx = this.services.getContext();
    if (ctx.hasDocument) {
      const detail = ctx.canUndo
        ? '저장하지 않은 변경이 있으면 잃을 수 있습니다.'
        : '현재 편집 중인 문서가 오른쪽과 다르면 에디터 내용이 오른쪽 파일로 바뀝니다.';
      const ok = window.confirm(
        `비교를 실행한 뒤 오른쪽 문서를 에디터에 불러옵니다.\n${detail}\n계속할까요?`,
      );
      if (!ok) return;
    }

    this.running = true;
    this.runBtn.disabled = true;
    this.openTwoPaneBtn.disabled = true;
    this.runBtn.textContent = '비교 중...';
    this.resultMetaEl.textContent = '비교 계산 중...';
    this.resultListEl.replaceChildren();

    try {
      const session = await compareDocuments(
        this.leftFile.bytes,
        this.leftFile.fileName,
        this.rightFile.bytes,
        this.rightFile.fileName,
        { ...DEFAULT_COMPARE_OPTS, caseSensitive: this.caseSensitiveCheck.checked },
      );

      await this.loadRightDocumentToEditor(this.rightFile);
      this.compareSessionStore.set(session);
      this.services.eventBus.emit('compare:mode-changed', true);

      const mode =
        session.textCompareStrategyUsed === 'identity' ? '본문=id(Map)' : '본문=정렬(alignment)';
      this.resultMetaEl.textContent =
        `${session.diffItems.length}개 차이 · ${mode} · "${this.leftFile.fileName}" vs "${this.rightFile.fileName}"`;
      this.renderDiffList(session.diffItems);
      this.openTwoPaneBtn.disabled = session.diffItems.length === 0;
      if (session.diffItems.length > 0) {
        this.compareSessionStore.gotoDiff(0);
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      this.resultMetaEl.textContent = `비교 실패: ${msg}`;
    } finally {
      this.running = false;
      this.runBtn.disabled = false;
      this.runBtn.textContent = '문서 비교 실행';
    }
  }

  private openResultWindow(): void {
    const sess = this.compareSessionStore.get();
    if (!sess || sess.diffItems.length === 0) {
      this.resultMetaEl.textContent = '먼저 문서 비교를 실행해 결과를 생성하세요.';
      return;
    }
    const idx = Math.max(0, sess.currentDiffIndex);
    this.resultWindow ??= new CompareResultWindow();
    if (!this.leftFile || !this.rightFile) return;
    this.resultWindow.show(sess, this.compareSessionStore, idx, {
      left: this.leftFile,
      right: this.rightFile,
    });
  }

  private async loadRightDocumentToEditor(file: CompareFile): Promise<void> {
    const requestId = `cmp_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
    const done = new Promise<void>((resolve, reject) => {
      const off = this.services.eventBus.on('open-document-bytes:done', (payload) => {
        const p = payload as { requestId?: string; ok: boolean; error?: string };
        if (p.requestId !== requestId) return;
        off();
        if (p.ok) resolve();
        else reject(new Error(p.error || '오른쪽 문서 로드 실패'));
      });
      setTimeout(() => {
        off();
        reject(new Error('오른쪽 문서 로드 타임아웃'));
      }, 90_000);
    });
    this.services.eventBus.emit('open-document-bytes', {
      bytes: new Uint8Array(file.bytes),
      fileName: file.fileName,
      fileHandle: null,
      requestId,
    });
    await done;
  }

  private renderDiffList(items: DiffItem[]): void {
    this.resultListEl.replaceChildren();
    for (const [idx, item] of items.entries()) {
      const li = document.createElement('li');
      li.className = 'compare-result-item';
      li.dataset.diffId = item.id;
      const location = formatDiffLocationCombined(item);
      const valueDiff = this.renderValueDiff(item);
      const leftPreview = this.formatPreviewText(this.sanitizeControlPreview(item.leftPreview));
      const rightPreview = this.formatPreviewText(this.sanitizeControlPreview(item.rightPreview));
      const previewLine = (item.kind === 'text' || item.severity !== 'modified')
        ? `<div class="compare-result-preview">L: ${this.escape(leftPreview)} / R: ${this.escape(rightPreview)}</div>`
        : '';
      li.innerHTML = `<strong>[${this.escape(this.kindLabel(item.kind))}] ${this.escape(item.title)}${location ? ` <span class="compare-result-location">(${this.escape(location)})</span>` : ''}</strong>${previewLine}${valueDiff}`;
      li.addEventListener('click', () => {
        this.compareSessionStore.gotoDiff(idx);
        const sess = this.compareSessionStore.get();
        if (sess) {
          this.resultWindow ??= new CompareResultWindow();
          if (this.leftFile && this.rightFile) {
            this.resultWindow.show(sess, this.compareSessionStore, idx, {
              left: this.leftFile,
              right: this.rightFile,
            });
          } else {
            this.resultWindow.show(sess, this.compareSessionStore, idx);
          }
        }
        this.resultListEl.querySelectorAll('.compare-result-item.active').forEach((el) => el.classList.remove('active'));
        li.classList.add('active');
        li.scrollIntoView({ block: 'nearest' });
      });
      this.resultListEl.appendChild(li);
    }
  }

  private formatPreviewText(text: string): string {
    const t = text.trim().replaceAll('\r\n', '\n').replaceAll('\n', ' ↵ ').replace(/\s{2,}/g, ' ');
    if (!t) return '(없음)';
    if (t.length <= 140) return t;
    return `${t.slice(0, 139)}…`;
  }

  private renderValueDiff(item: DiffItem): string {
    if (item.severity !== 'modified') return '';
    if (item.kind === 'text') {
      const l = this.formatPreviewText(item.leftPreview);
      const r = this.formatPreviewText(item.rightPreview);
      if (l === r) return '';
      return `<div class="compare-result-kv compare-result-kv-text"><div class="compare-result-kv-head">텍스트 변경</div><div class="compare-result-kv-line"><span class="k">기존</span><span class="v">${this.escape(l)}</span></div><div class="compare-result-kv-line"><span class="k">변경</span><span class="v">${this.escape(r)}</span></div></div>`;
    }
    const left = this.parseKvSummary(item.leftPreview);
    const right = this.parseKvSummary(item.rightPreview);
    const keys = new Set([...Object.keys(left), ...Object.keys(right)]);
    if (keys.size === 0) return '';

    const labels: Record<string, string> = {
      r: '행',
      c: '열',
      tprev: '텍스트',
      cprev: '셀 텍스트',
      txt: '텍스트 해시',
      props: '속성 해시',
      box: '크기',
      sig: '시그니처',
      crop: '자르기',
      effect: '효과',
      bc: '밝기/대비',
      rot: '회전',
      flip: '대칭',
      wrap: '본문배치',
      rel: '기준',
      pix: '시각 내용',
    };

    const rows: string[] = [];
    for (const k of keys) {
      if (k === 'txt' || k === 'sig' || k === 'csha') continue;
      const lv = left[k] ?? '(없음)';
      const rv = right[k] ?? '(없음)';
      if (lv === rv) continue;
      if (k === 'cprev') {
        const cellDiff = this.formatCellPreviewDiff(lv, rv, left.csha, right.csha);
        if (cellDiff) rows.push(`${labels[k] ?? k}: ${cellDiff}`);
        else rows.push(`${labels[k] ?? k}: ${this.formatFieldValue(k, lv)} → ${this.formatFieldValue(k, rv)}`);
        continue;
      }
      rows.push(`${labels[k] ?? k}: ${this.formatFieldValue(k, lv)} → ${this.formatFieldValue(k, rv)}`);
    }
    if (rows.length === 0) {
      if (item.title.includes('텍스트 변경')) {
        const changedCells = this.countChangedCellsFromHash(left.csha, right.csha);
        if (changedCells > 0) {
          return `<div class="compare-result-kv">변경값:<br/>변경 셀 ${changedCells}개 (셀 미리보기 범위를 벗어나거나 텍스트가 길어 일부 생략됨)</div>`;
        }
      }
      if (item.title.includes('속성 변경')) {
        const lp = left.props ?? '(없음)';
        const rp = right.props ?? '(없음)';
        if (lp !== rp) {
          return `<div class="compare-result-kv">변경값:<br/>속성 해시: ${this.escape(lp)} → ${this.escape(rp)}</div>`;
        }
        return '<div class="compare-result-kv">변경값:<br/>속성 값 변경</div>';
      }
      return '';
    }
    const body = rows.slice(0, 4).map((r) => this.escape(r)).join('<br/>');
    return `<div class="compare-result-kv">변경값:<br/>${body}</div>`;
  }

  private parseKvSummary(summary: string): Record<string, string> {
    const out: Record<string, string> = {};
    for (const m of summary.matchAll(/([a-z]+)=("([^"]*)"|[^\s]+)/g)) {
      const raw = m[2] ?? '';
      out[m[1]] = raw.startsWith('"') && raw.endsWith('"') ? raw.slice(1, -1) : raw;
    }
    return out;
  }

  private sanitizeControlPreview(text: string): string {
    return text
      .replace(/\s(?:txt|props|sig|cprev|csha|pix)=\"[^\"]*\"/g, '')
      .replace(/\s(?:sig|txt|props)=[^\s]+/g, '')
      .replace(/(?:^|\s)(sig|txt|props|csha|pix)=[^\s]+/g, '')
      .trim();
  }

  private formatFieldValue(key: string, value: string): string {
    if (value === '(없음)') return value;
    if (key === 'box') {
      const m = value.match(/^(-?\d+)x(-?\d+)$/);
      if (m) return `${m[1]}px × ${m[2]}px`;
    }
    if (key === 'crop') {
      const nums = value.split(',');
      if (nums.length === 4) return `좌${nums[0]}, 상${nums[1]}, 우${nums[2]}, 하${nums[3]}`;
    }
    if (key === 'cprev') {
      const map = this.parseCellPreviewMap(value);
      if (map.size > 0) {
        return [...map.entries()]
          .slice(0, 2)
          .map(([cell, text]) => `${cell}=${text}`)
          .join(' | ');
      }
      const normalized = value.replaceAll('&amp;', '&');
      return normalized || '(없음)';
    }
    if (key === 'rot') return `${value}도`;
    if (key === 'bc') {
      const [b, c] = value.split('/');
      if (b != null && c != null) return `밝기 ${b}, 대비 ${c}`;
    }
    if (key === 'flip') {
      if (value === '10') return '가로';
      if (value === '01') return '세로';
      if (value === '11') return '가로+세로';
      if (value === '00') return '없음';
    }
    return value;
  }

  private parseCellPreviewMap(value: string): Map<string, string> {
    const map = new Map<string, string>();
    if (!value || value === '(없음)') return map;
    const normalized = value.replaceAll('&amp;', '&');
    const parts = normalized.includes('&') ? normalized.split('&') : normalized.split(';');
    for (const part of parts) {
      const p = part.trim();
      if (!p) continue;
      const idx = p.includes('=') ? p.indexOf('=') : p.indexOf(':');
      if (idx <= 0) continue;
      const cell = p.slice(0, idx).trim();
      const raw = p.slice(idx + 1).trim();
      let text = raw;
      try {
        text = decodeURIComponent(raw);
      } catch {
        text = raw;
      }
      if (!cell) continue;
      map.set(cell, text || '(빈값)');
    }
    return map;
  }

  private parseCellHashMap(value: string): Map<string, string> {
    const map = new Map<string, string>();
    if (!value || value === '(없음)') return map;
    const normalized = value.replaceAll('&amp;', '&');
    const parts = normalized.includes('&') ? normalized.split('&') : normalized.split(';');
    for (const part of parts) {
      const p = part.trim();
      if (!p) continue;
      const idx = p.includes('=') ? p.indexOf('=') : p.indexOf(':');
      if (idx <= 0) continue;
      map.set(p.slice(0, idx).trim(), p.slice(idx + 1).trim());
    }
    return map;
  }

  private formatCellPreviewDiff(left: string, right: string, leftHashRaw?: string, rightHashRaw?: string): string {
    const lmap = this.parseCellPreviewMap(left);
    const rmap = this.parseCellPreviewMap(right);
    const lh = this.parseCellHashMap(leftHashRaw ?? '');
    const rh = this.parseCellHashMap(rightHashRaw ?? '');
    const unionKeys = [...new Set([...lmap.keys(), ...rmap.keys(), ...lh.keys(), ...rh.keys()])];
    const hashChangedKeys = unionKeys.filter((k) => (lh.get(k) ?? '') !== (rh.get(k) ?? ''));
    const keys = hashChangedKeys.length > 0 ? hashChangedKeys : unionKeys;
    const changes: string[] = [];
    for (const key of keys) {
      const lv = lmap.get(key) ?? '(없음)';
      const rv = rmap.get(key) ?? '(없음)';
      if (lv === rv) continue;
      const prettyKey = key.replace(/^r(\d+)c(\d+)$/i, '$1행$2열');
      changes.push(`${prettyKey} ${lv} → ${rv}`);
      if (changes.length >= 3) break;
    }
    if (changes.length === 0) return '';
    return changes.join(' / ');
  }

  private countChangedCellsFromHash(leftHashRaw?: string, rightHashRaw?: string): number {
    const lh = this.parseCellHashMap(leftHashRaw ?? '');
    const rh = this.parseCellHashMap(rightHashRaw ?? '');
    const keys = new Set<string>([...lh.keys(), ...rh.keys()]);
    let changed = 0;
    for (const key of keys) {
      if ((lh.get(key) ?? '') !== (rh.get(key) ?? '')) changed += 1;
    }
    return changed;
  }

  private escape(text: string): string {
    return text.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');
  }

  private kindLabel(kind: DiffItem['kind']): string {
    if (kind === 'table') return '표';
    if (kind === 'shape') return '도형';
    if (kind === 'image') return '이미지';
    if (kind === 'chart') return '그래프';
    if (kind === 'text') return '텍스트';
    return '메타';
  }
}

