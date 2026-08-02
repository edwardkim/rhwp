/**
 * camelCase ↔ snake_case **기계** 변환.
 *
 * 수기 개명을 금지하는 것이 요점이다. 사람이 이름을 다시 붙이기 시작하면 봉투에
 * 필드가 하나 늘 때마다 바인딩이 뒤처지고, 어느 쪽이 맞는지 아무도 모르게 된다.
 * 규칙이 코드로 고정돼 있어야 새 필드가 자동으로 따라온다 — 그 규칙을 여기서 못 박는다.
 */

import { describe, expect, it } from 'vitest';

import {
  camelKeys,
  isSafeIdentifier,
  propertyKey,
  snakeKeys,
  toCamel,
  toSnake,
} from '../src/naming.js';

/**
 * 실제 봉투에서 쓰이는 키들이다.
 *
 * 표를 크게 유지하는 이유: 변환 규칙은 정규식 두 줄인데, 그 두 줄이 약어·숫자·
 * 단일 단어에서 각각 다르게 동작한다. 대표 사례 몇 개만 두면 규칙을 "고쳤는데
 * 테스트는 통과하는" 구간이 생긴다.
 */
const TO_SNAKE: [string, string][] = [
  ['pageCount', 'page_count'],
  ['changedPages', 'changed_pages'],
  ['schemaVersion', 'schema_version'],
  ['planVersion', 'plan_version'],
  ['sourceA', 'source_a'],
  ['sourceB', 'source_b'],
  ['pageCountMismatch', 'page_count_mismatch'],
  ['irSchemaVersion', 'ir_schema_version'],
  ['dryRun', 'dry_run'],
  ['notFoundEmpty', 'not_found_empty'],
  ['maxDisp', 'max_disp'],
  ['structTextrunPm1', 'struct_textrun_pm1'],
  ['outputFormat', 'output_format'],
  ['filledCount', 'filled_count'],
  ['replacedCount', 'replaced_count'],
  ['matchCount', 'match_count'],
  ['tableCount', 'table_count'],
  ['diffCount', 'diff_count'],
  ['reparseError', 'reparse_error'],
  ['oldText', 'old_text'],
  ['keepStyle', 'keep_style'],
  ['caseSensitive', 'case_sensitive'],
  ['didYouMean', 'did_you_mean'],
  ['nextCall', 'next_call'],
  // 이미 snake 면 그대로 — 왕복 호출이 이름을 갉아먹으면 안 된다.
  ['already_snake', 'already_snake'],
  // 단일 단어.
  ['format', 'format'],
  ['pages', 'pages'],
];

const TO_CAMEL: [string, string][] = [
  ['page_count', 'pageCount'],
  ['dry_run', 'dryRun'],
  ['changed_pages', 'changedPages'],
  ['case_sensitive', 'caseSensitive'],
  ['next_call', 'nextCall'],
  ['plan_version', 'planVersion'],
  ['not_found_empty', 'notFoundEmpty'],
  // 이미 camel 이면 그대로.
  ['alreadyCamel', 'alreadyCamel'],
  ['single', 'single'],
];

describe('toSnake', () => {
  it.each(TO_SNAKE)('%s → %s', (camel, snake) => {
    expect(toSnake(camel)).toBe(snake);
  });

  it('연속 대문자(약어)의 경계를 살린다', () => {
    // 경계를 무시하면 `HTMLPage` 가 `h_t_m_l_page` 가 된다 — 사람이 읽을 수 없는
    // 이름은 생성 코드에서 그대로 굳어 버린다.
    expect(toSnake('HTMLPage')).toBe('html_page');
    expect(toSnake('exportPDF')).toBe('export_pdf');
    expect(toSnake('parseHWPXFile')).toBe('parse_hwpx_file');
  });

  it('숫자를 단어 경계로 오해하지 않는다', () => {
    expect(toSnake('structTextrunPm1')).toBe('struct_textrun_pm1');
    expect(toSnake('page1Count')).toBe('page1_count');
  });

  it('빈 문자열이 안전하다', () => {
    expect(toSnake('')).toBe('');
  });
});

describe('toCamel', () => {
  it.each(TO_CAMEL)('%s → %s', (snake, camel) => {
    expect(toCamel(snake)).toBe(camel);
  });

  it('빈 조각이 있어도 죽지 않는다', () => {
    // 봉투 키가 이렇게 생길 일은 없지만, 계획서를 사람이 손으로 쓸 수 있다.
    // 픽스처 하나 때문에 변환기가 죽으면 원인이 엉뚱한 곳에서 드러난다.
    expect(toCamel('a__b')).toBe('aB');
    expect(toCamel('trailing_')).toBe('trailing');
  });

  it('빈 문자열이 안전하다', () => {
    expect(toCamel('')).toBe('');
  });
});

describe('왕복', () => {
  it('봉투 키는 왕복해도 같다 — 아니면 계획서를 되돌려 보낼 수 없다', () => {
    for (const [, camel] of TO_CAMEL) {
      expect(toCamel(toSnake(camel))).toBe(camel);
    }
    for (const [camel] of TO_SNAKE) {
      // 표의 camel 쪽은 전부 실제 봉투 키다. 하나라도 왕복이 깨지면 그 필드는
      // 계획서로 되돌려 보내는 순간 이름이 달라져 rhwp 가 못 알아본다.
      if (camel.includes('_')) continue; // 이미 snake 인 항목은 대상이 아니다
      expect(toCamel(toSnake(camel))).toBe(camel);
    }
  });

  it('약어는 왕복하지 않는다 — 그래서 봉투 키에 연속 대문자를 쓰지 않는 것이 계약이다', () => {
    // 이 한계를 테스트로 남겨 둔다. 문서에만 적어 두면 언젠가 누군가
    // `HTMLPage` 같은 필드를 봉투에 넣고, 되돌려 보낼 때 조용히 이름이 달라진다.
    expect(toSnake('HTMLPage')).toBe('html_page');
    expect(toCamel('html_page')).toBe('htmlPage');
  });
});

describe('snakeKeys / camelKeys', () => {
  it('중첩과 배열 안까지 훑는다', () => {
    const source = {
      schemaVersion: '1.0',
      changedPages: [0, 1],
      verify: { diffCount: 0, identical: true },
      steps: [{ filledCount: 1, oldText: '값' }],
    };
    const result = snakeKeys<Record<string, unknown>>(source);

    expect(result['schema_version']).toBe('1.0');
    expect(result['changed_pages']).toEqual([0, 1]);
    expect(result['verify']).toEqual({ diff_count: 0, identical: true });
    expect(result['steps']).toEqual([{ filled_count: 1, old_text: '값' }]);
  });

  it('**값은 건드리지 않는다** — 필드 이름만 규약을 따르고 내용은 봉투 그대로다', () => {
    const source = {
      oldText: 'camelCase 라는 값',
      data: { 회사명: '테스트', 'A_B': 'x' },
    };
    const result = snakeKeys<Record<string, unknown>>(source);

    expect(result['old_text']).toBe('camelCase 라는 값');
    // 한글 키는 **사용자 데이터**(누름틀 이름)다 — 변환 대상이 아니다. 여기를
    // 건드리면 사용자가 붙인 이름이 조용히 달라지고 필드를 못 찾게 된다.
    expect(result['data']).toEqual({ 회사명: '테스트', a_b: 'x' });
  });

  it('원본을 고치지 않는다 — 봉투는 여러 곳에서 공유될 수 있다', () => {
    const source = { pageCount: 3, verify: { diffCount: 0 } };
    snakeKeys(source);
    expect(source).toEqual({ pageCount: 3, verify: { diffCount: 0 } });
  });

  it('null·원시값·빈 구조를 통과시킨다', () => {
    expect(snakeKeys(null)).toBeNull();
    expect(snakeKeys(3)).toBe(3);
    expect(snakeKeys('그대로')).toBe('그대로');
    expect(snakeKeys({})).toEqual({});
    expect(snakeKeys([])).toEqual([]);
    // null 값은 "검증 안 함" 같은 판정을 나르므로 사라지면 안 된다.
    expect(snakeKeys<Record<string, unknown>>({ changedPages: null })).toEqual({
      changed_pages: null,
    });
  });

  it('camelKeys 는 내보낼 계획서를 되돌린다', () => {
    const plan = {
      plan_version: '1.0',
      dry_run: true,
      steps: [{ case_sensitive: false, action: 'replace_text' }],
    };
    const result = camelKeys<Record<string, unknown>>(plan);

    expect(result['planVersion']).toBe('1.0');
    expect(result['dryRun']).toBe(true);
    // action 값은 계약상 snake 다 — 값이니까 그대로 남아야 한다.
    expect(result['steps']).toEqual([{ caseSensitive: false, action: 'replace_text' }]);
  });
});

describe('식별자 안전성', () => {
  it.each([['pageCount'], ['_private'], ['$dollar'], ['a1'], ['A_B']])(
    '%s 는 식별자로 안전하다',
    (name) => {
      expect(isSafeIdentifier(name)).toBe(true);
      expect(propertyKey(name)).toBe(name);
    },
  );

  it.each([['회사명'], ['a-b'], ['0start'], [''], ['with space'], ['dotted.key']])(
    '%s 는 따옴표가 필요하다',
    (name) => {
      expect(isSafeIdentifier(name)).toBe(false);
      expect(propertyKey(name)).toBe(JSON.stringify(name));
    },
  );

  it.each([['class'], ['function'], ['new'], ['typeof'], ['default']])(
    '예약어 %s 는 그대로 쓸 수 없다',
    (word) => {
      // 생성 코드에 예약어가 맨몸으로 들어가면 파일 전체가 파싱 불가가 된다.
      expect(isSafeIdentifier(word)).toBe(false);
      expect(propertyKey(word)).toBe(`"${word}"`);
    },
  );
});
