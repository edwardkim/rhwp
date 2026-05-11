import { formatDiffLocationCombined, formatParagraphLocationForSide, isComparePreviewAbsent } from '@/compare/diff-location-label';
import type { CompareSessionStore } from '@/compare/session';
import type { CompareSession, DiffAnchor, DiffItem } from '@/compare/types';
import { WasmBridge } from '@/core/wasm-bridge';

type CompareSourceDocument = {
  bytes: Uint8Array;
  fileName: string;
};

export class CompareResultWindow {
  private _open = false;
  private wrap!: HTMLDivElement;
  private titleEl!: HTMLSpanElement;
  private leftPane!: HTMLDivElement;
  private rightPane!: HTMLDivElement;
  private metaEl!: HTMLDivElement;
  private session: CompareSession | null = null;
  private store: CompareSessionStore | null = null;
  private leftTitleEl!: HTMLHeadingElement;
  private rightTitleEl!: HTMLHeadingElement;
  private leftCanvas!: HTMLCanvasElement;
  private rightCanvas!: HTMLCanvasElement;
  private leftMarker!: HTMLDivElement;
  private rightMarker!: HTMLDivElement;
  private leftCanvasWrap!: HTMLDivElement;
  private rightCanvasWrap!: HTMLDivElement;
  private leftStatusEl!: HTMLDivElement;
  private rightStatusEl!: HTMLDivElement;
  private leftWasm: WasmBridge | null = null;
  private rightWasm: WasmBridge | null = null;
  private leftDocKey = '';
  private rightDocKey = '';
  private leftSource: CompareSourceDocument | null = null;
  private rightSource: CompareSourceDocument | null = null;
  private loadingToken = 0;

  isOpen(): boolean {
    return this._open;
  }

  show(
    session: CompareSession,
    store: CompareSessionStore,
    initialIndex = 0,
    docs?: { left: CompareSourceDocument; right: CompareSourceDocument },
  ): void {
    this.session = session;
    this.store = store;
    if (docs) {
      this.leftSource = docs.left;
      this.rightSource = docs.right;
    }
    if (!this._open) {
      this._open = true;
      this.build();
      document.body.appendChild(this.wrap);
    }
    this.titleEl.textContent = `문서 비교 상세 · ${session.left.name} ↔ ${session.right.name}`;
    this.leftTitleEl.textContent = `왼쪽 문서: ${session.left.name}`;
    this.rightTitleEl.textContent = `오른쪽 문서: ${session.right.name}`;
    void this.focusDiff(initialIndex);
  }

  hide(): void {
    this._open = false;
    this.wrap?.remove();
    try {
      this.leftWasm?.releaseDocument();
      this.rightWasm?.releaseDocument();
    } catch {
      /* noop */
    }
    this.leftWasm = null;
    this.rightWasm = null;
    this.leftDocKey = '';
    this.rightDocKey = '';
    this.leftSource = null;
    this.rightSource = null;
    this.session = null;
    this.store = null;
  }

  async focusDiff(index: number): Promise<void> {
    if (!this.session) return;
    const item = this.session.diffItems[index];
    if (!item) return;
    await this.ensureCompareDocumentsLoaded();
    const locCombined = formatDiffLocationCombined(item);
    this.metaEl.textContent = locCombined
      ? `[${item.kind}] ${item.title}\n${locCombined}`
      : `[${item.kind}] ${item.title}`;
    this.leftPane.innerHTML = this.highlightPreview(item, 'left');
    this.rightPane.innerHTML = this.highlightPreview(item, 'right');
    this.renderRealDocumentPreview(item);
  }

  private build(): void {
    this.wrap = document.createElement('div');
    this.wrap.className = 'compare-inspector-window';

    const head = document.createElement('div');
    head.className = 'compare-inspector-head';
    this.titleEl = document.createElement('span');
    this.titleEl.textContent = '문서 비교 상세';
    const close = document.createElement('button');
    close.className = 'dialog-close';
    close.textContent = '\u00D7';
    close.addEventListener('click', () => this.hide());
    head.append(this.titleEl, close);

    const body = document.createElement('div');
    body.className = 'compare-inspector-body';
    this.metaEl = document.createElement('div');
    this.metaEl.className = 'compare-inspector-meta';
    this.metaEl.style.whiteSpace = 'pre-line';

    const panes = document.createElement('div');
    panes.className = 'compare-inspector-panes';
    const leftWrap = document.createElement('div');
    leftWrap.className = 'compare-inspector-pane';
    this.leftTitleEl = document.createElement('h4');
    this.leftTitleEl.textContent = '왼쪽 문서';
    this.leftStatusEl = document.createElement('div');
    this.leftStatusEl.className = 'compare-inspector-page-status';
    this.leftStatusEl.textContent = '페이지 준비 중...';
    this.leftCanvasWrap = document.createElement('div');
    this.leftCanvasWrap.className = 'compare-inspector-canvas-wrap';
    this.leftCanvas = document.createElement('canvas');
    this.leftCanvas.className = 'compare-inspector-canvas';
    this.leftMarker = document.createElement('div');
    this.leftMarker.className = 'compare-inspector-anchor-marker';
    this.leftCanvasWrap.append(this.leftCanvas, this.leftMarker);
    this.leftPane = document.createElement('div');
    this.leftPane.className = 'compare-inspector-content';
    leftWrap.append(this.leftTitleEl, this.leftStatusEl, this.leftCanvasWrap, this.leftPane);

    const rightWrap = document.createElement('div');
    rightWrap.className = 'compare-inspector-pane';
    this.rightTitleEl = document.createElement('h4');
    this.rightTitleEl.textContent = '오른쪽 문서';
    this.rightStatusEl = document.createElement('div');
    this.rightStatusEl.className = 'compare-inspector-page-status';
    this.rightStatusEl.textContent = '페이지 준비 중...';
    this.rightCanvasWrap = document.createElement('div');
    this.rightCanvasWrap.className = 'compare-inspector-canvas-wrap';
    this.rightCanvas = document.createElement('canvas');
    this.rightCanvas.className = 'compare-inspector-canvas';
    this.rightMarker = document.createElement('div');
    this.rightMarker.className = 'compare-inspector-anchor-marker';
    this.rightCanvasWrap.append(this.rightCanvas, this.rightMarker);
    this.rightPane = document.createElement('div');
    this.rightPane.className = 'compare-inspector-content';
    rightWrap.append(this.rightTitleEl, this.rightStatusEl, this.rightCanvasWrap, this.rightPane);
    panes.append(leftWrap, rightWrap);

    const nav = document.createElement('div');
    nav.className = 'compare-inspector-nav';
    const prev = document.createElement('button');
    prev.className = 'dialog-btn';
    prev.textContent = '이전 차이';
    prev.addEventListener('click', () => {
      const item = this.store?.prevDiff();
      if (!item || !this.session) return;
      void this.focusDiff(this.session.currentDiffIndex);
    });
    const next = document.createElement('button');
    next.className = 'dialog-btn';
    next.textContent = '다음 차이';
    next.addEventListener('click', () => {
      const item = this.store?.nextDiff();
      if (!item || !this.session) return;
      void this.focusDiff(this.session.currentDiffIndex);
    });
    nav.append(prev, next);

    body.append(this.metaEl, panes, nav);
    this.wrap.append(head, body);
  }

  private highlightPreview(item: DiffItem, side: 'left' | 'right'): string {
    const severity = item.severity;
    let leftText: string;
    let rightText: string;
    if (item.kind === 'table' && severity === 'modified') {
      const narrowed = this.formatTableCprevChangedCellsOnly(
        item.leftPreview || '',
        item.rightPreview || '',
      );
      if (narrowed) {
        leftText = narrowed.left;
        rightText = narrowed.right;
      } else {
        leftText = this.formatInspectorText(item.leftPreview || '(없음)');
        rightText = this.formatInspectorText(item.rightPreview || '(없음)');
      }
    } else {
      leftText = this.formatInspectorText(item.leftPreview || '(없음)');
      rightText = this.formatInspectorText(item.rightPreview || '(없음)');
    }
    const raw = side === 'left' ? item.leftPreview : item.rightPreview;
    const text = side === 'left' ? leftText : rightText;
    if (isComparePreviewAbsent(raw)) {
      return `<pre>${this.escape(text)}</pre>`;
    }
    if (severity === 'added' && side === 'right') {
      return `<pre><mark>${this.escape(text)}</mark></pre>`;
    }
    if (severity === 'removed' && side === 'left') {
      return `<pre><mark>${this.escape(text)}</mark></pre>`;
    }
    if (severity !== 'modified') return `<pre>${this.escape(text)}</pre>`;

    const a = leftText;
    const b = rightText;
    let start = 0;
    const minLen = Math.min(a.length, b.length);
    while (start < minLen && a.charCodeAt(start) === b.charCodeAt(start)) start += 1;
    let enda = a.length - 1;
    let endb = b.length - 1;
    while (enda >= start && endb >= start && a.charCodeAt(enda) === b.charCodeAt(endb)) {
      enda -= 1;
      endb -= 1;
    }
    const source = side === 'left' ? a : b;
    const end = side === 'left' ? enda : endb;
    const before = source.slice(0, start);
    const changed = source.slice(start, end + 1);
    const after = source.slice(end + 1);
    if (!changed) return `<pre>${this.escape(source)}</pre>`;
    return this.renderFocusedDiff(before, changed, after);
  }

  /**
   * 표 텍스트 변경: `cprev`/`tprev` 셀 맵을 비교해 **값이 달라진 셀만** 좌·우 각각 한 줄씩 만든다.
   * (기존 `formatInspectorText`는 앞 5셀만 잘라 노이즈가 컸음.)
   */
  private formatTableCprevChangedCellsOnly(
    leftRaw: string,
    rightRaw: string,
  ): { left: string; right: string } | null {
    const lk = this.parseKvSummary(leftRaw);
    const rk = this.parseKvSummary(rightRaw);
    const pick = (kv: Record<string, string>) => {
      const cp = kv.cprev;
      if (cp && cp !== '(없음)') return cp;
      const tp = kv.tprev;
      if (tp && tp !== '(없음)') return tp;
      return '';
    };
    const lc = pick(lk);
    const rc = pick(rk);
    if (!lc && !rc) return null;
    const Lm = this.parseCellPreviewToMap(lc);
    const Rm = this.parseCellPreviewToMap(rc);
    if (Lm.size === 0 && Rm.size === 0) return null;
    const keys = new Set<string>([...Lm.keys(), ...Rm.keys()]);
    const changed: string[] = [];
    for (const k of keys) {
      if ((Lm.get(k) ?? '') !== (Rm.get(k) ?? '')) changed.push(k);
    }
    changed.sort((ka, kb) => {
      const ma = ka.match(/^r(\d+)c(\d+)$/i);
      const mb = kb.match(/^r(\d+)c(\d+)$/i);
      if (!ma || !mb) return ka.localeCompare(kb);
      const ra = Number(ma[1]);
      const ca = Number(ma[2]);
      const rb = Number(mb[1]);
      const cb = Number(mb[2]);
      return ra !== rb ? ra - rb : ca - cb;
    });
    if (changed.length === 0) return { left: '(셀 텍스트 동일)', right: '(셀 텍스트 동일)' };
    const cellLabel = (k: string) => k.replace(/^r(\d+)c(\d+)$/i, '$1행$2열');
    const left = changed.map((k) => `${cellLabel(k)}: ${Lm.get(k) ?? '(없음)'}`).join('\n');
    const right = changed.map((k) => `${cellLabel(k)}: ${Rm.get(k) ?? '(없음)'}`).join('\n');
    return { left, right };
  }

  private parseCellPreviewToMap(raw: string): Map<string, string> {
    const m = new Map<string, string>();
    for (const [k, v] of this.parseCellPreview(raw)) m.set(k, v);
    return m;
  }

  private formatInspectorText(raw: string): string {
    if (!raw) return '(없음)';
    if (!raw.includes('=')) return raw;

    const kv = this.parseKvSummary(raw);
    if (Object.keys(kv).length === 0) return raw;

    const lines: string[] = [];
    const push = (label: string, value?: string) => {
      if (!value || value === '(없음)' || value === 'nopix' || value === 'nobox') return;
      lines.push(`${label}: ${value}`);
    };

    const cprev = kv.cprev;
    if (cprev && cprev !== '(없음)') {
      const cells = this.parseCellPreview(cprev);
      if (cells.length > 0) {
        for (const [cell, text] of cells.slice(0, 5)) {
          lines.push(`${cell.replace(/^r(\d+)c(\d+)$/i, '$1행$2열')}: ${text}`);
        }
        if (cells.length > 5) lines.push(`... 외 ${cells.length - 5}개 셀`);
      } else {
        push('셀 텍스트', cprev);
      }
    }

    push('행', kv.r);
    push('열', kv.c);
    push('크기', kv.box?.replace(/^(-?\d+)x(-?\d+)$/, '$1px × $2px'));
    push('텍스트', kv.text);
    push('자르기', kv.crop);
    push('효과', kv.effect);
    push('밝기/대비', kv.bc);
    push('회전', kv.rot ? `${kv.rot}도` : undefined);
    push('대칭', kv.flip);
    push('배치', kv.wrap);
    push('기준', kv.rel);

    if (lines.length === 0) return raw;
    return lines.join('\n');
  }

  private parseKvSummary(summary: string): Record<string, string> {
    const out: Record<string, string> = {};
    for (const m of summary.matchAll(/([a-z]+)=("([^"]*)"|[^\s]+)/g)) {
      const raw = m[2] ?? '';
      const unquoted = raw.startsWith('"') && raw.endsWith('"') ? raw.slice(1, -1) : raw;
      out[m[1]] = unquoted;
    }
    return out;
  }

  private parseCellPreview(raw: string): Array<[string, string]> {
    const out: Array<[string, string]> = [];
    const parts = raw.includes('&') ? raw.split('&') : raw.split(';');
    for (const p of parts) {
      const part = p.trim();
      if (!part) continue;
      const idx = part.indexOf('=');
      const legacyIdx = part.indexOf(':');
      const cut = idx > 0 ? idx : legacyIdx;
      if (cut <= 0) continue;
      const key = part.slice(0, cut).trim();
      const valRaw = part.slice(cut + 1).trim();
      let val = valRaw;
      try { val = decodeURIComponent(valRaw); } catch { val = valRaw; }
      out.push([key, val]);
    }
    return out;
  }

  private renderFocusedDiff(before: string, changed: string, after: string): string {
    const sideContext = 90;
    const hasBeforeTrim = before.length > sideContext;
    const hasAfterTrim = after.length > sideContext;
    const beforeSlice = hasBeforeTrim ? before.slice(before.length - sideContext) : before;
    const afterSlice = hasAfterTrim ? after.slice(0, sideContext) : after;
    const lead = hasBeforeTrim ? '…' : '';
    const tail = hasAfterTrim ? '…' : '';
    return `<pre>${this.escape(lead + beforeSlice)}<mark>${this.escape(changed)}</mark>${this.escape(afterSlice + tail)}</pre>`;
  }

  private escape(text: string): string {
    return text.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');
  }

  private async ensureCompareDocumentsLoaded(): Promise<void> {
    if (!this.leftSource || !this.rightSource) return;
    const token = ++this.loadingToken;
    this.leftStatusEl.textContent = '왼쪽 문서 로딩 중...';
    this.rightStatusEl.textContent = '오른쪽 문서 로딩 중...';
    try {
      const leftKey = `${this.leftSource.fileName}:${this.leftSource.bytes.byteLength}`;
      const rightKey = `${this.rightSource.fileName}:${this.rightSource.bytes.byteLength}`;

      if (!this.leftWasm) {
        this.leftWasm = new WasmBridge();
        await this.leftWasm.initialize();
      }
      if (!this.rightWasm) {
        this.rightWasm = new WasmBridge();
        await this.rightWasm.initialize();
      }
      if (this.loadingToken !== token) return;

      if (this.leftDocKey !== leftKey) {
        this.leftWasm.loadDocument(this.leftSource.bytes, this.leftSource.fileName);
        this.leftDocKey = leftKey;
      }
      if (this.rightDocKey !== rightKey) {
        this.rightWasm.loadDocument(this.rightSource.bytes, this.rightSource.fileName);
        this.rightDocKey = rightKey;
      }
      if (this.loadingToken !== token) return;
      try {
        this.leftWasm.refreshLayout();
      } catch {
        /* noop */
      }
      try {
        this.rightWasm.refreshLayout();
      } catch {
        /* noop */
      }
      if (this.loadingToken !== token) return;
      this.leftStatusEl.textContent = '왼쪽 문서 페이지 준비 완료';
      this.rightStatusEl.textContent = '오른쪽 문서 페이지 준비 완료';
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      this.leftStatusEl.textContent = `페이지 로드 실패: ${msg}`;
      this.rightStatusEl.textContent = `페이지 로드 실패: ${msg}`;
    }
  }

  private renderRealDocumentPreview(item: DiffItem): void {
    this.renderSidePage('left', this.leftWasm, this.leftCanvas, this.leftMarker, this.leftStatusEl, item);
    this.renderSidePage('right', this.rightWasm, this.rightCanvas, this.rightMarker, this.rightStatusEl, item);
  }

  /**
   * 스냅샷 비교 때 기록된 앵커가 한쪽만 있으면(**추가/삭제**) 반대 문서 패널에서는
   * 상대쪽 pageIndex를 쓰면 안 된다(pagination 불일치). alignment 짝 문단 경로가 있으면
   * `getCursorRect`로 해당 **문서** 기준 페이지·좌표를 구한다.
   */
  private resolveRenderAnchor(
    wasm: WasmBridge,
    item: DiffItem,
    side: 'left' | 'right',
  ): { anchor: DiffAnchor; fromDiffEngine: boolean } | null {
    const snapshot = side === 'left' ? item.leftAnchor : item.rightAnchor;
    if (snapshot && typeof snapshot.pageIndex === 'number') {
      return { anchor: snapshot, fromDiffEngine: true };
    }
    const peerPath = side === 'left' ? item.contextOnLeft : item.contextOnRight;
    if (!peerPath) return null;
    try {
      const rect = wasm.getCursorRect(peerPath.section, peerPath.paragraph, 0);
      const h = Math.max(14, rect.height);
      const anchor = {
        pageIndex: rect.pageIndex,
        x: rect.x,
        y: rect.y,
        width: Math.max(28, Math.min(520, h * 3)),
        height: h,
      };
      return { anchor, fromDiffEngine: false };
    } catch {
      return null;
    }
  }

  private renderSidePage(
    side: 'left' | 'right',
    wasm: WasmBridge | null,
    canvas: HTMLCanvasElement,
    marker: HTMLDivElement,
    statusEl: HTMLDivElement,
    item: DiffItem,
  ): void {
    if (!wasm) {
      statusEl.textContent = '문서가 아직 로드되지 않았습니다.';
      marker.style.display = 'none';
      return;
    }
    const resolved = this.resolveRenderAnchor(wasm, item, side);
    if (!resolved) {
      const locShort = formatParagraphLocationForSide(item, side);
      const base =
        side === 'left'
          ? '왼쪽: 스냅샷 직후 위치 정보가 없습니다. (텍스트 미리보기만 참고)'
          : '오른쪽: 스냅샷 직후 위치 정보가 없습니다. (텍스트 미리보기만 참고)';
      statusEl.textContent = locShort ? `${locShort} · ${base}` : base;
      const ctx = canvas.getContext('2d');
      if (ctx) ctx.clearRect(0, 0, canvas.width, canvas.height);
      marker.style.display = 'none';
      return;
    }

    const { anchor: ea, fromDiffEngine } = resolved;
    try {
      const info = wasm.getPageInfo(ea.pageIndex);
      const wrap = side === 'left' ? this.leftCanvasWrap : this.rightCanvasWrap;
      const maxWidth = Math.max(260, wrap.clientWidth - 10);
      const scale = Math.max(0.25, Math.min(1.25, maxWidth / Math.max(1, info.width)));
      const pageIdx = ea.pageIndex;
      const draw = (): void => {
        try {
          canvas.width = Math.max(1, Math.floor(info.width * scale));
          canvas.height = Math.max(1, Math.floor(info.height * scale));
          wasm.renderPageToCanvasFiltered(pageIdx, canvas, scale, 'all');
          const locShort = formatParagraphLocationForSide(item, side);
          const pageLine = `${ea.pageIndex + 1}쪽`;
          const contextNote = !fromDiffEngine ? ' · 직전 정렬 짝 문단 기준(마커 없음)' : '';
          if (fromDiffEngine) {
            marker.style.display = 'block';
            marker.style.left = `${Math.max(0, Math.floor(ea.x * scale))}px`;
            marker.style.top = `${Math.max(0, Math.floor(ea.y * scale))}px`;
            marker.style.width = `${Math.max(14, Math.floor(ea.width * scale))}px`;
            marker.style.height = `${Math.max(14, Math.floor(ea.height * scale))}px`;
          } else {
            marker.style.display = 'none';
          }
          statusEl.textContent = `${locShort ? `${locShort} · ` : ''}${pageLine} 실제 화면${contextNote}`;
          wrap.scrollTop = Math.max(0, marker.offsetTop - Math.floor(wrap.clientHeight * 0.15));
        } catch (err) {
          const msg = err instanceof Error ? err.message : String(err);
          statusEl.textContent = `페이지 렌더 실패: ${msg}`;
          marker.style.display = 'none';
        }
      };
      requestAnimationFrame(() => {
        requestAnimationFrame(draw);
      });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      statusEl.textContent = `페이지 렌더 실패: ${msg}`;
      marker.style.display = 'none';
    }
  }

}

