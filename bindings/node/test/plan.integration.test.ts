/**
 * 3층(계획) 통합 — 절차가 아니라 **의도**를 넘긴다.
 *
 * 다단 편집을 호출 체인으로 조립하면 두 가지가 반복해서 무너진다: 호출 사이에 상태가
 * 사라지고, 중간 실패가 반쪽 편집 문서를 남긴다. 계획 계층은 그래서 전 단계를 먼저
 * **선검증**하고, 통과했을 때만 한 번 저장한다.
 *
 * 이 파일이 지키는 핵심 두 가지:
 *
 * - **위반은 예외가 아니라 결과다.** 계획을 고쳐 다시 검사하는 것이 정상 흐름이므로,
 *   위반을 던지면 호출자는 정상 흐름을 `try/catch` 로 쓰게 된다.
 * - **위반은 한 번에 전부 보고된다.** 하나 고치면 다음 위반이 나오는 두더지잡기는
 *   에이전트 왕복을 선형으로 늘린다.
 */

import { existsSync } from 'node:fs';

import { describe, expect, it } from 'vitest';

import { Plan } from '../src/index.js';
import {
  fieldSample,
  fieldSampleReady,
  firstFieldName,
  supportsPlanDryRun,
  useTempDir,
} from './helpers/integration.js';

// 계획 실행기의 `--dry-run` 지원 여부는 **자기서술로** 확인한다. 버전 문자열로 추정하면
// 포크·패치 빌드에서 곧바로 틀리고, 없는 기능을 시험하면 진짜 계약 위반과 구분되지
// 않는 실패가 난다.
const dryRunSupported = await supportsPlanDryRun();

describe.skipIf(!fieldSampleReady)('3층 계획 — 선검증은 실행 전에 막는다', () => {
  const tempPath = useTempDir();

  it('위반은 예외가 아니라 결과로 오고 산출은 만들어지지 않는다', async () => {
    const out = tempPath('안나옴.hwp');

    const result = await new Plan(fieldSample(), out)
      .fillFields({ 존재하지않는필드XYZ: '값' })
      .run();

    expect(result.ok).toBe(false);
    expect(result.violations.length).toBeGreaterThanOrEqual(1);
    // 사람이 읽고 고칠 수 있어야 한다 — 위반 목록이 있는데 무엇이 문제인지 못 읽으면
    // 결국 원문 JSON 을 뒤지게 된다.
    expect(result.describeViolations()).toContain('존재하지않는필드XYZ');
    // 선검증이 실행을 0 단계에서 끊었으므로 디스크는 무변경이다.
    expect(existsSync(out), '선검증에서 막혔는데 파일이 생겼다').toBe(false);
  });

  it('위반 여러 건을 한 번에 보고한다 — 두더지잡기를 만들지 않는다', async () => {
    const out = tempPath('안나옴2.hwp');

    const result = await new Plan(fieldSample(), out)
      .fillFields({ 없는필드A: '값' })
      .replaceText('이런문자열은결코없다9999', 'X')
      .run();

    expect(result.ok).toBe(false);
    // 두 step 이 각각 잘못됐으니 두 건이 함께 와야 한다. 하나만 오면 호출자는
    // 고치고 다시 돌리기를 반복하게 되고, 왕복이 위반 수만큼 늘어난다.
    expect(result.violations, result.describeViolations()).toHaveLength(2);

    // 두 위반이 서로 다른 step 을 가리키는지는 설명문으로 확인한다 — 같은 step 을
    // 두 번 세는 구현이면 사용자는 고칠 곳을 하나밖에 못 찾는다.
    const described = result.describeViolations();
    expect(described).toContain('없는필드A');
    expect(described).toContain('이런문자열은결코없다9999');
    expect(existsSync(out)).toBe(false);
  });

  it('없는 누름틀은 계획에서 전부 아니면 전무로 막힌다', async () => {
    const name = await firstFieldName(fieldSample());
    if (name === undefined) return;
    const out = tempPath('부분채움금지.hwp');

    // 하나는 실재하고 하나는 없다. 편집 계층이라면 실재하는 쪽만 채우고 나머지를
    // `notFound` 로 보고하지만, 계획은 부분 성공을 인정하지 않는다.
    const result = await new Plan(fieldSample(), out)
      .fillFields({ [name]: '값', 절대로없는필드QQQ: '값' })
      .run();

    expect(result.ok).toBe(false);
    expect(result.describeViolations()).toContain('절대로없는필드QQQ');
    expect(existsSync(out)).toBe(false);
  });
});

describe.skipIf(!fieldSampleReady)('3층 계획 — 실행은 저널을 남긴다', () => {
  const tempPath = useTempDir();

  it('run() 이 step 저널·판정·산출을 함께 낸다', async () => {
    const name = await firstFieldName(fieldSample());
    if (name === undefined) return;
    const out = tempPath('계획실행.hwp');

    const journal = await new Plan(fieldSample(), out)
      .fillFields({ [name]: '실행값' })
      .verify()
      .run();

    expect(journal.ok, journal.describeViolations()).toBe(true);
    expect(journal.violations).toHaveLength(0);
    // 저널이 있어야 "무엇을 왜 바꿨는지"를 나중에 설명할 수 있다. 계획서와 저널이
    // 감사 추적의 양 끝이다.
    expect(journal.steps).toHaveLength(1);
    expect(journal.verify, 'verify() 를 걸었는데 판정이 없다').not.toBeNull();
    expect(journal.verify?.identical).toBe(true);
    expect(existsSync(out)).toBe(true);
    // 눈검증 대상 쪽도 함께 와야 렌더 확인이 상수 비용으로 끝난다.
    expect(journal.changedPages).not.toBeNull();
  });
});

// `check()` 는 계획을 **실행하지 않고** 미리 보는 축이라 도구의 `run --dry-run` 이
// 필요하다. 없는 버전에서 억지로 돌리면 미리보기가 실제 저장으로 바뀌어, 테스트가
// 검증하려던 바로 그 계약("디스크 무변경")을 스스로 깬다.
describe.skipIf(!fieldSampleReady || !dryRunSupported)('3층 계획 — check() 는 미리보기다', () => {
  const tempPath = useTempDir();

  it('check() 가 디스크를 건드리지 않고 step 미리보기를 낸다', async () => {
    const name = await firstFieldName(fieldSample());
    if (name === undefined) return;
    const out = tempPath('계획미리보기.hwp');

    const plan = new Plan(fieldSample(), out).fillFields({ [name]: '계획값' }).verify();
    const preview = await plan.check();

    expect(preview.isDryRun).toBe(true);
    expect(preview.ok, preview.describeViolations()).toBe(true);
    expect(preview.preview).toHaveLength(1);
    expect(existsSync(out), 'check() 가 파일을 만들었다').toBe(false);
  });

  it('같은 계획을 check() 로 통과시킨 뒤 run() 하면 그대로 산출된다', async () => {
    const name = await firstFieldName(fieldSample());
    if (name === undefined) return;
    const out = tempPath('검사후실행.hwp');

    const plan = new Plan(fieldSample(), out).fillFields({ [name]: '값' }).verify();

    const preview = await plan.check();
    expect(preview.ok).toBe(true);
    expect(existsSync(out)).toBe(false);

    // 미리보기와 실행이 같은 판정자를 쓴다는 것이 계획 계층의 전제다. 통과한 계획이
    // 실행에서 막히면 미리보기는 신뢰할 수 없는 정보가 된다.
    const journal = await plan.run();
    expect(journal.ok, journal.describeViolations()).toBe(true);
    expect(journal.isDryRun).toBe(false);
    expect(existsSync(out)).toBe(true);
  });
});
