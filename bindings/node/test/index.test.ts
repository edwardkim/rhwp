/**
 * 공개 표면 가드 — `src/index.ts` 가 무엇을 내보내는가.
 *
 * 이 파일이 지키는 것은 기능이 아니라 **경계**다. 새 모듈을 만들고 재수출을 잊으면
 * 소비자에게는 그 기능이 존재하지 않는다. 그런 누락은 컴파일도 테스트도 통과하고,
 * 누군가 `@rhwp/node` 에서 import 하다 실패하고 나서야 드러난다 — 그때는 이미
 * 패키지가 배포된 뒤다.
 *
 * 그래서 목록을 손으로 적지 않는다. **각 모듈의 실제 런타임 export 와 대조**한다.
 * 손으로 적은 목록은 그 자체로 또 하나의 뒤처지는 사본이 된다.
 *
 * 생성 파일(`ir.ts`·`envelopes.ts`)은 대상이 아니다 — 타입만 내보내므로 런타임에
 * 대조할 값이 없다(`export type * from` 은 컴파일 때 사라진다).
 */

import { describe, expect, it } from 'vitest';

import * as binary from '../src/binary.js';
import * as browser from '../src/browser.js';
import * as commands from '../src/commands.js';
import * as envelope from '../src/envelope.js';
import * as errors from '../src/errors.js';
import * as index from '../src/index.js';
import * as naming from '../src/naming.js';
import * as plan from '../src/plan.js';
import * as process_ from '../src/process.js';
import * as schema from '../src/schema.js';
import * as session from '../src/session.js';

/** 네임스페이스 객체를 이름으로 훑기 위한 얇은 뷰. */
function surfaceOf(namespace: object): Record<string, unknown> {
  return namespace as unknown as Record<string, unknown>;
}

/** 재수출 대상 모듈 — 파일 이름을 함께 들고 다녀야 실패 메시지가 쓸모 있다. */
const MODULES: [string, Record<string, unknown>][] = [
  ['binary.ts', surfaceOf(binary)],
  ['errors.ts', surfaceOf(errors)],
  ['envelope.ts', surfaceOf(envelope)],
  ['naming.ts', surfaceOf(naming)],
  ['process.ts', surfaceOf(process_)],
  ['commands.ts', surfaceOf(commands)],
  ['session.ts', surfaceOf(session)],
  ['plan.ts', surfaceOf(plan)],
  ['schema.ts', surfaceOf(schema)],
  ['browser.ts', surfaceOf(browser)],
];

/** `index.ts` 자신이 정의하는 값 — 어느 모듈에서도 오지 않는다. */
const META = ['VERSION', 'SUPPORTED_SCHEMA_VERSION'];

describe('index.ts — 재수출 누락 없음', () => {
  it('각 모듈의 값 export 를 하나도 빠뜨리지 않는다', () => {
    const surface = surfaceOf(index);
    const missing: string[] = [];
    const rebound: string[] = [];

    for (const [file, module] of MODULES) {
      for (const name of Object.keys(module)) {
        if (!(name in surface)) {
          missing.push(`${file}: ${name}`);
          continue;
        }
        // 같은 이름이라도 다른 객체를 내보내면 `instanceof` 가 깨진다. 예외 계층을
        // 쓰는 이 패키지에서는 그게 곧 "catch 가 안 잡힌다"는 뜻이다.
        if (surface[name] !== module[name]) rebound.push(`${file}: ${name}`);
      }
    }

    expect(
      missing,
      `재수출 누락 — src/index.ts 의 해당 모듈 블록에 추가하세요:\n  ${missing.join('\n  ')}`,
    ).toHaveLength(0);
    expect(rebound, `다른 객체로 재수출됨:\n  ${rebound.join('\n  ')}`).toHaveLength(0);
  });

  it('index 가 내보내는 값은 모두 출처가 있다', () => {
    // 위 단언의 반대 방향이다. 새 모듈이 생겼는데 이 목록에 넣지 않으면 그 모듈은
    // 영영 대조되지 않는다 — 여기서 미리 알려 준다.
    const known = new Set(MODULES.flatMap(([, module]) => Object.keys(module)));
    const orphans = Object.keys(index).filter(
      (name) => !META.includes(name) && !known.has(name),
    );

    expect(
      orphans,
      `출처 불명 export (MODULES 목록에 모듈을 추가하세요): ${orphans.join(', ')}`,
    ).toHaveLength(0);
  });

  it('3개 층과 스키마·브라우저 축의 진입점이 모두 서 있다', () => {
    // 위의 기계적 대조가 통과해도, 층 하나가 통째로 index 에서 빠지면(모듈 자체를
    // 안 실었다면) 아무도 눈치채지 못한다. 문서가 약속한 진입점을 못 박는다.
    for (const entry of [
      'info', // 1층 무상태
      'search',
      'openDocument', // 2층 세션
      'Session',
      'Document',
      'Plan', // 3층 계획
      'runPlan',
      'irSchema', // IR 스키마
      'IrSchema',
      'createBrowserClient', // 환경 독립
      'createNodeClient',
      'Envelope', // 공통 봉투
      'findBinary',
    ]) {
      expect(surfaceOf(index)[entry], `${entry} 이(가) 공개 표면에 없다`).toBeDefined();
    }
  });
});

describe('index.ts — 메타 상수', () => {
  it('패키지 버전과 지원 스키마 버전을 노출한다', () => {
    // 소비자가 런타임에 호환성을 판별할 유일한 근거다. 없으면 "왜 안 되는지"를
    // package.json 을 열어 추측하게 된다.
    expect(index.VERSION).toMatch(/^\d+\.\d+\.\d+/);
    expect(index.SUPPORTED_SCHEMA_VERSION).toMatch(/^\d+\.\d+$/);
  });
});

describe('index.ts — 예외 계층', () => {
  /** 모든 하위 예외는 RhwpError 하나로 잡혀야 한다. */
  const SUBCLASSES: [string, new (message: string) => errors.RhwpError][] = [
    ['BinaryNotFoundError', index.BinaryNotFoundError],
    ['UsageError', index.UsageError],
    ['RhwpRuntimeError', index.RhwpRuntimeError],
    ['VerdictFailed', index.VerdictFailed],
    ['ProtocolError', index.ProtocolError],
    ['SessionClosedError', index.SessionClosedError],
    ['RhwpTimeoutError', index.RhwpTimeoutError],
  ];

  it('모든 하위 예외가 RhwpError 이자 Error 다', () => {
    // 호출자가 `catch (e) { if (e instanceof RhwpError) ... }` 하나로 이 패키지의
    // 실패를 전부 구분할 수 있어야 한다. 계층이 끊기면 그 관용구가 조용히 새고,
    // 우리 예외가 "알 수 없는 오류"로 흘러 나간다.
    for (const [name, Subclass] of SUBCLASSES) {
      const error = new Subclass('메시지');
      expect(error, name).toBeInstanceOf(index.RhwpError);
      expect(error, name).toBeInstanceOf(Error);
      // 트랜스파일 이후에도 이름이 남아야 로그만 보고 종류를 안다.
      expect(error.name).toBe(name);
      expect(String(error)).toContain('메시지');
    }
  });

  it('형제 예외끼리는 서로가 아니다', () => {
    // 계층만 맞고 구분이 안 되면, 재시도해야 할 런타임 실패와 인자를 고쳐야 할
    // 사용법 오류를 같은 분기가 삼킨다.
    expect(new index.UsageError('x')).not.toBeInstanceOf(index.ProtocolError);
    expect(new index.ProtocolError('x')).not.toBeInstanceOf(index.UsageError);
    expect(new index.RhwpError('x')).not.toBeInstanceOf(index.SessionClosedError);
  });
});

describe('index.ts — 종료 코드 상수', () => {
  it('0~4 의 정수이고 서로 겹치지 않는다', () => {
    const codes = [
      index.EXIT_OK,
      index.EXIT_RUNTIME,
      index.EXIT_USAGE,
      index.EXIT_VERIFY,
      index.EXIT_VERIFY_PAGES,
    ];

    expect(codes).toEqual([0, 1, 2, 3, 4]);
    expect(new Set(codes).size).toBe(codes.length);
    expect(codes.every((code) => Number.isInteger(code) && code >= 0 && code <= 4)).toBe(true);
  });

  it('isKnownExitCode 가 사전에 있는 코드만 참으로 답한다', () => {
    // 새 종료 코드가 생겼는데 바인딩이 모르고 통과시키면, 실패한 작업이 성공으로
    // 보고된다. 모르는 코드는 모른다고 답해야 그 위층이 시끄럽게 실패한다.
    for (const code of [0, 1, 2, 3, 4]) expect(index.isKnownExitCode(code)).toBe(true);
    for (const code of [5, 42, -1, 127]) expect(index.isKnownExitCode(code)).toBe(false);
  });
});
