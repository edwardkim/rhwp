/**
 * 통합 테스트 공용 픽스처 — 실물 바이너리·실물 문서를 물린다.
 *
 * 이 파일이 존재하는 이유는 "없으면 실패"가 아니라 **"없으면 건너뛴다"** 를 한 곳에
 * 모으기 위해서다. 기여자 대부분은 Rust 툴체인 없이 TypeScript 만 만지고, 그들에게
 * 통합 실패를 보여 주면 게이트를 신뢰하지 않게 된다. 반대로 CI 의 통합 잡은 항상
 * `RHWP_BIN` 을 채워 돌리므로 건너뜀이 은폐가 되지도 않는다.
 *
 * 좌표·필드 이름을 **추측하지 않는 것**도 여기 모인 이유다. 표 번호나 누름틀 이름을
 * 테스트에 상수로 박으면 샘플이 조금만 바뀌어도 "바인딩 버그"처럼 보이는 실패가 난다.
 * 아래 조회 헬퍼들은 전부 도구에게 물어본 답을 그대로 쓴다.
 *
 * @packageDocumentation
 */

import { existsSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { afterEach, beforeEach } from 'vitest';

import {
  ENV_VAR,
  binaryName,
  capabilities,
  clearBinaryCache,
  exportTables,
  fields,
  runRaw,
  EXIT_USAGE,
  type Envelope,
} from '../../src/index.js';

// ── 위치 ──────────────────────────────────────────────────────────────────

const HERE = dirname(fileURLToPath(import.meta.url));

/** 저장소 루트. `bindings/node/test/helpers` 에서 네 단계 위. */
export const REPO_ROOT = resolve(HERE, '..', '..', '..', '..');

// ── 바이너리 ──────────────────────────────────────────────────────────────

/**
 * 실행 파일을 찾는다: `RHWP_BIN` → 로컬 빌드(release → debug).
 *
 * `findBinary()` 를 쓰지 않는 이유: 그쪽은 "패키지 동봉 → PATH" 를 보는 **소비자**
 * 경로이고, 여기는 "이 저장소를 방금 빌드한 개발자" 경로다. 저장소 빌드가 있는데
 * PATH 의 오래된 rhwp 가 잡히면, 방금 고친 동작이 테스트에 반영되지 않는다.
 */
function locateBinary(): string | undefined {
  const fromEnv = (process.env[ENV_VAR] ?? '').trim();
  if (fromEnv && existsSync(fromEnv)) return resolve(fromEnv);

  for (const profile of ['release', 'debug']) {
    const candidate = join(REPO_ROOT, 'target', profile, binaryName());
    if (existsSync(candidate)) return candidate;
  }
  return undefined;
}

/** 물린 실행 파일 경로. 없으면 undefined. */
export const BINARY: string | undefined = locateBinary();

/** 통합 테스트를 돌릴 수 있는가. */
export const hasBinary: boolean = BINARY !== undefined;

// 찾은 경로를 환경변수에 되꽂아 `findBinary()` 가 같은 것을 보게 한다. 탐색 캐시는
// 모듈 수명 동안 유지되므로, 되꽂은 뒤 반드시 비워야 이전 탐색 결과가 남지 않는다.
if (BINARY !== undefined && process.env[ENV_VAR] !== BINARY) {
  process.env[ENV_VAR] = BINARY;
  clearBinaryCache();
}

// ── 샘플 문서 ─────────────────────────────────────────────────────────────

function sample(name: string): string | undefined {
  const path = join(REPO_ROOT, 'samples', name);
  return existsSync(path) ? path : undefined;
}

/** 누름틀(ClickHere) 11개가 든 3쪽짜리 HWP5. 편집·판정 축의 기본 샘플. */
export const FIELD_SAMPLE: string | undefined = sample('field-01.hwp');

/** 표 53개(머리말 1 + 본문 최상위 52)가 든 HWPX. 표 좌표 축의 기본 샘플. */
export const TABLE_SAMPLE: string | undefined = sample(
  '2025년 기부·답례품 실적 지자체 보고서_양식.hwpx',
);

/** 누름틀 샘플로 돌 수 있는가 (바이너리 + 문서). */
export const fieldSampleReady: boolean = hasBinary && FIELD_SAMPLE !== undefined;

/** 표 샘플로 돌 수 있는가. */
export const tableSampleReady: boolean = hasBinary && TABLE_SAMPLE !== undefined;

/**
 * 누름틀 샘플 경로. `fieldSampleReady` 가 참인 블록 안에서만 부른다.
 *
 * `string | undefined` 를 매번 좁히는 대신 접근자를 두는 이유는, 단언(`!`)이
 * 테스트마다 흩어지면 "없을 수도 있다"는 사실이 코드에서 사라지기 때문이다.
 */
export function fieldSample(): string {
  if (FIELD_SAMPLE === undefined) {
    throw new Error('samples/field-01.hwp 가 없습니다 — 통합 대상이 아닙니다');
  }
  return FIELD_SAMPLE;
}

/** 표 샘플 경로. `tableSampleReady` 가 참인 블록 안에서만 부른다. */
export function tableSample(): string {
  if (TABLE_SAMPLE === undefined) {
    throw new Error('표 샘플(hwpx)이 없습니다 — 통합 대상이 아닙니다');
  }
  return TABLE_SAMPLE;
}

// ── 임시 디렉터리 ─────────────────────────────────────────────────────────

/**
 * 테스트마다 새 임시 디렉터리를 만들고 정리한다.
 *
 * 산출물을 저장소 안에 쓰지 않는 것이 요점이다 — 한 번이라도 흘리면 다음 `git status`
 * 가 지저분해지고, 그 상태가 커밋으로 새어 나간다.
 *
 * @returns 현재 테스트의 임시 디렉터리를 돌려주는 접근자.
 */
export function useTempDir(): (...segments: string[]) => string {
  let dir: string | undefined;

  beforeEach(() => {
    dir = mkdtempSync(join(tmpdir(), 'rhwp-node-'));
  });

  afterEach(() => {
    if (dir !== undefined) {
      rmSync(dir, { recursive: true, force: true });
      dir = undefined;
    }
  });

  return (...segments: string[]): string => {
    if (dir === undefined) {
      throw new Error('임시 디렉터리가 없습니다 — useTempDir() 은 describe 안에서 부르세요');
    }
    return segments.length ? join(dir, ...segments) : dir;
  };
}

// ── 자기서술(capabilities) 조회 ───────────────────────────────────────────

/** `capabilities` 가 선언하는 명령 하나. */
export interface DeclaredCommand {
  readonly name: string;
  readonly category?: string;
  readonly json?: boolean;
  readonly batch?: boolean;
  readonly flags?: readonly string[];
  readonly recordFields?: readonly string[];
  readonly summary?: string;
}

/** `capabilities --mcp` 가 선언하는 도구 하나. */
export interface DeclaredTool {
  readonly name: string;
  readonly description?: string;
  readonly cli?: { readonly command?: string; readonly args?: readonly string[] };
  readonly outputFields?: readonly string[];
}

// 자기서술은 프로세스 한 번당 한 번만 읽으면 충분하다 — 명령마다 다시 물으면
// 통합 잡이 느려지는 만큼 실행 빈도가 줄고, 결국 게이트가 늦게 돈다.
let capabilitiesCache: Envelope | undefined;
let mcpCapabilitiesCache: Envelope | undefined;

/** `capabilities` 봉투 (캐시). */
export async function loadCapabilities(): Promise<Envelope> {
  capabilitiesCache ??= await capabilities();
  return capabilitiesCache;
}

/** `capabilities --mcp` 봉투 (캐시). */
export async function loadMcpCapabilities(): Promise<Envelope> {
  mcpCapabilitiesCache ??= await capabilities({ mcp: true });
  return mcpCapabilitiesCache;
}

/** 선언된 명령을 이름으로 찾을 수 있게 정리한다. */
export async function declaredCommands(): Promise<ReadonlyMap<string, DeclaredCommand>> {
  const envelope = await loadCapabilities();
  const list = envelope.get<readonly DeclaredCommand[]>('commands');
  return new Map(list.map((command): [string, DeclaredCommand] => [command.name, command]));
}

/** 선언된 MCP 도구 목록. */
export async function declaredTools(): Promise<readonly DeclaredTool[]> {
  const envelope = await loadMcpCapabilities();
  return envelope.get<readonly DeclaredTool[]>('tools');
}

/**
 * 명령이 이 플래그를 **선언**하는가.
 *
 * 기능 유무를 버전 문자열로 추정하지 않는 이유: 자기서술이 단일 출처이고, 버전 비교는
 * 포크·패치 빌드에서 곧바로 틀린다.
 */
export async function commandDeclaresFlag(command: string, flag: string): Promise<boolean> {
  const commands = await declaredCommands();
  return (commands.get(command)?.flags ?? []).includes(flag);
}

/**
 * 이 인자 조합을 도구가 받아들이는가 (exit 2 = 사용법 오류가 아니면 참).
 *
 * 자기서술에 아직 등재되지 않은 신규 명령을 가려낼 때 쓴다.
 */
export async function acceptsInvocation(args: readonly string[]): Promise<boolean> {
  if (!hasBinary) return false;
  try {
    const result = await runRaw(args, { check: false });
    return result.exitCode !== EXIT_USAGE;
  } catch (error) {
    // 기능 탐지 단계의 실패는 "그 기능이 없다"로 취급한다. 여기서 던지면 파일 수집
    // 단계가 통째로 터져, 건너뛰었어야 할 것이 오류로 보고된다.
    console.warn(`[통합] 기능 탐지 실패 (${args.join(' ')}): ${String(error)}`);
    return false;
  }
}

/** 이 rhwp 가 `export-ir-schema` 를 갖고 있는가 (#3762 머지 여부). */
export async function supportsIrSchema(): Promise<boolean> {
  return acceptsInvocation(['export-ir-schema', '--json']);
}

/** 이 rhwp 의 계획 실행기가 `--dry-run` 을 지원하는가 (#3759 머지 여부). */
export async function supportsPlanDryRun(): Promise<boolean> {
  if (!hasBinary) return false;
  try {
    return await commandDeclaresFlag('run', '--dry-run');
  } catch (error) {
    console.warn(`[통합] 자기서술을 읽지 못했습니다: ${String(error)}`);
    return false;
  }
}

// ── 문서 조회 헬퍼 ────────────────────────────────────────────────────────

/** `fields` 봉투의 항목 한 개 (필요한 부분만). */
interface RawField {
  readonly name?: string;
}

/**
 * 문서의 첫 누름틀 이름. 없으면 undefined.
 *
 * 이름을 상수로 박지 않는 이유: 샘플이 바뀌면 실패 원인이 "바인딩"으로 오인된다.
 */
export async function firstFieldName(path: string): Promise<string | undefined> {
  const envelope = await fields(path);
  for (const field of envelope.get<readonly RawField[]>('fields')) {
    if (typeof field.name === 'string' && field.name.length > 0) return field.name;
  }
  return undefined;
}

/** `export-tables` 봉투의 셀 하나 (필요한 부분만). */
interface RawCell {
  readonly row: number;
  readonly col: number;
  readonly text?: string;
}

/** `export-tables` 봉투의 표 하나 (필요한 부분만). */
interface RawTable {
  readonly index: number;
  readonly rows: number;
  readonly cols: number;
  readonly cells?: readonly RawCell[];
  /** 비어 있어야 **본문 최상위** 표다. 머리말·꼬리말 안의 표도 목록에는 나온다. */
  readonly containerPath?: readonly unknown[];
}

/** `edit set-cell` 이 받아들이는 좌표 한 벌. */
export interface CellAddress {
  /** `--table` 에 그대로 넘길 번호. */
  readonly table: number;
  readonly row: number;
  readonly col: number;
  /** 덮어쓰기 전 값 — 편집이 실제로 반영됐는지 대조하는 데 쓴다. */
  readonly oldText: string;
}

/**
 * 실제로 쓸 수 있는 표 셀 좌표 하나를 **조회해서** 돌려준다.
 *
 * 두 가지를 추측하지 않는 것이 요점이다.
 *
 * 1. **표 번호** — `edit set-cell --table N` 은 `containerPath` 가 빈 표(본문 최상위)
 *    중에서 `index === N` 인 것을 찾는다. 머리말 표가 목록 앞에 오는 문서에서는
 *    최상위 표 번호가 0 에서 시작하지 않는다.
 * 2. **셀 좌표** — 병합으로 덮인 칸은 `cells` 에 아예 없다. 목록에 있는 앵커 좌표만
 *    기록 대상이므로 `(0,0)` 을 가정하면 병합 표에서 실패한다.
 */
export async function firstBodyTableCell(path: string): Promise<CellAddress | undefined> {
  const envelope = await exportTables(path);
  for (const table of envelope.get<readonly RawTable[]>('tables')) {
    if ((table.containerPath?.length ?? 0) > 0) continue;
    const cell = table.cells?.[0];
    if (cell === undefined) continue;
    return {
      table: table.index,
      row: cell.row,
      col: cell.col,
      oldText: cell.text ?? '',
    };
  }
  return undefined;
}

/**
 * 문서 본문에서 검색어로 쓸 만한 어휘 하나를 고른다.
 *
 * @param pageText - `export-text` 가 준 한 쪽의 평문.
 * @returns 두 글자 이상인 첫 어휘. 없으면 undefined.
 */
export function pickNeedle(pageText: string): string | undefined {
  for (const word of pageText.split(/\s+/)) {
    const trimmed = word.trim();
    if (trimmed.length >= 2) return trimmed;
  }
  return undefined;
}
