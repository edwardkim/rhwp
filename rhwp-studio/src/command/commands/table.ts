import type { CommandDef, EditorContext } from '../types';
import { TableCellPropsDialog } from '@/ui/table-cell-props-dialog';
import { TableCreateDialog } from '@/ui/table-create-dialog';
import { CellSplitDialog } from '@/ui/cell-split-dialog';
import { CellBorderBgDialog } from '@/ui/cell-border-bg-dialog';
import { FormulaDialog } from '@/ui/formula-dialog';

const inTable = (ctx: EditorContext) => ctx.inTable;

function safeTableOp(fn: () => void, label: string): void {
  try { fn(); } catch (e) { console.error(`[table] ${label} 실패:`, e); }
}

function stub(id: string, label: string, icon?: string, shortcut?: string): CommandDef {
  return {
    id,
    label,
    icon,
    shortcutLabel: shortcut,
    canExecute: inTable,
    execute() { /* TODO: 후속 타스크에서 구현 */ },
  };
}

function blockCalcCommand(id: string, label: string, func: string, shortcut: string): CommandDef {
  return {
    id,
    label,
    shortcutLabel: shortcut,
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      try {
        const cellInfo = services.wasm.getCellInfo(pos.sectionIndex, pos.parentParaIndex, pos.controlIndex, pos.cellIndex);
        const row = cellInfo.row;
        const col = cellInfo.col;
        const formula = `=${func}(above)`;
        const result = services.wasm.evaluateTableFormula(
          pos.sectionIndex, pos.parentParaIndex, pos.controlIndex,
          row, col, formula, true,
        );
        const parsed = JSON.parse(result);
        if (parsed.ok) {
          services.eventBus.emit('document-changed');
        }
      } catch (err) {
        console.warn(`[${id}] 블록 계산 실패:`, err);
      }
    },
  };
}

function openFormulaDialog(services: Parameters<CommandDef['execute']>[0]): void {
  const ih = services.getInputHandler();
  if (!ih) return;
  const pos = ih.getCursorPosition();
  if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
  const dialog = new FormulaDialog(services.wasm, services.eventBus, {
    sec: pos.sectionIndex,
    ppi: pos.parentParaIndex,
    ci: pos.controlIndex,
    cellIndex: pos.cellIndex,
  });
  dialog.show();
}

export const tableCommands: CommandDef[] = [
  { id: 'table:create', label: '표 만들기', icon: 'icon-table',
    canExecute: (ctx) => ctx.hasDocument && !ctx.inTable,
    execute(services, params) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex !== undefined) return;
      const dialog = new TableCreateDialog();
      dialog.onApply = (rows, cols) => {
        const ih2 = services.getInputHandler();
        if (!ih2) return;
        safeTableOp(() => ih2.executeOperation({
          kind: 'snapshot',
          operationType: 'createTable',
          operation: (wasm) => {
            const result = wasm.createTable(pos.sectionIndex, pos.paragraphIndex, pos.charOffset, rows, cols);
            if (result.ok) {
              return {
                sectionIndex: pos.sectionIndex,
                paragraphIndex: 0,
                charOffset: 0,
                parentParaIndex: result.paraIdx,
                controlIndex: result.controlIdx,
                cellIndex: 0,
                cellParaIndex: 0,
              };
            }
            return pos;
          },
        }), '표 만들기');
        // 대화상자 닫힘 후 편집 포커스 복원 — textarea 에 keydown 이 바인딩되어
        // 있어, 복원하지 않으면 직후 F5 등이 브라우저 기본동작으로 빠진다 (#1140)
        (ih2 as any).textarea?.focus();
      };
      dialog.show(params?.anchorEl as HTMLElement | undefined);
    },
  },
  {
    id: 'table:cell-props',
    label: '표/셀 속성',
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const tableCtx = { sec: pos.sectionIndex, ppi: pos.parentParaIndex, ci: pos.controlIndex };
      const ih2 = services.getInputHandler();
      const mode = ih2?.isInTableObjectSelection() ? 'table' as const : 'cell' as const;
      const dialog = new TableCellPropsDialog(services.wasm, services.eventBus, tableCtx, pos.cellIndex, mode);
      dialog.show();
    },
  },
  {
    id: 'table:border-each',
    label: '각 셀마다 적용(E)...',
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const tableCtx = { sec: pos.sectionIndex, ppi: pos.parentParaIndex, ci: pos.controlIndex };
      const dialog = new CellBorderBgDialog(services.wasm, services.eventBus, tableCtx, pos.cellIndex, 'each');
      dialog.show();
    },
  },
  {
    id: 'table:border-one',
    label: '하나의 셀처럼 적용(Z)...',
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const tableCtx = { sec: pos.sectionIndex, ppi: pos.parentParaIndex, ci: pos.controlIndex };
      const dialog = new CellBorderBgDialog(services.wasm, services.eventBus, tableCtx, pos.cellIndex, 'asOne');
      dialog.show();
    },
  },
  {
    id: 'table:insert-row-above',
    label: '위쪽에 줄 추가하기',
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const cellInfo = services.wasm.getCellInfo(pos.sectionIndex, pos.parentParaIndex, pos.controlIndex, pos.cellIndex);
      safeTableOp(() => ih.executeOperation({
        kind: 'snapshot',
        operationType: 'insertTableRow',
        operation: (wasm) => {
          wasm.insertTableRow(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, cellInfo.row, false);
          return pos;
        },
      }), '줄 추가');
    },
  },
  {
    id: 'table:insert-row-below',
    label: '아래쪽에 줄 추가하기',
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const cellInfo = services.wasm.getCellInfo(pos.sectionIndex, pos.parentParaIndex, pos.controlIndex, pos.cellIndex);
      safeTableOp(() => ih.executeOperation({
        kind: 'snapshot',
        operationType: 'insertTableRow',
        operation: (wasm) => {
          wasm.insertTableRow(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, cellInfo.row, true);
          return pos;
        },
      }), '줄 추가');
    },
  },
  {
    id: 'table:insert-col-left',
    label: '왼쪽에 칸 추가하기',
    shortcutLabel: 'Alt+Insert',
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const cellInfo = services.wasm.getCellInfo(pos.sectionIndex, pos.parentParaIndex, pos.controlIndex, pos.cellIndex);
      safeTableOp(() => ih.executeOperation({
        kind: 'snapshot',
        operationType: 'insertTableColumn',
        operation: (wasm) => {
          wasm.insertTableColumn(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, cellInfo.col, false);
          return pos;
        },
      }), '칸 추가');
    },
  },
  {
    id: 'table:insert-col-right',
    label: '오른쪽에 칸 추가하기',
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const cellInfo = services.wasm.getCellInfo(pos.sectionIndex, pos.parentParaIndex, pos.controlIndex, pos.cellIndex);
      safeTableOp(() => ih.executeOperation({
        kind: 'snapshot',
        operationType: 'insertTableColumn',
        operation: (wasm) => {
          wasm.insertTableColumn(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, cellInfo.col, true);
          return pos;
        },
      }), '칸 추가');
    },
  },
  {
    id: 'table:delete-row',
    label: '줄 지우기',
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const cellInfo = services.wasm.getCellInfo(pos.sectionIndex, pos.parentParaIndex, pos.controlIndex, pos.cellIndex);
      safeTableOp(() => ih.executeOperation({
        kind: 'snapshot',
        operationType: 'deleteTableRow',
        operation: (wasm) => {
          wasm.deleteTableRow(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, cellInfo.row);
          return pos;
        },
      }), '줄 지우기');
    },
  },
  {
    id: 'table:delete-col',
    label: '칸 지우기',
    shortcutLabel: 'Alt+Delete',
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const cellInfo = services.wasm.getCellInfo(pos.sectionIndex, pos.parentParaIndex, pos.controlIndex, pos.cellIndex);
      safeTableOp(() => ih.executeOperation({
        kind: 'snapshot',
        operationType: 'deleteTableColumn',
        operation: (wasm) => {
          wasm.deleteTableColumn(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, cellInfo.col);
          return pos;
        },
      }), '칸 지우기');
    },
  },
  {
    id: 'table:cell-split',
    label: '셀 나누기',
    shortcutLabel: 'S',
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;

      // F5 셀 선택 모드: 범위 선택 여부 확인
      const range = ih.getSelectedCellRange?.();
      const tableCtx = ih.getCellTableContext?.();
      const isMultiCell = range && tableCtx &&
        (range.startRow !== range.endRow || range.startCol !== range.endCol);

      const cellInfo = services.wasm.getCellInfo(pos.sectionIndex, pos.parentParaIndex, pos.controlIndex, pos.cellIndex);
      const isMerged = !isMultiCell && (cellInfo.rowSpan > 1 || cellInfo.colSpan > 1);

      const dialog = new CellSplitDialog(isMerged);
      dialog.onApply = (nRows, mCols, equalHeight, mergeFirst) => {
        const ih2 = services.getInputHandler();
        if (!ih2) return;
        safeTableOp(() => ih2.executeOperation({
          kind: 'snapshot',
          operationType: 'splitTableCell',
          operation: (wasm) => {
            if (isMultiCell && range && tableCtx) {
              wasm.splitTableCellsInRange(
                tableCtx.sec, tableCtx.ppi, tableCtx.ci,
                range.startRow, range.startCol, range.endRow, range.endCol,
                nRows, mCols, equalHeight,
              );
            } else {
              wasm.splitTableCellInto(
                pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!,
                cellInfo.row, cellInfo.col,
                nRows, mCols, equalHeight, mergeFirst,
              );
            }
            return pos;
          },
        }), '셀 나누기');
        if (isMultiCell) ih2.exitCellSelectionMode?.();
        // 대화상자 닫힘 후 편집 포커스 복원 (#1140 — 표 만들기와 동일 결함)
        (ih2 as any).textarea?.focus();
      };
      dialog.show();
    },
  },
  {
    id: 'table:cell-merge',
    label: '셀 합치기',
    shortcutLabel: 'M',
    canExecute: (ctx) => ctx.inCellSelectionMode,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const range = ih.getSelectedCellRange();
      const tableCtx = ih.getCellTableContext();
      if (!range || !tableCtx) return;
      if (range.startRow === range.endRow && range.startCol === range.endCol) return;
      safeTableOp(() => ih.executeOperation({
        kind: 'snapshot',
        operationType: 'mergeTableCells',
        operation: (wasm) => {
          wasm.mergeTableCells(tableCtx.sec, tableCtx.ppi, tableCtx.ci, range.startRow, range.startCol, range.endRow, range.endCol);
          return ih.getCursorPosition();
        },
      }), '셀 합치기');
      ih.exitCellSelectionMode();
    },
  },
  {
    id: 'table:delete',
    label: '표 지우기',
    canExecute: (ctx) => ctx.inTable || ctx.inTableObjectSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const ref = ih.getSelectedTableRef();
      if (ref) {
        safeTableOp(() => ih.executeOperation({
          kind: 'snapshot',
          operationType: 'deleteTable',
          operation: (wasm) => {
            wasm.deleteTableControl(ref.sec, ref.ppi, ref.ci);
            return { sectionIndex: ref.sec, paragraphIndex: ref.ppi, charOffset: 0 };
          },
        }), '표 지우기');
        return;
      }
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined) return;
      safeTableOp(() => ih.executeOperation({
        kind: 'snapshot',
        operationType: 'deleteTable',
        operation: (wasm) => {
          wasm.deleteTableControl(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!);
          return { sectionIndex: pos.sectionIndex, paragraphIndex: pos.parentParaIndex!, charOffset: 0 };
        },
      }), '표 지우기');
    },
  },
  {
    id: 'table:caption-toggle',
    label: '캡션 넣기',
    canExecute: (ctx) => ctx.inTable || ctx.inTableObjectSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      // 표 참조 획득 (표 객체 선택 또는 셀 내부)
      let sec: number, ppi: number, ci: number;
      const ref = ih.getSelectedTableRef();
      if (ref) {
        sec = ref.sec; ppi = ref.ppi; ci = ref.ci;
      } else {
        const pos = ih.getCursorPosition();
        if (pos.parentParaIndex === undefined || pos.controlIndex === undefined) return;
        sec = pos.sectionIndex; ppi = pos.parentParaIndex; ci = pos.controlIndex;
      }
      // 현재 캡션 상태 조회
      let props: any;
      try { props = services.wasm.getTableProperties(sec, ppi, ci); } catch { return; }
      if (!props) return;
      let charOffset = 0;
      if (!props.hasCaption) {
        safeTableOp(() => ih.executeOperation({
          kind: 'snapshot',
          operationType: 'toggleTableCaption',
          operation: (wasm) => {
            const result: any = wasm.setTableProperties(sec, ppi, ci, { hasCaption: true });
            charOffset = result?.captionCharOffset ?? 3;
            return { sectionIndex: sec, paragraphIndex: ppi, charOffset: 0 };
          },
        }), '캡션 넣기');
      } else {
        try {
          const len = services.wasm.getCellParagraphLength(sec, ppi, ci, 65534, 0);
          charOffset = len;
        } catch { charOffset = 0; }
      }
      // 표 내부 편집 모드 종료 후 캡션 편집 진입
      if (ref) {
        ih.exitTableObjectSelection();
      }
      ih.enterTableCaptionEditing(sec, ppi, ci, charOffset);
    },
  },
  {
    id: 'table:cell-height-equal',
    label: '셀 높이를 같게',
    shortcutLabel: 'H',
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const sec = pos.sectionIndex, ppi = pos.parentParaIndex, ci = pos.controlIndex;
      try {
        const dims = services.wasm.getTableDimensions(sec, ppi, ci);
        const cells: Array<{ idx: number; row: number; height: number }> = [];
        const rowHeights = new Map<number, { sum: number; count: number }>();
        for (let i = 0; i < dims.cellCount; i++) {
          const info = services.wasm.getCellInfo(sec, ppi, ci, i);
          if (info.rowSpan > 1) continue;
          const h = services.wasm.getCellProperties(sec, ppi, ci, i).height;
          cells.push({ idx: i, row: info.row, height: h });
          const entry = rowHeights.get(info.row);
          if (entry) { entry.sum += h; entry.count++; }
          else rowHeights.set(info.row, { sum: h, count: 1 });
        }
        if (rowHeights.size < 2) return;
        let totalHeight = 0;
        for (const v of rowHeights.values()) totalHeight += v.sum / v.count;
        const avgHeight = Math.round(totalHeight / rowHeights.size);
        const updates: Array<{ cellIdx: number; heightDelta: number }> = [];
        for (const c of cells) {
          const delta = avgHeight - c.height;
          if (delta !== 0) updates.push({ cellIdx: c.idx, heightDelta: delta });
        }
        if (updates.length === 0) return;
        services.wasm.resizeTableCells(sec, ppi, ci, updates);
        services.eventBus.emit('document-changed');
      } catch (err) {
        console.warn('[table:cell-height-equal] 높이 균등화 실패:', err);
      }
    },
  },
  {
    id: 'table:cell-width-equal',
    label: '셀 너비를 같게',
    shortcutLabel: 'W',
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const sec = pos.sectionIndex, ppi = pos.parentParaIndex, ci = pos.controlIndex;
      try {
        const dims = services.wasm.getTableDimensions(sec, ppi, ci);
        const cells: Array<{ idx: number; col: number; width: number }> = [];
        const colWidths = new Map<number, { sum: number; count: number }>();
        for (let i = 0; i < dims.cellCount; i++) {
          const info = services.wasm.getCellInfo(sec, ppi, ci, i);
          if (info.colSpan > 1) continue;
          const w = services.wasm.getCellProperties(sec, ppi, ci, i).width;
          cells.push({ idx: i, col: info.col, width: w });
          const entry = colWidths.get(info.col);
          if (entry) { entry.sum += w; entry.count++; }
          else colWidths.set(info.col, { sum: w, count: 1 });
        }
        if (colWidths.size < 2) return;
        let totalWidth = 0;
        for (const v of colWidths.values()) totalWidth += v.sum / v.count;
        const avgWidth = Math.round(totalWidth / colWidths.size);
        const updates: Array<{ cellIdx: number; widthDelta: number }> = [];
        for (const c of cells) {
          const delta = avgWidth - c.width;
          if (delta !== 0) updates.push({ cellIdx: c.idx, widthDelta: delta });
        }
        if (updates.length === 0) return;
        services.wasm.resizeTableCells(sec, ppi, ci, updates);
        services.eventBus.emit('document-changed');
      } catch (err) {
        console.warn('[table:cell-width-equal] 너비 균등화 실패:', err);
      }
    },
  },
  {
    id: 'table:formula',
    label: '계산식(F)...',
    shortcutLabel: 'Ctrl+M,F',
    canExecute: inTable,
    execute(services) { openFormulaDialog(services); },
  },
  {
    id: 'table:block-formula',
    label: '블록 계산식',
    canExecute: inTable,
    execute(services) { openFormulaDialog(services); },
  },
  blockCalcCommand('table:block-sum', '블록 합계', 'SUM', 'Ctrl+Shift+S'),
  blockCalcCommand('table:block-avg', '블록 평균', 'AVERAGE', 'Ctrl+Shift+A'),
  blockCalcCommand('table:block-product', '블록 곱', 'PRODUCT', 'Ctrl+Shift+P'),
  {
    id: 'table:thousand-sep',
    label: '1,000 단위 구분 쉼표',
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const sec = pos.sectionIndex, ppi = pos.parentParaIndex, ci = pos.controlIndex, cei = pos.cellIndex;
      const cpi = pos.cellParaIndex ?? 0;
      try {
        const len = services.wasm.getCellParagraphLength(sec, ppi, ci, cei, cpi);
        if (len <= 0) return;
        const text = services.wasm.getTextInCell(sec, ppi, ci, cei, cpi, 0, len);
        const trimmed = text.trim();
        if (!trimmed) return;
        const stripped = trimmed.replace(/,/g, '');
        const numMatch = stripped.match(/^([+-]?)(\d+)(\.?\d*)$/);
        if (!numMatch) return;
        const [, sign, intPart, decPart] = numMatch;
        let result: string;
        if (trimmed.includes(',')) {
          result = sign + intPart + decPart;
        } else {
          const formatted = intPart.replace(/\B(?=(\d{3})+(?!\d))/g, ',');
          result = sign + formatted + decPart;
        }
        if (result === text) return;
        services.wasm.deleteTextInCell(sec, ppi, ci, cei, cpi, 0, len);
        services.wasm.insertTextInCell(sec, ppi, ci, cei, cpi, 0, result);
        services.eventBus.emit('document-changed');
      } catch (err) {
        console.warn('[table:thousand-sep] 구분 쉼표 변환 실패:', err);
      }
    },
  },
  {
    id: 'table:decimal-add',
    label: '자릿점 넣기',
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const sec = pos.sectionIndex, ppi = pos.parentParaIndex, ci = pos.controlIndex, cei = pos.cellIndex;
      const cpi = pos.cellParaIndex ?? 0;
      try {
        const len = services.wasm.getCellParagraphLength(sec, ppi, ci, cei, cpi);
        if (len <= 0) return;
        const text = services.wasm.getTextInCell(sec, ppi, ci, cei, cpi, 0, len);
        const trimmed = text.trim();
        const raw = trimmed.replace(/,/g, '');
        const match = raw.match(/^([+-]?)(\d+)(\.(\d*))?$/);
        if (!match) return;
        const [, sign, intPart, , decimals] = match;
        const newDecimals = (decimals ?? '') + '0';
        const hasCommas = trimmed.includes(',');
        const fmtInt = hasCommas ? intPart.replace(/\B(?=(\d{3})+(?!\d))/g, ',') : intPart;
        const result = sign + fmtInt + '.' + newDecimals;
        if (result === text) return;
        services.wasm.deleteTextInCell(sec, ppi, ci, cei, cpi, 0, len);
        services.wasm.insertTextInCell(sec, ppi, ci, cei, cpi, 0, result);
        services.eventBus.emit('document-changed');
      } catch (err) {
        console.warn('[table:decimal-add] 자릿점 넣기 실패:', err);
      }
    },
  },
  {
    id: 'table:decimal-remove',
    label: '자릿점 빼기',
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const sec = pos.sectionIndex, ppi = pos.parentParaIndex, ci = pos.controlIndex, cei = pos.cellIndex;
      const cpi = pos.cellParaIndex ?? 0;
      try {
        const len = services.wasm.getCellParagraphLength(sec, ppi, ci, cei, cpi);
        if (len <= 0) return;
        const text = services.wasm.getTextInCell(sec, ppi, ci, cei, cpi, 0, len);
        const trimmed = text.trim();
        const raw = trimmed.replace(/,/g, '');
        const match = raw.match(/^([+-]?)(\d+)\.(\d+)$/);
        if (!match) return;
        const [, sign, intPart, decimals] = match;
        const hasCommas = trimmed.includes(',');
        const fmtInt = hasCommas ? intPart.replace(/\B(?=(\d{3})+(?!\d))/g, ',') : intPart;
        const newDecimals = decimals.slice(0, -1);
        const result = newDecimals ? sign + fmtInt + '.' + newDecimals : sign + fmtInt;
        if (result === text) return;
        services.wasm.deleteTextInCell(sec, ppi, ci, cei, cpi, 0, len);
        services.wasm.insertTextInCell(sec, ppi, ci, cei, cpi, 0, result);
        services.eventBus.emit('document-changed');
      } catch (err) {
        console.warn('[table:decimal-remove] 자릿점 빼기 실패:', err);
      }
    },
  },
];
