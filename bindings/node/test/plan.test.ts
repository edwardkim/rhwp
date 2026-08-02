/**
 * 계획 빌더 — 문법 검사와 직렬화.
 *
 * 빌더는 **문법만** 검사한다(값 타입·필수 인자·좌표 범위). 실행 가능성(그 필드가
 * 문서에 있는가, 그 좌표가 병합으로 덮였는가)은 rhwp 의 선검증이 판정한다.
 * 판정자를 두 곳에 두면 반드시 어긋나고, 그때 어느 쪽이 맞는지 아무도 모른다.
 *
 * 여기서는 프로세스를 띄우지 않는다 — 계획서 **모양**과 판정 **읽기**만 본다.
 * `check()`/`run()` 은 실물 rhwp 가 필요하므로 통합 쪽 몫이다.
 */

import { describe, expect, it } from 'vitest';

import { Envelope } from '../src/envelope.js';
import { Plan, PlanResult } from '../src/plan.js';

/**
 * 계획서를 원문 맵으로 본다.
 *
 * 계획서는 rhwp 에 그대로 건너가는 **JSON 계약**이다. 빌더가 어떤 정적 타입으로
 * 선언했든 최종적으로 맞아야 하는 것은 키 이름과 값이므로, 타입을 벗기고 확인한다.
 */
function payloadOf(value: unknown): Record<string, unknown> {
  return value as Record<string, unknown>;
}

/** step 배열을 원문 맵 배열로. */
function stepsOf(payload: Record<string, unknown>): Record<string, unknown>[] {
  const steps = payload['steps'];
  expect(Array.isArray(steps)).toBe(true);
  return steps as Record<string, unknown>[];
}

/**
 * 저널 항목의 필드를 읽는다.
 *
 * 항목이 {@link Envelope} 로 감싸여 오든 원문 객체로 오든 같은 방식으로 읽는다 —
 * 이 테스트가 고정하려는 것은 포장이 아니라 **판정 내용**이다.
 */
function field(item: unknown, key: string): unknown {
  if (item instanceof Envelope) return item.getOr<unknown>(key, undefined);
  return (item as Record<string, unknown>)[key];
}

describe('Plan — 계획서 조립', () => {
  it('빌더가 계약 모양의 계획서를 만든다', () => {
    const plan = new Plan('서식.hwp', '제출본.hwp')
      .fillFields({ 성명: '홍길동' })
      .replaceText('2025년', '2026년')
      .setCell(1, 0, 0, '값')
      .setCheckbox(1)
      .verify();
    const payload = payloadOf(plan.toJSON());

    expect(payload['planVersion']).toBe('1.0');
    expect(payload['input']).toBe('서식.hwp');
    expect(payload['output']).toBe('제출본.hwp');
    // action 이름은 rhwp 선검증이 문자열로 분기하는 값이다 — camel 로 바꾸면
    // "알 수 없는 action" 으로 되돌아온다.
    expect(stepsOf(payload).map((s) => s['action'])).toEqual([
      'fill_fields',
      'replace_text',
      'set_cell',
      'set_checkbox',
    ]);
  });

  it('단언은 assertions 아래에 모인다', () => {
    const plan = new Plan('a.hwp', 'b.hwp')
      .fillFields({ 이름: '값' })
      .verify()
      .requireAllFieldsFound();
    const assertions = payloadOf(payloadOf(plan.toJSON())['assertions']);

    expect(assertions['verify']).toBe(true);
    // 못 찾은 필드가 하나라도 있으면 실패로 친다. 서식 제출처럼 빠진 칸이 곧
    // 반려인 경우엔 조용한 성공이 더 위험하다.
    expect(assertions['notFoundEmpty']).toBe(true);
  });

  it('단언을 끌 수도 있다 — 껐다는 사실이 계획서에 남아야 한다', () => {
    const plan = new Plan('a.hwp', 'b.hwp').fillFields({ 이름: '값' }).verify(false);
    expect(payloadOf(payloadOf(plan.toJSON())['assertions'])['verify']).toBe(false);
  });

  it('dryRun 은 인자가 아니라 **계획서 필드**로 들어간다', () => {
    // 계획서가 dryRun 을 실으므로 MCP 경로도 인자 추가 없이 같은 계약을 얻는다.
    const plan = new Plan('a.hwp', 'b.hwp').fillFields({ 이름: '값' });
    expect(payloadOf(plan.toJSON())).not.toHaveProperty('dryRun');
    expect(payloadOf(plan.toJSON({ dryRun: true }))['dryRun']).toBe(true);
  });

  it('JSON.stringify(plan) 이 그대로 계획서가 된다', () => {
    // 계획서를 파일로 남기면 감사 추적·재현이 따라온다. `toJSON` 이라는 이름은
    // 우연이 아니다 — 다만 JSON.stringify 는 인자로 **키 문자열**을 넘기므로,
    // 옵션 객체를 그대로 믿는 구현은 여기서 깨진다.
    const plan = new Plan('a.hwp', 'b.hwp').replaceText('가', '나').verify();
    const serialized = JSON.parse(JSON.stringify(plan)) as unknown;
    expect(payloadOf(serialized)).toEqual(payloadOf(plan.toJSON()));
    expect(payloadOf(serialized)).not.toHaveProperty('dryRun');
  });

  it('체이닝이 같은 인스턴스를 돌려준다', () => {
    const plan = new Plan('a.hwp', 'b.hwp');
    expect(plan.fillFields({ a: 'b' })).toBe(plan);
    expect(plan.replaceText('가', '나')).toBe(plan);
    expect(plan.setCell(0, 0, 0, '값')).toBe(plan);
    expect(plan.setCheckbox(0)).toBe(plan);
    expect(plan.verify()).toBe(plan);
    expect(plan.requireAllFieldsFound()).toBe(plan);
  });
});

describe('Plan — step 직렬화', () => {
  it('fillFields 는 data 를 그대로 싣는다', () => {
    const plan = new Plan('a.hwp', 'b.hwp').fillFields({ 성명: '홍길동', '이름#1': '값' });
    const step = stepsOf(payloadOf(plan.toJSON()))[0];
    // `이름#1` 같은 순번 표기는 **사용자 데이터**다 — 이름 변환 대상이 아니다.
    expect(step?.['data']).toEqual({ 성명: '홍길동', '이름#1': '값' });
  });

  it('occurrence 는 줬을 때만 직렬화된다', () => {
    // 안 준 것과 0 번째를 지목한 것은 다른 요청이다. 기본값을 채워 넣으면
    // "전부 치환"이 조용히 "첫 번째만 치환"으로 바뀐다.
    const withOccurrence = new Plan('a.hwp', 'b.hwp').replaceText('가', '나', { occurrence: 2 });
    expect(stepsOf(payloadOf(withOccurrence.toJSON()))[0]?.['occurrence']).toBe(2);

    const without = new Plan('a.hwp', 'b.hwp').replaceText('가', '나');
    expect(stepsOf(payloadOf(without.toJSON()))[0]).not.toHaveProperty('occurrence');
  });

  it('caseSensitive 를 끄면 계획서에 남는다', () => {
    const plan = new Plan('a.hwp', 'b.hwp').replaceText('가', '나', { caseSensitive: false });
    expect(stepsOf(payloadOf(plan.toJSON()))[0]?.['caseSensitive']).toBe(false);
  });

  it('setCell 좌표와 값이 그대로 간다', () => {
    const plan = new Plan('a.hwp', 'b.hwp').setCell(2, 3, 4, '값', { keepStyle: true });
    const step = stepsOf(payloadOf(plan.toJSON()))[0];
    expect(step?.['table']).toBe(2);
    expect(step?.['row']).toBe(3);
    expect(step?.['col']).toBe(4);
    expect(step?.['text']).toBe('값');
    expect(step?.['keepStyle']).toBe(true);
  });

  it('setCheckbox 는 순번을 싣는다', () => {
    const plan = new Plan('a.hwp', 'b.hwp').setCheckbox(3);
    expect(stepsOf(payloadOf(plan.toJSON()))[0]?.['occurrence']).toBe(3);
  });
});

describe('Plan — 빌더가 즉시 거부하는 것', () => {
  it('step 이 하나도 없는 계획은 직렬화되지 않는다', () => {
    // 빈 계획을 보내면 rhwp 가 exit 2 로 돌려주는데, 그 왕복은 프로세스 하나를
    // 낭비하고 진단도 한 단계 멀어진다.
    expect(() => new Plan('a.hwp', 'b.hwp').toJSON()).toThrow();
  });

  it('빈 data 는 거부한다', () => {
    expect(() => new Plan('a.hwp', 'b.hwp').fillFields({})).toThrow();
  });

  it('빈 find 는 거부한다', () => {
    // 빈 문자열 치환은 문서 전체에 값을 흩뿌린다.
    expect(() => new Plan('a.hwp', 'b.hwp').replaceText('', '값')).toThrow();
  });

  it('문자열이 아닌 replace 는 거부한다', () => {
    expect(() =>
      new Plan('a.hwp', 'b.hwp').replaceText('가', 123 as unknown as string),
    ).toThrow();
  });

  it('셀 값의 줄바꿈·탭을 거부한다 — 셀은 한 줄 값이다', () => {
    // CLI 선검증과 같은 규칙을 빌더에서도 즉시 잡는다. 왕복 한 번을 아끼는 것보다,
    // 오류가 값을 만든 코드 옆에서 나는 것이 중요하다.
    expect(() => new Plan('a.hwp', 'b.hwp').setCell(0, 0, 0, '두\n줄')).toThrow();
    expect(() => new Plan('a.hwp', 'b.hwp').setCell(0, 0, 0, '탭\t들어감')).toThrow();
  });

  it.each([[-1], [1.5]])('좌표 %s 는 거부한다 — 0 이상의 정수만 좌표다', (bad) => {
    expect(() => new Plan('a.hwp', 'b.hwp').setCell(bad, 0, 0, '값')).toThrow();
    expect(() => new Plan('a.hwp', 'b.hwp').setCell(0, bad, 0, '값')).toThrow();
    expect(() => new Plan('a.hwp', 'b.hwp').setCell(0, 0, bad, '값')).toThrow();
  });

  it('NaN 좌표를 거부한다', () => {
    // `NaN < 0` 은 거짓이라 음수 검사만으로는 통과한다. 그런데 JSON.stringify 는
    // NaN 을 **null 로** 직렬화하므로, 통과시키면 계획서에 좌표가 사라진 채
    // rhwp 까지 가서 "table·row·col 이 필요합니다"로 되돌아온다 — 원인에서
    // 가장 먼 곳에서 드러나는 오류다.
    expect(() => new Plan('a.hwp', 'b.hwp').setCell(Number.NaN, 0, 0, '값')).toThrow();
  });

  it('음수 순번은 거부한다', () => {
    expect(() => new Plan('a.hwp', 'b.hwp').setCheckbox(-1)).toThrow();
    expect(() =>
      new Plan('a.hwp', 'b.hwp').replaceText('가', '나', { occurrence: -1 }),
    ).toThrow();
  });
});

describe('PlanResult — 위반은 예외가 아니라 결과다', () => {
  it('선검증 위반을 데이터로 돌려준다', () => {
    // 계획 실행에서 위반은 **설계된 흐름의 일부**다 — 검사하고, 고치고, 다시
    // 검사한다. 예외로 만들면 catch 안에서 계획을 고치는 어색한 코드가 된다.
    const result = new PlanResult({
      schemaVersion: '1.0',
      planVersion: '1.0',
      invalid: [
        { step: 0, action: 'fill_fields', reason: "필드 '없음' 이(가) 없습니다" },
        { step: 1, action: 'replace_text', reason: "'X' 일치 0건" },
      ],
    });

    expect(result.ok).toBe(false);
    expect(result.violations).toHaveLength(2);
    const described = result.describeViolations();
    expect(described).toContain('step 0');
    expect(described).toContain('일치 0건');
  });

  it('위반이 없으면 통과다', () => {
    const result = new PlanResult({ schemaVersion: '1.0', steps: [{ step: 0 }] });
    expect(result.ok).toBe(true);
    expect(result.violations).toEqual([]);
    expect(result.describeViolations()).toContain('위반 없음');
  });

  it('검사 결과는 preview 를 주고 steps 는 비어 있다', () => {
    const result = new PlanResult({
      schemaVersion: '1.0',
      dryRun: true,
      invalid: [],
      preview: [{ step: 0, action: 'replace_text', matches: 7, willReplace: 7 }],
    });

    expect(result.isDryRun).toBe(true);
    expect(result.ok).toBe(true);
    expect(result.preview).toHaveLength(1);
    expect(field(result.preview[0], 'matches')).toBe(7);
    // 검사는 디스크를 건드리지 않았다 — 적용 결과가 있을 수 없다.
    expect(result.steps).toEqual([]);
  });

  it('실행 결과는 steps·verify·changedPages 를 준다', () => {
    const result = new PlanResult({
      schemaVersion: '1.0',
      steps: [{ step: 0, action: 'fill_fields', filledCount: 1 }],
      verify: { identical: true, diffCount: 0 },
      changedPages: [0],
    });

    expect(result.isDryRun).toBe(false);
    expect(result.steps).toHaveLength(1);
    expect(field(result.steps[0], 'filledCount')).toBe(1);
    // PlanResult 도 봉투다 — 판정 필드를 그대로 갖는다.
    expect(result.verify?.identical).toBe(true);
    expect(result.changedPages).toEqual([0]);
    expect(result.preview).toEqual([]);
  });

  it('검증하지 않은 실행의 verify 는 null 이다 — 실패와 구분된다', () => {
    const result = new PlanResult({ schemaVersion: '1.0', steps: [], verify: null });
    expect(result.verify).toBeNull();
  });

  it('판정 실패는 verify 로 드러난다 — ok 는 선검증 위반만 본다', () => {
    // 단언 실패(exit 3)는 "계획이 틀렸다"가 아니라 "저장본이 계획과 다르다"다.
    // 두 신호를 한 불리언에 뭉치면 무엇을 고쳐야 할지 알 수 없다.
    const result = new PlanResult({
      schemaVersion: '1.0',
      steps: [{ step: 0 }],
      verify: { identical: false, diffCount: 2 },
      error: 'verify 단언 실패 — 디스크 무변경',
    });

    expect(result.ok).toBe(true);
    expect(result.verify?.identical).toBe(false);
    expect(result.verify?.diffCount).toBe(2);
  });
});
