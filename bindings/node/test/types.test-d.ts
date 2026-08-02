/**
 * 타입 수준 검증 — 컴파일러가 계약을 강제하는지 본다.
 *
 * 런타임 테스트가 "값이 맞나"를 보는 동안 이 파일은 **"틀린 코드가 컴파일되지 않나"**
 * 를 본다. 이 바인딩에서 그 구분이 중요한 이유는, 봉투가 `null`(검증 안 함)과
 * `false`(검증 실패), `undefined`(모름)와 `[]`(없음)을 구분하는 계약을 갖기 때문이다.
 * 타입이 그 구분을 뭉개면 런타임 테스트는 통과하는데 사용자 코드가 조용히 틀린다.
 *
 * 파일 이름이 `*.test-d.ts` 인 것은 우연이 아니다 — vitest 의 실행 대상 글로브
 * (`*.test.ts`)에 걸리지 않으므로 이 파일은 **`tsc` 만이 검사한다**. 실행할 것이 없는
 * 검사를 러너에 얹으면 "테스트 없음"으로 실패한다.
 */

import { expectTypeOf } from 'vitest';

import {
  batch,
  fillFields,
  info,
  search,
  setCell,
  type BatchRecord,
  type BinaryNotFoundError,
  type CamelCase,
  type CommandOptions,
  type EditOptions,
  type Envelope,
  type ProtocolError,
  type RawEnvelope,
  type RhwpError,
  type RhwpRuntimeError,
  type RhwpTimeoutError,
  type SearchOptions,
  type SessionClosedError,
  type SetCellOptions,
  type SnakeCase,
  type UsageError,
  type VerdictFailed,
  type VerifyReport,
} from '../src/index.js';

// ── 봉투 제네릭 ───────────────────────────────────────────────────────────

/** 생성기가 `capabilities` 에서 뽑아 낼 모양의 축소판. */
interface InfoShape extends RawEnvelope {
  readonly pageCount: number;
  readonly format: string;
  readonly fonts: readonly string[];
}

declare const typedEnvelope: Envelope<InfoShape>;

// 타입 파라미터를 주면 `.raw` 가 그 모양으로 좁혀진다. 좁혀지지 않으면 생성 타입을
// 만드는 노력 전체가 헛수고가 된다 — 사용자는 결국 `as` 로 캐스팅하게 된다.
expectTypeOf(typedEnvelope.raw).toEqualTypeOf<InfoShape>();
expectTypeOf(typedEnvelope.raw.pageCount).toEqualTypeOf<number>();
expectTypeOf(typedEnvelope.raw.fonts).toEqualTypeOf<readonly string[]>();

declare const plainEnvelope: Envelope;

// 타입 파라미터가 없으면 안전한 상한으로 남는다. `any` 로 떨어지면 오타가 통과한다.
expectTypeOf(plainEnvelope.raw).toEqualTypeOf<RawEnvelope>();
expectTypeOf(plainEnvelope.get('pageCount')).toEqualTypeOf<unknown>();
expectTypeOf(plainEnvelope.get<number>('pageCount')).toEqualTypeOf<number>();

// ── null 계약을 타입으로 고정한다 ─────────────────────────────────────────

// `null`(검증 안 함)이 타입에서 사라지면 호출자는 `verify.identical` 을 바로 읽게 되고,
// 검증하지 않은 저장이 "통과"로 읽힌다.
expectTypeOf(plainEnvelope.verify).toEqualTypeOf<VerifyReport | null>();

// `null`(어느 쪽이 바뀌었는지 모름)과 `[]`(바뀐 쪽 없음)는 다른 결론이다.
expectTypeOf(plainEnvelope.changedPages).toEqualTypeOf<number[] | null>();

// 스키마 버전은 없을 수 있다(구버전 봉투) — `string` 으로 단정하면 그 사실이 사라진다.
expectTypeOf(plainEnvelope.schemaVersion).toEqualTypeOf<string | undefined>();

// ── 이름 변환 규칙 ────────────────────────────────────────────────────────

// 런타임 `toSnake`/`toCamel` 과 **같은 규칙**이어야 한다. 두 규칙이 갈라지면 생성
// 타입의 키와 실제 봉투의 키가 어긋나고, 그 어긋남은 런타임에야 드러난다.
expectTypeOf<SnakeCase<'pageCount'>>().toEqualTypeOf<'page_count'>();
expectTypeOf<SnakeCase<'sourceA'>>().toEqualTypeOf<'source_a'>();
expectTypeOf<SnakeCase<'irSchemaVersion'>>().toEqualTypeOf<'ir_schema_version'>();
expectTypeOf<SnakeCase<'already_snake'>>().toEqualTypeOf<'already_snake'>();

expectTypeOf<CamelCase<'dry_run'>>().toEqualTypeOf<'dryRun'>();
expectTypeOf<CamelCase<'page_count'>>().toEqualTypeOf<'pageCount'>();
expectTypeOf<CamelCase<'alreadyCamel'>>().toEqualTypeOf<'alreadyCamel'>();

// ── 예외 계층 ─────────────────────────────────────────────────────────────

declare const usage: UsageError;
declare const runtimeFailure: RhwpRuntimeError;
declare const verdict: VerdictFailed;
declare const protocol: ProtocolError;
declare const sessionClosed: SessionClosedError;
declare const binaryMissing: BinaryNotFoundError;
declare const timedOut: RhwpTimeoutError;

// 전부 기반 예외로 받을 수 있어야 `catch (e) { if (e instanceof RhwpError) ... }` 하나로
// 모든 실패를 잡는 흐름이 성립한다.
const hierarchy: RhwpError[] = [
  usage,
  runtimeFailure,
  verdict,
  protocol,
  sessionClosed,
  binaryMissing,
  timedOut,
];
expectTypeOf(hierarchy).toEqualTypeOf<RhwpError[]>();

// 하위 클래스 고유 멤버는 그 클래스에서만 보여야 한다.
expectTypeOf(usage.suggestion).toEqualTypeOf<string | undefined>();
expectTypeOf(verdict.isPageCountMismatch).toEqualTypeOf<boolean>();
expectTypeOf(usage.stderr).toEqualTypeOf<string>();
expectTypeOf(usage.lastDiagnostic).toEqualTypeOf<string>();
expectTypeOf(usage.exitCode).toEqualTypeOf<number | undefined>();

declare const anyFailure: RhwpError;
// @ts-expect-error — 계층은 한 방향이다. 기반 예외를 사용법 오류로 받으면
// `suggestion` 이 없는 값에서 교정 제안을 읽게 된다.
const _wrongDirection: UsageError = anyFailure;

// ── batch 는 봉투가 아니라 레코드 목록이다 ────────────────────────────────

declare const records: Awaited<ReturnType<typeof batch>>;

// 부분 실패를 `error` 필드로 판별하는 것이 이 축의 계약이라 `Envelope` 로 감싸지
// 않는다 — `Envelope.get` 은 없는 필드에 예외를 던져 `error` 유무 검사를 방해한다.
expectTypeOf(batch).toBeFunction();
expectTypeOf(records).toEqualTypeOf<BatchRecord[]>();

// `noUncheckedIndexedAccess` 가 켜져 있어야 "빈 결과"를 성공으로 오독하지 않는다.
expectTypeOf(records[0]).toEqualTypeOf<BatchRecord | undefined>();
expectTypeOf(records[0]?.error).toEqualTypeOf<string | undefined>();

// ── 명령 시그니처 ─────────────────────────────────────────────────────────

expectTypeOf(info).toBeFunction();
expectTypeOf(search).toBeFunction();
expectTypeOf(setCell).toBeFunction();
expectTypeOf(fillFields).toBeFunction();

declare const infoResult: Awaited<ReturnType<typeof info>>;
expectTypeOf(infoResult).toEqualTypeOf<Envelope>();

expectTypeOf<Parameters<typeof search>[1]>().toEqualTypeOf<string>();
expectTypeOf<Parameters<typeof setCell>[1]>().toEqualTypeOf<number>();

// 누름틀 데이터는 `any` 가 아니라 `unknown` 값의 맵이다 — 값의 의미를 지우지 않되,
// 아무 연산이나 허용하지도 않는다.
expectTypeOf<Parameters<typeof fillFields>[1]>().toEqualTypeOf<Readonly<Record<string, unknown>>>();

// ── exactOptionalPropertyTypes 아래의 옵션 객체 ───────────────────────────

declare const maybeOut: string | undefined;

// 조건부로 옵션을 조립해도 성립해야 한다. 성립하지 않으면 호출자는 분기마다 옵션
// 객체를 따로 만들게 되고, 그 중복이 곧 어긋난다.
const editOptions: EditOptions = { out: maybeOut, dryRun: true, verify: undefined };
expectTypeOf(editOptions).toEqualTypeOf<EditOptions>();

const searchOptions: SearchOptions = { caseSensitive: false, limit: 10 };
expectTypeOf(searchOptions).toEqualTypeOf<SearchOptions>();

const cellOptions: SetCellOptions = { keepStyle: true, out: maybeOut };
expectTypeOf(cellOptions).toEqualTypeOf<SetCellOptions>();

// `null` 은 "무제한"이라는 뜻이고 `undefined` 는 "기본값"이라는 뜻이다. 둘을 같은
// 타입으로 묶으면 무제한 실행을 기본값으로 되돌리는 사고가 조용히 일어난다.
const noTimeout: CommandOptions = { timeoutMs: null };
expectTypeOf(noTimeout).toEqualTypeOf<CommandOptions>();

// @ts-expect-error — 판정 플래그에 문자열을 넣으면 CLI 에 `--verify` 가 붙지 않는다.
const _wrongVerify: EditOptions = { verify: 'true' };

// @ts-expect-error — 오타 난 옵션 이름은 조용히 무시되는 대신 컴파일에서 걸려야 한다.
const _typo: EditOptions = { drynRun: true };
