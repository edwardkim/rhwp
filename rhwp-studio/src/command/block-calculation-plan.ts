export type BlockCalculationFunction = 'SUM' | 'AVERAGE' | 'PRODUCT';

export interface BlockCalculationCellState {
  empty: boolean;
  rowSpan?: number;
  colSpan?: number;
}

export interface BlockCalculationInput {
  range: { startRow: number; startCol: number; endRow: number; endCol: number };
  /** 선택 범위의 startRow/startCol에 맞춰 정렬된 셀 상태 행렬. */
  cells: BlockCalculationCellState[][];
  functionName: BlockCalculationFunction;
  hasExcludedCells?: boolean;
  nested?: boolean;
}

export interface BlockCalculationJob {
  targetRow: number;
  targetCol: number;
  formula: string;
}

export interface BlockCalculationPlan {
  orientation: 'horizontal' | 'vertical';
  jobs: BlockCalculationJob[];
}

function columnName(zeroBasedColumn: number): string | null {
  if (!Number.isInteger(zeroBasedColumn) || zeroBasedColumn < 0) return null;
  let value = zeroBasedColumn + 1;
  let result = '';
  while (value > 0) {
    const digit = (value - 1) % 26;
    result = String.fromCharCode(65 + digit) + result;
    value = Math.floor((value - 1) / 26);
  }
  return result;
}

function cellReference(row: number, col: number): string | null {
  const column = columnName(col);
  if (!column || !Number.isInteger(row) || row < 0) return null;
  return `${column}${row + 1}`;
}

function hasMergedCell(cells: BlockCalculationCellState[][]): boolean {
  return cells.some(row => row.some(cell =>
    (cell.rowSpan ?? 1) !== 1 || (cell.colSpan ?? 1) !== 1));
}

export function planBlockCalculation(input: BlockCalculationInput): BlockCalculationPlan | null {
  const { range, cells } = input;
  const rowCount = range.endRow - range.startRow + 1;
  const colCount = range.endCol - range.startCol + 1;
  if (input.nested || input.hasExcludedCells) return null;
  if (rowCount <= 0 || colCount <= 0 || rowCount * colCount <= 1) return null;
  if (cells.length !== rowCount || cells.some(row => row.length !== colCount)) return null;
  if (hasMergedCell(cells)) return null;

  const hasHorizontalResultEdge = colCount >= 2 && cells.every(row => row[colCount - 1].empty);
  const hasVerticalResultEdge = rowCount >= 2 && cells[rowCount - 1].every(cell => cell.empty);
  if (hasHorizontalResultEdge === hasVerticalResultEdge) return null;

  if (hasHorizontalResultEdge) {
    const startCol = columnName(range.startCol);
    const endCol = columnName(range.endCol - 1);
    if (!startCol || !endCol) return null;
    return {
      orientation: 'horizontal',
      jobs: Array.from({ length: rowCount }, (_, rowOffset) => {
        const row = range.startRow + rowOffset;
        return {
          targetRow: row,
          targetCol: range.endCol,
          formula: `=${input.functionName}(${startCol}${row + 1}:${endCol}${row + 1})`,
        };
      }),
    };
  }

  const jobs: BlockCalculationJob[] = [];
  for (let col = range.startCol; col <= range.endCol; col += 1) {
    const start = cellReference(range.startRow, col);
    const end = cellReference(range.endRow - 1, col);
    if (!start || !end) return null;
    jobs.push({
      targetRow: range.endRow,
      targetCol: col,
      formula: `=${input.functionName}(${start}:${end})`,
    });
  }
  return { orientation: 'vertical', jobs };
}

export function preflightBlockCalculationJobs(
  jobs: BlockCalculationJob[],
  evaluate: (job: BlockCalculationJob, writeResult: false) => { ok: boolean },
): boolean {
  if (jobs.length === 0) return false;
  try {
    return jobs.every(job => evaluate(job, false).ok);
  } catch {
    return false;
  }
}
