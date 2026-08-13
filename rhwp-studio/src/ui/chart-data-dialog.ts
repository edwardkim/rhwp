/**
 * [#4694] 차트 데이터 편집 대화상자 — 그리드(행=포인트, 열=계열)로 숫자 값을 고친다.
 *
 * 편집 판단은 전부 순수 함수(`@/core/chart-data-target`)와 코어 검증기에 있다:
 * 입력시 선제 검증(cellInputIssue) → [확인] 시 dryRun(코어가 단일 진실) →
 * executeOperation(snapshot/objectProps) 실쓰기. 무변경이면 어떤 쓰기·undo 기록도
 * 없이 닫는다. 쓰기 직전 operation 클로저 안에서 재열거·재대조해 index 드리프트를
 * 차단하고, 재대조 실패는 무기록 + 안내다(오매칭으로 다른 차트를 고치는 것이 최악).
 *
 * B1 경계: 행·열 추가/삭제 버튼이 없고 계열명 헤더가 비활성인 것 자체가 B2 와의
 * 경계 표시다. 라벨 열은 분산형 공유 X축에서만 편집 가능(코어 계약과 동형).
 */
import { ModalDialog } from './dialog';
import type { WasmBridge } from '@/core/wasm-bridge';
import type { EventBus } from '@/core/event-bus';
import type { CommandServices } from '@/command/types';
import {
  buildChartEdits,
  cellInputIssue,
  hasAnyEdit,
  labelsEditable,
  type ChartContainerRefJson,
  type ChartDataResult,
  type ChartInvalidEntry,
  type ChartRefJson,
} from '@/core/chart-data-target';

const B2_NOTE = '행·열 추가/삭제와 계열 이름·카테고리 라벨 변경은 다음 단계(B2)에서 지원합니다.';
const EMPTY_CELL_NOTE = '빈 값(결측)은 편집할 수 없습니다 — 점 구조 변경은 다음 단계(B2)입니다.';
const SERIES_LOCK_NOTE = '계열 이름 변경은 다음 단계(B2)에서 지원합니다.';

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
  private gridWrap!: HTMLDivElement;
  private errorEl!: HTMLDivElement;
  /** [seriesIdx][pointIdx] — 비편집 셀(원본 빈 값)은 null. */
  private valueInputs: (HTMLInputElement | null)[][] = [];
  private labelInputs: HTMLInputElement[] = [];

  constructor(wasm: WasmBridge, eventBus: EventBus, services?: CommandServices) {
    super('차트 데이터 편집', 560, false);
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
    note.textContent = B2_NOTE;
    body.appendChild(note);

    return body;
  }

  private renderGrid(): void {
    const data = this.data;
    if (!data || !data.series) return;
    this.hideError();
    this.gridWrap.textContent = '';
    this.valueInputs = data.series.map(() => []);
    this.labelInputs = [];

    const scatter = data.axis === 'scatter';
    const labels = data.labels ?? [];
    const editableLabels = labelsEditable(data);
    const rowCount = Math.max(labels.length, ...data.series.map((s) => s.values.length));

    const table = document.createElement('table');
    table.className = 'chart-data-grid';

    const head = table.createTHead().insertRow();
    const corner = document.createElement('th');
    corner.textContent = scatter ? 'X' : '카테고리';
    head.appendChild(corner);
    data.series.forEach((s, i) => {
      const th = document.createElement('th');
      th.className = 'chart-data-series-locked';
      th.textContent = s.name ?? `계열 ${i + 1}`;
      th.title = SERIES_LOCK_NOTE;
      head.appendChild(th);
    });

    const tbody = table.createTBody();
    for (let r = 0; r < rowCount; r++) {
      const row = tbody.insertRow();

      const labelCell = document.createElement('th');
      if (editableLabels) {
        const input = document.createElement('input');
        input.className = 'chart-data-input';
        input.inputMode = 'decimal';
        input.value = labels[r] ?? '';
        input.addEventListener('input', () => this.markCell(input));
        this.labelInputs.push(input);
        labelCell.appendChild(input);
      } else {
        labelCell.textContent = labels[r] ?? String(r + 1);
        labelCell.title = scatter
          ? '계열마다 X 가 달라 한 열로 편집할 수 없습니다.'
          : '카테고리 라벨 변경은 다음 단계(B2)에서 지원합니다.';
      }
      row.appendChild(labelCell);

      data.series.forEach((s, si) => {
        const cell = row.insertCell();
        const original = s.values[r];
        if (original === undefined) {
          this.valueInputs[si][r] = null;
          return;
        }
        const input = document.createElement('input');
        input.className = 'chart-data-input';
        input.inputMode = 'decimal';
        input.value = original;
        if (original === '') {
          input.disabled = true;
          input.title = EMPTY_CELL_NOTE;
          this.valueInputs[si][r] = null;
        } else {
          input.addEventListener('input', () => this.markCell(input));
          this.valueInputs[si][r] = input;
        }
        cell.appendChild(input);
      });
    }

    this.gridWrap.appendChild(table);
  }

  /** 입력시 선제 표시 — 최종 판정은 [확인]의 dryRun 이다. */
  private markCell(input: HTMLInputElement): void {
    input.classList.toggle('chart-data-invalid', cellInputIssue(input.value) !== null);
  }

  private collectValues(): string[][] {
    const data = this.data;
    if (!data || !data.series) return [];
    return data.series.map((s, si) =>
      s.values.map((original, r) => this.valueInputs[si][r]?.value ?? original),
    );
  }

  private collectLabels(): string[] | undefined {
    if (this.labelInputs.length === 0) return undefined;
    return this.labelInputs.map((input) => input.value);
  }

  private showError(message: string): void {
    this.errorEl.textContent = message;
    this.errorEl.style.display = '';
  }

  private showInvalid(invalid: ChartInvalidEntry[] | undefined): void {
    const lines = (invalid ?? []).map((e) => e.message ?? e.reason);
    this.showError(lines.length > 0 ? lines.join('\n') : '차트가 편집을 거부했습니다.');
  }

  private hideError(): void {
    this.errorEl.style.display = 'none';
  }

  protected onConfirm(): void | boolean {
    const chart = this.chart;
    const data = this.data;
    if (!chart || !data || !data.series) return;

    const values = this.collectValues();
    const labels = this.collectLabels();

    // 변경된 셀만 선제 검증 — 원본 그대로인 셀(빈 값 포함)은 코어가 무기록 처리한다.
    let broken = false;
    data.series.forEach((s, si) => {
      this.valueInputs[si].forEach((input, r) => {
        if (!input || input.value === s.values[r]) return;
        if (cellInputIssue(input.value) !== null) {
          this.markCell(input);
          broken = true;
        }
      });
    });
    this.labelInputs.forEach((input, r) => {
      if (input.value === (data.labels ?? [])[r]) return;
      if (cellInputIssue(input.value) !== null) {
        this.markCell(input);
        broken = true;
      }
    });
    if (broken) {
      this.showError('숫자가 아닌 값이 있습니다 — 표시된 칸을 고쳐 주세요.');
      return false;
    }

    // 무변경이면 쓰기도 undo 기록도 없이 닫는다.
    if (!hasAnyEdit(data, values, labels)) return;

    const edits = buildChartEdits(data, values, labels);

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

    let applied = false;
    let raceInvalid: ChartInvalidEntry[] | null = null;
    ih.executeOperation({
      kind: 'snapshot',
      operationType: 'objectProps',
      operation: () => {
        // 다이얼로그가 열린 사이 문서가 바뀌었을 수 있다 — 주소로 다시 찾는다.
        const found = this.wasm.listCharts().find((c) => sameChartAddress(c, chart));
        if (!found) return null;
        const res = this.wasm.setChartDataByIndex(found.index, edits);
        if (!res.ok) {
          raceInvalid = res.invalid ?? [];
          return null;
        }
        if ((res.changedCount ?? 0) === 0) return null;
        applied = true;
        return ih.getCursorPosition();
      },
    });
    if (!applied) {
      if (raceInvalid) this.showInvalid(raceInvalid);
      else this.showError('차트를 다시 찾지 못했습니다 — 문서가 바뀌었습니다. 다시 열어 주세요.');
      return false;
    }
  }
}
