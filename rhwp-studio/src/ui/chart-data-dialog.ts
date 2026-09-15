/**
 * [#4694] 차트 데이터 편집 대화상자 — 그리드(행=포인트, 열=계열)로 데이터를 고친다.
 *
 * 편집 판단은 전부 순수 함수(`@/core/chart-data-target`, `@/core/chart-grid-model`)와
 * 코어 검증기에 있다: 입력시 선제 검증 → [확인] 시 dryRun(코어가 단일 진실) →
 * executeOperation(snapshot/objectProps) 실쓰기. 무변경이면 어떤 쓰기·undo 기록도
 * 없이 닫는다. 쓰기 직전 operation 클로저 안에서 재열거·재대조해 index 드리프트를
 * 차단하고, 재대조 실패는 무기록 + 안내다(오매칭으로 다른 차트를 고치는 것이 최악).
 *
 * [#6053 B2] 그리드 셀 우클릭으로 행·열을 더하고 지우며, 계열명과 카테고리 라벨도 고친다.
 * 상태는 DOM 이 아니라 `GridModel` 이 쥔다 — 재렌더가 무손실이어야 구조 연산이 성립한다.
 * 페이로드가 코어 B1 검증의 네 거부를 건드릴 때만 `structure: true` 를 켠다(`needsStructure`).
 *
 * 사전 비활성은 **봉투에서 유도되는 것만** 한다(다층·비공유 라벨, 마지막 행·열, 캔들 양끝).
 * 그 밖의 거부(빈 값 자리 이동, `c:tx` 부재 등)는 막지 않고 코어 dryRun 이 판정한다 —
 * 코어 규칙을 UI 가 재현하기 시작하면 둘이 어긋나고, 과잉 차단이 미달 차단보다 나쁘다.
 */
import { ModalDialog } from './dialog';
import { LocalContextMenu, type LocalMenuItem } from './local-context-menu';
import type { WasmBridge } from '@/core/wasm-bridge';
import type { EventBus } from '@/core/event-bus';
import type { CommandServices } from '@/command/types';
import {
  buildChartEdits,
  cellInputIssue,
  hasAnyEdit,
  needsStructure,
  unsafeTextIssue,
  type ChartContainerRefJson,
  type ChartDataResult,
  type ChartInvalidEntry,
  type ChartRefJson,
} from '@/core/chart-data-target';
import {
  deleteColumn,
  deleteRow,
  gridFromChartData,
  gridLabels,
  gridSeriesNames,
  gridValues,
  insertColumn,
  insertRow,
  setCell,
  setLabel,
  setSeriesName,
  type GridModel,
} from '@/core/chart-grid-model';

import { t } from '../i18n/index.ts';
const EMPTY_CELL_NOTE =
  '원본이 빈 값(결측)이라 이 자리는 고칠 수 없습니다 — 행을 새로 넣으면 값을 쓸 수 있습니다.';
const ABSENT_CELL_NOTE = '원본에 이 계열의 값이 없던 자리입니다.';
const NO_NAME_NOTE = '이 계열은 이름 칸(c:tx)이 없어 이름을 넣을 자리가 없습니다.';
const GRID_NOTE = '셀을 우클릭하면 행·계열을 넣거나 지울 수 있습니다.';
const PIE_EXTRA_SERIES_NOTE =
  '원형 차트는 첫 계열만 그리므로, 추가한 계열은 화면에 나타나지 않습니다.';
const CANDLE_NOTE =
  '주식형 캔들은 첫·끝 계열을 몸통으로 삼습니다 — 양끝을 바꾸면 그림이 깨집니다.';

/** 주소 동일성 — 재열거 결과에서 같은 차트를 다시 찾는 기준(순번은 흔들려도 주소는 남는다). */
function sameChartAddress(a: ChartRefJson, b: ChartRefJson): boolean {
  const ac = a.container ?? [];
  const bc = b.container ?? [];
  return (
    a.section === b.section &&
    a.paragraph === b.paragraph &&
    a.control === b.control &&
    ac.length === bc.length &&
    ac.every((level: ChartContainerRefJson, i: number) =>
      level.kind === bc[i].kind &&
      level.control === bc[i].control &&
      level.paragraph === bc[i].paragraph &&
      level.cell === bc[i].cell)
  );
}

export class ChartDataDialog extends ModalDialog {
  private wasm: WasmBridge;
  private eventBus: EventBus;
  private services?: CommandServices;

  private chart: ChartRefJson | null = null;
  private data: ChartDataResult | null = null;
  /** 편집의 단일 진실. DOM 은 이것의 투영일 뿐이다. */
  private model: GridModel | null = null;
  private gridWrap!: HTMLDivElement;
  private errorEl!: HTMLDivElement;
  private menu = new LocalContextMenu();

  constructor(wasm: WasmBridge, eventBus: EventBus, services?: CommandServices) {
    super(t('dialog.chartData.sameChartAddress.title'), 560, false);
    this.wasm = wasm;
    this.eventBus = eventBus;
    this.services = services;
  }

  /**
   * 열거 항목(matchChartRef 의 결과)으로 연다. 데이터를 읽지 못하면 열지 않고
   * false — 호출부(커맨드)가 안내를 맡는다.
   */
  open(chart: ChartRefJson): boolean {
    const data = this.wasm.getChartDataByIndex(chart.index);
    if (!data.ok || !data.series || data.series.length === 0) return false;
    this.chart = chart;
    this.data = data;
    this.model = gridFromChartData(data);
    this.show();
    this.renderGrid();
    return true;
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'chart-data-body';

    this.gridWrap = document.createElement('div');
    this.gridWrap.className = 'chart-data-grid-wrap';
    body.appendChild(this.gridWrap);

    this.errorEl = document.createElement('div');
    this.errorEl.className = 'chart-data-error';
    this.errorEl.style.display = 'none';
    body.appendChild(this.errorEl);

    const note = document.createElement('div');
    note.className = 'chart-data-note';
    note.textContent = GRID_NOTE;
    body.appendChild(note);

    return body;
  }

  /** 모델을 갈아 끼우고 그리드를 다시 그린다 — 모델이 진실이라 입력값이 살아남는다. */
  private applyModel(next: GridModel): void {
    if (next === this.model) return;
    this.model = next;
    this.renderGrid();
  }

  private renderGrid(): void {
    const model = this.model;
    if (!model) return;
    this.hideError();
    this.menu.hide();
    this.gridWrap.textContent = '';

    const scatter = model.axis === 'scatter';
    const labelsOpen = model.labelsUsable;

    const table = document.createElement('table');
    table.className = 'chart-data-grid';

    const head = table.createTHead().insertRow();
    const corner = document.createElement('th');
    corner.textContent = scatter ? 'X' : t('dialog.chartData.corner.text');
    head.appendChild(corner);
    model.series.forEach((s, si) => {
      const th = document.createElement('th');
      th.dataset.series = String(si);
      if (s.name === null) {
        // 이름 칸이 없는 계열은 열지 않는다 — 열면 코어가 반드시 거부한다.
        // `계열 N` 은 표시용 대체 문구일 뿐이므로 모델에 들어가지 않는다.
        th.className = 'chart-data-series-locked';
        th.textContent = t('dialog.chartData.th.text', { p1: si + 1 });
        th.title = NO_NAME_NOTE;
      } else {
        const input = this.textInput(s.name, t('dialog.chartData.textInput.label'));
        input.addEventListener('input', () => {
          this.markText(input);
          this.model = setSeriesName(this.model!, si, input.value);
        });
        th.appendChild(input);
      }
      head.appendChild(th);
    });

    const tbody = table.createTBody();
    for (let r = 0; r < model.rowCount; r++) {
      const row = tbody.insertRow();

      const labelCell = document.createElement('th');
      labelCell.dataset.row = String(r);
      if (labelsOpen) {
        const label = model.labels[r];
        const input = scatter
          ? this.numberInput(label.text)
          : this.textInput(label.text, scatter ? t('dialog.chartData.renderGrid.label') : t('dialog.chartData.renderGrid.label.xc19837'));
        if (label.origin === 'new') input.classList.add('chart-data-new');
        input.addEventListener('input', () => {
          if (scatter) this.markCell(input);
          else this.markText(input);
          this.model = setLabel(this.model!, r, input.value);
        });
        labelCell.appendChild(input);
      } else {
        labelCell.textContent = model.labels[r].text || String(r + 1);
        labelCell.title = scatter
          ? t('dialog.chartData.labelCell.tooltip')
          : t('dialog.chartData.labelCell.tooltip.x3df2db');
      }
      row.appendChild(labelCell);

      model.series.forEach((s, si) => {
        const td = row.insertCell();
        td.dataset.series = String(si);
        td.dataset.row = String(r);
        const cell = s.cells[r];
        const input = this.numberInput(cell.text);
        if (cell.origin.kind === 'empty' || cell.origin.kind === 'absent') {
          // 코어가 제자리 치환을 거부하는 자리다(valueNotPatchable). 값만 바꾸는 편집으로는
          // 손댈 수 없고, 행을 새로 넣어야 쓸 수 있다.
          input.disabled = true;
          input.title = cell.origin.kind === 'empty' ? EMPTY_CELL_NOTE : ABSENT_CELL_NOTE;
        } else {
          if (cell.origin.kind === 'new') input.classList.add('chart-data-new');
          input.addEventListener('input', () => {
            this.markCell(input);
            this.model = setCell(this.model!, si, r, input.value);
          });
        }
        td.appendChild(input);
      });
    }

    table.addEventListener('contextmenu', (e) => this.onGridContextMenu(e));
    this.gridWrap.appendChild(table);
  }

  /** 수치 칸 — `cellInputIssue` 로 검증한다. */
  private numberInput(value: string): HTMLInputElement {
    const input = document.createElement('input');
    input.className = 'chart-data-input';
    input.inputMode = 'decimal';
    input.dataset.kind = 'number';
    input.value = value;
    return input;
  }

  /** 텍스트 칸(계열명·카테고리 라벨) — `unsafeTextIssue` 로 검증한다. */
  private textInput(value: string, title: string): HTMLInputElement {
    const input = document.createElement('input');
    input.className = 'chart-data-input chart-data-text';
    input.dataset.kind = 'text';
    input.title = title;
    input.value = value;
    return input;
  }

  // ── 우클릭 메뉴 ─────────────────────────────────────────────────────────

  private onGridContextMenu(e: MouseEvent): void {
    const model = this.model;
    const data = this.data;
    if (!model || !data) return;
    const cell = (e.target as HTMLElement | null)?.closest('td,th') as HTMLElement | null;
    if (!cell) return;
    e.preventDefault();

    const row = cell.dataset.row === undefined ? null : Number(cell.dataset.row);
    const series = cell.dataset.series === undefined ? null : Number(cell.dataset.series);
    const items = this.menuItems(model, data, row, series);
    if (items.length === 0) return;
    this.menu.show(e.clientX, e.clientY, items);
  }

  /**
   * 메뉴 항목과 사전 비활성. 봉투에서 유도되는 것만 막는다 —
   * 라벨 구조(다층·비공유), 마지막 행·열, 캔들 장치의 양끝 계열(#6037).
   */
  private menuItems(
    model: GridModel,
    data: ChartDataResult,
    row: number | null,
    series: number | null,
  ): LocalMenuItem[] {
    const items: LocalMenuItem[] = [];

    // 행 연산은 라벨 열을 목표 상태로 함께 보내야 성립한다(labelsRequired/scatterXYMismatch).
    const rowsBlocked = model.labelsUsable
      ? undefined
      : data.labelsMultiLevel === true
        ? '다층 카테고리 차트는 행 구조를 바꿀 수 없습니다.'
        : '계열마다 라벨이 달라 행 구조를 바꿀 수 없습니다.';

    if (row !== null) {
      items.push(
        {
          type: 'command',
          label: t('dialog.chartData.menuItems.label'),
          disabledReason: rowsBlocked,
          run: () => this.applyModel(insertRow(model, row)),
        },
        {
          type: 'command',
          label: t('dialog.chartData.menuItems.label.x3b565a'),
          disabledReason: rowsBlocked,
          run: () => this.applyModel(insertRow(model, row + 1)),
        },
        {
          type: 'command',
          label: t('dialog.chartData.menuItems.label.x928e00'),
          disabledReason:
            rowsBlocked ?? (model.rowCount <= 1 ? '마지막 행은 지울 수 없습니다.' : undefined),
          run: () => this.applyModel(deleteRow(model, row)),
        },
      );
    }

    if (series !== null) {
      if (items.length > 0) items.push({ type: 'separator' });
      const last = model.series.length - 1;
      // [#6037] c:upDownBars 는 첫 계열과 끝 계열을 캔들 몸통으로 삼는다. 어느 쪽이든
      // 바뀌면 몸통이 엉뚱한 짝으로 다시 잡혀 전부 검은 박스가 된다(candleAnchorBroken).
      // 중간 삽입·중간 삭제는 양끝이 그대로라 정상이므로 막지 않는다.
      const candle = data.hasUpDownBars === true;
      // 원형은 첫 계열만 그린다 — 파손이 아니라 무효과라 막지 않고 알려만 준다(#6037).
      const pieNote = data.plot === 'pie' || data.plot === 'ofPie' ? PIE_EXTRA_SERIES_NOTE : undefined;

      items.push(
        {
          type: 'command',
          label: t('dialog.chartData.menuItems.label.x49f4ca'),
          disabledReason: candle && series === 0 ? CANDLE_NOTE : undefined,
          note: pieNote,
          run: () => this.applyModel(insertColumn(model, series)),
        },
        {
          type: 'command',
          label: t('dialog.chartData.menuItems.label.x1e0262'),
          disabledReason: candle && series === last ? CANDLE_NOTE : undefined,
          note: pieNote,
          run: () => this.applyModel(insertColumn(model, series + 1)),
        },
        {
          type: 'command',
          label: t('dialog.chartData.menuItems.label.xb8a829'),
          disabledReason:
            model.series.length <= 1
              ? '마지막 계열은 지울 수 없습니다.'
              : candle && (series === 0 || series === last)
                ? CANDLE_NOTE
                : undefined,
          run: () => this.applyModel(deleteColumn(model, series)),
        },
      );
    }

    return items;
  }

  // ── 선제 검증 — 최종 판정은 [확인]의 dryRun 이다 ──────────────────────────

  private markCell(input: HTMLInputElement): void {
    input.classList.toggle('chart-data-invalid', cellInputIssue(input.value) !== null);
  }

  /** 계열명·카테고리 라벨은 수치가 아니다 — 코어 `is_safe_text` 와 같은 문자 집합을 본다. */
  private markText(input: HTMLInputElement): void {
    input.classList.toggle('chart-data-invalid', unsafeTextIssue(input.value) !== null);
  }

  /** 화면에 남은 위반 칸을 표시하고 하나라도 있으면 true. */
  private hasBrokenInput(): boolean {
    let broken = false;
    for (const input of Array.from(this.gridWrap.querySelectorAll('input'))) {
      if (input.disabled) continue;
      const bad =
        input.dataset.kind === 'text'
          ? unsafeTextIssue(input.value) !== null
          : cellInputIssue(input.value) !== null;
      input.classList.toggle('chart-data-invalid', bad);
      if (bad) broken = true;
    }
    return broken;
  }

  private showError(message: string): void {
    this.errorEl.textContent = message;
    this.errorEl.style.display = '';
  }

  private showInvalid(invalid: ChartInvalidEntry[] | undefined): void {
    const lines = (invalid ?? []).map((e) => e.message ?? e.reason);
    this.showError(lines.length > 0 ? lines.join('\n') : t('dialog.chartData.showInvalid.label'));
  }

  private hideError(): void {
    this.errorEl.style.display = 'none';
  }

  protected onConfirm(): void | boolean {
    const chart = this.chart;
    const data = this.data;
    const model = this.model;
    if (!chart || !data || !model) return;

    const values = gridValues(model);
    const labels = model.labelsUsable ? gridLabels(model) : undefined;
    const names = gridSeriesNames(model);

    if (this.hasBrokenInput()) {
      this.showError(t('dialog.chartData.showError.label'));
      return false;
    }

    // 무변경이면 쓰기도 undo 기록도 없이 닫는다.
    if (!hasAnyEdit(data, values, labels, names)) return;

    // 코어 B1 검증의 네 거부가 설 페이로드일 때만 구조 의도를 켠다 — 항상 켜면 그리드
    // 조립 버그가 거부 대신 조용한 계열 절단이 된다.
    const structure = needsStructure(data, values, labels, names);
    const edits = buildChartEdits(data, values, labels, structure ? { structure, names } : undefined);

    // 코어 검증기가 단일 진실 — dryRun 거부면 닫지 않고 사유를 보여준다.
    const probe = this.wasm.setChartDataByIndex(chart.index, { ...edits, dryRun: true });
    if (!probe.ok) {
      this.showInvalid(probe.invalid);
      return false;
    }
    if ((probe.changedCount ?? 0) === 0) return;

    const ih = this.services?.getInputHandler();
    if (!ih) {
      const res = this.wasm.setChartDataByIndex(chart.index, edits);
      if (!res.ok) {
        this.showInvalid(res.invalid);
        return false;
      }
      this.eventBus.emit('document-changed');
      return;
    }

    // null 반환은 무기록 신호인데 사유가 셋이라 결과로 구분한다 — 뭉개면
    // "쓰기 시점 이미 같은 값"(무해)에 오안내를 내보내게 된다.
    // (객체 홀더인 이유: 클로저 안 대입을 CFA 가 못 봐 유니온 let 은 협착된다.)
    const result: {
      outcome: 'applied' | 'noop' | 'refused' | 'notFound';
      invalid?: ChartInvalidEntry[];
    } = { outcome: 'notFound' };
    ih.executeOperation({
      kind: 'snapshot',
      operationType: 'objectProps',
      operation: () => {
        // 다이얼로그가 열린 사이 문서가 바뀌었을 수 있다 — 주소로 다시 찾는다.
        const found = this.wasm.listCharts().find((c) => sameChartAddress(c, chart));
        if (!found) return null;
        const res = this.wasm.setChartDataByIndex(found.index, edits);
        if (!res.ok) {
          result.outcome = 'refused';
          result.invalid = res.invalid ?? [];
          return null;
        }
        if ((res.changedCount ?? 0) === 0) {
          result.outcome = 'noop';
          return null;
        }
        result.outcome = 'applied';
        return ih.getCursorPosition();
      },
    });
    if (result.outcome === 'refused') {
      this.showInvalid(result.invalid);
      return false;
    }
    if (result.outcome === 'notFound') {
      this.showError(t('dialog.chartData.showError.label.xc15233'));
      return false;
    }
    // applied·noop(쓰기 시점 이미 같은 값) 모두 닫는다.
  }
}
