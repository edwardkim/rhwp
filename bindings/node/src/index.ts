/**
 * `@rhwp/node` — HWP/HWPX 문서 엔진의 Node/TypeScript 바인딩.
 *
 * **바인딩은 새 표면이 아니라 기존 계약의 재포장이다**
 * (`mydocs/tech/bindings_foundation.md`). CLI `--json` 봉투와 `mcp-serve` 세션 도구가
 * 이미 증명한 계약 위에만 서고, TypeScript 쪽에서 판정 로직을 새로 만들지 않는다.
 * 그래서 rhwp 본체에 명령이 늘면 바인딩은 자동으로 따라온다.
 *
 * ## 1층 — 무상태 (호출 하나 = 작업 하나)
 *
 * ```ts
 * import { info, search } from '@rhwp/node';
 *
 * const meta = await info('보고서.hwp');
 * console.log(meta.get<number>('pageCount'));
 *
 * for (const match of (await search('보고서.hwp', '예산')).children('matches')) {
 *   console.log(match.get('page'), match.get('snippet'));
 * }
 * ```
 *
 * ## 2층 — 세션 (같은 문서를 반복해서 만질 때)
 *
 * ```ts
 * import { openDocument } from '@rhwp/node';
 *
 * const doc = await openDocument('서식.hwp');
 * try {
 *   await doc.fillFields({ 성명: '홍길동' });
 *   const saved = await doc.save('제출본.hwp', { verify: true });
 *   if (!saved.verify?.identical) console.warn('저장본이 다르다');
 * } finally {
 *   await doc.close();
 * }
 * ```
 *
 * ## 3층 — 계획 (의도를 선언하면 안전은 도구가 보장)
 *
 * ```ts
 * import { Plan } from '@rhwp/node';
 *
 * const plan = new Plan('서식.hwp', '제출본.hwp')
 *   .fillFields({ 성명: '홍길동' })
 *   .verify();
 *
 * const preview = await plan.check();   // 디스크 무변경
 * if (preview.ok) await plan.run();
 * ```
 *
 * ## 판정 vs 고장
 *
 * `--verify` 불일치나 시각 회귀는 **예외가 아니다** — 도구는 정상 동작했고 문서에
 * 대한 단언이 실패한 것이다. 판정은 반환값(`result.verify?.identical`)으로 읽는다.
 * 예외를 원하면 `throwOnVerdict: true` 를 명시한다.
 *
 * @packageDocumentation
 */

// ── 메타 ────────────────────────────────────────────────────────────────────

/** 바인딩 패키지 버전. rhwp 본체 버전과 별개다. */
export const VERSION = '0.1.0';

/** 이 바인딩이 검증한 봉투 스키마 버전. 본체가 major 를 올리면 여기도 올린다. */
export const SUPPORTED_SCHEMA_VERSION = '1.0';

// ── 바이너리 탐색 ───────────────────────────────────────────────────────────

export {
  ENV_VAR,
  binaryName,
  bundledDir,
  clearBinaryCache,
  findBinary,
  type FindBinaryOptions,
} from './binary.js';

// ── 예외·종료 코드 ──────────────────────────────────────────────────────────

export {
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
  type ErrorContext,
  type KnownExitCode,
  type RaiseForExitOptions,
} from './errors.js';

// ── 봉투 ────────────────────────────────────────────────────────────────────

export {
  Envelope,
  VerifyReport,
  asEnvelope,
  type BatchRecord,
  type RawEnvelope,
  type RawVerifyReport,
} from './envelope.js';

// ── 이름 변환 ───────────────────────────────────────────────────────────────

export {
  camelKeys,
  isSafeIdentifier,
  propertyKey,
  snakeKeys,
  toCamel,
  toSnake,
  type CamelCase,
  type CamelKeys,
  type JsonValue,
  type SnakeCase,
  type SnakeKeys,
} from './naming.js';

// ── 저수준 실행 ─────────────────────────────────────────────────────────────

export {
  DEFAULT_TIMEOUT_MS,
  iterNdjson,
  runJson,
  runNdjson,
  runRaw,
  type Argument,
  type CompletedRun,
  type RunOptions,
} from './process.js';

// ── 1층: 무상태 명령 ────────────────────────────────────────────────────────

export {
  batch,
  buildFromIngest,
  capabilities,
  convert,
  csvToTable,
  digest,
  explain,
  exportAgentManifest,
  exportCapabilitiesSchema,
  exportDoclang,
  exportHml,
  exportHwpx,
  exportIrSchema,
  exportPlanSchema,
  exportMarkdown,
  exportPdf,
  exportProvenanceMap,
  exportStructure,
  exportSvg,
  exportTables,
  exportText,
  extractData,
  extractPages,
  fields,
  fillFields,
  info,
  inspect,
  irDiff,
  renderDiff,
  replaceText,
  search,
  setCell,
  tableToCsv,
  thumbnail,
  type AgentManifestOptions,
  type BatchOptions,
  type BuildFromIngestOptions,
  type CapabilitiesOptions,
  type CapabilitiesSchemaOptions,
  type CommandOptions,
  type ConvertOptions,
  type CsvToTableOptions,
  type DigestOptions,
  type EditOptions,
  type ExportDoclangOptions,
  type ExportPdfOptions,
  type ExportStructureOptions,
  type ExportSvgOptions,
  type ExportTextOptions,
  type ExtractDataKind,
  type ExtractDataOptions,
  type InspectHiddenTextOptions,
  type InspectInjectionOptions,
  type InspectOptions,
  type InspectTarget,
  type InspectUnicodeOptions,
  type IrDiffOptions,
  type IrSchemaOptions,
  type OutputOptions,
  type PagedOutputOptions,
  type PathLike,
  type PlanSchemaOptions,
  type PdfBackend,
  type RenderDiffOptions,
  type RenderProfile,
  type ReplaceTextOptions,
  type RoundtripVia,
  type SearchOptions,
  type SetCellOptions,
  type StructureMode,
  type TableToCsvOptions,
  type ThumbnailOptions,
  type UnicodeInspectionKind,
  type VerifiableOptions,
} from './commands.js';

// ── 2층: 세션 ───────────────────────────────────────────────────────────────

export {
  Document,
  Session,
  openDocument,
  type OpenOptions,
  type SessionOptions,
} from './session.js';

// ── 3층: 계획 ───────────────────────────────────────────────────────────────

export {
  Plan,
  PlanResult,
  clearPlanCapabilityCache,
  runPlan,
  type PlanDocument,
  type PlanReplaceOptions,
  type PlanSetCellOptions,
  type PlanStep,
} from './plan.js';

// ── IR 스키마 ───────────────────────────────────────────────────────────────

export {
  FieldDef,
  IrSchema,
  TypeDef,
  capabilitiesSchema,
  irSchema,
  type SchemaNode,
} from './schema.js';

// ── 브라우저·환경 독립 ──────────────────────────────────────────────────────

export {
  createBrowserClient,
  createNodeClient,
  type DocumentSource,
  type RhwpClient,
  type RhwpWasmDocument,
  type RhwpWasmModule,
} from './browser.js';

// ── 생성 타입 ───────────────────────────────────────────────────────────────
//
// `tools/gen-types.ts` 가 `export-ir-schema`·`capabilities` 에서 만든다.
// 손으로 고치지 않는다 — 스키마가 단일 출처다.

export type * from './ir.js';
export type * from './envelopes.js';
