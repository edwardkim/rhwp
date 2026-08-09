/**
 * 프로세스 실행 계약 — 가짜 바이너리로 각 종료 코드 경로를 검증한다.
 *
 * 이 모듈은 봉투 계약을 **신뢰하되 검증한다**. 계약이 깨졌는데 조용히 넘기면
 * 호출자는 빈 결과를 "차이 없음"으로, 반쪽 JSON 을 "성공"으로 오독한다. 그래서
 * 여기서 확인하는 것은 "잘 되는 경우"가 아니라 **깨졌을 때 시끄러운가**이다.
 */

import { existsSync } from 'node:fs';
import { join } from 'node:path';

import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import { ENV_VAR, clearBinaryCache } from '../src/binary.js';
import {
  ProtocolError,
  RhwpRuntimeError,
  RhwpTimeoutError,
  UsageError,
  VerdictFailed,
} from '../src/errors.js';
import { iterNdjson, runJson, runNdjson, runRaw } from '../src/process.js';
import type { Argument } from '../src/process.js';
import { createFakeBinary, type FakeBinary } from './helpers/fake-binary.js';

let fake: FakeBinary;
let savedBin: string | undefined;

beforeAll(() => {
  savedBin = process.env[ENV_VAR];
  fake = createFakeBinary();
  process.env[ENV_VAR] = fake.path;
  clearBinaryCache();
});

afterAll(() => {
  if (savedBin === undefined) delete process.env[ENV_VAR];
  else process.env[ENV_VAR] = savedBin;
  clearBinaryCache();
  fake.dispose();
});

/** 거부를 값으로 잡는다 — 던져진 객체의 필드까지 봐야 계약을 확인할 수 있다. */
async function captureAsync(promise: Promise<unknown>): Promise<unknown> {
  try {
    await promise;
  } catch (error) {
    return error;
  }
  throw new Error('거부되어야 하는데 정상 이행했습니다');
}

/** 비동기 이터레이터를 전량 모은다. */
async function collect<T>(source: AsyncIterable<T>): Promise<T[]> {
  const out: T[] = [];
  for await (const item of source) out.push(item);
  return out;
}

function sleep(ms: number): Promise<void> {
  return new Promise<void>((resolve) => {
    setTimeout(resolve, ms);
  });
}

describe('runJson — 종료 코드별 경로', () => {
  it('성공하면 봉투를 돌려준다', async () => {
    const env = await runJson(fake.args('ok'));
    expect(env['schemaVersion']).toBe('1.0');
    expect(env['ok']).toBe(true);
  });

  it('한글이 UTF-8 로 온전히 건너온다', async () => {
    // 실물 rhwp 는 콘솔 코드페이지와 무관하게 UTF-8 을 낸다. 이 단언이 깨지는 곳은
    // 대개 바인딩이 아니라 인코딩 가정이며, 윈도우에서만 드러나 진단이 늦어진다.
    const env = await runJson(fake.args('ok'));
    expect(env['note']).toBe('한글도 UTF-8 로 나간다');
  });

  it('exit 2 는 UsageError 이고 힌트를 구조화해 준다', async () => {
    const error = await captureAsync(runJson(fake.args('usage')));
    expect(error).toBeInstanceOf(UsageError);
    expect((error as UsageError).suggestion).toBe("가장 가까운 명령은 'export-svg' 입니다");
  });

  it('exit 1 은 RhwpRuntimeError 이고 stderr 를 싣는다', async () => {
    const error = await captureAsync(runJson(fake.args('runtime')));
    expect(error).toBeInstanceOf(RhwpRuntimeError);
    expect((error as RhwpRuntimeError).stderr).toContain('읽을 수 없습니다');
  });

  it('exit 3 은 예외가 아니다 — 판정 근거가 담긴 봉투가 나온다', async () => {
    // 여기서 던지면 호출자는 catch 로 넘기고 diffCount 를 영영 읽지 않는다.
    const env = await runJson(fake.args('verdict'));
    expect(env['verify']).toEqual({ identical: false, diffCount: 3 });
  });

  it('exit 3 을 예외로 올리더라도 봉투를 잃지 않는다', async () => {
    const error = await captureAsync(runJson(fake.args('verdict'), { throwOnVerdict: true }));
    expect(error).toBeInstanceOf(VerdictFailed);
    const verdict = error as VerdictFailed;
    // 종료 코드 검사를 **파싱 뒤에** 하기 때문에 가능한 일이다. 순서를 뒤집으면
    // 가장 중요한 정보를 버린 채 예외만 남는다.
    expect(verdict.envelope?.['verify']).toEqual({ identical: false, diffCount: 3 });
  });

  it('exit 4 는 페이지 수 불일치로 구분된다', async () => {
    const error = await captureAsync(runJson(fake.args('pages'), { throwOnVerdict: true }));
    expect((error as VerdictFailed).isPageCountMismatch).toBe(true);
  });

  it('exit 4 도 기본으로는 봉투를 돌려준다', async () => {
    const env = await runJson(fake.args('pages'));
    expect(env['pageCount']).toBe(2);
  });

  it('사전에 없는 종료 코드를 그대로 보고한다', async () => {
    const error = await captureAsync(runJson(fake.args('unknown-exit')));
    expect(error).toBeInstanceOf(RhwpRuntimeError);
    expect((error as RhwpRuntimeError).message).toContain('42');
  });
});

describe('runJson — 봉투 계약 위반', () => {
  it('stdout 이 JSON 이 아니면 ProtocolError', async () => {
    const error = await captureAsync(runJson(fake.args('garbage')));
    expect(error).toBeInstanceOf(ProtocolError);
    expect((error as ProtocolError).message).toContain('순수 JSON');
  });

  it('JSON 이지만 객체가 아니면 ProtocolError', async () => {
    // 배열을 봉투로 받으면 이후 `get('pageCount')` 가 이상한 곳에서 죽는다.
    const error = await captureAsync(runJson(fake.args('array')));
    expect(error).toBeInstanceOf(ProtocolError);
    expect((error as ProtocolError).message).toContain('객체');
  });

  it('성공했는데 stdout 이 비면 ProtocolError', async () => {
    // 빈 결과를 "처리할 게 없었다"로 넘기면 실패가 성공으로 보고된다.
    const error = await captureAsync(runJson(fake.args('empty')));
    expect(error).toBeInstanceOf(ProtocolError);
    expect((error as ProtocolError).message).toContain('비어 있습니다');
  });
});

describe('runRaw — 원문 접근과 인자 조립', () => {
  it('종료 코드와 두 스트림을 그대로 노출한다', async () => {
    const result = await runRaw(fake.args('runtime'), { check: false });
    expect(result.exitCode).toBe(1);
    expect(result.stderr).toContain('읽을 수 없습니다');
    // 실패 경로의 stdout 은 0바이트가 계약이다 — 반쪽 JSON 을 흘리지 않는다.
    expect(result.stdout).toBe('');
  });

  it('check 를 끄지 않으면 종료 코드를 검사한다', async () => {
    await expect(runRaw(fake.args('runtime'))).rejects.toBeInstanceOf(RhwpRuntimeError);
  });

  it('불리언은 인자 값이 될 수 없다', async () => {
    // "true"/"false" 로 나가면 CLI 가 못 읽는다. 플래그로 표현해야 하므로
    // 값 위치에 온 불리언은 호출 조립 버그이고, 실행 전에 잡아야 한다.
    const error = await captureAsync(
      runRaw([...fake.args('ok'), true as unknown as Argument], { check: false }),
    );
    expect(error).toBeInstanceOf(TypeError);
    expect((error as TypeError).message).toContain('플래그');
  });

  it('숫자·공백·한글 인자가 그대로 자식에게 닿는다', async () => {
    // 셸을 태우지 않는 이유가 여기 있다. 셸을 거치면 윈도우 인용 규칙 때문에
    // 공백 있는 한글 경로가 조용히 두 인자로 쪼개진다.
    const env = await runJson([...fake.args('argv', '공백 있는 문서.hwp'), 7]);
    expect(env['argv']).toEqual(['공백 있는 문서.hwp', '7']);
  });

  it('argv 에 실제 실행한 명령줄이 남는다', async () => {
    const result = await runRaw(fake.args('ok'), { check: false });
    expect(result.argv[0]).toBe(fake.path);
    expect(result.argv.slice(1)).toEqual(fake.args('ok'));
  });

  it('stdin 을 UTF-8 로 흘려 넣는다', async () => {
    const payload = '가나다\n라마바';
    const env = await runJson(fake.args('stdin-echo'), { stdin: payload });
    expect(env['stdin']).toBe(payload);
  });

  it('제한 시간을 넘기면 자식을 죽이고 RhwpTimeoutError 를 던진다', async () => {
    const error = await captureAsync(
      runRaw(fake.args('slow'), { timeoutMs: 300, check: false }),
    );
    expect(error).toBeInstanceOf(RhwpTimeoutError);
    expect((error as RhwpTimeoutError).message).toContain('300ms');
  });

  it('[D-20] envelopeHint 를 주면 실패 예외에 봉투가 실린다', async () => {
    // runJson 은 stdout 을 직접 파싱해 자동으로 봉투를 실어 보내지만, runRaw 는
    // 원문만 돌려주므로 호출자가 미리 파싱해 둔 봉투가 있으면 여기로 넘겨야
    // 판정 근거가 예외에서 빠지지 않는다 — 파이썬 `_process.py` 의
    // `envelope_hint` 와 대칭.
    const hint = { verify: { identical: false, diffCount: 3 } };
    const error = await captureAsync(
      runRaw(fake.args('runtime'), { envelopeHint: hint }),
    );
    expect(error).toBeInstanceOf(RhwpRuntimeError);
    expect((error as RhwpRuntimeError).envelope).toEqual(hint);
  });

  it('[D-20] envelopeHint 를 안 주면 예외의 봉투는 undefined 다 (기존 동작 불변)', async () => {
    const error = await captureAsync(runRaw(fake.args('runtime')));
    expect((error as RhwpRuntimeError).envelope).toBeUndefined();
  });
});

describe('runNdjson — 배치는 부분 실패도 실패다', () => {
  it('모든 레코드를 돌려준다', async () => {
    const records = await runNdjson(fake.args('ndjson'));
    expect(records).toHaveLength(3);
    expect(records.map((r) => r['pageCount'])).toEqual([1, 2, 3]);
  });

  it('부분 실패에서도 성공 레코드를 버리지 않는다', async () => {
    // 스트림을 통째로 버리면 성공분까지 잃는다. 실패는 레코드의 `error` 필드로
    // 남아 있어야 호출자가 무엇이 빠졌는지 안다.
    const records = await runNdjson(fake.args('ndjson-partial'));
    expect(records).toHaveLength(2);
    expect(records[0]?.['error']).toBeUndefined();
    expect(records[1]?.['error']).toBe('읽기 실패');
  });

  it('exit 2 는 예외다 — 스트림이 아예 성립하지 않았다는 뜻이다', async () => {
    await expect(runNdjson(fake.args('usage'))).rejects.toBeInstanceOf(UsageError);
  });

  it('빈 스트림은 오류가 아니다 — 처리할 것이 없는 것과 실패는 다르다', async () => {
    expect(await runNdjson(fake.args('empty'))).toEqual([]);
  });

  it('중간 줄이 JSON 이 아니면 ProtocolError', async () => {
    await expect(runNdjson(fake.args('garbage'))).rejects.toBeInstanceOf(ProtocolError);
  });
});

describe('iterNdjson — 스트리밍과 자식 정리', () => {
  it('레코드가 나오는 대로 넘긴다', async () => {
    const seen: unknown[] = [];
    for await (const record of iterNdjson(fake.args('ndjson'))) {
      seen.push(record['pageCount']);
    }
    expect(seen).toEqual([1, 2, 3]);
  });

  it('스트리밍과 일괄 수집이 같은 결과를 낸다', async () => {
    const streamed = (await collect(iterNdjson(fake.args('ndjson')))).map((r) => r['source']);
    const collected = (await runNdjson(fake.args('ndjson'))).map((r) => r['source']);
    expect(streamed).toEqual(collected);
  });

  it('실패 레코드도 스트림에 남는다', async () => {
    const records = await collect(iterNdjson(fake.args('ndjson-partial')));
    expect(records).toHaveLength(2);
    expect(records.some((r) => r['error'] !== undefined)).toBe(true);
  });

  it('잘못된 줄을 그대로 흘려보내지 않는다', async () => {
    await expect(collect(iterNdjson(fake.args('garbage')))).rejects.toBeInstanceOf(ProtocolError);
  });

  it('끝까지 소비하면 자식이 제 수명을 다 산다 (대조군)', async () => {
    // 아래 "중단하면 자식을 정리한다"의 짝이다. 마커가 원래 안 만들어지는
    // 픽스처였다면 그 테스트는 아무것도 증명하지 않고 통과한다.
    const marker = join(fake.dir, 'marker-완주.txt');
    const records = await collect(iterNdjson(fake.args('ndjson-marker', marker)));
    expect(records).toHaveLength(3);
    expect(existsSync(marker)).toBe(true);
  });

  it('중단하면 자식을 정리한다 — 남으면 파일을 잡고 있어 다음 작업이 막힌다', async () => {
    const marker = join(fake.dir, 'marker-중단.txt');
    for await (const record of iterNdjson(fake.args('ndjson-marker', marker))) {
      expect(record['pageCount']).toBe(1);
      break; // 첫 레코드만 보고 중단
    }
    // 자식은 400ms 뒤 마커를 쓴다. 정리됐다면 그 시점이 오지 않는다.
    await sleep(1000);
    expect(existsSync(marker)).toBe(false);
  });
});
