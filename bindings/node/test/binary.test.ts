/**
 * 바이너리 탐색 계약 — **순서가 계약이다.**
 *
 * `RHWP_BIN` → 패키지 동봉 → `PATH`. 이 순서가 뒤집히면 개발자가 로컬 빌드를
 * 가리켜도 동봉본이 실행돼 "왜 내 수정이 반영 안 되지"라는 진단 불가 상황이 생긴다.
 * 그리고 환경변수를 **줬는데 못 쓰는** 경우 조용히 다음 후보로 넘어가면, 사용자는
 * 자기가 지정한 바이너리가 도는 줄 알면서 전혀 다른 결과를 본다.
 */

import { chmodSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { ENV_VAR, binaryName, clearBinaryCache, findBinary } from '../src/binary.js';
import { BinaryNotFoundError } from '../src/errors.js';

let root: string;
let savedBin: string | undefined;
let savedPath: string | undefined;
const HOME_VAR = process.platform === 'win32' ? 'USERPROFILE' : 'HOME';
let savedHome: string | undefined;

/** 예외를 값으로 잡는다 — 메시지까지 봐야 "어디에 두면 되는지" 계약을 확인할 수 있다. */
function capture(run: () => unknown): unknown {
  try {
    run();
  } catch (error) {
    return error;
  }
  throw new Error('예외가 발생해야 하는데 정상 반환했습니다');
}

/** 환경변수 복원 — 지워야 하는 것과 되돌려야 하는 것을 구분한다. */
function restoreEnv(key: string, value: string | undefined): void {
  if (value === undefined) delete process.env[key];
  else process.env[key] = value;
}

/** 플랫폼에 맞는 '실행 가능한' 더미 파일을 만든다. */
function makeExecutable(dir: string): string {
  mkdirSync(dir, { recursive: true });
  // 윈도우는 실행 비트가 없어 확장자로 판단하므로 이름이 곧 자격이다.
  const target = join(dir, binaryName());
  writeFileSync(target, '#!/bin/sh\nexit 0\n', 'utf8');
  if (process.platform !== 'win32') chmodSync(target, 0o755);
  return target;
}

beforeEach(() => {
  savedBin = process.env[ENV_VAR];
  savedPath = process.env['PATH'];
  savedHome = process.env[HOME_VAR];
  root = mkdtempSync(join(tmpdir(), 'rhwp-node-bin-'));
  delete process.env[ENV_VAR];
  // PATH 를 비워 둔다 — 개발 머신에 진짜 rhwp 가 설치돼 있으면 탐색 결과가
  // 머신 상태에 따라 달라진다. 테스트가 이 PC 에서만 다르게 도는 것이 가장 나쁘다.
  process.env['PATH'] = '';
  clearBinaryCache();
});

afterEach(() => {
  restoreEnv(ENV_VAR, savedBin);
  restoreEnv('PATH', savedPath);
  restoreEnv(HOME_VAR, savedHome);
  clearBinaryCache();
  rmSync(root, { recursive: true, force: true });
});

describe('binaryName', () => {
  it('플랫폼별 실행 파일 이름을 준다', () => {
    expect(binaryName()).toBe(process.platform === 'win32' ? 'rhwp.exe' : 'rhwp');
  });
});

describe('탐색 순서', () => {
  it('RHWP_BIN 이 최우선이다', () => {
    const target = makeExecutable(join(root, 'env'));
    process.env[ENV_VAR] = target;
    expect(findBinary()).toBe(resolve(target));
  });

  it('RHWP_BIN 이 PATH 를 이긴다 — 로컬 빌드를 가리킬 수 있어야 한다', () => {
    const wanted = makeExecutable(join(root, 'env'));
    const onPath = makeExecutable(join(root, 'path'));
    process.env[ENV_VAR] = wanted;
    process.env['PATH'] = join(root, 'path');

    expect(findBinary()).toBe(resolve(wanted));
    expect(findBinary()).not.toBe(onPath);
  });

  it('RHWP_BIN 이 없으면 PATH 에서 찾는다', () => {
    const onPath = makeExecutable(join(root, 'path'));
    process.env['PATH'] = join(root, 'path');
    expect(findBinary()).toBe(onPath);
  });

  it('PATH 의 앞 항목이 먼저다', () => {
    const first = makeExecutable(join(root, 'first'));
    makeExecutable(join(root, 'second'));
    process.env['PATH'] = [join(root, 'first'), join(root, 'second')].join(
      process.platform === 'win32' ? ';' : ':',
    );
    expect(findBinary()).toBe(first);
  });
});

describe('RHWP_BIN 해석', () => {
  it('디렉터리를 줘도 그 안의 실행 파일을 찾는다 — 흔한 사용 실수를 흡수한다', () => {
    const target = makeExecutable(join(root, 'bin'));
    process.env[ENV_VAR] = join(root, 'bin');
    expect(findBinary()).toBe(resolve(target));
  });

  it('절대 경로로 굳혀서 돌려준다 — 실행 시점의 cwd 에 결과가 흔들리면 안 된다', () => {
    const target = makeExecutable(join(root, 'env'));
    process.env[ENV_VAR] = target;
    const found = findBinary();
    expect(found).toBe(resolve(found));
  });

  it('앞뒤 공백은 무시한다', () => {
    const target = makeExecutable(join(root, 'env'));
    process.env[ENV_VAR] = `  ${target}  `;
    expect(findBinary()).toBe(resolve(target));
  });

  it('빈 문자열은 "미설정"으로 본다 — 빈 값 때문에 전체 탐색이 막히면 안 된다', () => {
    const onPath = makeExecutable(join(root, 'path'));
    process.env[ENV_VAR] = '   ';
    process.env['PATH'] = join(root, 'path');
    expect(findBinary()).toBe(onPath);
  });

  it('환경변수를 줬는데 못 쓰면 즉시 실패한다 — 조용히 다음으로 넘어가지 않는다', () => {
    // PATH 에는 멀쩡한 바이너리를 둔다. 그래도 잘못된 RHWP_BIN 이 이겨야 한다 —
    // 사용자는 자기가 지정한 바이너리가 돈다고 믿는데 다른 게 실행되면
    // 결과가 왜 다른지 설명할 방법이 없다.
    makeExecutable(join(root, 'path'));
    process.env['PATH'] = join(root, 'path');
    process.env[ENV_VAR] = join(root, '없는파일');

    const error = capture(() => findBinary());

    expect(error).toBeInstanceOf(BinaryNotFoundError);
    expect((error as BinaryNotFoundError).message).toContain(ENV_VAR);
    expect((error as BinaryNotFoundError).message).toContain('없는파일');
  });

  it('이름만 같은 디렉터리를 실행 파일로 착각하지 않는다', () => {
    // `<root>/rhwp` 라는 **디렉터리**를 만든다. stat 만 보고 넘기면 여기서 걸린다.
    mkdirSync(join(root, binaryName()), { recursive: true });
    process.env[ENV_VAR] = root;
    expect(() => findBinary()).toThrow(BinaryNotFoundError);
  });

  it('[D-16] 선행 ~ 를 홈 디렉터리로 확장한다', () => {
    // 파이썬 _binary.py:70 의 Path(raw).expanduser() 와 대칭 — RHWP_BIN=~/bin/rhwp
    // 가 파이썬에서는 됐지만 Node 의 resolve() 는 ~ 를 리터럴로 다뤄 안 됐다.
    process.env[HOME_VAR] = root;
    const target = makeExecutable(join(root, 'bin'));
    process.env[ENV_VAR] = '~/bin/' + binaryName();
    expect(findBinary()).toBe(resolve(target));
  });

  it('[D-16] 단독 ~ 하나도 홈 디렉터리로 확장한다', () => {
    process.env[HOME_VAR] = root;
    const target = makeExecutable(root);
    process.env[ENV_VAR] = '~';
    expect(findBinary()).toBe(resolve(target));
  });
});

describe('못 찾았을 때', () => {
  it('시도한 세 경로를 전부 알려준다', () => {
    // "없다"만 알려주면 사용자가 어디에 둬야 할지 모른다. 메시지가 곧 해결책이어야 한다.
    const error = capture(() => findBinary());

    expect(error).toBeInstanceOf(BinaryNotFoundError);
    const message = (error as BinaryNotFoundError).message;
    expect(message).toContain(ENV_VAR);
    expect(message).toContain('동봉');
    expect(message).toContain('PATH');
    expect(message).toContain(binaryName());
  });
});

describe('캐시', () => {
  it('한 번 찾으면 다시 훑지 않는다', () => {
    const target = makeExecutable(join(root, 'env'));
    process.env[ENV_VAR] = target;
    const first = findBinary();

    // 환경변수를 지워도 캐시가 살아 있어야 한다 — 명령 하나마다 PATH 를 훑을 이유가 없다.
    delete process.env[ENV_VAR];
    expect(findBinary()).toBe(first);
  });

  it('refresh 는 캐시를 우회한다 — 테스트와 재설치 시나리오가 여기에 걸린다', () => {
    const first = makeExecutable(join(root, 'a'));
    const second = makeExecutable(join(root, 'b'));

    process.env[ENV_VAR] = first;
    expect(findBinary()).toBe(resolve(first));

    process.env[ENV_VAR] = second;
    expect(findBinary()).toBe(resolve(first)); // 캐시가 그대로
    expect(findBinary({ refresh: true })).toBe(resolve(second));
  });

  it('clearBinaryCache 가 다음 호출을 다시 탐색하게 만든다', () => {
    const target = makeExecutable(join(root, 'env'));
    process.env[ENV_VAR] = target;
    expect(findBinary()).toBe(resolve(target));

    delete process.env[ENV_VAR];
    clearBinaryCache();
    // 캐시가 비었으니 이제는 진짜로 못 찾아야 한다.
    expect(() => findBinary()).toThrow(BinaryNotFoundError);
  });
});
