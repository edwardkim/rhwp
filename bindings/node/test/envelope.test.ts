/**
 * 봉투 모델 — 세 가지 접근 방식과 "모름 vs 없음" 구분.
 *
 * 가장 중요한 계약: **오타가 조용한 `undefined` 가 되면 안 된다.** 없는 필드를 물으면
 * 실패해야 한다. 그렇지 않으면 필드 이름을 잘못 쓴 코드가 "값이 없네"로 흘러가
 * 예외 없이 잘못된 결과를 내는데, 그게 가장 찾기 어려운 버그가 된다.
 */

import { describe, expect, it } from 'vitest';

import { Envelope, VerifyReport, asEnvelope } from '../src/envelope.js';
import type { RawEnvelope } from '../src/envelope.js';

/**
 * 생성기가 만들 봉투 타입의 대역.
 *
 * `interface` 가 아니라 `type` 인 것이 중요하다 — interface 는 암묵적 인덱스 시그니처를
 * 얻지 못해 `RawEnvelope` 제약을 만족하지 못한다. 생성기가 이 규칙을 어기면 생성 타입을
 * `Envelope<T>` 에 못 꽂는다.
 */
type InfoEnvelope = {
  schemaVersion: string;
  pageCount: number;
};

/** 없는 필드 접근이 던지는 것을 값으로 잡는다. */
function capture(run: () => unknown): unknown {
  try {
    run();
  } catch (error) {
    return error;
  }
  throw new Error('예외가 발생해야 하는데 정상 반환했습니다');
}

describe('세 가지 접근', () => {
  it('원문 키·snake 표기·raw 가 같은 값에 닿는다', () => {
    // 봉투 키는 camel 이지만 호출자는 snake 로 물을 수 있어야 한다 — 표기 하나
    // 때문에 "없는 필드" 예외가 나면 규약이 오히려 방해가 된다.
    const env = new Envelope({ pageCount: 3, schemaVersion: '1.0' });
    expect(env.get('pageCount')).toBe(3); // 원문(camel) 키
    expect(env.get('page_count')).toBe(3); // snake 로 물어도
    expect(env.raw['pageCount']).toBe(3); // 원문 그대로
    expect(env.getPath('pageCount')).toBe(3); // 점 경로(한 조각)
  });

  it('생성 타입을 주면 raw 가 정적으로 좁혀진다', () => {
    const env = new Envelope<InfoEnvelope>({ schemaVersion: '1.0', pageCount: 3 });
    // 아래 두 줄은 런타임 단언이자 **컴파일 단언**이다. `raw.pageCount` 가
    // `unknown` 으로 새면 여기서 타입 검사가 깨진다.
    const count: number = env.raw.pageCount;
    expect(count).toBe(3);
    expect(env.get<number>('pageCount')).toBe(3);
  });

  it('has 가 세 표기를 모두 인정한다', () => {
    const env = new Envelope({ changedPages: [0] });
    expect(env.has('changedPages')).toBe(true);
    expect(env.has('changed_pages')).toBe(true);
    expect(env.has('없는필드')).toBe(false);
  });

  it('keys 는 원문 키를 그대로 준다 — 봉투를 다시 조립할 수 있어야 한다', () => {
    const env = new Envelope({ pageCount: 3, schemaVersion: '1.0' });
    expect(env.keys().sort()).toEqual(['pageCount', 'schemaVersion']);
  });
});

describe('없는 필드', () => {
  it('조용한 undefined 가 아니라 예외다', () => {
    const env = new Envelope({ pageCount: 3, schemaVersion: '1.0' });
    const error = capture(() => env.get('pageConut')); // 오타
    expect(error).toBeInstanceOf(Error);
    // 있는 필드를 함께 알려줘야 사용자가 즉시 고칠 수 있다. "없습니다"만 알려주는
    // 오류는 사용자를 봉투 문서 찾기로 내몬다.
    expect((error as Error).message).toContain('pageConut');
    expect((error as Error).message).toContain('pageCount');
    expect((error as Error).message).toContain('schemaVersion');
  });

  it('getOr 는 **선택** 필드용이다', () => {
    const env = new Envelope({ pageCount: 3 });
    expect(env.getOr('pageCount', 0)).toBe(3);
    expect(env.getOr('truncated', false)).toBe(false);
    // 필수 필드에 getOr 를 쓰면 위의 보호를 스스로 버리는 것이다 — 그래서
    // 기본값을 명시하도록 강제한다(인자 하나짜리 오버로드가 없다).
    expect(env.getOr<number | null>('changedPages', null)).toBeNull();
  });

  it('값이 undefined 인 필드는 "있는" 필드다', () => {
    // 키가 없는 것과 값이 undefined 인 것은 다른 상태다. JSON 봉투에는
    // undefined 가 올 수 없지만, 손으로 만든 봉투가 이 경계를 밟는다.
    const env = new Envelope({ verify: undefined });
    expect(env.has('verify')).toBe(true);
    expect(env.get('verify')).toBeUndefined();
  });
});

describe('getPath', () => {
  const env = new Envelope({
    verify: { identical: true, diffCount: 0 },
    steps: [{ action: 'fill_fields' }],
  });

  it('점 경로로 중첩 값을 꺼낸다', () => {
    expect(env.getPath('verify.identical')).toBe(true);
    expect(env.getPath('verify.diffCount')).toBe(0);
  });

  it('경로 조각도 snake 로 쓸 수 있다', () => {
    expect(env.getPath('verify.diff_count')).toBe(0);
  });

  it('없는 경로는 undefined — 여기서만 조용한 undefined 가 허용된다', () => {
    // `getPath` 는 "있으면 보고 싶다"는 용도다. 필수 필드는 `get` 으로 읽는다.
    expect(env.getPath('verify.없음')).toBeUndefined();
    expect(env.getPath('없는.경로')).toBeUndefined();
    // 중간이 원시값이면 더 파고들지 않는다(`0.identical` 같은 접근으로 죽지 않게).
    expect(env.getPath('verify.diffCount.identical')).toBeUndefined();
  });
});

describe('verify — null(검증 안 함)과 실패는 다르다', () => {
  it('요청하지 않았으면 null 이다', () => {
    // 이 둘을 섞으면 **검증하지 않은 저장을 통과로 읽는다.**
    expect(new Envelope({ output: 'a.hwp', verify: null }).verify).toBeNull();
    expect(new Envelope({ output: 'a.hwp' }).verify).toBeNull();
  });

  it('실패한 검증은 보고서로 나온다', () => {
    const report = new Envelope({ verify: { identical: false, diffCount: 2 } }).verify;
    expect(report).toBeInstanceOf(VerifyReport);
    expect(report?.identical).toBe(false);
    expect(report?.diffCount).toBe(2);
    expect(report?.reparseError).toBeUndefined();
  });

  it('통과한 검증도 같은 모양이다', () => {
    const report = new Envelope({ verify: { identical: true, diffCount: 0 } }).verify;
    expect(report?.identical).toBe(true);
    expect(String(report)).toBe('동일');
  });

  it('identical 이 없으면 통과로 보지 않는다', () => {
    // 필드가 빠진 봉투를 "동일"로 읽으면 검증 없는 저장이 통과가 된다.
    const report = new Envelope({ verify: {} }).verify;
    expect(report?.identical).toBe(false);
  });

  it('재파싱 실패는 "판정 불가"가 아니라 실패로 드러난다', () => {
    const report = new Envelope({
      verify: { identical: false, diffCount: null, reparseError: '손상' },
    }).verify;
    expect(report?.identical).toBe(false);
    // 차이를 셀 수 없었다 — 0 이 아니라 null 이어야 한다.
    expect(report?.diffCount).toBeNull();
    expect(report?.reparseError).toBe('손상');
    expect(String(report)).toContain('재파싱 실패');
  });

  it('보고서의 raw 도 사본이다', () => {
    const report = new VerifyReport({ identical: true, diffCount: 0 });
    // 원문은 readonly 로 선언돼 있으니 캐스팅으로 뚫고 고쳐 본다 — 타입이 아니라
    // **런타임 격리**를 확인하는 것이 이 테스트의 목적이다.
    (report.raw as { identical?: boolean }).identical = false;
    expect(report.identical).toBe(true);
  });
});

describe('changedPages — null(모름)과 []( 없음)은 다른 결론이다', () => {
  it('확정 불가는 null 이다', () => {
    // 둘을 falsy 로 뭉뚱그리면 "확인할 게 없다"고 잘못 판단하고, 눈으로 봐야 할
    // 페이지를 건너뛴다.
    expect(new Envelope({ changedPages: null }).changedPages).toBeNull();
    expect(new Envelope({}).changedPages).toBeNull();
  });

  it('바뀐 쪽이 없으면 빈 배열이다', () => {
    expect(new Envelope({ changedPages: [] }).changedPages).toEqual([]);
  });

  it('쪽 번호를 그대로 준다', () => {
    expect(new Envelope({ changedPages: [0, 2] }).changedPages).toEqual([0, 2]);
  });

  it('숫자가 아닌 항목은 걸러낸다 — 쪽 번호 자리에 문자열이 오면 계약 위반이다', () => {
    expect(new Envelope({ changedPages: [0, 'x', 2] }).changedPages).toEqual([0, 2]);
  });
});

describe('raw 는 사본이다', () => {
  it('돌려받은 원문을 고쳐도 봉투는 안 바뀐다', () => {
    // 봉투는 도구가 내놓은 **판정**이다. 호출자가 흘려보내다 고친 값이 원본으로
    // 되비치면 판정의 출처를 잃는다.
    const env = new Envelope({ pageCount: 3 });
    const raw = env.raw;
    raw['pageCount'] = 99;
    expect(env.get('pageCount')).toBe(3);
  });

  it('toJSON 이 원문을 그대로 내보낸다', () => {
    const source = { schemaVersion: '1.0', pageCount: 3 };
    const env = new Envelope(source);
    expect(env.toJSON()).toEqual(source);
    expect(JSON.parse(JSON.stringify(env))).toEqual(source);
  });
});

describe('중첩 감싸기', () => {
  it('child 가 하위 객체를 봉투로 감싼다', () => {
    const env = new Envelope({ verify: { diffCount: 0, identical: true } });
    const child = env.child('verify');
    expect(child).toBeInstanceOf(Envelope);
    expect(child?.get('diff_count')).toBe(0);
  });

  it('child 는 객체가 아니면 null 이다', () => {
    const env = new Envelope({ pageCount: 3, pages: [1, 2], nothing: null });
    expect(env.child('pageCount')).toBeNull();
    expect(env.child('pages')).toBeNull();
    expect(env.child('nothing')).toBeNull();
    expect(env.child('없는필드')).toBeNull();
  });

  it('children 이 배열 항목마다 봉투를 만든다', () => {
    const env = new Envelope({
      pages: [
        { pageNo: 0, text: '가' },
        { pageNo: 1, text: '나' },
      ],
    });
    const pages = env.children('pages');
    expect(pages).toHaveLength(2);
    expect(pages[1]?.get('page_no')).toBe(1);
  });

  it('children 은 배열이 아니면 빈 배열이다', () => {
    const env = new Envelope({ pageCount: 3 });
    expect(env.children('pageCount')).toEqual([]);
    expect(env.children('없는필드')).toEqual([]);
  });
});

describe('입력 검증', () => {
  it.each([
    ['배열', [1, 2, 3]],
    ['null', null],
    ['문자열', '봉투 아님'],
    ['숫자', 3],
  ])('%s 은 봉투가 아니다', (_label, value) => {
    // 봉투가 아닌 것을 감싸면 이후 모든 접근이 이상하게 실패한다. 감싸는 지점에서
    // 막아야 원인이 드러난다.
    expect(() => new Envelope(value as unknown as RawEnvelope)).toThrow(TypeError);
  });

  it('오류 메시지가 무엇을 받았는지 알려준다', () => {
    const error = capture(() => new Envelope([1, 2] as unknown as RawEnvelope));
    expect((error as TypeError).message).toContain('array');
  });
});

describe('asEnvelope', () => {
  it('원문을 감싼다', () => {
    expect(asEnvelope({ pageCount: 3 })).toBeInstanceOf(Envelope);
  });

  it('이미 봉투면 그대로 돌려준다 — 이중 포장은 접근 경로만 늘린다', () => {
    const env = new Envelope({ pageCount: 3 });
    expect(asEnvelope(env)).toBe(env);
  });
});

describe('표현', () => {
  it('schemaVersion 을 꺼낸다', () => {
    expect(new Envelope({ schemaVersion: '1.0' }).schemaVersion).toBe('1.0');
    expect(new Envelope({}).schemaVersion).toBeUndefined();
    // 문자열이 아니면 버전으로 인정하지 않는다.
    expect(new Envelope({ schemaVersion: 1 }).schemaVersion).toBeUndefined();
  });

  it('toString 이 키를 요약해 보여준다', () => {
    const env = new Envelope({ pageCount: 3, schemaVersion: '1.0' });
    expect(String(env)).toBe('Envelope(pageCount, schemaVersion)');
  });

  it('키가 많으면 잘라서 보여준다 — 로그 한 줄이 화면을 덮으면 안 된다', () => {
    const source: RawEnvelope = {};
    for (let i = 0; i < 10; i += 1) source[`k${i}`] = i;
    expect(String(new Envelope(source))).toContain('…');
  });
});
