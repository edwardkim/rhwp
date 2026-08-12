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
 * camelCase → snake_case.
 *
 * ```ts
 * toSnake('pageCount')       // 'page_count'
 * toSnake('sourceA')         // 'source_a'
 * toSnake('irSchemaVersion') // 'ir_schema_version'
 * toSnake('already_snake')   // 'already_snake'
 * ```
 */
export declare function toSnake(name: string): string;
/**
 * snake_case → camelCase (봉투로 되돌려 보낼 때).
 *
 * ```ts
 * toCamel('page_count')  // 'pageCount'
 * toCamel('dry_run')     // 'dryRun'
 * toCamel('alreadyCamel')// 'alreadyCamel'
 * ```
 */
export declare function toCamel(name: string): string;
/** JSON 으로 표현 가능한 값. */
export type JsonValue = string | number | boolean | null | JsonValue[] | {
    [key: string]: JsonValue;
};
/**
 * 중첩 구조 전체의 키를 snake_case 로 변환한다 (배열 내부까지).
 *
 * **값은 건드리지 않는다** — 필드 *이름*만 규약을 따르고, 내용은 봉투 그대로다.
 * 사용자 데이터(예: 한글 누름틀 이름)가 키인 경우도 그대로 둔다.
 */
export declare function snakeKeys<T = unknown>(value: T): T;
/** 중첩 구조 전체의 키를 camelCase 로 되돌린다 (계획서를 보낼 때). */
export declare function camelKeys<T = unknown>(value: T): T;
/**
 * 타입 수준 camelCase → snake_case.
 *
 * 런타임 {@link toSnake} 와 같은 규칙이어야 한다. 생성기가 만든 인터페이스를
 * 변환할 때 타입이 실제 결과를 따라가도록 쓴다.
 */
export type SnakeCase<S extends string> = S extends `${infer Head}${infer Tail}` ? Tail extends Uncapitalize<Tail> ? `${Lowercase<Head>}${SnakeCase<Tail>}` : `${Lowercase<Head>}_${SnakeCase<Uncapitalize<Tail>>}` : S;
/** 타입 수준 snake_case → camelCase. */
export type CamelCase<S extends string> = S extends `${infer Head}_${infer Tail}` ? `${Head}${Capitalize<CamelCase<Tail>>}` : S;
/** 객체 타입의 모든 키를 snake_case 로. */
export type SnakeKeys<T> = {
    [K in keyof T as K extends string ? SnakeCase<K> : K]: T[K];
};
/** 객체 타입의 모든 키를 camelCase 로. */
export type CamelKeys<T> = {
    [K in keyof T as K extends string ? CamelCase<K> : K]: T[K];
};
/** 식별자로 안전한 이름인지. 아니면 따옴표로 감싸야 한다. */
export declare function isSafeIdentifier(name: string): boolean;
/** 생성 코드에서 프로퍼티 이름을 안전하게 표기한다. */
export declare function propertyKey(name: string): string;
