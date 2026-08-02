/**
 * 봉투 키(camelCase) ↔ 속성(snake_case) **기계** 변환.
 *
 * 수기 개명을 금지하는 것이 이 모듈의 요점이다 (`bindings_foundation.md` §3).
 * 사람이 이름을 다시 붙이기 시작하면 봉투에 필드가 하나 늘 때마다 바인딩이 뒤처지고,
 * 어느 쪽이 맞는지 알 수 없게 된다. 규칙을 코드로 고정하면 새 필드는 자동으로 따라온다.
 *
 * TypeScript 에서는 여기에 더해 **타입 수준 변환**도 제공한다 — 생성기가 만든
 * 인터페이스와 런타임 변환이 같은 규칙을 쓰는지 컴파일러가 검사할 수 있다.
 *
 * @packageDocumentation
 */

/**
 * 연속 대문자(약어) 경계를 살린다: `HTMLPage` → `html_page`.
 *
 * 앞 그룹이 `[A-Z]+`(가변 길이)이면 `AAAA…Aa` 같은 입력에서 역추적이 다항으로
 * 늘어난다(ReDoS). 고정 길이 `[A-Z]` 로 바꿔도 **결과가 같다** — 경계는 "대문자
 * 하나 뒤에 대문자+소문자"라는 국소 조건이고, 앞쪽 대문자를 얼마나 먹는지는
 * 삽입 위치를 바꾸지 않기 때문이다. (`HTMLPage`·`ABCd`·`AAAa` 로 확인.)
 */
const ACRONYM_BOUNDARY = /([A-Z])([A-Z][a-z])/g;
/** 일반 단어 경계: `pageCount` → `page_count`. */
const WORD_BOUNDARY = /([a-z0-9])([A-Z])/g;

/**
 * camelCase → snake_case.
 *
 * ```ts
 * toSnake('pageCount')       // 'page_count'
 * toSnake('sourceA')         // 'source_a'
 * toSnake('irSchemaVersion') // 'ir_schema_version'
 * toSnake('already_snake')   // 'already_snake'
 * ```
 */
export function toSnake(name: string): string {
  if (!name) return name;
  return name
    .replace(ACRONYM_BOUNDARY, '$1_$2')
    .replace(WORD_BOUNDARY, '$1_$2')
    .toLowerCase();
}

/**
 * snake_case → camelCase (봉투로 되돌려 보낼 때).
 *
 * ```ts
 * toCamel('page_count')  // 'pageCount'
 * toCamel('dry_run')     // 'dryRun'
 * toCamel('alreadyCamel')// 'alreadyCamel'
 * ```
 */
export function toCamel(name: string): string {
  if (!name || !name.includes('_')) return name;
  const [head, ...rest] = name.split('_');
  return (
    (head ?? '') +
    rest
      .filter((part) => part.length > 0)
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join('')
  );
}

/** JSON 으로 표현 가능한 값. */
export type JsonValue =
  | string
  | number
  | boolean
  | null
  | JsonValue[]
  | { [key: string]: JsonValue };

/**
 * 중첩 구조 전체의 키를 snake_case 로 변환한다 (배열 내부까지).
 *
 * **값은 건드리지 않는다** — 필드 *이름*만 규약을 따르고, 내용은 봉투 그대로다.
 * 사용자 데이터(예: 한글 누름틀 이름)가 키인 경우도 그대로 둔다.
 */
export function snakeKeys<T = unknown>(value: T): T {
  return walk(value, toSnake) as T;
}

/** 중첩 구조 전체의 키를 camelCase 로 되돌린다 (계획서를 보낼 때). */
export function camelKeys<T = unknown>(value: T): T {
  return walk(value, toCamel) as T;
}

function walk(value: unknown, transform: (key: string) => string): unknown {
  if (Array.isArray(value)) {
    return value.map((item) => walk(item, transform));
  }
  if (value !== null && typeof value === 'object') {
    const out: Record<string, unknown> = {};
    for (const [key, item] of Object.entries(value as Record<string, unknown>)) {
      out[transform(key)] = walk(item, transform);
    }
    return out;
  }
  return value;
}

/**
 * 타입 수준 camelCase → snake_case.
 *
 * 런타임 {@link toSnake} 와 같은 규칙이어야 한다. 생성기가 만든 인터페이스를
 * 변환할 때 타입이 실제 결과를 따라가도록 쓴다.
 */
export type SnakeCase<S extends string> = S extends `${infer Head}${infer Tail}`
  ? Tail extends Uncapitalize<Tail>
    ? `${Lowercase<Head>}${SnakeCase<Tail>}`
    : `${Lowercase<Head>}_${SnakeCase<Uncapitalize<Tail>>}`
  : S;

/** 타입 수준 snake_case → camelCase. */
export type CamelCase<S extends string> = S extends `${infer Head}_${infer Tail}`
  ? `${Head}${Capitalize<CamelCase<Tail>>}`
  : S;

/** 객체 타입의 모든 키를 snake_case 로. */
export type SnakeKeys<T> = {
  [K in keyof T as K extends string ? SnakeCase<K> : K]: T[K];
};

/** 객체 타입의 모든 키를 camelCase 로. */
export type CamelKeys<T> = {
  [K in keyof T as K extends string ? CamelCase<K> : K]: T[K];
};

/** 예약어와 충돌하면 뒤에 밑줄을 붙인다 (생성기가 쓴다). */
const RESERVED = new Set([
  'break', 'case', 'catch', 'class', 'const', 'continue', 'debugger', 'default',
  'delete', 'do', 'else', 'enum', 'export', 'extends', 'false', 'finally', 'for',
  'function', 'if', 'import', 'in', 'instanceof', 'new', 'null', 'return', 'super',
  'switch', 'this', 'throw', 'true', 'try', 'typeof', 'var', 'void', 'while', 'with',
]);

/** 식별자로 안전한 이름인지. 아니면 따옴표로 감싸야 한다. */
export function isSafeIdentifier(name: string): boolean {
  return /^[A-Za-z_$][A-Za-z0-9_$]*$/.test(name) && !RESERVED.has(name);
}

/** 생성 코드에서 프로퍼티 이름을 안전하게 표기한다. */
export function propertyKey(name: string): string {
  return isSafeIdentifier(name) ? name : JSON.stringify(name);
}
