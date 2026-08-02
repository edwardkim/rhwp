/**
 * 종료 코드 → 예외 매핑 계약.
 *
 * 이 파일이 지키는 핵심 구분: **판정 실패는 고장이 아니다.** exit 3/4 를 기본으로
 * 예외로 만들면 호출자가 `try/catch` 로 "고장"처럼 다루게 되고, 정작 봉투에 담긴
 * 판정 근거(diffCount·pages)를 읽지 않는다. 그 규약이 무너졌는지 여기서 잡는다.
 */

import { describe, expect, it } from 'vitest';

import {
  BinaryNotFoundError,
  EXIT_OK,
  EXIT_RUNTIME,
  EXIT_USAGE,
  EXIT_VERIFY,
  EXIT_VERIFY_PAGES,
  ProtocolError,
  RhwpError,
  RhwpRuntimeError,
  RhwpTimeoutError,
  SessionClosedError,
  UsageError,
  VerdictFailed,
  isKnownExitCode,
  raiseForExit,
} from '../src/errors.js';

/**
 * 예외를 **값으로** 잡는다.
 *
 * `expect(...).toThrow()` 만 쓰면 던져진 객체의 필드를 못 본다. 이 바인딩의 계약은
 * "무엇이 던져졌나"가 아니라 "무엇을 실어 던졌나"에 있으므로 객체가 필요하다.
 */
function capture(run: () => void): unknown {
  try {
    run();
  } catch (error) {
    return error;
  }
  throw new Error('예외가 발생해야 하는데 정상 반환했습니다');
}

describe('raiseForExit — 종료 코드 사전', () => {
  it('성공(0)은 아무것도 던지지 않는다', () => {
    expect(() => raiseForExit(EXIT_OK, { argv: ['rhwp', 'info', 'a.hwp'] })).not.toThrow();
  });

  it('exit 2 는 UsageError — 호출을 조립한 우리 쪽 버그다', () => {
    const error = capture(() =>
      raiseForExit(EXIT_USAGE, { argv: ['rhwp', 'expot-svg'], stderr: '오류: 알 수 없는 명령' }),
    );
    expect(error).toBeInstanceOf(UsageError);
    expect((error as UsageError).exitCode).toBe(EXIT_USAGE);
  });

  it('exit 1 은 RhwpRuntimeError — 인자를 고쳐도 해결되지 않는다', () => {
    const error = capture(() =>
      raiseForExit(EXIT_RUNTIME, { argv: ['rhwp', 'info', '없음.hwp'], stderr: '오류: 읽기 실패' }),
    );
    expect(error).toBeInstanceOf(RhwpRuntimeError);
    expect(error).not.toBeInstanceOf(UsageError);
  });

  it.each([[EXIT_VERIFY], [EXIT_VERIFY_PAGES]])(
    'exit %i 는 기본적으로 예외가 아니다 — 판정은 반환값으로 다룬다',
    (code) => {
      // 도구는 정상 동작했다. 실패한 것은 *문서에 대한 단언*이다. 이걸 예외로
      // 만들면 호출자가 봉투의 판정 필드를 읽지 않고 catch 로 넘겨 버린다.
      expect(() =>
        raiseForExit(code, { argv: ['rhwp', 'export-hwpx', 'a.hwp', '--verify'] }),
      ).not.toThrow();
    },
  );

  it.each([[EXIT_VERIFY], [EXIT_VERIFY_PAGES]])(
    'exit %i 는 throwOnVerdict 를 명시했을 때만 VerdictFailed 다',
    (code) => {
      const envelope = { verify: { identical: false, diffCount: 7 } };
      const error = capture(() =>
        raiseForExit(code, {
          argv: ['rhwp', 'export-hwpx', 'a.hwp', '--verify'],
          envelope,
          throwOnVerdict: true,
        }),
      );
      expect(error).toBeInstanceOf(VerdictFailed);
      const verdict = error as VerdictFailed;
      // 판정 근거가 예외에 실려 있어야 한다 — 없으면 왜 실패했는지 알 수 없다.
      expect(verdict.envelope).toEqual(envelope);
      expect(verdict.isPageCountMismatch).toBe(code === EXIT_VERIFY_PAGES);
    },
  );

  it('페이지 수 불일치(4)와 일반 검증 실패(3)는 메시지가 다르다', () => {
    const pages = capture(() => raiseForExit(EXIT_VERIFY_PAGES, { throwOnVerdict: true }));
    const verify = capture(() => raiseForExit(EXIT_VERIFY, { throwOnVerdict: true }));
    expect((pages as VerdictFailed).message).toContain('페이지 수');
    expect((verify as VerdictFailed).message).not.toContain('페이지 수');
  });

  it('사전에 없는 코드는 조용히 통과시키지 않는다', () => {
    // 새 종료 코드가 생겼는데 바인딩이 모르고 통과시키면, 실패한 작업이 성공으로
    // 보고된다 — 잘못된 결과가 아무 신호 없이 파이프라인 아래로 흘러간다.
    const error = capture(() => raiseForExit(42, { argv: ['rhwp', 'info', 'a.hwp'] }));
    expect(error).toBeInstanceOf(RhwpRuntimeError);
    expect((error as RhwpRuntimeError).message).toContain('42');
    expect((error as RhwpRuntimeError).exitCode).toBe(42);
  });

  it('맥락(argv·stderr·cause)을 예외에 그대로 실어 보낸다', () => {
    const cause = new Error('원인');
    const error = capture(() =>
      raiseForExit(EXIT_RUNTIME, { argv: ['rhwp', 'info'], stderr: '진단', cause }),
    );
    const runtime = error as RhwpRuntimeError;
    expect(runtime.argv).toEqual(['rhwp', 'info']);
    expect(runtime.stderr).toBe('진단');
    expect(runtime.cause).toBe(cause);
  });
});

describe('UsageError — 교정 단서', () => {
  it('stderr 의 `힌트:` 줄에서 did-you-mean 제안을 뽑는다', () => {
    const error = capture(() =>
      raiseForExit(EXIT_USAGE, {
        stderr: "오류: 알 수 없는 명령입니다\n힌트: 가장 가까운 명령은 'export-svg' 입니다",
      }),
    );
    expect((error as UsageError).suggestion).toBe("가장 가까운 명령은 'export-svg' 입니다");
  });

  it('힌트가 없으면 undefined — 없는 제안을 지어내지 않는다', () => {
    const error = capture(() =>
      raiseForExit(EXIT_USAGE, { stderr: '오류: 인자가 필요합니다' }),
    );
    expect((error as UsageError).suggestion).toBeUndefined();
  });

  it('힌트가 여러 줄이면 마지막 것을 쓴다 — 가장 최근 진단이 가장 구체적이다', () => {
    const error = capture(() =>
      raiseForExit(EXIT_USAGE, { stderr: '힌트: 오래된 제안\n오류: ...\n힌트: 최신 제안' }),
    );
    expect((error as UsageError).suggestion).toBe('최신 제안');
  });

  it('봉투의 nextCall 을 그대로 꺼낸다 — 기계가 따라할 수 있는 교정이다', () => {
    const nextCall = {
      name: 'hwp_export_svg',
      arguments: { path: 'a.hwp' },
      why: '명령 이름을 잘못 썼습니다',
    };
    const error = capture(() => raiseForExit(EXIT_USAGE, { envelope: { nextCall } }));
    expect((error as UsageError).nextCall).toEqual(nextCall);
  });

  it('nextCall 이 없거나 모양이 아니면 undefined', () => {
    const none = capture(() => raiseForExit(EXIT_USAGE, { envelope: { error: '...' } }));
    expect((none as UsageError).nextCall).toBeUndefined();

    // `name` 없는 객체는 교정 호출이 아니다. 모양을 안 보고 넘기면 호출자가
    // `undefined.name` 으로 죽는다.
    const malformed = capture(() => raiseForExit(EXIT_USAGE, { envelope: { nextCall: 'export-svg' } }));
    expect((malformed as UsageError).nextCall).toBeUndefined();
  });
});

describe('RhwpError — 재현과 진단', () => {
  it('command 가 공백 있는 인자를 따옴표로 감싼다', () => {
    const error = new RhwpError('실패', {
      argv: ['rhwp', 'info', '공백 있는 파일.hwp', '--json'],
    });
    // 버그 리포트에 그대로 붙여넣을 수 있어야 한다. 따옴표가 없으면 인자가
    // 둘로 쪼개진 다른 명령이 되고, 재현이 안 되면 보고는 무용지물이다.
    expect(error.command).toContain('rhwp info');
    expect(error.command).toContain('"공백 있는 파일.hwp"');
  });

  it('command 가 빈 인자와 따옴표 있는 인자도 잃지 않는다', () => {
    const error = new RhwpError('실패', { argv: ['rhwp', 'search', '', 'say "hi"'] });
    // 빈 문자열이 그냥 사라지면 인자 개수가 달라진 명령이 된다.
    expect(error.command).toBe('rhwp search "" "say \\"hi\\""');
  });

  it('argv 가 없으면 command 는 빈 문자열이다', () => {
    expect(new RhwpError('실패').command).toBe('');
  });

  it('lastDiagnostic 이 stderr 의 마지막 유의미한 줄을 고른다', () => {
    const error = new RhwpError('실패', {
      stderr: '진행: 1/3\n진행: 2/3\n오류: 진짜 사유는 여기\n\n  \n',
    });
    // 마지막 줄이 가장 구체적이다. 빈 줄에 걸려 ''를 돌려주면 진단이 사라진다.
    expect(error.lastDiagnostic).toBe('오류: 진짜 사유는 여기');
  });

  it('stderr 가 없으면 빈 문자열 — undefined 를 흘리지 않는다', () => {
    const error = new RhwpError('실패');
    expect(error.stderr).toBe('');
    expect(error.lastDiagnostic).toBe('');
  });

  it('toString 이 이름·종료 코드·마지막 진단을 한 줄에 담는다', () => {
    const error = capture(() =>
      raiseForExit(EXIT_RUNTIME, {
        argv: ['rhwp', 'info', 'a.hwp'],
        stderr: '첫 줄\n오류: 진짜 사유는 여기',
      }),
    );
    const text = String(error);
    expect(text).toContain('RhwpRuntimeError');
    expect(text).toContain('exit 1');
    expect(text).toContain('진짜 사유는 여기');
  });

  it('argv·envelope 를 사본으로 보관한다 — 호출자가 나중에 고쳐도 증거가 안 바뀐다', () => {
    const argv = ['rhwp', 'info'];
    const envelope: Record<string, unknown> = { pageCount: 3 };
    const error = new RhwpError('실패', { argv, envelope });

    argv.push('오염');
    envelope['pageCount'] = 99;

    expect(error.argv).toEqual(['rhwp', 'info']);
    expect(error.envelope).toEqual({ pageCount: 3 });
  });
});

describe('예외 계층 — instanceof 가 실제로 동작해야 한다', () => {
  // 프로토타입 복원이 없으면 트랜스파일 이후 `instanceof` 가 조용히 거짓이 된다.
  // 그러면 호출자의 `catch (e) { if (e instanceof UsageError) ... }` 분기가 통째로
  // 죽는데, 예외는 여전히 던져지므로 아무도 눈치채지 못한다.
  const subclasses: [string, RhwpError][] = [
    ['BinaryNotFoundError', new BinaryNotFoundError('x')],
    ['UsageError', new UsageError('x')],
    ['RhwpRuntimeError', new RhwpRuntimeError('x')],
    ['VerdictFailed', new VerdictFailed('x')],
    ['ProtocolError', new ProtocolError('x')],
    ['SessionClosedError', new SessionClosedError('x')],
    ['RhwpTimeoutError', new RhwpTimeoutError('x')],
  ];

  it.each(subclasses)('%s 는 Error·RhwpError 이자 자기 자신이다', (name, error) => {
    expect(error).toBeInstanceOf(Error);
    expect(error).toBeInstanceOf(RhwpError);
    expect(error.name).toBe(name);
  });

  it('형제 예외끼리는 서로 instanceof 가 아니다', () => {
    expect(new UsageError('x')).not.toBeInstanceOf(RhwpRuntimeError);
    expect(new VerdictFailed('x')).not.toBeInstanceOf(ProtocolError);
  });

  it('스택 트레이스가 남는다 — 진단 없는 예외는 절반만 예외다', () => {
    expect(new UsageError('x').stack).toBeTruthy();
  });
});

describe('isKnownExitCode', () => {
  it.each([[EXIT_OK], [EXIT_RUNTIME], [EXIT_USAGE], [EXIT_VERIFY], [EXIT_VERIFY_PAGES]])(
    '%i 는 사전에 있는 코드다',
    (code) => {
      expect(isKnownExitCode(code)).toBe(true);
    },
  );

  it.each([[5], [42], [-1], [127]])('%i 는 사전에 없다', (code) => {
    expect(isKnownExitCode(code)).toBe(false);
  });
});
