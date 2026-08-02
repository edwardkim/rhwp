/**
 * 재현 명령(`RhwpError.command`)의 셸 안전성.
 *
 * 예외에 담기는 `command` 는 "버그 리포트에 그대로 붙여넣으면 재현된다"가 계약이다.
 * 인용이 새면 그 약속이 깨질 뿐 아니라, 최악의 경우 **붙여넣은 사람이 의도하지 않은
 * 명령을 실행**하게 된다. 그래서 별도 파일로 떼어 고정한다.
 *
 * CodeQL `js/incomplete-sanitization` 이 실제로 잡은 결함의 회귀 테스트이기도 하다 —
 * 따옴표만 이스케이프하고 역슬래시를 빠뜨렸던 판이 있었다.
 */

import { describe, expect, it } from 'vitest';

import { RhwpError } from '../src/errors.js';

/** 인자 하나짜리 재현 명령. */
function quoted(arg: string): string {
  return new RhwpError('x', { argv: [arg] }).command;
}

describe('quoteArgument — 특수문자가 없으면 그대로 둔다', () => {
  it('평범한 토큰은 감싸지 않는다', () => {
    expect(quoted('plain')).toBe('plain');
    expect(quoted('a/b/c.hwp')).toBe('a/b/c.hwp');
    expect(quoted('--json')).toBe('--json');
  });

  it('빈 문자열은 감싼다 — 감싸지 않으면 인자가 통째로 사라진다', () => {
    expect(quoted('')).toBe('""');
  });
});

describe('quoteArgument — 감싸야 하는 경우', () => {
  it('공백', () => {
    expect(quoted('공백 있음')).toBe('"공백 있음"');
  });

  it('작은따옴표', () => {
    expect(quoted("it's")).toBe(`"it's"`);
  });

  it('큰따옴표는 이스케이프한다', () => {
    expect(quoted('say"hi')).toBe('"say\\"hi"');
  });
});

describe('quoteArgument — 역슬래시 (CodeQL 이 잡은 실제 결함)', () => {
  it('역슬래시만 있어도 감싼다', () => {
    // 이전 판은 `[\s"']` 만 봐서 `back\slash` 를 날것으로 내보냈다.
    expect(quoted('back\\slash')).toBe('"back\\\\slash"');
  });

  it('끝 역슬래시가 닫는 따옴표를 먹지 않는다', () => {
    // 따옴표만 이스케이프하면 `C:\경로\` → `"C:\경로\"` 가 되어
    // 닫는 따옴표가 이스케이프돼 버린다.
    expect(quoted('C:\\경로\\')).toBe('"C:\\\\경로\\\\"');
  });

  it('역슬래시로 끝나는 인자가 다음 인자를 삼키지 않는다', () => {
    const joined = new RhwpError('x', { argv: ['C:\\dir\\', 'next'] }).command;
    expect(joined).toBe('"C:\\\\dir\\\\" next');
  });

  it('이스케이프 순서가 뒤바뀌지 않는다 (이중 이스케이프 금지)', () => {
    // 따옴표를 먼저 처리하면 그때 넣은 역슬래시를 역슬래시 단계가 또 이스케이프한다.
    // 반드시 역슬래시 → 따옴표 순서여야 한다.
    expect(quoted('a"b\\c')).toBe('"a\\"b\\\\c"');
  });

  it('연속 역슬래시도 각각 이스케이프한다', () => {
    expect(quoted('a\\\\b')).toBe('"a\\\\\\\\b"');
  });
});

describe('command — 인자가 없을 때', () => {
  it('argv 가 없으면 빈 문자열', () => {
    expect(new RhwpError('x').command).toBe('');
  });

  it('argv 가 빈 배열이면 빈 문자열', () => {
    expect(new RhwpError('x', { argv: [] }).command).toBe('');
  });
});
