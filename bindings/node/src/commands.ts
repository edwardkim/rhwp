/**
 * API 1층 — 무상태 명령 래퍼.
 *
 * 각 함수는 CLI `--json` 봉투를 {@link Envelope} 로 돌려준다. 호출 한 번 = 프로세스
 * 한 번 = 문서 재파싱 한 번이다. 같은 문서를 반복해서 만질 거라면
 * {@link module:session} 의 2층이 빠르다.
 *
 * 판정 규약: exit 3/4 는 예외가 아니라 봉투의 판정 필드다. 예외를 원하면
 * `throwOnVerdict: true` 를 넘긴다.
 *
 * @packageDocumentation
 */

import { Envelope, type BatchRecord, type RawEnvelope } from './envelope.js';
import { EXIT_USAGE, UsageError } from './errors.js';
import { runJson, runNdjson, type Argument, type RunOptions } from './process.js';

/** 경로로 쓸 수 있는 값. */
export type PathLike = string;

/** 모든 명령이 공유하는 옵션. */
export interface CommandOptions {
  /** 제한 시간(ms). `null` 이면 무제한. */
  readonly timeoutMs?: number | null | undefined;
  /** 작업 디렉터리. */
  readonly cwd?: string | undefined;
}

/** 산출 파일을 내는 명령의 공통 옵션. */
export interface OutputOptions extends CommandOptions {
  /** 산출 경로. 생략하면 도구 기본 위치. */
  readonly out?: PathLike | undefined;
}

/** 판정(`--verify`)을 낼 수 있는 명령의 공통 옵션. */
export interface VerifiableOptions extends OutputOptions {
  /** 저장 직후 자기검증을 요청한다. */
  readonly verify?: boolean | undefined;
  /**
   * 판정 실패(exit 3/4)를 예외로 올릴지.
   *
   * 기본은 거짓 — 판정은 반환값(`envelope.verify`)으로 읽는 것이 이 바인딩의 규약이다.
   */
  readonly throwOnVerdict?: boolean | undefined;
}

/** 편집 명령의 공통 옵션. */
export interface EditOptions extends VerifiableOptions {
  /** 디스크를 건드리지 않고 예정 결과만 본다. */
  readonly dryRun?: boolean | undefined;
}

/** `CommandOptions` 를 프로세스 실행 옵션으로 옮긴다. */
function toRunOptions(
  options: CommandOptions & { throwOnVerdict?: boolean | undefined },
): RunOptions {
  return {
    timeoutMs: options.timeoutMs,
    cwd: options.cwd,
    throwOnVerdict: options.throwOnVerdict,
  };
}

/** 값이 있으면 `--name value` 를 붙인다. */
function flag(args: Argument[], name: string, value: Argument | undefined): void {
  if (value !== undefined) args.push(name, value);
}

/** 참이면 플래그만 붙인다. */
function toggle(args: Argument[], name: string, enabled: boolean | undefined): void {
  if (enabled) args.push(name);
}

/**
 * 여러 번 지정할 수 있는 값 플래그 (`--font-path` 처럼).
 *
 * 문자열 하나만 줘도 되고 목록을 줘도 된다 — CLI 는 같은 플래그가 반복되면 값을
 * 누적하므로, 목록을 쉼표로 이어 붙이면 안 된다(경로에 쉼표가 들어갈 수 있다).
 */
function repeat(
  args: Argument[],
  name: string,
  values: string | readonly string[] | undefined,
): void {
  if (values === undefined) return;
  for (const value of typeof values === 'string' ? [values] : values) {
    args.push(name, value);
  }
}

/** 공통 산출·편집 플래그를 한꺼번에 붙인다. */
function editFlags(args: Argument[], options: EditOptions): void {
  flag(args, '-o', options.out);
  toggle(args, '--dry-run', options.dryRun);
  toggle(args, '--verify', options.verify);
}

/** 봉투를 감싸 돌려주는 공통 실행. */
async function call<T extends RawEnvelope = RawEnvelope>(
  args: readonly Argument[],
  options: CommandOptions & { throwOnVerdict?: boolean | undefined } = {},
): Promise<Envelope<T>> {
  return new Envelope(await runJson<T>(args, toRunOptions(options)));
}

// ── 조회 ──────────────────────────────────────────────────────────────────

/** 문서 요약 — 포맷·쪽수·구역수·문단수·글꼴. */
export async function info(path: PathLike, options: CommandOptions = {}): Promise<Envelope> {
  return call(['info', path, '--json'], options);
}

/** {@link exportText} 옵션. */
export interface ExportTextOptions extends CommandOptions {
  /**
   * 이 쪽만 뽑는다 (0 기준).
   *
   * 긴 문서에서 필요한 쪽만 읽어 문맥을 아끼고 싶을 때. 생략하면 전 쪽.
   */
  readonly page?: number | undefined;
}

/**
 * 쪽별 평문 추출.
 *
 * `-o` 는 노출하지 않는다 — CLI 가 `--json` 모드에서 파일을 쓰지 않기 때문이다
 * (실측: `-o` 를 줘도 디렉터리가 생기지 않는다). 여기서 받아 주면 "저장했다"는
 * 거짓말이 된다. 파일이 필요하면 봉투의 `pages[].text` 를 직접 쓰라.
 */
export async function exportText(
  path: PathLike,
  options: ExportTextOptions = {},
): Promise<Envelope> {
  const args: Argument[] = ['export-text', path];
  flag(args, '-p', options.page);
  args.push('--json');
  return call(args, options);
}

/** 구조 분류 방식. */
export type StructureMode = 'auto' | 'outline' | 'clause';

/** {@link exportStructure} 옵션. */
export interface ExportStructureOptions extends CommandOptions {
  /**
   * 계층 분류 방식. 기본 `auto`.
   *
   * - `outline` — 개요 번호(1./가./1)) 기준. 보고서·기획서처럼 목차가 있는 문서.
   * - `clause` — 조문(편·장·절·관·조·항·호·목) 기준. 법령·규정·정관.
   * - `auto` — 문서를 보고 둘 중 하나를 고른다. 고른 결과는 봉투의 `mode` 에 담긴다.
   *
   * 자동 판정이 기대와 다를 때(예: 규정 문서를 개요로 읽었을 때) 명시로 되돌린다.
   */
  readonly mode?: StructureMode | undefined;
}

/**
 * 문서 구조 (제목 계층·절).
 *
 * `-o` 는 노출하지 않는다 — {@link exportText} 와 같은 이유로 `--json` 모드에서
 * 무시된다(실측: 파일이 생기지 않는다).
 */
export async function exportStructure(
  path: PathLike,
  options: ExportStructureOptions = {},
): Promise<Envelope> {
  const args: Argument[] = ['export-structure', path];
  flag(args, '--mode', options.mode);
  args.push('--json');
  return call(args, options);
}

/**
 * 표 전량을 셀 좌표와 함께. 병합 셀은 좌상단 좌표로만 나온다.
 *
 * `-o` 는 노출하지 않는다 — 실측하면 `-o` 를 준 순간 stdout 이 사람용 문장
 * ("표 추출 완료: N개 → 경로")으로 바뀌어 `--json` 봉투 계약이 깨진다. 받아 주면
 * 파싱이 `ProtocolError` 로 터진다. 파일이 필요하면 봉투를 직접 쓰라.
 */
export async function exportTables(
  path: PathLike,
  options: CommandOptions = {},
): Promise<Envelope> {
  return call(['export-tables', path, '--json'], options);
}

/** 누름틀(필드) 목록 — 이름·순번·현재값. */
export async function fields(path: PathLike, options: CommandOptions = {}): Promise<Envelope> {
  return call(['fields', path, '--json'], options);
}

/** {@link search} 옵션. */
export interface SearchOptions extends CommandOptions {
  /** 대소문자를 구분할지. 기본 참. */
  readonly caseSensitive?: boolean | undefined;
  /** 최대 매치 수. */
  readonly limit?: number | undefined;
}

/**
 * 주소가 붙은 검색 — 매치마다 (구역·문단·**쪽**·문자 오프셋).
 *
 * `-` 로 시작하는 검색어도 그대로 넘길 수 있다 — 내부에서 `--` 구분자를 써서
 * 옵션이 아닌 값으로 읽히게 한다.
 */
export async function search(
  path: PathLike,
  query: string,
  options: SearchOptions = {},
): Promise<Envelope> {
  const args: Argument[] = ['search', path];
  flag(args, '--limit', options.limit);
  if (options.caseSensitive === false) args.push('--ignore-case');
  args.push('--json', '--', query);
  return call(args, options);
}

/** {@link digest} 옵션. */
export interface DigestOptions extends CommandOptions {
  /** 절 단위 청킹 (주소 보존). */
  readonly sections?: boolean | undefined;
  /** 쪽 범위 창 (`"0..4"`). */
  readonly pages?: string | undefined;
  /**
   * 발췌 최대 문자 수. 기본 2000, `sections: true` 면 절마다 240.
   *
   * 문맥 창이 좁은 모델에 넘길 때 줄이고, 한 번에 더 읽히고 싶을 때 늘린다.
   * 이 값이 없으면 긴 문서에서 발췌만으로 창을 다 먹는다.
   */
  readonly maxChars?: number | undefined;
}

/** 요약·RAG 용 청킹. 주소를 보존하므로 인용에 "몇 쪽"을 답할 수 있다. */
export async function digest(path: PathLike, options: DigestOptions = {}): Promise<Envelope> {
  const args: Argument[] = ['digest', path];
  toggle(args, '--sections', options.sections);
  flag(args, '--pages', options.pages);
  flag(args, '--max-chars', options.maxChars);
  args.push('--json');
  return call(args, options);
}

/** `explain`은 문서의 형식·쪽수·표·필드·각주를 한 번에 요약한다. */
export async function explain(path: PathLike, options: CommandOptions = {}): Promise<Envelope> {
  return call(['explain', path, '--json'], options);
}

/** {@link capabilities} 옵션. */
export interface CapabilitiesOptions extends CommandOptions {
  /** MCP 도구 매니페스트를 받을지. */
  readonly mcp?: boolean | undefined;
}

/**
 * 도구 자기서술 — 명령 목록·플래그·봉투 필드·종료 코드 사전.
 *
 * 이 봉투가 바인딩의 단일 출처다. 명령이 늘었는지, 어떤 필드가 나오는지를 여기서
 * 읽으면 수기 목록을 둘 필요가 없다.
 */
export async function capabilities(options: CapabilitiesOptions = {}): Promise<Envelope> {
  const args: Argument[] = ['capabilities'];
  toggle(args, '--mcp', options.mcp);
  return call(args, options);
}

/** {@link exportIrSchema} 옵션. */
export interface IrSchemaOptions extends CommandOptions {
  /** 봉투 없이 스키마 본문만 (JSON Schema 도구 입력용). */
  readonly bare?: boolean | undefined;
}

/**
 * 공개 IR 의 JSON Schema.
 *
 * 문서를 입력으로 받지 않는다 — 스키마는 **타입의 자기서술**이지 특정 문서의
 * 속성이 아니다.
 */
export async function exportIrSchema(options: IrSchemaOptions = {}): Promise<Envelope> {
  const args: Argument[] = ['export-ir-schema'];
  toggle(args, '--bare', options.bare);
  args.push('--json');
  return call(args, options);
}

/** {@link exportPlanSchema} 옵션. */
export interface PlanSchemaOptions extends IrSchemaOptions, OutputOptions {}

/** `run` 계획서 문법의 JSON Schema. */
export async function exportPlanSchema(options: PlanSchemaOptions = {}): Promise<Envelope> {
  const args: Argument[] = ['export-plan-schema'];
  toggle(args, '--bare', options.bare);
  flag(args, '-o', options.out);
  args.push('--json');
  return call(args, options);
}

/** {@link exportAgentManifest} 옵션. */
export interface AgentManifestOptions extends IrSchemaOptions {}

/** capabilities·IR·provenance·plan schema를 한 봉투로 조립한 에이전트 매니페스트. */
export async function exportAgentManifest(
  options: AgentManifestOptions = {},
): Promise<Envelope> {
  const args: Argument[] = ['export-agent-manifest'];
  toggle(args, '--bare', options.bare);
  args.push('--json');
  return call(args, options);
}

/** {@link exportCapabilitiesSchema} 옵션. */
export interface CapabilitiesSchemaOptions extends IrSchemaOptions, OutputOptions {
  /**
   * 스키마를 이 파일로 저장한다.
   *
   * 저장해도 stdout 은 봉투를 유지한다(`output`·`bytes` 가 담긴다) — 그래서
   * {@link exportText}·{@link exportTables} 와 달리 안전하게 노출한다.
   * 타입 생성기가 스키마를 저장소에 커밋해 두고 diff 로 표면 변화를 감시할 때 쓴다.
   */
  readonly out?: PathLike | undefined;
}

/** 명령 표면(capabilities)의 JSON Schema — 타입 생성기가 읽는다. */
export async function exportCapabilitiesSchema(
  options: CapabilitiesSchemaOptions = {},
): Promise<Envelope> {
  const args: Argument[] = ['export-capabilities-schema'];
  toggle(args, '--bare', options.bare);
  flag(args, '-o', options.out);
  args.push('--json');
  return call(args, options);
}

export {
  csvToTable,
  exportProvenanceMap,
  extractData,
  inspect,
  tableToCsv,
  type CsvToTableOptions,
  type ExtractDataKind,
  type ExtractDataOptions,
  type InspectHiddenTextOptions,
  type InspectInjectionOptions,
  type InspectOptions,
  type InspectTarget,
  type InspectUnicodeOptions,
  type TableToCsvOptions,
  type UnicodeInspectionKind,
} from './document-analysis.js';

// ── 산출 ──────────────────────────────────────────────────────────────────

/** 쪽을 골라 낼 수 있는 산출 명령의 공통 옵션. */
export interface PagedOutputOptions extends OutputOptions {
  /** 특정 쪽만 (0 기준). 생략하면 전 쪽. */
  readonly page?: number | undefined;
}

/** {@link exportSvg} 옵션. */
export interface ExportSvgOptions extends PagedOutputOptions {}

/** SVG 렌더. */
export async function exportSvg(
  path: PathLike,
  options: ExportSvgOptions = {},
): Promise<Envelope> {
  const args: Argument[] = ['export-svg', path];
  flag(args, '-o', options.out);
  flag(args, '-p', options.page);
  args.push('--json');
  return call(args, options);
}

/**
 * 산출 명령을 만드는 공장 — 같은 모양을 다섯 번 쓰지 않기 위해.
 *
 * `-o` 와 `--json` 은 모든 산출 명령이 공유하므로 여기서 붙이고, 명령마다 다른
 * 플래그는 `extra` 콜백이 붙인다. 공장을 버리고 명령별로 함수를 복사하면 `-o`
 * 위치나 `--json` 순서가 한 곳에서 어긋나도 아무도 모르게 된다.
 *
 * `extra` 를 인자 스펙(배열)이 아니라 콜백으로 받는 이유: 플래그마다 값·토글·
 * 반복으로 붙는 방식이 다르고, 그 차이를 데이터로 표현하면 표를 읽는 코드가
 * 표보다 길어진다.
 */
function outputCommand<O extends OutputOptions>(
  command: string,
  extra?: (args: Argument[], options: O) => void,
): (path: PathLike, options?: O) => Promise<Envelope> {
  return async (path: PathLike, options?: O): Promise<Envelope> => {
    const args: Argument[] = [command, path];
    flag(args, '-o', options?.out);
    if (extra !== undefined && options !== undefined) extra(args, options);
    args.push('--json');
    return call(args, options ?? {});
  };
}

/** PDF backend. */
export type PdfBackend = 'svg' | 'direct';

/** 렌더 출력 프로필. */
export type RenderProfile = 'screen' | 'print' | 'high-quality' | 'fast-preview';

/** {@link exportPdf} 옵션. */
export interface ExportPdfOptions extends PagedOutputOptions {
  /**
   * PDF 생성 방식. 기본 `svg`.
   *
   * `direct` 는 `native-skia` 기능을 켜서 빌드한 바이너리에서만 동작한다 —
   * 없는 빌드에 주면 실행 오류(exit 1)로 끝난다. 실제로 쓴 backend 는 봉투의
   * `backend` 필드에 담기므로 결과로 확인할 수 있다.
   */
  readonly backend?: PdfBackend | undefined;
  /**
   * 레이어 출력 프로필. 화면용(`screen`)과 인쇄용(`print`)의 렌더 품질이 다르다.
   *
   * 인쇄물로 넘길 PDF 면 `print`, 미리보기라면 `fast-preview` 로 시간을 아낀다.
   */
  readonly profile?: RenderProfile | undefined;
  /**
   * 폰트 파일 탐색 경로. 여러 개를 줄 수 있다.
   *
   * 한컴 전용 폰트(HY견명조 등)가 시스템에 없는 서버·CI 에서 글자가 깨지거나
   * 대체 폰트로 밀릴 때, 폰트를 모아 둔 디렉터리를 지정한다.
   */
  readonly fontPath?: string | readonly string[] | undefined;
}

/** PDF 산출. */
export const exportPdf = outputCommand<ExportPdfOptions>('export-pdf', (args, options) => {
  flag(args, '-p', options.page);
  flag(args, '--backend', options.backend);
  flag(args, '--profile', options.profile);
  repeat(args, '--font-path', options.fontPath);
});

/** Markdown 산출. */
export const exportMarkdown = outputCommand<PagedOutputOptions>(
  'export-markdown',
  (args, options) => {
    flag(args, '-p', options.page);
  },
);

/** HML 재직렬화. `out` 은 CLI 가 요구한다 — 원본 덮어쓰기를 막기 위해서다. */
export const exportHml = outputCommand<OutputOptions>('export-hml');

/** {@link exportDoclang} 옵션. */
export interface ExportDoclangOptions extends OutputOptions {
  /**
   * 그림 등 이진 자원을 이 디렉터리에 **파일로** 기록한다.
   *
   * 생략하면 자원이 base64 data URI 로 XML 안에 인라인된다 — 그림이 많은 문서는
   * XML 이 수십 MB 로 부풀어 파서가 감당하지 못한다. 그럴 때 분리한다.
   * 기록 결과는 봉투의 `assetsDir`·`assetCount` 로 확인한다.
   */
  readonly assetsDir?: PathLike | undefined;
}

/** DocLang XML 산출. */
export const exportDoclang = outputCommand<ExportDoclangOptions>(
  'export-doclang',
  (args, options) => {
    flag(args, '--assets-dir', options.assetsDir);
  },
);

/**
 * {@link thumbnail} 옵션.
 *
 * `base64`·`dataUri` 는 파일 저장을 **대체한다** — 둘 중 하나를 켜면 `out` 을 줘도
 * 파일이 생기지 않고 봉투의 `output` 이 `null` 이 된다(실측). 파일과 문자열을
 * 둘 다 원하면 파일로 뽑은 뒤 직접 읽어라.
 *
 * 둘은 **서로 배타적**이기도 하다. 함께 켜면 나중 플래그가 이겨 `dataUri` 만
 * 온다 — 봉투에 `base64` 가 있을 거라 믿고 읽으면 `undefined` 를 만난다.
 * 하나만 고르라.
 */
export interface ThumbnailOptions extends OutputOptions {
  /**
   * 이미지 바이트를 봉투의 `base64` 필드로 받는다.
   *
   * 파일을 거치지 않고 바로 HTTP 응답이나 DB 에 실을 때 쓴다. 디스크를 쓸 수 없는
   * 서버리스·읽기전용 컨테이너에서 특히.
   */
  readonly base64?: boolean | undefined;
  /**
   * 봉투의 `dataUri` 필드로 `data:image/png;base64,…` 를 받는다.
   *
   * `<img src>` 에 그대로 넣을 수 있는 형태 — {@link ThumbnailOptions.base64} 와
   * 달리 MIME 접두가 붙는다.
   */
  readonly dataUri?: boolean | undefined;
}

/** 첫 쪽 미리보기 이미지 (PrvImage). */
export const thumbnail = outputCommand<ThumbnailOptions>('thumbnail', (args, options) => {
  toggle(args, '--base64', options.base64);
  toggle(args, '--data-uri', options.dataUri);
});

/**
 * 쪽 범위를 잘라 새 문서로.
 *
 * @param from - 시작 쪽 (**1 기준**, 포함).
 * @param to - 끝 쪽 (**1 기준**, 포함).
 *
 * 범위를 `"2-4"` 같은 문자열이 아니라 두 인자로 받는다 — CLI 가 `--from`/`--to`
 * 를 쓰기 때문이다(`--pages` 는 `digest` 쪽 어휘라 섞으면 exit 2 가 난다).
 *
 * 쪽 번호가 **1 기준**인 것에 주의하라. 이 바인딩의 다른 쪽 인자(`page`,
 * `digest` 의 `pages`)는 전부 0 기준인데 이 명령만 다르다 — CLI 가 그렇다
 * (`--from 0` 은 "쪽 범위가 잘못됐습니다 … (1 기준)" 으로 exit 1).
 */
export async function extractPages(
  path: PathLike,
  from: number,
  to: number,
  options: OutputOptions = {},
): Promise<Envelope> {
  const args: Argument[] = ['extract-pages', path, '--from', from, '--to', to];
  flag(args, '-o', options.out);
  args.push('--json');
  return call(args, options);
}

/** {@link buildFromIngest} 옵션. */
export interface BuildFromIngestOptions extends OutputOptions {
  /**
   * 명세가 참조하는 그림 파일이 들어 있는 디렉터리.
   *
   * 생략하거나 없는 경로를 주면 이미지가 placeholder 로 처리되고 경고만 남는다
   * (실패하지 않는다) — 그림이 빠진 산출물을 정상으로 오해하기 쉬우니, 그림이
   * 있는 명세라면 반드시 지정하라.
   */
  readonly mediaDir?: PathLike | undefined;
}

/** 구조 명세(JSON)에서 새 문서를 생성. `out` 은 CLI 가 요구한다. */
export async function buildFromIngest(
  spec: PathLike,
  options: BuildFromIngestOptions = {},
): Promise<Envelope> {
  const args: Argument[] = ['build-from-ingest', spec];
  flag(args, '--media-dir', options.mediaDir);
  flag(args, '-o', options.out);
  args.push('--json');
  return call(args, options);
}

// ── 변환·비교 ─────────────────────────────────────────────────────────────

/** {@link exportHwpx} 옵션. */
export interface ConvertOptions extends VerifiableOptions {
  /** 페이지 수 일치까지 단언한다 (불일치 시 exit 4). */
  readonly verifyPages?: boolean | undefined;
}

/**
 * HWP → HWPX 변환.
 *
 * `verify: true` 면 봉투에 `verify.identical` 이 담긴다. 판정 실패(exit 3)는
 * 기본적으로 예외가 아니다 — 봉투를 읽어 판단하라.
 *
 * 산출 경로는 **위치 인자**로 넘어간다. 이 명령은 `-o` 를 모른다("알 수 없는
 * 옵션: -o", exit 2). 생략하면 `<입력 stem>.hwpx`.
 */
export async function exportHwpx(
  path: PathLike,
  options: ConvertOptions = {},
): Promise<Envelope> {
  const args: Argument[] = ['export-hwpx', path];
  if (options.out !== undefined) args.push(options.out);
  toggle(args, '--verify', options.verify);
  toggle(args, '--verify-pages', options.verifyPages);
  args.push('--json');
  return call(args, options);
}

/**
 * HWPX·배포용 → 편집 가능 HWP 변환.
 *
 * `exportHwpx` 와 마찬가지로 산출 경로가 **위치 인자**다. 다만 이쪽은 **필수**다
 * — 기본 경로가 없어서, 빠뜨리면 CLI 가 사용법 오류로 끝난다. 프로세스를 띄우기
 * 전에 여기서 같은 판정을 내려 무엇이 빠졌는지 이름으로 알린다.
 *
 * @throws {UsageError} `out` 을 주지 않았을 때.
 */
export async function convert(
  path: PathLike,
  options: ConvertOptions = {},
): Promise<Envelope> {
  if (options.out === undefined) {
    throw new UsageError('convert 는 산출 경로가 필요합니다 — options.out 을 지정하세요', {
      argv: ['convert', String(path), '--json'],
      exitCode: EXIT_USAGE,
    });
  }
  const args: Argument[] = ['convert', path, options.out];
  toggle(args, '--verify', options.verify);
  toggle(args, '--verify-pages', options.verifyPages);
  args.push('--json');
  return call(args, options);
}

/** {@link irDiff} 옵션. */
export interface IrDiffOptions extends CommandOptions {
  /**
   * 이 구역만 비교 (0 기준).
   *
   * 차이가 수백 건 나오는 문서에서 범위를 좁혀 원인을 이분법으로 찾을 때.
   */
  readonly section?: number | undefined;
  /**
   * 이 **문단**만 비교 (0 기준). CLI 의 `-p`/`--para` 다.
   *
   * 쪽이 아니라 문단이다 — 다른 명령의 `page` 와 헷갈리지 말 것.
   * `section` 과 함께 주면 그 구역의 그 문단으로 좁힌다.
   */
  readonly paragraph?: number | undefined;
}

/** 두 문서의 IR 차이 — 무엇이 달라졌는지 범주별로. */
export async function irDiff(
  a: PathLike,
  b: PathLike,
  options: IrDiffOptions = {},
): Promise<Envelope> {
  const args: Argument[] = ['ir-diff', a, b];
  flag(args, '-s', options.section);
  flag(args, '-p', options.paragraph);
  args.push('--json');
  return call(args, options);
}

/** 라운드트립 경유 포맷. */
export type RoundtripVia = 'hwpx' | 'hwp';

/** {@link renderDiff} 옵션. */
export interface RenderDiffOptions extends CommandOptions {
  /**
   * 라운드트립 경유 포맷. 기본 `hwpx`.
   *
   * 원본을 이 포맷으로 직렬화했다가 다시 읽어 기하를 비교한다 — HWPX 저장이
   * 무엇을 잃는지, HWP5 저장이 무엇을 잃는지를 갈라서 본다.
   * `pathB` 를 준 두 파일 비교(pair) 에서는 **무시된다**(봉투의 `via` 가 null).
   */
  readonly via?: RoundtripVia | undefined;
  /**
   * 이 쪽만 판정 (0 기준).
   *
   * 회귀가 난 쪽을 좁혀 볼 때. 문서에 없는 쪽 번호면 사용법 오류(exit 2)다.
   */
  readonly page?: number | undefined;
  /**
   * 변위 임계(px). 기본 1.0. 초과한 쪽이 있으면 `status` 가 `OVER` 가 된다.
   *
   * 봉투에는 `threshold` 로 담긴다. 렌더 반올림 수준의 흔들림을 회귀로 세지
   * 않으려면 올리고, 더 촘촘한 게이트가 필요하면 내린다.
   */
  readonly maxDisp?: number | undefined;
  /**
   * 회귀 검출(exit 3)을 예외로 올릴지.
   *
   * 기본은 거짓 — 회귀는 도구의 고장이 아니라 **문서에 대한 판정**이므로
   * 봉투(`status`·`regression`)로 읽는 것이 이 바인딩의 규약이다.
   */
  readonly throwOnVerdict?: boolean | undefined;
}

/**
 * 렌더 기하 차이로 시각 회귀를 판정한다.
 *
 * `pathB` 를 주면 두 파일 직접 비교(`mode: "pair"`), 없으면 자기 라운드트립
 * (`mode: "roundtrip"`). **회귀는 예외가 아니라** `status`·`regression` 필드로
 * 온다 — 도구는 정상 동작했고 문서에 대한 판정이 실패한 것이다. 예외를 원하면
 * {@link RenderDiffOptions.throwOnVerdict} 를 준다.
 *
 * 라운드트립인데 옵션을 주고 싶으면 `pathB` 자리에 `undefined` 를 넘긴다:
 * `renderDiff('a.hwp', undefined, { via: 'hwp' })`.
 *
 * CLI 의 `--batch` 축(폴더 일괄)은 **여기서 감싸지 않는다** — 그쪽은 한 줄 봉투가
 * 아니라 NDJSON 스트림이라 반환 타입이 다르다. 이 함수가 둘 다 처리하면 호출자가
 * 받은 값이 봉투인지 배열인지 타입으로 알 수 없게 된다. 폴더 일괄이 필요하면
 * `-o <출력폴더>` 와 함께 CLI 를 직접 부르거나, `runNdjson` 을 쓰라.
 */
export async function renderDiff(
  path: PathLike,
  pathB?: PathLike | undefined,
  options: RenderDiffOptions = {},
): Promise<Envelope> {
  const args: Argument[] = ['render-diff', path];
  if (pathB !== undefined) args.push(pathB);
  flag(args, '--via', options.via);
  flag(args, '-p', options.page);
  flag(args, '--max-disp', options.maxDisp);
  args.push('--json');
  return call(args, options);
}

/** {@link verify} 옵션 — 기대 조건이 최소 1개 있어야 한다. */
export interface VerifyOptions extends CommandOptions {
  /** 총 쪽수 기대값 (`--expect-pages`). */
  readonly expectPages?: number | undefined;
  /** 최소 쪽수 (`--expect-min-pages`). */
  readonly expectMinPages?: number | undefined;
  /** 최대 쪽수 (`--expect-max-pages`). */
  readonly expectMaxPages?: number | undefined;
  /** 본문 최소 문자 수 (`--expect-min-chars`). */
  readonly expectMinChars?: number | undefined;
  /** 최소 표 개수 (`--expect-min-tables`). */
  readonly expectMinTables?: number | undefined;
  /** 표 개수 정확 일치 (`--expect-table-count`). */
  readonly expectTableCount?: number | undefined;
  /** 본문에 있어야 하는 문자열 (`--expect-contains`, 반복 가능). */
  readonly expectContains?: string | readonly string[] | undefined;
  /** 본문에 없어야 하는 문자열 (`--expect-not-contains`, 반복 가능). */
  readonly expectNotContains?: string | readonly string[] | undefined;
  /** `이름=값` 필드 기대값 (`--expect-field`, 반복 가능). */
  readonly expectField?: string | readonly string[] | undefined;
  /** 컨테이너 포맷 기대값 (`--expect-format`). */
  readonly expectFormat?: 'hwp5' | 'hwpx' | 'hwp3' | 'hml' | undefined;
  /**
   * 기대 위반(exit 3)을 예외로 올릴지.
   *
   * 기본은 거짓 — 위반은 도구의 고장이 아니라 **문서에 대한 판정**이므로
   * 봉투(`verdict`·`passCount`·`failCount`)로 읽는 것이 이 바인딩의 규약이다.
   */
  readonly throwOnVerdict?: boolean | undefined;
}

/**
 * 독립 사후검증 게이트 — 임의 파일에 기대 조건 집합을 대조한다.
 *
 * 편집 축의 `verify: true`(저장 직후 자기검증)와 달리 **임의 시점·임의 파일**에
 * 쓴다. 기대 조건이 하나도 없으면 CLI 가 사용법 오류(exit 2)를 낸다.
 */
export async function verify(path: PathLike, options: VerifyOptions = {}): Promise<Envelope> {
  const args: Argument[] = ['verify', path];
  flag(args, '--expect-pages', options.expectPages);
  flag(args, '--expect-min-pages', options.expectMinPages);
  flag(args, '--expect-max-pages', options.expectMaxPages);
  flag(args, '--expect-min-chars', options.expectMinChars);
  flag(args, '--expect-min-tables', options.expectMinTables);
  flag(args, '--expect-table-count', options.expectTableCount);
  repeat(args, '--expect-contains', options.expectContains);
  repeat(args, '--expect-not-contains', options.expectNotContains);
  repeat(args, '--expect-field', options.expectField);
  flag(args, '--expect-format', options.expectFormat);
  args.push('--json');
  return call(args, options);
}

// ── 편집 ──────────────────────────────────────────────────────────────────

/**
 * 누름틀 채우기 (메일머지).
 *
 * @param data - `{ "필드이름": "값" }`. 동명 필드는 `"이름#1"` 로 순번 지정.
 */
export async function fillFields(
  path: PathLike,
  data: Readonly<Record<string, unknown>>,
  options: EditOptions = {},
): Promise<Envelope> {
  const args: Argument[] = ['edit', 'fill-fields', path, '--data', JSON.stringify(data)];
  editFlags(args, options);
  args.push('--json');
  return call(args, options);
}

/** {@link replaceText} 옵션. */
export interface ReplaceTextOptions extends EditOptions {
  /** 이 순번 하나만 치환 (0 기준). */
  readonly occurrence?: number | undefined;
  /** 대소문자를 무시할지. */
  readonly ignoreCase?: boolean | undefined;
}

/** 문자열 치환. `occurrence` 를 주면 그 순번 하나만. */
export async function replaceText(
  path: PathLike,
  find: string,
  replace: string,
  options: ReplaceTextOptions = {},
): Promise<Envelope> {
  const args: Argument[] = [
    'edit',
    'replace-text',
    path,
    '--find',
    find,
    '--replace',
    replace,
  ];
  flag(args, '--occurrence', options.occurrence);
  toggle(args, '--ignore-case', options.ignoreCase);
  editFlags(args, options);
  args.push('--json');
  return call(args, options);
}

/** {@link setCell} 옵션. */
export interface SetCellOptions extends EditOptions {
  /** 기존 글자 모양을 유지할지. */
  readonly keepStyle?: boolean | undefined;
}

/**
 * 표 셀에 값 기록.
 *
 * 좌표는 {@link exportTables} 로 확인한다 — 최상위 표 인덱스가 0 에서 시작하지
 * 않는 문서도 있으므로 추측하면 안 된다.
 */
export async function setCell(
  path: PathLike,
  table: number,
  row: number,
  col: number,
  text: string,
  options: SetCellOptions = {},
): Promise<Envelope> {
  const args: Argument[] = [
    'edit',
    'set-cell',
    path,
    '--table',
    table,
    '--row',
    row,
    '--col',
    col,
    '--text',
    text,
  ];
  toggle(args, '--keep-style', options.keepStyle);
  editFlags(args, options);
  args.push('--json');
  return call(args, options);
}

// ── 대량 ──────────────────────────────────────────────────────────────────

/** {@link batch} 옵션. */
export interface BatchOptions extends CommandOptions {
  /**
   * 파일 **간** 병렬 스레드 수. 기본은 CPU 코어 수.
   *
   * 낮추는 쪽으로 쓰는 값이다 — 공유 CI 러너나 메모리가 빠듯한 컨테이너에서
   * 코어 수만큼 문서를 동시에 펼치면 OOM 으로 끝난다.
   */
  readonly threads?: number | undefined;
  /**
   * `export-structure` 축 전용 — 계층 분류 방식.
   *
   * 단건 {@link exportStructure} 의 `mode` 와 같은 값이다.
   */
  readonly mode?: StructureMode | undefined;
  /**
   * `search` 축 전용 — 찾을 문자열. 이 축에서는 **필수**다(없으면 exit 2).
   *
   * 단건 {@link search} 와 달리 `--` 구분자를 쓰지 않으므로, `-` 로 시작하는
   * 검색어는 이 경로로 넘기지 말 것.
   */
  readonly query?: string | undefined;
  /**
   * `convert` 축 전용 — 산출물을 모을 폴더. 이 축에서는 **필수**다.
   *
   * 산출 이름은 `<입력이름>.hwp` 로 고정이다. 대소문자만 다른 이름을 포함해
   * 이름이 겹치면 한 건도 쓰지 않고 exit 2 로 끝난다.
   */
  readonly outDir?: PathLike | undefined;
  /** `convert` 축 전용 — 재파싱 IR 비교. 차이가 있으면 집계 종료 코드가 3. */
  readonly verify?: boolean | undefined;
  /** `convert` 축 전용 — 재파싱 쪽수 비교. 불일치면 집계 종료 코드가 4. */
  readonly verifyPages?: boolean | undefined;
  /**
   * 서브커맨드에 덧붙일 인자 — 위 옵션이 못 담는 축의 탈출구.
   *
   * 이름 붙은 옵션 **뒤에** 놓이므로, 같은 플래그를 다시 주면 이쪽이 이긴다.
   */
  readonly extraArgs?: readonly Argument[] | undefined;
}

/**
 * 폴더/목록 일괄 처리 — NDJSON 레코드 목록을 돌려준다.
 *
 * 부분 실패도 실패다. 실패한 항목은 `error` 필드를 단 레코드로 남으므로, 스트림을
 * 통째로 버리지 말고 레코드별로 판단하라.
 *
 * `verify`/`verifyPages` 판정은 예외로 오지 않는다 — batch 는 스트림을 끝까지
 * 흘린 뒤 집계 종료 코드(3/4)로만 신호하므로, 판정은 레코드를 읽어 판단한다.
 * (그래서 이 축에는 `throwOnVerdict` 가 없다.)
 *
 * 기본 제한 시간은 무제한이다 — 대량 작업은 오래 걸린다.
 */
export async function batch<T extends BatchRecord = BatchRecord>(
  subcommand: string,
  paths: readonly PathLike[],
  options: BatchOptions = {},
): Promise<T[]> {
  if (paths.length === 0) {
    throw new Error('처리할 파일이 없습니다 — batch 는 최소 1개가 필요합니다');
  }
  const args: Argument[] = ['batch', subcommand];
  flag(args, '--threads', options.threads);
  flag(args, '--mode', options.mode);
  flag(args, '--query', options.query);
  flag(args, '--out-dir', options.outDir);
  toggle(args, '--verify', options.verify);
  toggle(args, '--verify-pages', options.verifyPages);
  args.push(...(options.extraArgs ?? []), '--json');
  return runNdjson<T>(args, {
    stdin: `${paths.join('\n')}\n`,
    timeoutMs: options.timeoutMs ?? null,
    cwd: options.cwd,
  });
}
