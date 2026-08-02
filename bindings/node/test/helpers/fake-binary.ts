/**
 * 시나리오대로 행동하는 가짜 rhwp — 프로세스 계약을 **바이너리 없이** 검증한다.
 *
 * 실물 rhwp 를 요구하면 바인딩 기여자가 Rust 툴체인을 갖춰야 하고, 그러면 기여가
 * 줄고 줄어든 기여는 곧 뒤처짐이다(DESIGN D10). 종료 코드·봉투·NDJSON 계약은
 * 순수 프로토콜이므로 흉내 낼 수 있다.
 *
 * ## 인코딩 — 이 픽스처의 가장 중요한 규칙
 *
 * 실물 rhwp(Rust)는 콘솔 코드페이지와 **무관하게 항상 UTF-8 바이트**를 낸다.
 * 픽스처가 플랫폼 기본 인코딩을 따르면 윈도우에서만 한글이 깨지고, 그 깨짐이
 * "바인딩 버그"로 오인된다(파이썬판이 이 함정에 두 번 걸렸다). 그래서 이 스크립트는
 * stdout·stderr 에 문자열을 넘기지 않고 **`Buffer.from(text, 'utf8')` 로 명시 인코딩한**
 * 바이트를 쓰고, stdin 도 받은 바이트를 명시적으로 UTF-8 로 해석한다.
 *
 * ## 왜 윈도우에서 `.cmd` 래퍼가 아닌가
 *
 * 파이썬판 픽스처는 `rhwp.cmd` 배치 래퍼를 썼다. Node 에서는 쓸 수 없다 —
 * `child_process.spawn` 은 `shell: true` 없이 `.bat`/`.cmd` 를 실행하지 못하는데
 * (Node 문서 "Spawning .bat and .cmd files on Windows"; 최신 Node 는 `EINVAL` 로
 * 거부한다), `src/process.ts` 는 인용 규칙 때문에 `shell: false` 를 고정하고 있다.
 * 그래서 윈도우에서는 **node 실행 파일 자체를 rhwp 로 삼고** 스크립트 경로를 첫 인자로
 * 앞세운다. 어느 쪽이든 자식이 보는 `process.argv.slice(2)` 는 `[시나리오, ...나머지]`
 * 로 같아서 스크립트 본문은 한 벌이다.
 *
 * @packageDocumentation
 */

import { chmodSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

/**
 * 가짜 rhwp 가 흉내 내는 상황들.
 *
 * 유니온으로 고정하는 이유: 오타 난 시나리오 이름은 `default` 분기로 떨어져
 * exit 1 이 되는데, 그러면 "런타임 실패 경로가 잘 도네"라고 **테스트가 거짓 통과**한다.
 */
export type Scenario =
  /** 성공 + 봉투 하나. */
  | 'ok'
  /** 검증 단언 실패 — exit 3 인데 판정 근거가 담긴 봉투가 나온다. */
  | 'verdict'
  /** 페이지 수 불일치 — exit 4. */
  | 'pages'
  /** 사용법 오류 — exit 2 + `힌트:` 줄. */
  | 'usage'
  /** 런타임 실패 — exit 1. */
  | 'runtime'
  /** stdout 이 JSON 이 아니다. */
  | 'garbage'
  /** stdout 이 JSON 이지만 객체가 아니다(배열). */
  | 'array'
  /** 성공했는데 stdout 이 0바이트 — 봉투 계약 위반. */
  | 'empty'
  /** NDJSON 3줄. */
  | 'ndjson'
  /** NDJSON 성공 1 + 실패 1, exit 1 (부분 실패). */
  | 'ndjson-partial'
  /** NDJSON 3줄 뒤 잠시 살아 있다가 마커 파일을 쓴다 — 자식 정리 관측용. */
  | 'ndjson-marker'
  /** 사전에 없는 종료 코드. */
  | 'unknown-exit'
  /** 끝나지 않는다 — 타임아웃 유발. */
  | 'slow'
  /** 받은 인자를 그대로 봉투에 실어 돌려준다. */
  | 'argv'
  /** stdin 을 UTF-8 로 읽어 그대로 돌려준다. */
  | 'stdin-echo';

/** 가짜 실행 파일 한 벌. */
export interface FakeBinary {
  /** `RHWP_BIN` 에 넣을 경로. */
  readonly path: string;
  /** 픽스처 임시 디렉터리 — 마커 파일 같은 부산물을 여기 둔다. */
  readonly dir: string;
  /**
   * 시나리오 호출 인자.
   *
   * 플랫폼별 실행 방식 차이(윈도우는 `node <script>` 접두)를 여기서 흡수하므로
   * 테스트 본문은 어디서나 같다.
   */
  args(scenario: Scenario, ...rest: string[]): string[];
  /** 임시 디렉터리 정리. */
  dispose(): void;
}

/**
 * 가짜 rhwp 본문.
 *
 * `String.raw` 인 이유: 이 안의 `\n` 은 **생성되는 JS 파일에 그대로 남아야 하는**
 * 두 글자다. 보통 템플릿 리터럴이면 TS 가 여기서 줄바꿈으로 바꿔 버려, 자식이
 * 문자열 안에 진짜 줄바꿈을 담은 채로 실행된다.
 *
 * `process.exit()` 를 쓰지 않고 `process.exitCode` 만 정하는 것도 계약이다 —
 * 파이프로 나가는 stdout 은 비동기라 `exit()` 가 봉투를 잘라먹을 수 있고, 잘린 봉투는
 * `ProtocolError` 로 나타나 "바인딩이 JSON 을 못 읽는다"로 오진된다.
 */
const FAKE_SCRIPT = String.raw`'use strict';

const fs = require('node:fs');

// 문자열을 그대로 넘기지 않는다 — 인코딩을 플랫폼에 맡기지 않겠다는 뜻이다.
function out(text) { process.stdout.write(Buffer.from(text, 'utf8')); }
function err(text) { process.stderr.write(Buffer.from(text, 'utf8')); }
function line(value) { out(JSON.stringify(value) + '\n'); }
function done(code) { process.exitCode = code; }

const argv = process.argv.slice(2);
const scenario = argv.length > 0 ? argv[0] : 'ok';
const rest = argv.slice(1);

switch (scenario) {
  case 'ok':
    line({ schemaVersion: '1.0', ok: true, note: '한글도 UTF-8 로 나간다' });
    done(0);
    break;

  case 'verdict':
    line({ schemaVersion: '1.0', output: 'a.hwp', verify: { identical: false, diffCount: 3 } });
    done(3);
    break;

  case 'pages':
    line({ schemaVersion: '1.0', pageCount: 2, expectedPageCount: 3 });
    done(4);
    break;

  case 'usage':
    err("오류: 알 수 없는 명령입니다\n힌트: 가장 가까운 명령은 'export-svg' 입니다\n");
    done(2);
    break;

  case 'runtime':
    err('오류: 파일을 읽을 수 없습니다\n');
    done(1);
    break;

  case 'garbage':
    out('이건 JSON 이 아니다\n');
    done(0);
    break;

  case 'array':
    line([1, 2, 3]);
    done(0);
    break;

  case 'empty':
    done(0);
    break;

  case 'ndjson':
    for (let i = 0; i < 3; i += 1) {
      line({ schemaVersion: '1.0', source: 'f' + i + '.hwp', pageCount: i + 1 });
    }
    done(0);
    break;

  case 'ndjson-partial':
    line({ schemaVersion: '1.0', source: 'ok.hwp', pageCount: 1 });
    line({ schemaVersion: '1.0', source: 'bad.hwp', error: '읽기 실패' });
    done(1);
    break;

  case 'ndjson-marker': {
    const marker = rest[0];
    for (let i = 0; i < 3; i += 1) {
      line({ schemaVersion: '1.0', source: 'f' + i + '.hwp', pageCount: i + 1 });
    }
    // 소비자가 중간에 멈췄다면 부모가 여기까지 오기 전에 죽인다.
    setTimeout(function () {
      fs.writeFileSync(marker, '자식이 살아남았다', 'utf8');
      done(0);
    }, 400);
    break;
  }

  case 'unknown-exit':
    done(42);
    break;

  case 'slow':
    // 이벤트 루프를 붙잡아 둔다 — 부모의 제한 시간이 먼저 끝나야 한다.
    setTimeout(function () { done(0); }, 60000);
    break;

  case 'argv':
    line({ schemaVersion: '1.0', argv: rest });
    done(0);
    break;

  case 'stdin-echo': {
    const chunks = [];
    process.stdin.on('data', function (chunk) { chunks.push(chunk); });
    process.stdin.on('end', function () {
      // 받은 바이트를 명시적으로 UTF-8 로 해석한다.
      line({ schemaVersion: '1.0', stdin: Buffer.concat(chunks).toString('utf8') });
      done(0);
    });
    break;
  }

  default:
    err('알 수 없는 시나리오: ' + scenario + '\n');
    done(1);
}
`;

/**
 * shebang 으로 실행 파일을 흉내 낼 수 있는 환경인가.
 *
 * 윈도우는 애초에 불가하고, node 경로에 공백이 있거나 너무 길면 shebang 이 깨진다
 * (리눅스의 `#!` 줄 길이 제한은 127바이트다). 그런 환경에서는 node 를 직접 띄운다 —
 * 픽스처가 깨져서 나는 실패는 바인딩 결함으로 오인되기 때문에, 애매하면 확실한 쪽을 쓴다.
 */
function canUseShebang(): boolean {
  if (process.platform === 'win32') return false;
  if (process.execPath.includes(' ')) return false;
  return `#!${process.execPath}`.length <= 100;
}

/**
 * 가짜 rhwp 한 벌을 임시 디렉터리에 만든다.
 *
 * 호출자는 `path` 를 `RHWP_BIN` 에 물리고, 끝나면 {@link FakeBinary.dispose} 를 부른다.
 */
export function createFakeBinary(): FakeBinary {
  const dir = mkdtempSync(join(tmpdir(), 'rhwp-node-fake-'));

  // 확장자 없는 실행 파일과 `.cjs` 를 모두 CommonJS 로 못 박는다. 임시 디렉터리
  // 상위에 누군가 `"type": "module"` 을 둔 package.json 이 있으면 픽스처가
  // "require is not defined" 로 죽는데, 그 실패는 바인딩과 아무 관계가 없다.
  writeFileSync(join(dir, 'package.json'), '{ "type": "commonjs" }\n', 'utf8');

  const script = join(dir, 'fake-rhwp.cjs');
  writeFileSync(script, FAKE_SCRIPT, 'utf8');

  if (!canUseShebang()) {
    return {
      path: process.execPath,
      dir,
      args: (scenario, ...rest) => [script, scenario, ...rest],
      dispose: () => rmSync(dir, { recursive: true, force: true }),
    };
  }

  // POSIX 에서는 실물과 같은 이름의 실행 파일로 둔다 — `RHWP_BIN` 이 진짜 rhwp 를
  // 가리키는 상황과 최대한 같게 만든다.
  const launcher = join(dir, 'rhwp');
  writeFileSync(
    launcher,
    `#!${process.execPath}\nrequire(${JSON.stringify(script)});\n`,
    'utf8',
  );
  chmodSync(launcher, 0o755);

  return {
    path: launcher,
    dir,
    args: (scenario, ...rest) => [scenario, ...rest],
    dispose: () => rmSync(dir, { recursive: true, force: true }),
  };
}
