import type { CommandDef, CommandServices, EditorContext } from '../types';
import { TableCellPropsDialog } from '@/ui/table-cell-props-dialog';
import { TableCreateDialog } from '@/ui/table-create-dialog';
import type { TableCreateOptions } from '@/ui/table-create-dialog';
import { CellSplitDialog } from '@/ui/cell-split-dialog';
import { CellBorderBgDialog } from '@/ui/cell-border-bg-dialog';
import { FormulaDialog } from '@/ui/formula-dialog';
import {
  TableDeleteRowColumnDialog,
  TableInsertRowColumnDialog,
  type TableDeleteRowColumnMode,
  type TableInsertRowColumnMode,
} from '@/ui/table-row-column-dialog';

const inTable = (ctx: EditorContext) => ctx.inTable;
const inTableOrCellSelection = (ctx: EditorContext) => ctx.inTable || ctx.inCellSelectionMode;
const hasMultiCellSelection = (ctx: EditorContext) => ctx.hasMultiCellSelection;

type CellRange = { startRow: number; startCol: number; endRow: number; endCol: number };
type TableDimensions = { rowCount: number; colCount: number; cellCount: number };
type TableCellCommandContext = {
  ih: NonNullable<ReturnType<CommandServices['getInputHandler']>>;
  pos: ReturnType<NonNullable<ReturnType<CommandServices['getInputHandler']>>['getCursorPosition']>;
  cellInfo: ReturnType<CommandServices['wasm']['getCellInfo']>;
};

function safeTableOp(fn: () => void, label: string): void {
  try { fn(); } catch (e) { console.error(`[table] ${label} 실패:`, e); }
}

function equalizeTargetRange(ih: ReturnType<CommandServices['getInputHandler']>, dims: TableDimensions): CellRange {
  const range = ih?.isInCellSelectionMode?.() ? ih.getSelectedCellRange?.() : null;
  return range ?? {
    startRow: 0,
    startCol: 0,
    endRow: Math.max(0, dims.rowCount - 1),
    endCol: Math.max(0, dims.colCount - 1),
  };
}

function hasNonRectangularCellSelection(ih: ReturnType<CommandServices['getInputHandler']>): boolean {
  return Boolean(ih?.isInCellSelectionMode?.() && ih.hasExcludedCellSelection?.());
}

function isTransposeTargetOverflowError(err: unknown): boolean {
  const message = err instanceof Error ? err.message : String(err);
  return message.includes('표 크기') && message.includes('초과');
}

function isCellInRange(cell: { row: number; col: number }, range: CellRange): boolean {
  return cell.row >= range.startRow && cell.row <= range.endRow &&
    cell.col >= range.startCol && cell.col <= range.endCol;
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
        // [블록계산 이관] write=true 는 결과를 셀에 써서 문자 수를 바꾼다 — 미기록 시 후속
        // undo 오프셋 오염(#2344 셀 숫자 서식과 동일 계열). dry-run(write=false)으로 ok 를
        // 확인한 뒤 commit 을 snapshot 으로 라우팅한다(라우터가 refresh → 수동 emit 제거).
        const check = JSON.parse(services.wasm.evaluateTableFormula(
          pos.sectionIndex, pos.parentParaIndex, pos.controlIndex, row, col, formula, false,
        ));
        if (!check.ok) return;
        safeTableOp(() => ih.executeOperation({
          kind: 'snapshot',
          operationType: 'tableBlockCalc',
          operation: (wasm) => {
            wasm.evaluateTableFormula(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, row, col, formula, true);
            return pos;
          },
        }), '블록 계산');
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
  }, services);
  dialog.show();
}

function currentTableCellContext(services: CommandServices): TableCellCommandContext | null {
  const ih = services.getInputHandler();
  if (!ih) return null;
  const pos = ih.getCursorPosition();
  if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return null;
  const cellInfo = services.wasm.getCellInfo(
    pos.sectionIndex,
    pos.parentParaIndex,
    pos.controlIndex,
    pos.cellIndex,
  );
  return { ih, pos, cellInfo };
}

function restoreEditorFocus(ih: TableCellCommandContext['ih']): void {
  const textarea = (ih as unknown as { textarea?: HTMLTextAreaElement }).textarea;
  textarea?.focus();
}

function applyTableInsertRowColumn(
  services: CommandServices,
  mode: TableInsertRowColumnMode,
  count: number,
): void {
  const ctx = currentTableCellContext(services);
  if (!ctx) return;
  const { ih, pos, cellInfo } = ctx;
  safeTableOp(() => ih.executeOperation({
    kind: 'snapshot',
    operationType: mode.startsWith('row') ? 'insertTableRow' : 'insertTableColumn',
    operation: (wasm) => {
      for (let i = 0; i < count; i += 1) {
        switch (mode) {
          case 'row-above':
            wasm.insertTableRow(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, cellInfo.row, false);
            break;
          case 'row-below':
            wasm.insertTableRow(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, cellInfo.row, true);
            break;
          case 'col-left':
            wasm.insertTableColumn(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, cellInfo.col, false);
            break;
          case 'col-right':
            wasm.insertTableColumn(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, cellInfo.col, true);
            break;
        }
      }
      return pos;
    },
  }), '줄/칸 추가');
  restoreEditorFocus(ih);
}

/**
 * 줄/칸 지우기 후 커서 셀 보정 (#1483).
 *
 * 삭제로 셀 수가 줄면 기존 cellIndex가 새 표 범위를 벗어나 updateRect가 "셀 인덱스 초과"로
 * 실패한다. 삭제 후 표 크기(rowCount/colCount) 내로 (row,col)을 clamp하고, 해당 위치의
 * cellIndex를 getTableCellBboxes로 역조회한다 (병합 셀은 rowSpan/colSpan 범위로 매칭).
 * 표가 소멸(rowCount/colCount<=0)하면 null을 반환한다.
 */
function clampedCellAfterDelete(
  wasm: CommandServices['wasm'],
  sec: number,
  parentPara: number,
  controlIdx: number,
  origRow: number,
  origCol: number,
  rowCount: number,
  colCount: number,
): { cellIndex: number; cellParaIndex: number } | null {
  if (rowCount <= 0 || colCount <= 0) return null;
  const row = Math.min(origRow, rowCount - 1);
  const col = Math.min(origCol, colCount - 1);
  const bboxes = wasm.getTableCellBboxes(sec, parentPara, controlIdx);
  const hit = bboxes.find(
    (b) =>
      row >= b.row && row < b.row + b.rowSpan &&
      col >= b.col && col < b.col + b.colSpan,
  );
  return { cellIndex: hit ? hit.cellIdx : 0, cellParaIndex: 0 };
}

function applyTableDeleteRowColumn(
  services: CommandServices,
  mode: TableDeleteRowColumnMode,
): void {
  const ctx = currentTableCellContext(services);
  if (!ctx) return;
  const { ih, pos, cellInfo } = ctx;
  safeTableOp(() => ih.executeOperation({
    kind: 'snapshot',
    operationType: mode === 'row' ? 'deleteTableRow' : 'deleteTableColumn',
    operation: (wasm) => {
      const res = mode === 'row'
        ? wasm.deleteTableRow(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, cellInfo.row)
        : wasm.deleteTableColumn(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, cellInfo.col);
      if (!res.ok) return pos;
      // 삭제 후 셀 수가 줄면 기존 cellIndex가 범위를 벗어날 수 있어 보정한다 (#1483).
      const corrected = clampedCellAfterDelete(
        wasm, pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!,
        cellInfo.row, cellInfo.col, res.rowCount, res.colCount,
      );
      if (!corrected) {
        // 표 소멸 → 표 밖 본문 위치로 폴백.
        return { sectionIndex: pos.sectionIndex, paragraphIndex: pos.parentParaIndex ?? 0, charOffset: 0 };
      }
      return { ...pos, charOffset: 0, ...corrected };
    },
  }), '줄/칸 지우기');
  restoreEditorFocus(ih);
}

export const tableCommands: CommandDef[] = [
  { id: 'table:create', label: '표 만들기', icon: 'icon-table',
    opensDialog: true,
    canExecute: (ctx) => ctx.hasDocument && !ctx.inTable,
    execute(services, params) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex !== undefined) return;
      const dialog = new TableCreateDialog();
      dialog.onApply = (rows, cols, options?: TableCreateOptions) => {
        const ih2 = services.getInputHandler();
        if (!ih2) return;
        safeTableOp(() => ih2.executeOperation({
          kind: 'snapshot',
          operationType: 'createTable',
          operation: (wasm) => {
            const result = options
              ? wasm.createTableEx({
                  sectionIdx: pos.sectionIndex,
                  paraIdx: pos.paragraphIndex,
                  charOffset: pos.charOffset,
                  rowCount: rows,
                  colCount: cols,
                  ...options,
                })
              : wasm.createTable(pos.sectionIndex, pos.paragraphIndex, pos.charOffset, rows, cols);
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
    opensDialog: true,
    label: '표/셀 속성',
    canExecute: (ctx) => ctx.inTable || ctx.inCellSelectionMode || ctx.inTableObjectSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      if (ih.isInTableObjectSelection()) {
        const ref = ih.getSelectedTableRef();
        if (!ref) return;
        const tableCtx = { sec: ref.sec, ppi: ref.ppi, ci: ref.ci };
        const dialog = new TableCellPropsDialog(services.wasm, services.eventBus, tableCtx, 0, 'table', services);
        dialog.show();
        return;
      }

      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const tableCtx = { sec: pos.sectionIndex, ppi: pos.parentParaIndex, ci: pos.controlIndex };
      const dialog = new TableCellPropsDialog(services.wasm, services.eventBus, tableCtx, pos.cellIndex, 'cell', services);
      dialog.show();
    },
  },
  {
    id: 'table:border-each',
    opensDialog: true,
    label: '각 셀마다 적용(E)...',
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const tableCtx = { sec: pos.sectionIndex, ppi: pos.parentParaIndex, ci: pos.controlIndex };
      const selectionRange = ih.isInCellSelectionMode?.() ? ih.getSelectedCellRange?.() ?? null : null;
      const dialog = new CellBorderBgDialog(
        services.wasm,
        services.eventBus,
        tableCtx,
        pos.cellIndex,
        'each',
        selectionRange,
        services,
      );
      dialog.show();
    },
  },
  {
    id: 'table:border-one',
    opensDialog: true,
    label: '하나의 셀처럼 적용(Z)...',
    canExecute: hasMultiCellSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      if (!ih.hasMultiCellSelection()) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const tableCtx = { sec: pos.sectionIndex, ppi: pos.parentParaIndex, ci: pos.controlIndex };
      const dialog = new CellBorderBgDialog(
        services.wasm,
        services.eventBus,
        tableCtx,
        pos.cellIndex,
        'asOne',
        ih.getSelectedCellRange(),
        services,
      );
      dialog.show();
    },
  },
  {
    id: 'table:insert-row-col',
    opensDialog: true,
    label: '줄/칸 추가하기(I)...',
    shortcutLabel: 'Alt+Enter',
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const dialog = new TableInsertRowColumnDialog();
      dialog.onApply = ({ mode, count }) => applyTableInsertRowColumn(services, mode, count);
      dialog.afterClose = () => restoreEditorFocus(ih);
      dialog.show();
    },
  },
  {
    id: 'table:delete-row-col',
    opensDialog: true,
    label: '줄/칸 지우기(E)...',
    shortcutLabel: 'Alt+Delete',
    canExecute: inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const dialog = new TableDeleteRowColumnDialog();
      dialog.onApply = ({ mode }) => applyTableDeleteRowColumn(services, mode);
      dialog.afterClose = () => restoreEditorFocus(ih);
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
    opensDialog: true,
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
    id: 'table:transpose-copy',
    label: '행/열 바꿈 복사',
    canExecute: (ctx) => ctx.inCellSelectionMode,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const range = ih.getSelectedCellRange?.();
      const tableCtx = ih.getCellTableContext?.();
      if (!range || !tableCtx) return;
      if (hasNonRectangularCellSelection(ih)) return;
      if (tableCtx.cellPath && tableCtx.cellPath.length > 1) return;

      safeTableOp(() => {
        services.wasm.copyTableCellsTransposed(
          tableCtx.sec,
          tableCtx.ppi,
          tableCtx.ci,
          range.startRow,
          range.startCol,
          range.endRow,
          range.endCol,
        );
      }, '행/열 바꿈 복사');
      restoreEditorFocus(ih);
    },
  },
  {
    id: 'table:transpose-paste',
    label: '행/열 바꿈 붙여넣기',
    canExecute: (ctx) => ctx.hasDocument && ctx.hasTableTransposeClipboard,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if ((pos.cellPath?.length ?? 0) > 1) return;

      safeTableOp(() => ih.executeOperation({
        kind: 'snapshot',
        operationType: 'pasteTableCellsTransposed',
        operation: (wasm) => {
          const pasteAsNewTable = (sectionIndex: number, paragraphIndex: number, charOffset: number) => {
            const result = wasm.pasteTableCellsTransposedAsTable(
              sectionIndex,
              paragraphIndex,
              charOffset,
            );
            if (result.ok && result.paraIdx !== undefined && result.controlIdx !== undefined) {
              return {
                sectionIndex,
                paragraphIndex: 0,
                charOffset: 0,
                parentParaIndex: result.paraIdx,
                controlIndex: result.controlIdx,
                cellIndex: 0,
                cellParaIndex: 0,
              };
            }
            return pos;
          };

          const selectionTableCtx = ih.isInCellSelectionMode?.() ? ih.getCellTableContext?.() : null;
          if (selectionTableCtx) {
            if ((selectionTableCtx.cellPath?.length ?? 0) > 1) return pos;
            const range = ih.getSelectedCellRange?.();
            const dims = wasm.getTableDimensions(
              selectionTableCtx.sec,
              selectionTableCtx.ppi,
              selectionTableCtx.ci,
            );
            const isWholeTable = range
              && range.startRow === 0
              && range.startCol === 0
              && range.endRow === dims.rowCount - 1
              && range.endCol === dims.colCount - 1;
            if (isWholeTable) {
              wasm.transposeTableCellsInPlace(
                selectionTableCtx.sec,
                selectionTableCtx.ppi,
                selectionTableCtx.ci,
              );
              return {
                sectionIndex: selectionTableCtx.sec,
                paragraphIndex: 0,
                charOffset: 0,
                parentParaIndex: selectionTableCtx.ppi,
                controlIndex: selectionTableCtx.ci,
                cellIndex: 0,
                cellParaIndex: 0,
              };
            }
            if (range) {
              try {
                wasm.pasteTableCellsTransposed(
                  selectionTableCtx.sec,
                  selectionTableCtx.ppi,
                  selectionTableCtx.ci,
                  range.startRow,
                  range.startCol,
                );
                return {
                  sectionIndex: selectionTableCtx.sec,
                  paragraphIndex: 0,
                  charOffset: 0,
                  parentParaIndex: selectionTableCtx.ppi,
                  controlIndex: selectionTableCtx.ci,
                  cellIndex: 0,
                  cellParaIndex: 0,
                };
              } catch (err) {
                if (!isTransposeTargetOverflowError(err)) throw err;
              }
            }
            return pasteAsNewTable(selectionTableCtx.sec, selectionTableCtx.ppi, 0);
          }

          if (pos.parentParaIndex !== undefined && pos.controlIndex !== undefined && pos.cellIndex !== undefined) {
            const cellInfo = services.wasm.getCellInfo(
              pos.sectionIndex,
              pos.parentParaIndex,
              pos.controlIndex,
              pos.cellIndex,
            );
            try {
              wasm.pasteTableCellsTransposed(
                pos.sectionIndex,
                pos.parentParaIndex,
                pos.controlIndex,
                cellInfo.row,
                cellInfo.col,
              );
            } catch (err) {
              if (!isTransposeTargetOverflowError(err)) throw err;
              return pasteAsNewTable(pos.sectionIndex, pos.parentParaIndex, 0);
            }
            return pos;
          }

          return pasteAsNewTable(pos.sectionIndex, pos.paragraphIndex, pos.charOffset);
        },
      }), '행/열 바꿈 붙여넣기');
      restoreEditorFocus(ih);
    },
  },
  {
    id: 'table:split',
    label: '표 나누기',
    shortcutLabel: 'Ctrl+M,A',
    canExecute: (ctx) => ctx.inTable,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos?.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      // 중첩 표(cellPath 깊이 2+)는 flat 인덱스가 바깥 표를 가리켜 오동작한다 —
      // path 기반 API 가 생기기 전까지는 최상위 표에서만 허용.
      if ((pos.cellPath?.length ?? 0) > 1) return;
      const { sectionIndex: sec, parentParaIndex: ppi, controlIndex: ci, cellIndex } = pos;
      safeTableOp(() => {
        // Rust 쪽에서도 첫 행을 거부하지만, snapshot 을 뜬 뒤 실패하면 빈 undo
        // 항목이 남으므로 확정 실패는 executeOperation 전에 걸러낸다.
        const info = services.wasm.getCellInfo(sec, ppi, ci, cellIndex);
        if (info.row === 0) {
          console.warn('[table:split] 첫 번째 줄에서는 표 나누기를 할 수 없습니다.');
          return;
        }
        ih.executeOperation({
          kind: 'snapshot',
          operationType: 'splitTable',
          operation: (wasm) => {
            // 커서 셀(row >= 분할행)은 예외 없이 뒤 표로 옮겨지므로, 작업 전
            // 위치를 그대로 반환하면 앞 표의 범위 밖 cellIndex 가 된다
            // (redo 시에도 같은 무효 위치로 복원). 뒤 표 기준으로 재계산한다.
            const res = wasm.splitTable(sec, ppi, ci, info.row);
            const frontCells = wasm.getTableDimensions(sec, ppi, ci).cellCount;
            return {
              sectionIndex: sec,
              paragraphIndex: 0,
              charOffset: 0,
              parentParaIndex: res.backParaIdx,
              controlIndex: 0,
              cellIndex: Math.max(0, cellIndex - frontCells),
              cellParaIndex: 0,
            };
          },
        });
      }, '표 나누기');
    },
  },
  {
    // 한컴 용어는 '붙이기'(attach)지만 의미는 다음 표와의 행 병합이라
    // WASM API 는 mergeTableWithNext, 이벤트는 TablesMerged 를 쓴다.
    id: 'table:attach',
    label: '표 붙이기',
    shortcutLabel: 'Ctrl+M,Z',
    canExecute: (ctx) => ctx.inTable || ctx.inTableObjectSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      // 셀 컨텍스트가 필요 없는 명령이라 표 객체 선택 상태도 지원한다
      // (table:delete 와 같은 패턴 — 테두리 클릭 진입 시 커서는 셀 밖이다).
      let sec: number, ppi: number, ci: number;
      const ref = ih.getSelectedTableRef();
      if (ref) {
        sec = ref.sec; ppi = ref.ppi; ci = ref.ci;
      } else {
        const pos = ih.getCursorPosition();
        if (pos?.parentParaIndex === undefined || pos.controlIndex === undefined) return;
        // 중첩 표(cellPath 깊이 2+)는 flat 인덱스가 바깥 표를 가리켜 오동작한다 —
        // path 기반 API 가 생기기 전까지는 최상위 표에서만 허용.
        if ((pos.cellPath?.length ?? 0) > 1) return;
        sec = pos.sectionIndex; ppi = pos.parentParaIndex; ci = pos.controlIndex;
      }
      safeTableOp(() => {
        ih.executeOperation({
          kind: 'snapshot',
          operationType: 'mergeTableWithNext',
          operation: (wasm) => {
            wasm.mergeTableWithNext(sec, ppi, ci);
            return ih.getCursorPosition()!;
          },
        });
      }, '표 붙이기');
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
    canExecute: inTableOrCellSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const sec = pos.sectionIndex, ppi = pos.parentParaIndex, ci = pos.controlIndex;
      try {
        if (hasNonRectangularCellSelection(ih)) return;
        const dims = services.wasm.getTableDimensions(sec, ppi, ci);
        const range = equalizeTargetRange(ih, dims);
        const bboxes = services.wasm.getTableCellBboxes(sec, ppi, ci);
        const bboxByCellIdx = new Map(bboxes.map(bbox => [bbox.cellIdx, bbox]));
        const cells: Array<{ idx: number; height: number; renderHeight: number }> = [];
        for (let i = 0; i < dims.cellCount; i++) {
          const info = services.wasm.getCellInfo(sec, ppi, ci, i);
          if (!isCellInRange(info, range)) continue;
          if (info.rowSpan > 1) continue;
          const h = services.wasm.getCellProperties(sec, ppi, ci, i).height;
          const bbox = bboxByCellIdx.get(i);
          const renderHeight = bbox ? Math.round(bbox.h * 75) : h;
          cells.push({ idx: i, height: h, renderHeight });
        }
        if (cells.length < 2) return;
        const totalHeight = cells.reduce((sum, cell) => sum + cell.renderHeight, 0);
        const avgHeight = Math.round(totalHeight / cells.length);
        const updates: Parameters<CommandServices['wasm']['resizeTableCells']>[3] = [];
        let changed = false;
        for (const c of cells) {
          if (c.renderHeight !== avgHeight) changed = true;
          updates.push({
            cellIdx: c.idx,
            heightDelta: 0,
            localResize: true,
            renderHeight: avgHeight,
          });
        }
        if (!changed) return;
        safeTableOp(() => ih.executeOperation({
          kind: 'snapshot',
          operationType: 'equalizeTableCellHeights',
          operation: (wasm) => {
            wasm.resizeTableCells(sec, ppi, ci, updates);
            return pos;
          },
        }), '셀 높이를 같게');
        restoreEditorFocus(ih);
      } catch (err) {
        console.warn('[table:cell-height-equal] 높이 균등화 실패:', err);
      }
    },
  },
  {
    id: 'table:cell-width-equal',
    label: '셀 너비를 같게',
    shortcutLabel: 'W',
    canExecute: inTableOrCellSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined || pos.cellIndex === undefined) return;
      const sec = pos.sectionIndex, ppi = pos.parentParaIndex, ci = pos.controlIndex;
      try {
        if (hasNonRectangularCellSelection(ih)) return;
        const dims = services.wasm.getTableDimensions(sec, ppi, ci);
        const range = equalizeTargetRange(ih, dims);
        const bboxes = services.wasm.getTableCellBboxes(sec, ppi, ci);
        const bboxByCellIdx = new Map(bboxes.map(bbox => [bbox.cellIdx, bbox]));
        const cells: Array<{ idx: number; col: number; width: number; renderWidth: number }> = [];
        for (let i = 0; i < dims.cellCount; i++) {
          const info = services.wasm.getCellInfo(sec, ppi, ci, i);
          if (!isCellInRange(info, range)) continue;
          if (info.rowSpan > 1) continue;
          const w = services.wasm.getCellProperties(sec, ppi, ci, i).width;
          const bbox = bboxByCellIdx.get(i);
          const renderWidth = bbox ? Math.round(bbox.w * 75) : w;
          cells.push({ idx: i, col: info.col, width: w, renderWidth });
        }
        if (cells.length < 2) return;
        const totalWidth = cells.reduce((sum, cell) => sum + cell.renderWidth, 0);
        const avgWidth = Math.round(totalWidth / cells.length);
        const updates: Parameters<CommandServices['wasm']['resizeTableCells']>[3] = [];
        let changed = false;
        for (const c of cells) {
          const delta = avgWidth - c.width;
          if (delta !== 0 || c.renderWidth !== avgWidth) changed = true;
          updates.push({
            cellIdx: c.idx,
            widthDelta: delta,
            localResize: true,
            renderWidth: avgWidth,
          });
        }
        if (!changed) return;
        safeTableOp(() => ih.executeOperation({
          kind: 'snapshot',
          operationType: 'equalizeTableCellWidths',
          operation: (wasm) => {
            wasm.resizeTableCells(sec, ppi, ci, updates);
            return pos;
          },
        }), '셀 너비를 같게');
        restoreEditorFocus(ih);
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
        // [#2344] delete+insert 를 하나의 snapshot 으로 원자화해 라우팅 — 미기록 시 셀 문자
        // 수가 바뀌어 후속 undo 오프셋이 오염되고 텍스트가 손상된다("1234567"→쉼표→Ctrl+Z="67").
        // 라우터가 refresh 하므로 수동 document-changed emit 은 제거.
        // [Task #2370] 종전에는 여기를 safeTableOp 으로 한 겹 더 감쌌으나, 이 문장이 바깥
        // try 의 마지막이라 바깥 catch 는 도달할 수 없었다. 관측 차이는 로그뿐이고 바깥
        // catch 의 메시지가 더 구체적이므로 한 겹만 남긴다.
        ih.executeOperation({
          kind: 'snapshot',
          operationType: 'cellNumberFormat',
          operation: (wasm) => {
            wasm.deleteTextInCell(sec, ppi, ci, cei, cpi, 0, len);
            wasm.insertTextInCell(sec, ppi, ci, cei, cpi, 0, result);
            return pos;
          },
        });
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
        // [#2344] delete+insert 를 하나의 snapshot 으로 원자화해 라우팅 — 미기록 시 셀 문자
        // 수가 바뀌어 후속 undo 오프셋이 오염되고 텍스트가 손상된다("1234567"→쉼표→Ctrl+Z="67").
        // 라우터가 refresh 하므로 수동 document-changed emit 은 제거.
        // [Task #2370] 종전에는 여기를 safeTableOp 으로 한 겹 더 감쌌으나, 이 문장이 바깥
        // try 의 마지막이라 바깥 catch 는 도달할 수 없었다. 관측 차이는 로그뿐이고 바깥
        // catch 의 메시지가 더 구체적이므로 한 겹만 남긴다.
        ih.executeOperation({
          kind: 'snapshot',
          operationType: 'cellNumberFormat',
          operation: (wasm) => {
            wasm.deleteTextInCell(sec, ppi, ci, cei, cpi, 0, len);
            wasm.insertTextInCell(sec, ppi, ci, cei, cpi, 0, result);
            return pos;
          },
        });
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
        // [#2344] delete+insert 를 하나의 snapshot 으로 원자화해 라우팅 — 미기록 시 셀 문자
        // 수가 바뀌어 후속 undo 오프셋이 오염되고 텍스트가 손상된다("1234567"→쉼표→Ctrl+Z="67").
        // 라우터가 refresh 하므로 수동 document-changed emit 은 제거.
        // [Task #2370] 종전에는 여기를 safeTableOp 으로 한 겹 더 감쌌으나, 이 문장이 바깥
        // try 의 마지막이라 바깥 catch 는 도달할 수 없었다. 관측 차이는 로그뿐이고 바깥
        // catch 의 메시지가 더 구체적이므로 한 겹만 남긴다.
        ih.executeOperation({
          kind: 'snapshot',
          operationType: 'cellNumberFormat',
          operation: (wasm) => {
            wasm.deleteTextInCell(sec, ppi, ci, cei, cpi, 0, len);
            wasm.insertTextInCell(sec, ppi, ci, cei, cpi, 0, result);
            return pos;
          },
        });
      } catch (err) {
        console.warn('[table:decimal-remove] 자릿점 빼기 실패:', err);
      }
    },
  },
];
