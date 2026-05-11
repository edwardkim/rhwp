/**
 * 원본 오픈소스 대비 이력관리 UI 확장 (상세)
 *
 * [역할]
 * - 단순 비교 호출 UI가 아니라, "버전 스냅샷 저장소" 프론트 게이트 역할을 한다.
 * - 사용자 기준 액션: 저장 / 목록 조회 / 삭제 / 전체 비우기 / 선택 버전 vs 현재 비교.
 *
 * [저장 정책]
 * - 우선 저장 포맷: IR 스냅샷(JSON)
 *   -> 문단 stable_id를 보존하여 같은 문서의 시계열 비교에서 identity 모드를 유도한다.
 * - 레거시 포맷(bytes)도 읽을 수 있게 유지
 *   -> 기존 사용자 데이터 유실 방지(하위호환).
 *
 * [비교 실행 정책]
 * - IR 항목: `compareSnapshots(leftSnap, rightSnap, opts)` 사용
 * - legacy bytes 항목: `compareDocuments(...)`로 폴백
 * - 결과는 `CompareSessionStore.set(session)` 후 `gotoDiff(0)`로 즉시 탐색 가능하게 연결한다.
 *
 * [UI/탐색 연결]
 * - 이 다이얼로그는 자체 스크롤/렌더 목록만 담당하고,
 *   실제 문서 이동/하이라이트는 `compare:navigate-diff` 체인(main.ts)으로 위임한다.
 *
 * [유지보수 포인트]
 * - 결과 리스트 포맷(미리보기/위치 표기)은 사용성 이슈가 가장 잦다.
 * - 위치 표기는 `leftSectionPage/rightSectionPage` 우선, 없으면 anchor.pageIndex 폴백 순서 유지.
 */
import type { CommandServices } from '@/command/types';
import { formatDiffLocationCombined } from '@/compare/diff-location-label';
import { buildSnapshotFromWasm, compareDocuments, compareSnapshots } from '@/compare/diff-engine';
import type { CompareSessionStore } from '@/compare/session';
import type { CompareOptions, DiffItem, DiffKind } from '@/compare/types';
import { clearHistory, deleteHistorySnapshot, getHistoryPayload, listHistoryMeta, saveHistoryIrSnapshot } from '@/history/idb-store';
import type { DocHistoryEntryMeta } from '@/history/types';

const DEFAULT_KINDS: DiffKind[] = ['text', 'table', 'shape', 'image', 'chart', 'paragraphMeta'];

const HISTORY_COMPARE_OPTS: CompareOptions = {
  caseSensitive: true,
  ignoreWhitespace: true,
  kinds: DEFAULT_KINDS,
  // 이력 관리는 같은 문서 계통 비교가 목적이므로 stable_id(identity) 고정.
  strategy: 'identity',
  performanceTuning: {
    // identity는 O(N) 성격이므로 타임버짓을 넉넉히 유지(실질 영향 작음).
    maxComputeMs: 3000,
  },
};

export class HistoryDialog {
  private readonly services: CommandServices;
  private readonly compareSessionStore: CompareSessionStore;
  private _open = false;

  private wrap!: HTMLDivElement;
  private labelInput!: HTMLInputElement;
  private listEl!: HTMLUListElement;
  private resultMetaEl!: HTMLSpanElement;
  private resultListEl!: HTMLUListElement;
  private selectedId: string | null = null;
  private entries: DocHistoryEntryMeta[] = [];

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
    void this.refreshList();
  }

  hide(): void {
    this._open = false;
    this.wrap?.remove();
  }

  private build(): void {
    this.wrap = document.createElement('div');
    this.wrap.className = 'compare-dialog history-dialog';

    const title = document.createElement('div');
    title.className = 'compare-dialog-title';
    title.innerHTML = '<span>문서 이력 관리</span>';
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
      '이력은 문단 stable_id가 보존된 IR 스냅샷(JSON)으로 저장됩니다. "선택과 현재 비교"는 같은 편집 세션에서 identity(Map) 비교가 됩니다. 예전에 HWP 바이트만 저장된 항목(legacy)은 비교 시 정렬(alignment)로 폴백됩니다.';
    body.appendChild(hint);

    const saveRow = document.createElement('div');
    saveRow.className = 'compare-row';
    const lab = document.createElement('label');
    lab.className = 'compare-label';
    lab.textContent = '스냅샷';
    lab.htmlFor = 'history-snap-label';
    this.labelInput = document.createElement('input');
    this.labelInput.id = 'history-snap-label';
    this.labelInput.type = 'text';
    this.labelInput.className = 'history-label-input';
    this.labelInput.placeholder = '메모 (비우면 시각 기본값)';
    this.labelInput.value = '';
    const saveBtn = document.createElement('button');
    saveBtn.className = 'dialog-btn';
    saveBtn.textContent = '현재 문서 저장';
    saveBtn.addEventListener('click', () => void this.onSaveSnapshot());
    saveRow.append(lab, this.labelInput, saveBtn);
    body.appendChild(saveRow);

    const listTitle = document.createElement('div');
    listTitle.className = 'compare-kinds-title';
    listTitle.textContent = '저장된 이력 (클릭하여 선택)';
    body.appendChild(listTitle);
    this.listEl = document.createElement('ul');
    this.listEl.className = 'history-list';
    body.appendChild(this.listEl);

    const actions = document.createElement('div');
    actions.className = 'compare-actions';
    const delBtn = document.createElement('button');
    delBtn.className = 'dialog-btn';
    delBtn.textContent = '선택 삭제';
    delBtn.addEventListener('click', () => void this.onDeleteSelected());
    const clrBtn = document.createElement('button');
    clrBtn.className = 'dialog-btn';
    clrBtn.textContent = '전체 비우기';
    clrBtn.addEventListener('click', () => void this.onClearAll());
    const cmpBtn = document.createElement('button');
    cmpBtn.className = 'dialog-btn';
    cmpBtn.textContent = '선택과 현재 문서 비교';
    cmpBtn.addEventListener('click', () => void this.onCompareWithCurrent());
    actions.append(delBtn, clrBtn, cmpBtn);
    body.appendChild(actions);

    const resTitle = document.createElement('div');
    resTitle.className = 'compare-kinds-title';
    resTitle.textContent = '비교 결과';
    body.appendChild(resTitle);
    this.resultMetaEl = document.createElement('span');
    this.resultMetaEl.className = 'compare-result-meta';
    this.resultMetaEl.textContent = '비교 실행 전';
    this.resultListEl = document.createElement('ul');
    this.resultListEl.className = 'compare-result-list';
    body.appendChild(this.resultMetaEl);
    body.appendChild(this.resultListEl);

    this.wrap.appendChild(body);
  }

  private async refreshList(): Promise<void> {
    this.entries = await listHistoryMeta();
    this.listEl.replaceChildren();
    for (const e of this.entries) {
      const li = document.createElement('li');
      li.className = 'history-entry';
      if (e.id === this.selectedId) li.classList.add('selected');
      li.dataset.id = e.id;
      const dt = new Date(e.createdAt).toLocaleString('ko-KR');
      const kindNote = e.storageKind === 'legacy' ? ' · 구바이트' : '';
      li.innerHTML = `<strong>${this.escape(e.label)}</strong><div class="history-entry-meta">${this.escape(e.sourceFileName)} · ${(e.byteLength / 1024).toFixed(1)} KB${kindNote} · ${dt}</div>`;
      li.addEventListener('click', () => {
        this.selectedId = e.id;
        this.listEl.querySelectorAll('.history-entry').forEach((el) => el.classList.remove('selected'));
        li.classList.add('selected');
      });
      this.listEl.appendChild(li);
    }
    if (!this.entries.find((x) => x.id === this.selectedId)) this.selectedId = null;
  }

  private async onSaveSnapshot(): Promise<void> {
    const { wasm } = this.services;
    try {
      const label = this.labelInput.value.trim() || new Date().toLocaleString('ko-KR');
      const snap = buildSnapshotFromWasm(wasm, label, HISTORY_COMPARE_OPTS);
      await saveHistoryIrSnapshot(label, wasm.fileName, snap);
      this.labelInput.value = '';
      await this.refreshList();
      this.resultMetaEl.textContent = '스냅샷을 저장했습니다.';
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.resultMetaEl.textContent = `저장 실패: ${msg}`;
    }
  }

  private async onDeleteSelected(): Promise<void> {
    if (!this.selectedId) {
      this.resultMetaEl.textContent = '삭제할 항목을 목록에서 먼저 선택하세요.';
      return;
    }
    await deleteHistorySnapshot(this.selectedId);
    this.selectedId = null;
    await this.refreshList();
    this.resultMetaEl.textContent = '삭제했습니다.';
    this.resultListEl.replaceChildren();
  }

  private async onClearAll(): Promise<void> {
    if (!window.confirm('저장된 문서 이력을 모두 지울까요?')) return;
    await clearHistory();
    this.selectedId = null;
    await this.refreshList();
    this.resultMetaEl.textContent = '이력을 비웠습니다.';
    this.resultListEl.replaceChildren();
  }

  private async onCompareWithCurrent(): Promise<void> {
    const { wasm } = this.services;
    if (!this.selectedId) {
      this.resultMetaEl.textContent = '비교할 스냅샷을 목록에서 선택하세요.';
      return;
    }
    const payload = await getHistoryPayload(this.selectedId);
    if (!payload) {
      this.resultMetaEl.textContent = '스냅샷 데이터를 읽을 수 없습니다.';
      return;
    }
    const meta = this.entries.find((x) => x.id === this.selectedId);
    const leftName = meta?.label ?? '이력 스냅샷';
    const rightName = wasm.fileName || '현재 문서.hwp';
    this.resultMetaEl.textContent = '비교 중...';
    this.resultListEl.replaceChildren();
    try {
      let session;
      if (payload.kind === 'ir') {
        const rightSnap = buildSnapshotFromWasm(wasm, rightName, HISTORY_COMPARE_OPTS);
        session = compareSnapshots(payload.snapshot, rightSnap, HISTORY_COMPARE_OPTS);
      } else {
        let cur: Uint8Array;
        try {
          cur = wasm.exportHwp();
        } catch {
          this.resultMetaEl.textContent = '현재 문서가 없습니다. 문서를 연 뒤 다시 시도하세요.';
          return;
        }
        session = await compareDocuments(payload.bytes, leftName, cur, rightName, HISTORY_COMPARE_OPTS);
      }
      console.log('[rhwp:history] 최종 Diff 배열', session.diffItems);
      this.compareSessionStore.set(session);
      const mode =
        session.textCompareStrategyUsed === 'identity' ? '본문=id(Map)' : '본문=정렬(alignment)';
      this.resultMetaEl.textContent = `${session.diffItems.length}개 차이 · ${mode} · "${leftName}" vs "${rightName}"`;
      this.renderDiffList(session.diffItems);
      this.services.eventBus.emit('compare:mode-changed', true);
      if (session.diffItems.length > 0) {
        this.compareSessionStore.gotoDiff(0);
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      this.resultMetaEl.textContent = `비교 실패: ${msg}`;
    }
  }

  private renderDiffList(items: DiffItem[]): void {
    this.resultListEl.replaceChildren();
    for (const [idx, item] of items.entries()) {
      const li = document.createElement('li');
      li.className = 'compare-result-item';
      li.dataset.diffId = item.id;
      const location = formatDiffLocationCombined(item);
      const leftText = this.formatPreviewText(this.sanitizeControlPreview(item.leftPreview));
      const rightText = this.formatPreviewText(this.sanitizeControlPreview(item.rightPreview));
      const previewLine = (item.kind === 'text' || item.severity !== 'modified')
        ? `<div class="compare-result-preview">L: ${this.escape(leftText)} / R: ${this.escape(rightText)}</div>`
        : '';
      const inline =
        item.inlineTextDiff !== undefined && item.inlineTextDiff !== ''
          ? `<div class="compare-result-inline-diff">${this.escape(this.formatInlineDiffText(item.inlineTextDiff))}</div>`
          : '';
      const valueDiff = this.renderValueDiff(item);
      li.innerHTML = `<strong>[${this.escape(this.kindLabel(item.kind))}] ${this.escape(item.title)}${location ? ` <span class="compare-result-location">(${this.escape(location)})</span>` : ''}</strong>${previewLine}${valueDiff}${inline}`;
      li.addEventListener('click', () => {
        this.compareSessionStore.gotoDiff(idx);
        this.resultListEl.querySelectorAll('.compare-result-item.active').forEach((el) => el.classList.remove('active'));
        li.classList.add('active');
        li.scrollIntoView({ block: 'nearest' });
      });
      this.resultListEl.appendChild(li);
    }
  }

  private escape(text: string): string {
    return text.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');
  }

  private formatPreviewText(text: string): string {
    const trimmed = text.trim();
    if (!trimmed) return '(없음)';
    const visible = this.makeWhitespaceVisible(trimmed);
    return this.truncateText(visible, 140);
  }

  private formatInlineDiffText(text: string): string {
    const compact = text
      .replaceAll('\r\n', '\n')
      .split('\n')
      .map((line) => this.makeWhitespaceVisible(line).trimEnd())
      .filter((line, idx, arr) => !(line === '' && idx > 0 && arr[idx - 1] === ''))
      .join('\n');
    return this.truncateText(compact, 500);
  }

  private makeWhitespaceVisible(text: string): string {
    return text
      .replaceAll('\t', '⇥ ')
      .replaceAll('\r\n', '\n')
      .replaceAll('\n', ' ↵ ')
      .replace(/\s{2,}/g, ' ');
  }

  private truncateText(text: string, maxLen: number): string {
    if (text.length <= maxLen) return text;
    return `${text.slice(0, maxLen - 1)}…`;
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

  private kindLabel(kind: DiffItem['kind']): string {
    if (kind === 'table') return '표';
    if (kind === 'shape') return '도형';
    if (kind === 'image') return '이미지';
    if (kind === 'chart') return '그래프';
    if (kind === 'text') return '텍스트';
    return '메타';
  }
}
