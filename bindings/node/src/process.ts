/**
 * rhwp 프로세스 실행 — 봉투 계약을 지키는 얇은 껍데기.
 *
 * 계약 요지 (`--json` 모드):
 *
 * - stdout 은 **순수 JSON**(배치는 NDJSON). 진단·진행·요약은 stderr.
 * - 실패 경로의 stdout 은 **0바이트** — 반쪽 JSON 을 흘리지 않는다.
 * - 종료 코드는 #2707 사전을 따른다 ({@link module:errors} 참조).
 *
 * 이 모듈은 그 계약을 신뢰하되 **검증한다**. 계약이 깨졌을 때 조용히 넘기면
 * 호출자는 빈 결과를 "차이 없음"으로 오독한다.
 *
 * @packageDocumentation
 */

import { spawn, type ChildProcessWithoutNullStreams } from 'node:child_process';

import { findBinary } from './binary.js';
import type { BatchRecord, RawEnvelope } from './envelope.js';
import {
  ProtocolError,
  RhwpError,
  RhwpTimeoutError,
  raiseForExit,
} from './errors.js';

/**
 * 기본 제한 시간(ms). 대형 문서 렌더가 수십 초 걸릴 수 있어 넉넉히 잡는다.
 * `null` 을 넘기면 무제한.
 */
export const DEFAULT_TIMEOUT_MS = 300_000;

/** 인자로 받을 수 있는 값. 불리언은 **값 위치에 올 수 없다**(플래그로 표현해야 한다). */
export type Argument = string | number;

/** 실행 옵션. */
export interface RunOptions {
  /** 표준 입력으로 흘려 넣을 문자열 (batch 파일 목록, 암호 등). */
  readonly stdin?: string | undefined;
  /** 제한 시간(ms). `null` 이면 무제한. */
  readonly timeoutMs?: number | null | undefined;
  /** 작업 디렉터리. */
  readonly cwd?: string | undefined;
  /** exit 3/4 도 예외로 올릴지. 기본은 판정을 값으로 다룬다. */
  readonly throwOnVerdict?: boolean | undefined;
}

/** 실행 결과 원문. */
export interface CompletedRun {
  /** 실제 실행한 명령줄. */
  readonly argv: readonly string[];
  /** 종료 코드. */
  readonly exitCode: number;
  /** 표준 출력. */
  readonly stdout: string;
  /** 표준 오류. */
  readonly stderr: string;
}

/** 인자 하나를 문자열로. */
function stringify(value: Argument): string {
  if (typeof value === 'boolean') {
    // 불리언이 "true"/"false" 로 나가면 CLI 가 못 읽는다. 플래그로 표현해야 하므로
    // 값 위치에 오면 호출 조립 버그다.
    throw new TypeError('불리언은 인자 값이 될 수 없습니다 — 플래그로 표현하세요');
  }
  return String(value);
}

/** 자식 프로세스를 띄우고 전체 출력을 모은다. */
function spawnCollected(
  argv: readonly string[],
  options: RunOptions,
): Promise<CompletedRun> {
  const timeoutMs =
    options.timeoutMs === null
      ? null
      : (options.timeoutMs ?? DEFAULT_TIMEOUT_MS);

  return new Promise<CompletedRun>((resolve, reject) => {
    let child: ChildProcessWithoutNullStreams;
    try {
      child = spawn(argv[0] as string, argv.slice(1), {
        cwd: options.cwd,
        // 실행 파일 경로는 우리가 탐색한 것이므로 셸을 태우지 않는다 —
        // 셸을 거치면 윈도우 인용 규칙 때문에 한글 경로가 깨진다.
        shell: false,
        windowsHide: true,
      }) as ChildProcessWithoutNullStreams;
    } catch (cause) {
      reject(new RhwpError(`rhwp 실행에 실패했습니다: ${String(cause)}`, { argv, cause }));
      return;
    }

    const stdoutChunks: Buffer[] = [];
    const stderrChunks: Buffer[] = [];
    let settled = false;
    let timer: NodeJS.Timeout | undefined;

    const finish = (fn: () => void): void => {
      if (settled) return;
      settled = true;
      if (timer) clearTimeout(timer);
      fn();
    };

    child.stdout.on('data', (chunk: Buffer) => stdoutChunks.push(chunk));
    child.stderr.on('data', (chunk: Buffer) => stderrChunks.push(chunk));

    child.on('error', (cause) => {
      finish(() =>
        reject(new RhwpError(`rhwp 실행에 실패했습니다: ${cause.message}`, { argv, cause })),
      );
    });

    child.on('close', (code) => {
      finish(() =>
        resolve({
          argv,
          exitCode: code ?? 1,
          // 봉투는 UTF-8 이 계약이다. 잘못된 바이트가 섞여도 죽지 않고 치환하되,
          // 그 경우 JSON 파싱이 실패해 ProtocolError 로 드러난다.
          stdout: Buffer.concat(stdoutChunks).toString('utf8'),
          stderr: Buffer.concat(stderrChunks).toString('utf8'),
        }),
      );
    });

    if (timeoutMs !== null) {
      timer = setTimeout(() => {
        child.kill('SIGKILL');
        finish(() =>
          reject(
            new RhwpTimeoutError(`제한 시간 ${timeoutMs}ms 를 초과했습니다`, {
              argv,
              stderr: Buffer.concat(stderrChunks).toString('utf8'),
            }),
          ),
        );
      }, timeoutMs);
      // 타이머가 이벤트 루프를 붙잡지 않게 한다.
      timer.unref?.();
    }

    if (options.stdin !== undefined) {
      child.stdin.on('error', () => {
        // 자식이 stdin 을 읽기 전에 죽으면 EPIPE 가 난다. close 핸들러가 실제
        // 실패를 보고하므로 여기서는 삼킨다.
      });
      child.stdin.end(options.stdin, 'utf8');
    } else {
      child.stdin.end();
    }
  });
}

/**
 * rhwp 를 실행하고 원문 결과를 돌려준다.
 *
 * @param args - 실행 인자 (프로그램 이름 제외).
 * @param options - 실행 옵션. `check` 는 없다 — 검사는 호출자가
 *   {@link raiseForExit} 로 명시한다.
 */
export async function runRaw(
  args: readonly Argument[],
  options: RunOptions & { readonly check?: boolean } = {},
): Promise<CompletedRun> {
  const binary = findBinary();
  const argv = [binary, ...args.map(stringify)];
  const result = await spawnCollected(argv, options);

  if (options.check !== false) {
    raiseForExit(result.exitCode, {
      argv: result.argv,
      stderr: result.stderr,
      throwOnVerdict: options.throwOnVerdict ?? false,
    });
  }
  return result;
}

/**
 * `--json` 명령을 실행하고 봉투를 돌려준다.
 *
 * 종료 코드 검사는 **파싱 뒤**에 한다 — exit 3(판정 실패)일 때도 봉투가 나오고,
 * 그 봉투에 판정 근거가 들어 있기 때문이다. 순서를 뒤집으면 가장 중요한 정보를
 * 버리게 된다.
 *
 * @throws {ProtocolError} stdout 이 JSON 이 아니거나, 성공했는데 비어 있을 때.
 */
export async function runJson<T extends RawEnvelope = RawEnvelope>(
  args: readonly Argument[],
  options: RunOptions = {},
): Promise<T> {
  const result = await runRaw(args, { ...options, check: false });

  let envelope: T | undefined;
  const text = result.stdout.trim();
  if (text) {
    let parsed: unknown;
    try {
      parsed = JSON.parse(text);
    } catch (cause) {
      throw new ProtocolError(`stdout 이 순수 JSON 이 아닙니다: ${String(cause)}`, {
        argv: result.argv,
        exitCode: result.exitCode,
        stderr: result.stderr,
        cause,
      });
    }
    if (parsed === null || typeof parsed !== 'object' || Array.isArray(parsed)) {
      throw new ProtocolError(
        `봉투는 JSON 객체여야 합니다 (받음: ${Array.isArray(parsed) ? 'array' : typeof parsed})`,
        { argv: result.argv, exitCode: result.exitCode, stderr: result.stderr },
      );
    }
    envelope = parsed as T;
  }

  // 봉투를 예외에 실어 판정 근거를 보존한 채로 코드 검사.
  raiseForExit(result.exitCode, {
    argv: result.argv,
    stderr: result.stderr,
    envelope,
    throwOnVerdict: options.throwOnVerdict ?? false,
  });

  if (envelope === undefined) {
    // 성공(또는 판정 실패)인데 stdout 이 비었다 = 계약 위반.
    throw new ProtocolError('성공했는데 stdout 이 비어 있습니다 — --json 봉투 계약 위반입니다', {
      argv: result.argv,
      exitCode: result.exitCode,
      stderr: result.stderr,
    });
  }
  return envelope;
}

/** NDJSON 한 줄을 파싱한다. */
function parseLine(
  line: string,
  lineNo: number,
  argv: readonly string[],
  exitCode: number,
  stderr: string,
): Record<string, unknown> | undefined {
  const trimmed = line.trim();
  if (!trimmed) return undefined;
  let parsed: unknown;
  try {
    parsed = JSON.parse(trimmed);
  } catch (cause) {
    throw new ProtocolError(`NDJSON ${lineNo}번째 줄이 JSON 이 아닙니다: ${String(cause)}`, {
      argv,
      exitCode,
      stderr,
      cause,
    });
  }
  if (parsed === null || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new ProtocolError(`NDJSON ${lineNo}번째 줄이 객체가 아닙니다`, {
      argv,
      exitCode,
      stderr,
    });
  }
  return parsed as Record<string, unknown>;
}

/**
 * batch 계열을 실행하고 NDJSON 레코드 목록을 돌려준다.
 *
 * batch 는 **부분 실패도 실패**다 — 성공 레코드는 스트림에 남고 종료 코드가
 * 신호한다. 그래서 여기서는 exit 1 을 예외로 올리지 않고, 레코드에 담긴 `error`
 * 필드를 호출자가 보게 한다. 스트림을 통째로 버리면 성공분까지 잃는다.
 */
export async function runNdjson<T extends BatchRecord = BatchRecord>(
  args: readonly Argument[],
  options: RunOptions = {},
): Promise<T[]> {
  const result = await runRaw(args, { ...options, check: false });

  const records: T[] = [];
  const lines = result.stdout.split('\n');
  for (let i = 0; i < lines.length; i += 1) {
    const record = parseLine(
      lines[i] as string,
      i + 1,
      result.argv,
      result.exitCode,
      result.stderr,
    );
    if (record) records.push(record as T);
  }

  // 사용법 오류(2)는 스트림이 아예 성립하지 않은 것이므로 예외.
  if (result.exitCode === 2) {
    raiseForExit(2, { argv: result.argv, stderr: result.stderr });
  }
  return records;
}

/**
 * NDJSON 을 **스트리밍**으로 읽는다 — 대량 배치에서 메모리를 아낀다.
 *
 * 전량을 모으는 {@link runNdjson} 과 달리 레코드가 나오는 대로 넘긴다.
 * 소비자가 중간에 멈추면(`break`) 자식 프로세스도 정리한다 — 남으면 파일을
 * 잡고 있어 다음 작업이 막힌다.
 */
export async function* iterNdjson<T extends BatchRecord = BatchRecord>(
  args: readonly Argument[],
  options: RunOptions = {},
): AsyncIterableIterator<T> {
  const binary = findBinary();
  const argv = [binary, ...args.map(stringify)];

  const child = spawn(argv[0] as string, argv.slice(1), {
    cwd: options.cwd,
    shell: false,
    windowsHide: true,
  }) as ChildProcessWithoutNullStreams;

  if (options.stdin !== undefined) {
    child.stdin.on('error', () => {
      /* 자식이 먼저 죽으면 EPIPE — close 가 실제 실패를 보고한다. */
    });
    child.stdin.end(options.stdin, 'utf8');
  } else {
    child.stdin.end();
  }

  let buffer = '';
  let lineNo = 0;
  try {
    child.stdout.setEncoding('utf8');
    for await (const chunk of child.stdout) {
      buffer += chunk as string;
      let index = buffer.indexOf('\n');
      while (index >= 0) {
        const line = buffer.slice(0, index);
        buffer = buffer.slice(index + 1);
        lineNo += 1;
        const record = parseLine(line, lineNo, argv, 0, '');
        if (record) yield record as T;
        index = buffer.indexOf('\n');
      }
    }
    if (buffer.trim()) {
      lineNo += 1;
      const record = parseLine(buffer, lineNo, argv, 0, '');
      if (record) yield record as T;
    }
  } finally {
    // 소비자가 break 로 빠져나가도 자식이 남지 않게 한다.
    if (child.exitCode === null && child.signalCode === null) {
      child.kill('SIGKILL');
    }
  }
}
