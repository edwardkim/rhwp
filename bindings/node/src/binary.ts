/**
 * rhwp 실행 파일 탐색.
 *
 * 탐색 순서는 `mydocs/tech/bindings_foundation.md` §3 이 고정한 그대로다:
 *
 * 1. 환경변수 `RHWP_BIN`
 * 2. 패키지 동봉 (`dist/_bin/`)
 * 3. `PATH`
 *
 * 순서 자체가 계약이다 — 개발자가 로컬 빌드를 가리키고 싶을 때(1) 패키지 동봉본(2)이
 * 가로채면 "왜 내 수정이 반영 안 되지"라는 진단 불가 상황이 생긴다.
 *
 * @packageDocumentation
 */

import { accessSync, constants, statSync } from 'node:fs';
import { delimiter, dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { BinaryNotFoundError } from './errors.js';

/** 바이너리 경로 환경변수 이름 — 문서 §3 고정. */
export const ENV_VAR = 'RHWP_BIN';

/** 탐색은 프로세스 수명 동안 캐시한다 — 명령 하나마다 PATH 를 훑을 이유가 없다. */
let cached: string | undefined;

/** 플랫폼별 실행 파일 이름. */
export function binaryName(): string {
  return process.platform === 'win32' ? 'rhwp.exe' : 'rhwp';
}

/**
 * 탐색 캐시를 비운다.
 *
 * 테스트에서 환경변수를 바꿔 가며 검사할 때 필요하다.
 */
export function clearBinaryCache(): void {
  cached = undefined;
}

/** 패키지 동봉 바이너리가 놓이는 디렉터리. */
export function bundledDir(): string {
  // ESM/CJS 듀얼이라 __dirname 을 그대로 못 쓴다. 빌드 산출물 기준 위치를 잡는다.
  const here =
    typeof __dirname === 'string'
      ? __dirname
      : dirname(fileURLToPath(import.meta.url));
  return join(here, '_bin');
}

/** 실행 가능한 **파일**인지. 디렉터리·깨진 링크·권한 없음을 모두 건다. */
function isExecutableFile(path: string): boolean {
  let stat;
  try {
    stat = statSync(path);
  } catch {
    // 경로가 없거나, 너무 길거나, 권한이 없어 stat 자체가 실패하는 경우.
    return false;
  }
  if (!stat.isFile()) return false;

  if (process.platform === 'win32') {
    // 윈도우는 실행 비트가 없다 — 확장자로 판단한다.
    return /\.(exe|bat|cmd)$/i.test(path);
  }
  try {
    accessSync(path, constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

/**
 * `RHWP_BIN` 이 가리키는 경로. 디렉터리를 줬으면 그 안의 실행 파일도 본다.
 *
 * @throws {BinaryNotFoundError} 환경변수를 **줬는데** 쓸 수 없을 때.
 *   조용히 다음 경로로 넘어가지 않는다 — 사용자는 그 바이너리를 쓰고 있다고
 *   믿는데 다른 게 실행되면 디버깅이 불가능해진다.
 */
function fromEnv(): string | undefined {
  const raw = (process.env[ENV_VAR] ?? '').trim();
  if (!raw) return undefined;

  let candidate = resolve(raw);
  try {
    if (statSync(candidate).isDirectory()) {
      candidate = join(candidate, binaryName());
    }
  } catch {
    // stat 실패는 아래 검사에서 BinaryNotFoundError 로 이어진다.
  }

  if (isExecutableFile(candidate)) return candidate;

  throw new BinaryNotFoundError(
    `${ENV_VAR} 가 가리키는 실행 파일을 쓸 수 없습니다: ${raw}\n` +
      '  (존재하지 않거나, 파일이 아니거나, 실행 권한이 없습니다)',
  );
}

/** 패키지에 동봉된 바이너리. */
function fromBundle(): string | undefined {
  const candidate = join(bundledDir(), binaryName());
  return isExecutableFile(candidate) ? candidate : undefined;
}

/** `PATH` 에서 찾기. */
function fromPath(): string | undefined {
  const name = binaryName();
  const entries = (process.env.PATH ?? '').split(delimiter).filter(Boolean);
  for (const entry of entries) {
    const candidate = join(entry, name);
    if (isExecutableFile(candidate)) return candidate;
  }
  return undefined;
}

/** {@link findBinary} 옵션. */
export interface FindBinaryOptions {
  /** 참이면 캐시를 무시하고 다시 탐색한다. */
  readonly refresh?: boolean;
}

/**
 * rhwp 실행 파일 경로를 돌려준다.
 *
 * @throws {BinaryNotFoundError} 세 경로 모두에서 찾지 못했을 때. 메시지에 시도한
 *   위치를 전부 담는다 — "없다"만 알려주면 사용자가 어디에 둬야 할지 모른다.
 */
export function findBinary(options: FindBinaryOptions = {}): string {
  if (cached !== undefined && !options.refresh) return cached;

  const tried: string[] = [];

  const fromEnvironment = fromEnv(); // 환경변수가 잘못되면 여기서 바로 던진다.
  if (fromEnvironment) {
    cached = fromEnvironment;
    return cached;
  }
  tried.push(`${ENV_VAR} (미설정)`);

  const bundled = fromBundle();
  if (bundled) {
    cached = bundled;
    return cached;
  }
  tried.push(`패키지 동봉 (${join(bundledDir(), binaryName())})`);

  const onPath = fromPath();
  if (onPath) {
    cached = onPath;
    return cached;
  }
  tried.push(`PATH (${binaryName()} 없음)`);

  throw new BinaryNotFoundError(
    'rhwp 실행 파일을 찾지 못했습니다. 다음 순서로 탐색했습니다:\n' +
      tried.map((t, i) => `  ${i + 1}. ${t}`).join('\n') +
      `\n\n해결: rhwp 를 설치해 PATH 에 두거나, ${ENV_VAR} 로 경로를 지정하세요.`,
  );
}
