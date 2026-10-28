import test from 'node:test';
import assert from 'node:assert/strict';

type CellState = {
  empty: boolean;
  rowSpan?: number;
  colSpan?: number;
};

type BlockCalculationInput = {
  range: { startRow: number; startCol: number; endRow: number; endCol: number };
  cells: CellState[][];
  functionName: 'SUM';
  hasExcludedCells?: boolean;
  nested?: boolean;
};

type BlockCalculationPlan = {
  orientation: 'horizontal' | 'vertical';
  jobs: Array<{ targetRow: number; targetCol: number; formula: string }>;
};

type BlockCalculationPlanner = (input: BlockCalculationInput) => BlockCalculationPlan | null;
type BlockCalculationJob = BlockCalculationPlan['jobs'][number];
type BlockCalculationPreflight = (
  jobs: BlockCalculationJob[],
  evaluate: (job: BlockCalculationJob, writeResult: false) => { ok: boolean },
) => boolean;

async function calculationModule(): Promise<{
  planBlockCalculation?: BlockCalculationPlanner;
  preflightBlockCalculationJobs?: BlockCalculationPreflight;
}> {
  const modulePath = '../src/command/block-calculation-plan.ts';
  return await import(modulePath) as unknown as {
    planBlockCalculation?: BlockCalculationPlanner;
    preflightBlockCalculationJobs?: BlockCalculationPreflight;
  };
}

async function planner(): Promise<BlockCalculationPlanner> {
  const module = await calculationModule();
  assert.equal(
    typeof module.planBlockCalculation,
    'function',
    'Recovery R1: 선택 범위 블록 계산 planner가 필요하다',
  );
  return module.planBlockCalculation;
}

const filled = (): CellState => ({ empty: false });
const blank = (): CellState => ({ empty: true });

test('Recovery R1: 오른쪽 빈 결과 열을 포함한 블록은 행별 합계 job을 만든다', async () => {
  const plan = await planner();
  assert.deepEqual(plan({
    range: { startRow: 0, startCol: 0, endRow: 1, endCol: 3 },
    cells: [
      [filled(), filled(), filled(), blank()],
      [filled(), filled(), filled(), blank()],
    ],
    functionName: 'SUM',
  }), {
    orientation: 'horizontal',
    jobs: [
      { targetRow: 0, targetCol: 3, formula: '=SUM(A1:C1)' },
      { targetRow: 1, targetCol: 3, formula: '=SUM(A2:C2)' },
    ],
  });
});

test('Recovery R1: 아래 빈 결과 행을 포함한 블록은 열별 합계 job을 만든다', async () => {
  const plan = await planner();
  assert.deepEqual(plan({
    range: { startRow: 0, startCol: 0, endRow: 2, endCol: 1 },
    cells: [
      [filled(), filled()],
      [filled(), filled()],
      [blank(), blank()],
    ],
    functionName: 'SUM',
  }), {
    orientation: 'vertical',
    jobs: [
      { targetRow: 2, targetCol: 0, formula: '=SUM(A1:A2)' },
      { targetRow: 2, targetCol: 1, formula: '=SUM(B1:B2)' },
    ],
  });
});

test('Recovery R1: Z 다음 열도 AA/AB 표기로 선택 범위 계산식을 만든다', async () => {
  const plan = await planner();
  assert.deepEqual(plan({
    range: { startRow: 4, startCol: 25, endRow: 4, endCol: 27 },
    cells: [[filled(), filled(), blank()]],
    functionName: 'SUM',
  }), {
    orientation: 'horizontal',
    jobs: [
      { targetRow: 4, targetCol: 27, formula: '=SUM(Z5:AA5)' },
    ],
  });
});

test('Recovery R1: 결과 가장자리가 없거나 양축이 모두 비면 문서를 바꿀 계획을 만들지 않는다', async () => {
  const plan = await planner();
  assert.equal(plan({
    range: { startRow: 0, startCol: 0, endRow: 1, endCol: 1 },
    cells: [
      [filled(), filled()],
      [filled(), filled()],
    ],
    functionName: 'SUM',
  }), null, '빈 결과 가장자리가 없으면 거절');

  assert.equal(plan({
    range: { startRow: 0, startCol: 0, endRow: 2, endCol: 2 },
    cells: [
      [filled(), filled(), blank()],
      [filled(), filled(), blank()],
      [blank(), blank(), blank()],
    ],
    functionName: 'SUM',
  }), null, '오른쪽과 아래가 모두 빈 모호한 선택은 거절');
});

test('Recovery R1: 단일·불연속·병합·중첩 선택은 fail-closed한다', async () => {
  const plan = await planner();
  assert.equal(plan({
    range: { startRow: 0, startCol: 0, endRow: 0, endCol: 0 },
    cells: [[blank()]],
    functionName: 'SUM',
  }), null, '단일 셀 거절');

  const horizontal: BlockCalculationInput = {
    range: { startRow: 0, startCol: 0, endRow: 0, endCol: 2 },
    cells: [[filled(), filled(), blank()]],
    functionName: 'SUM',
  };
  assert.equal(plan({ ...horizontal, hasExcludedCells: true }), null, '불연속 선택 거절');
  assert.equal(plan({ ...horizontal, nested: true }), null, '중첩 표 거절');
  assert.equal(plan({
    ...horizontal,
    cells: [[{ empty: false, colSpan: 2 }, filled(), blank()]],
  }), null, '병합 셀 거절');
});

test('Recovery R1: 일부 dry-run 실패 시 preflight는 write 없이 전체 작업을 거절한다', async () => {
  const module = await calculationModule();
  assert.equal(
    typeof module.preflightBlockCalculationJobs,
    'function',
    'Recovery R1: 블록 계산 job 전체를 먼저 dry-run하는 helper가 필요하다',
  );
  const jobs: BlockCalculationJob[] = [
    { targetRow: 0, targetCol: 2, formula: '=SUM(A1:B1)' },
    { targetRow: 1, targetCol: 2, formula: '=SUM(A2:B2)' },
  ];
  const calls: Array<{ job: BlockCalculationJob; writeResult: false }> = [];
  const accepted = module.preflightBlockCalculationJobs!(jobs, (job, writeResult) => {
    calls.push({ job, writeResult });
    return { ok: job.targetRow === 0 };
  });

  assert.equal(accepted, false);
  assert.ok(calls.length >= 1);
  assert.ok(calls.every(call => call.writeResult === false), 'preflight에서 write=true 호출 금지');
});
