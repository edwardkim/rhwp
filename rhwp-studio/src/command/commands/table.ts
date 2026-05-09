import type { CommandDef, EditorContext } from '../types';
import { TableCellPropsDialog } from '@/ui/table-cell-props-dialog';
import { TableCreateDialog } from '@/ui/table-create-dialog';
import { CellSplitDialog } from '@/ui/cell-split-dialog';
import { CellBorderBgDialog } from '@/ui/cell-border-bg-dialog';
import { FormulaDialog } from '@/ui/formula-dialog';

const inTable = (ctx: EditorContext) => ctx.inTable;

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

export const tableCommands: CommandDef[] = [
  { id: 'table:create', label: '표 만들기', icon: 'icon-table',
    canExecute: (ctx) => ctx.hasDocument && !ctx.inTable,
    execute(services, params) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const pos = ih.getCursorPosition();
      // 셀 내부에서는 사용 불가 (canExecute에서 걸리지만 방어)
      if (pos.parentParaIndex !== undefined) return;
      const dialog = new TableCreateDialog();
      dialog.onApply = (rows, cols) => {
        try {
          const result = services.wasm.createTable(
            pos.sectionIndex, pos.paragraphIndex, pos.charOffset,
            rows, cols,
          );
          if (result.ok) {
            services.eventBus.emit('document-changed');
            // 표 생성 후 첫 번째 셀로 커서 이동
            const ih = services.getInputHandler();
            if (ih) {
              ih.moveCursorTo({
                sectionIndex: pos.sectionIndex,
                paragraphIndex: 0,
                charOffset: 0,
                parentParaIndex: result.paraIdx,
                controlIndex: 0,
                cellIndex: 0,
                cellParaIndex: 0,
              });
            }
          }
        } catch (e) {
          console.error('표 만들기 실패:', e);
        }
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
      try {
        services.wasm.insertTableRow(pos.sectionIndex, pos.parentParaIndex, pos.controlIndex, cellInfo.row, false);
        services.eventBus.emit('document-changed');
      } catch (e) {
        console.error('줄 추가 실패:', e);
      }
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
      try {
        services.wasm.insertTableRow(pos.sectionIndex, pos.parentParaIndex, pos.controlIndex, cellInfo.row, true);
        services.eventBus.emit('document-changed');
      } catch (e) {
        console.error('줄 추가 실패:', e);
      }
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
      try {
        services.wasm.insertTableColumn(pos.sectionIndex, pos.parentParaIndex, pos.controlIndex, cellInfo.col, false);
        services.eventBus.emit('document-changed');
      } catch (e) {
        console.error('칸 추가 실패:', e);
      }
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
      try {
        services.wasm.insertTableColumn(pos.sectionIndex, pos.parentParaIndex, pos.controlIndex, cellInfo.col, true);
        services.eventBus.emit('document-changed');
      } catch (e) {
        console.error('칸 추가 실패:', e);
      }
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
      try {
        services.wasm.deleteTableRow(pos.sectionIndex, pos.parentParaIndex, pos.controlIndex, cellInfo.row);
        services.eventBus.emit('document-changed');
      } catch (e) {
        console.error('줄 지우기 실패:', e);
      }
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
      try {
        services.wasm.deleteTableColumn(pos.sectionIndex, pos.parentParaIndex, pos.controlIndex, cellInfo.col);
        services.eventBus.emit('document-changed');
      } catch (e) {
        console.error('칸 지우기 실패:', e);
      }
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
        try {
          if (isMultiCell && range && tableCtx) {
            // 다중 셀: 범위 내 각 셀을 개별 분할
            services.wasm.splitTableCellsInRange(
              tableCtx.sec, tableCtx.ppi, tableCtx.ci,
              range.startRow, range.startCol, range.endRow, range.endCol,
              nRows, mCols, equalHeight,
            );
            ih.exitCellSelectionMode?.();
          } else {
            // 단일 셀 분할
            services.wasm.splitTableCellInto(
              pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!,
              cellInfo.row, cellInfo.col,
              nRows, mCols, equalHeight, mergeFirst,
            );
          }
          services.eventBus.emit('document-changed');
        } catch (e) {
          console.error('셀 나누기 실패:', e);
        }
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
      try {
        services.wasm.mergeTableCells(tableCtx.sec, tableCtx.ppi, tableCtx.ci, range.startRow, range.startCol, range.endRow, range.endCol);
        ih.exitCellSelectionMode();
        services.eventBus.emit('document-changed');
      } catch (e) {
        console.error('셀 합치기 실패:', e);
      }
    },
  },
  {
    id: 'table:delete',
    label: '표 지우기',
    canExecute: (ctx) => ctx.inTable || ctx.inTableObjectSelection,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      // 표 객체 선택 모드면 선택된 표 참조 사용
      const ref = ih.getSelectedTableRef();
      if (ref) {
        try {
          services.wasm.deleteTableControl(ref.sec, ref.ppi, ref.ci);
          services.eventBus.emit('document-changed');
        } catch (e) {
          console.error('표 지우기 실패:', e);
        }
        return;
      }
      // 셀 내부에서 커맨드 실행
      const pos = ih.getCursorPosition();
      if (pos.parentParaIndex === undefined || pos.controlIndex === undefined) return;
      try {
        services.wasm.deleteTableControl(pos.sectionIndex, pos.parentParaIndex, pos.controlIndex);
        services.eventBus.emit('document-changed');
      } catch (e) {
        console.error('표 지우기 실패:', e);
      }
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
        try {
          const result: any = services.wasm.setTableProperties(sec, ppi, ci, { hasCaption: true });
          charOffset = result?.captionCharOffset ?? 3;
          services.eventBus.emit('document-changed');
        } catch (e) { console.error('표 캡션 생성 실패:', e); return; }
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
    shortcutLabel: 'Ctrl+N,F',
    canExecute: inTable,
    execute(services) {
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
    },
  },
  stub('table:block-formula', '블록 계산식'),
  stub('table:block-sum', '블록 합계', undefined, 'Ctrl+Shift+S'),
  stub('table:block-avg', '블록 평균', undefined, 'Ctrl+Shift+A'),
  stub('table:block-product', '블록 곱', undefined, 'Ctrl+Shift+P'),
  stub('table:thousand-sep', '1,000 단위 구분 쉼표'),
  stub('table:decimal-add', '자릿점 넣기'),
  stub('table:decimal-remove', '자릿점 빼기'),
];
