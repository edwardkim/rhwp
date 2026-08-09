/**
 * 2층(세션)의 **프로토콜 취급** — 실물 rhwp 없이 가짜 `mcp-serve` 로 검증한다.
 *
 * 문서를 진짜로 여는 일은 통합 쪽(`session.integration.test.ts`)이 본다. 여기서
 * 고정하는 것은 그 위층, 즉 **JSON-RPC 를 어떻게 다루는가**다. 이 층이 조용히
 * 어긋나면 증상은 "가끔 엉뚱한 응답을 받는다"로 나타나고, 그건 실물 문서로
 * 재현하기 가장 어려운 종류의 버그다:
 *
 * - 알림 프레임을 응답으로 오인하면 **다음 호출의 결과가 한 칸씩 밀린다**
 * - 요청을 직렬화하지 않으면 id 대조가 무너져 **A 의 답이 B 에게 간다**
 * - 실패 경로에서 자식을 정리하지 않으면 서버가 남아 **다음 작업이 파일을 못 연다**
 *
 * ## 가짜 서버를 어떻게 띄우는가 (윈도우 제약이 설계를 정한다)
 *
 * `helpers/fake-binary.ts` 의 픽스처는 시나리오를 **인자**로 받는다. 세션은 그럴 수
 * 없다 — `Session` 이 argv 를 스스로 조립하고(`[binary, 'mcp-serve']`) 호출자가
 * 끼어들 자리가 없다. 그래서 그 문서가 설명한 윈도우 해법을 한 걸음 더 민다:
 *
 * - `RHWP_BIN` 을 **node 실행 파일 자체**로 둔다. `.cmd` 래퍼는 `shell: false` 에서
 *   띄울 수 없고(`session.ts` 는 인용 규칙 때문에 `shell: false` 를 고정한다),
 *   node 는 어느 플랫폼에서나 실행 파일이다.
 * - 가짜 서버 스크립트를 임시 디렉터리에 **`mcp-serve` 라는 이름**(확장자 없음)으로
 *   두고 `cwd` 를 그 디렉터리로 준다. 그러면 `node mcp-serve` 가 곧 우리 서버다.
 * - 시나리오는 `--profile` 로 고른다. `Session` 이 실제로 실어 보내는 유일한
 *   옵션이라, 시나리오가 골라지는 것 자체가 "profile 이 자식에게 닿는다"의 증명이다.
 *
 * 인코딩은 `fake-binary.ts` 의 규칙을 그대로 따른다 — stdout 은 `Buffer.from(…, 'utf8')`,
 * stdin 은 `setEncoding('utf8')`. 플랫폼 기본을 따르면 윈도우 cp949 에서만 한글이
 * 깨지고, 그 깨짐이 "바인딩 버그"로 오인된다.
 */

import { existsSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { afterAll, afterEach, beforeAll, describe, expect, it } from 'vitest';

import { ENV_VAR, clearBinaryCache } from '../src/binary.js';
import { Envelope } from '../src/envelope.js';
import { ProtocolError, RhwpTimeoutError, SessionClosedError, UsageError } from '../src/errors.js';
import { Document, Session, openDocument } from '../src/session.js';

// ── 가짜 mcp-serve ──────────────────────────────────────────────────────────

/**
 * 가짜 서버 본문.
 *
 * `String.raw` 인 이유는 픽스처와 같다 — 이 안의 `\n` 은 **생성되는 JS 파일에 그대로
 * 남아야 하는** 두 글자다. 보통 템플릿 리터럴이면 TS 가 여기서 줄바꿈으로 바꿔 버려,
 * 자식이 문자열 안에 진짜 줄바꿈을 담은 채로 실행된다.
 *
 * `process.exit()` 를 쓰지 않는 것도 픽스처와 같은 이유다. 파이프로 나가는 출력은
 * 비동기라 `exit()` 가 마지막 프레임을 잘라먹을 수 있고, 잘린 프레임은 `ProtocolError`
 * 로 나타나 "바인딩이 JSON 을 못 읽는다"로 오진된다.
 */
const FAKE_SERVER = String.raw`'use strict';

const fs = require('node:fs');
const path = require('node:path');

// 인코딩을 플랫폼에 맡기지 않는다 — 실물 rhwp 는 콘솔 코드페이지와 무관하게 UTF-8 이다.
function out(text) { process.stdout.write(Buffer.from(text, 'utf8')); }
function err(text) { process.stderr.write(Buffer.from(text, 'utf8')); }
function send(frame) { out(JSON.stringify(frame) + '\n'); }

const argv = process.argv.slice(2);
const flag = argv.indexOf('--profile');
const profile = flag >= 0 && argv.length > flag + 1 ? argv[flag + 1] : '';

// 받은 바이트를 명시적으로 UTF-8 로 해석한다 — 멀티바이트 경계도 StringDecoder 가 본다.
process.stdin.setEncoding('utf8');

if (profile === 'crash') {
  // 첫 요청을 받은 **뒤** 응답 없이 죽는다. 순서를 고정해야 부모가 stderr 를 반드시
  // 싣는다 — 기동 즉시 죽이면 부모가 쓰기도 전에 끝나 진단이 비는 경합이 생긴다.
  process.stdin.on('data', function () {
    err('치명적: 문서 서버가 응답 전에 죽었습니다\n');
    process.exitCode = 1;
    process.stdin.destroy();
  });
} else {
  runServer();
}

function runServer() {
  // 부모가 세션을 정리했다면 이 시점이 오지 않는다 — 누수의 물증.
  const marker = path.join(__dirname, 'leak-marker.txt');
  let markerTimer = null;
  if (profile === 'no-docid') {
    markerTimer = setTimeout(function () {
      fs.writeFileSync(marker, '서버가 살아남았다', 'utf8');
    }, 400);
  }

  let docSeq = 0;
  let inFlight = 0;
  let maxInFlight = 0;
  let lastOpen = null;
  let buffer = '';

  process.stdin.on('data', function (chunk) {
    buffer += chunk;
    let index = buffer.indexOf('\n');
    while (index >= 0) {
      const line = buffer.slice(0, index).trim();
      buffer = buffer.slice(index + 1);
      if (line) accept(line);
      index = buffer.indexOf('\n');
    }
  });

  process.stdin.on('end', function () {
    if (markerTimer) clearTimeout(markerTimer);
    process.exitCode = 0;
  });

  function accept(line) {
    let request;
    try {
      request = JSON.parse(line);
    } catch {
      err('요청이 JSON-RPC 프레임이 아닙니다\n');
      return;
    }
    // 동시 요청 수를 관측한다 — 부모가 직렬화하는지가 여기서 드러난다.
    inFlight += 1;
    if (inFlight > maxInFlight) maxInFlight = inFlight;
    setTimeout(function () {
      respond(request);
      inFlight -= 1;
    }, profile === 'slow-echo' ? 40 : 0);
  }

  function respond(request) {
    const id = request.id;
    const params = request.params || {};
    const name = params.name;
    const args = params.arguments || {};

    if (profile === 'notify') {
      // id 가 없는 진행 알림 — 응답이 아니다.
      send({ jsonrpc: '2.0', method: 'notifications/progress', params: { tool: name, progress: 1 } });
      // id 가 null 인 프레임도 응답이 아니다.
      send({ jsonrpc: '2.0', id: null, method: 'notifications/message', params: { level: 'info' } });
      // 우리가 기다리지 않는 id — 무시가 안전하다.
      send({ jsonrpc: '2.0', id: 999999, result: { structuredContent: { 유령: true } } });
    }

    if (profile === 'rpc-error') {
      send({ jsonrpc: '2.0', id: id, error: { code: -32601, message: '알 수 없는 도구입니다: ' + name } });
      return;
    }

    if (profile === 'reject') {
      send({ jsonrpc: '2.0', id: id, result: {
        isError: true,
        structuredContent: {
          schemaVersion: '1.0',
          error: name + ' 은(는) 지금 쓸 수 없습니다',
          nextCall: {
            name: 'hwp_doc_fields',
            arguments: { docId: args.docId },
            why: '누름틀 목록을 먼저 확인하세요',
          },
        },
      } });
      return;
    }

    // 열기 인자를 기억해 둔다 — openDocument 는 열기 봉투를 삼키므로, 나중 호출의
    // 응답에 실어 보내야 "무엇을 실어 보냈는지"를 밖에서 확인할 수 있다.
    if (name === 'hwp_open') lastOpen = args;
    const body = {
      schemaVersion: '1.0',
      tool: name,
      args: args,
      lastOpen: lastOpen,
      maxInFlight: maxInFlight,
    };
    if (name === 'hwp_open') {
      body.note = '한글도 UTF-8 로 나간다';
      if (profile !== 'no-docid') {
        docSeq += 1;
        body.docId = 'doc-' + docSeq;
      }
    }

    if (profile === 'text-json') {
      send({ jsonrpc: '2.0', id: id, result: { content: [{ type: 'text', text: JSON.stringify(body) }] } });
      return;
    }
    if (profile === 'text-plain') {
      send({ jsonrpc: '2.0', id: id, result: { content: [{ type: 'text', text: '평문 결과: ' + name }] } });
      return;
    }
    send({ jsonrpc: '2.0', id: id, result: { structuredContent: body } });
  }
}
`;

/** 가짜 서버 한 벌. */
interface FakeServer {
  /** 서버 스크립트가 놓인 디렉터리. 모든 세션의 `cwd` 가 된다. */
  readonly dir: string;
  /** 자식이 살아남으면 남기는 마커 파일 경로. */
  readonly markerPath: string;
  /** 마커를 지운다 — 누수 검사 짝이 서로를 오염시키지 않도록. */
  clearMarker(): void;
  dispose(): void;
}

function createFakeServer(): FakeServer {
  const dir = mkdtempSync(join(tmpdir(), 'rhwp-node-mcp-'));

  // 임시 디렉터리 상위에 `"type": "module"` 인 package.json 이 있으면 확장자 없는
  // 진입점이 ESM 으로 해석돼 "require is not defined" 로 죽는다 — 바인딩과 아무
  // 관계 없는 실패이므로 여기서 못 박는다.
  writeFileSync(join(dir, 'package.json'), '{ "type": "commonjs" }\n', 'utf8');
  // 파일 **이름이 곧 하위 명령**이다. `Session` 은 언제나 `mcp-serve` 를 첫 인자로
  // 넘기므로, cwd 안의 이 파일이 그대로 진입점이 된다.
  writeFileSync(join(dir, 'mcp-serve'), FAKE_SERVER, 'utf8');

  const markerPath = join(dir, 'leak-marker.txt');
  return {
    dir,
    markerPath,
    clearMarker: () => rmSync(markerPath, { force: true }),
    dispose: () => {
      try {
        rmSync(dir, { recursive: true, force: true });
      } catch (error) {
        // 자식이 아직 이 디렉터리를 cwd 로 잡고 있으면 윈도우에서 삭제가 막힌다.
        // 정리 실패로 테스트 결과를 뒤집지는 않는다 — 게이트의 신호가 흐려진다.
        console.warn('가짜 서버 정리 실패(무시):', String(error));
      }
    },
  };
}

// ── 준비·정리 ──────────────────────────────────────────────────────────────

let fake: FakeServer;
let savedBin: string | undefined;
/** 테스트가 직접 만든 세션 — 실패로 빠져나가도 반드시 닫는다. */
const live: Session[] = [];

beforeAll(() => {
  savedBin = process.env[ENV_VAR];
  fake = createFakeServer();
  // node 자신을 rhwp 로 삼는다 (파일 머리말 참고).
  process.env[ENV_VAR] = process.execPath;
  clearBinaryCache();
});

afterEach(async () => {
  // 자식이 남으면 다음 테스트의 마커 검사와 디렉터리 정리가 모두 오염된다.
  while (live.length > 0) await live.pop()?.close();
});

afterAll(() => {
  if (savedBin === undefined) delete process.env[ENV_VAR];
  else process.env[ENV_VAR] = savedBin;
  clearBinaryCache();
  fake.dispose();
});

// ── 도우미 ──────────────────────────────────────────────────────────────────

/** 시나리오를 물린 세션 하나. 정리는 `afterEach` 가 책임진다. */
function spawnSession(profile?: string): Session {
  const session = new Session({ cwd: fake.dir, profile });
  live.push(session);
  return session;
}

/** 거부를 값으로 잡는다 — 던져진 객체의 필드까지 봐야 계약을 확인할 수 있다. */
async function capture(promise: Promise<unknown>): Promise<unknown> {
  try {
    await promise;
  } catch (error) {
    return error;
  }
  throw new Error('거부되어야 하는데 정상 이행했습니다');
}

/** 가짜 서버가 되돌려준 도구 인자 원문. */
function argsOf(envelope: Envelope): Record<string, unknown> {
  return envelope.get<Record<string, unknown>>('args');
}

function sleep(ms: number): Promise<void> {
  return new Promise<void>((resolve) => {
    setTimeout(resolve, ms);
  });
}

// ── 문서 열기 ───────────────────────────────────────────────────────────────

describe('openDocument — 핸들 발급', () => {
  it('서버가 준 docId 로 Document 를 만든다', async () => {
    const doc = await openDocument('문서 이름.hwp', { cwd: fake.dir });
    try {
      expect(doc).toBeInstanceOf(Document);
      expect(doc.docId).toBe('doc-1');
      expect(doc.toString()).toBe('Document(doc-1, open)');
    } finally {
      await doc.close();
    }
    // 열림·닫힘이 표기에 드러나야 로그만 보고도 수명 문제를 짚을 수 있다.
    expect(doc.toString()).toBe('Document(doc-1, closed)');
  });

  it('경로는 그대로, 암호는 줬을 때만 실어 보낸다', async () => {
    // 공백이 든 한글 경로가 쪼개지면 셸을 태운 것이다(`shell: false` 계약 위반).
    // 그리고 언제나 `password: undefined` 를 보내면 서버가 "암호 있는 문서"로
    // 오해할 수 있다 — 없는 옵션은 키 자체가 없어야 한다.
    const plain = await openDocument('내 문서/보고서 최종.hwp', { cwd: fake.dir });
    try {
      expect((await plain.info()).get('lastOpen')).toEqual({ path: '내 문서/보고서 최종.hwp' });
    } finally {
      await plain.close();
    }

    const locked = await openDocument('보호문서.hwp', { cwd: fake.dir, password: '비밀번호' });
    try {
      expect((await locked.info()).get('lastOpen')).toEqual({
        path: '보호문서.hwp',
        password: '비밀번호',
      });
    } finally {
      await locked.close();
    }
  });

  it('docId 가 없으면 ProtocolError — 핸들 없는 Document 를 만들지 않는다', async () => {
    // 여기서 조용히 넘어가면 이후 모든 도구 호출이 `docId: undefined` 로 나가고,
    // 서버는 "그런 문서 없음"이라는 엉뚱한 진단을 돌려준다. 원인은 두 단계 앞이다.
    const session = spawnSession('no-docid');
    const error = await capture(openDocument('문서.hwp', { session }));

    expect(error).toBeInstanceOf(ProtocolError);
    expect((error as ProtocolError).message).toContain('docId');
    // 서버가 실제로 준 봉투를 진단에 담아야 무엇이 왔는지 알 수 있다.
    expect((error as ProtocolError).message).toContain('hwp_open');
  });
});

// ── 프레임 취급 ─────────────────────────────────────────────────────────────

describe('Session — JSON-RPC 프레임 취급', () => {
  it('도구 호출에 docId 를 자동으로 실어 보낸다', async () => {
    // 호출자가 매번 docId 를 손으로 넘겨야 한다면 2층은 1층보다 나을 게 없다.
    const doc = await openDocument('문서.hwp', { cwd: fake.dir });
    try {
      const fields = await doc.fields();
      expect(fields.get<string>('tool')).toBe('hwp_doc_fields');
      expect(argsOf(fields)).toEqual({ docId: doc.docId });
    } finally {
      await doc.close();
    }
  });

  it('알림 프레임을 건너뛰고 제 응답을 찾는다', async () => {
    // 서버는 진행 알림(id 없음), id 가 null 인 프레임, 우리가 기다리지 않는 id 를
    // 응답 앞에 흘린다. 하나라도 응답으로 오인하면 **다음 호출의 결과가 한 칸씩
    // 밀려** 오래도록 드러나지 않는 오염이 된다.
    const session = spawnSession('notify');
    const first = await session.call('hwp_doc_info', { docId: 'doc-1' });
    const second = await session.call('hwp_doc_fields', { docId: 'doc-1' });

    expect(first.get<string>('tool')).toBe('hwp_doc_info');
    expect(second.get<string>('tool')).toBe('hwp_doc_fields');
  });

  it('동시에 보낸 호출이 각자 제 응답을 받는다 — 요청은 큐로 직렬화된다', async () => {
    // 이 바인딩은 id 로만 응답을 대조한다. 그래서 요청을 직렬화하지 않으면 느린
    // 응답과 빠른 응답이 엇갈리는 순간 A 의 답이 B 에게 간다.
    const session = spawnSession('slow-echo');
    const results = await Promise.all([
      session.call('hwp_doc_text', { page: 0 }),
      session.call('hwp_doc_text', { page: 1 }),
      session.call('hwp_doc_text', { page: 2 }),
    ]);

    expect(results.map((r) => argsOf(r)['page'])).toEqual([0, 1, 2]);
    // 서버가 관측한 동시 요청 수. 2 이상이면 큐가 무너진 것이고, 그 뒤로 id 대조는
    // 운에 맡겨진다 — 단언을 여기 둬야 "가끔 이상하다"가 결정론적 실패가 된다.
    expect(results.map((r) => r.get<number>('maxInFlight'))).toEqual([1, 1, 1]);
  });

  it('한글 인자가 UTF-8 로 온전히 왕복한다', async () => {
    const session = spawnSession();
    const result = await session.call('hwp_doc_replace_text', {
      find: '가나다 라마바',
      replace: '②③④ 한글',
    });
    expect(argsOf(result)).toEqual({ find: '가나다 라마바', replace: '②③④ 한글' });
  });
});

// ── 제한 시간(D-14) ─────────────────────────────────────────────────────────

describe('Session — 호출 제한 시간', () => {
  it('제한 시간을 넘기면 RhwpTimeoutError 이고 세션은 죽지 않는다', async () => {
    // 파이썬판 Session(timeout=300.0) 과 대칭 — 예전엔 Node 에 이 옵션이 아예
    // 없어 응답이 영원히 안 와도 끊을 방법이 없었다(D-14). stdio 가 이벤트
    // 기반이라 process.ts 의 전체-프로세스 타임아웃과 달리, 호출 하나만 정리하고
    // 자식 프로세스나 세션 전체는 건드리지 않는다.
    const session = new Session({ cwd: fake.dir, profile: 'slow-echo', timeoutMs: 5 });
    live.push(session);

    const first = await capture(session.call('hwp_doc_info', { docId: 'doc-1' }));
    expect(first).toBeInstanceOf(RhwpTimeoutError);
    expect((first as RhwpTimeoutError).message).toContain('hwp_doc_info');
    expect((first as RhwpTimeoutError).message).toContain('5ms');

    // 뒤이은 호출도 독립적으로 타임아웃돼야 한다 — 첫 호출이 세션을 닫거나
    // 다음 요청의 id 대조를 어그러뜨리면 안 된다.
    const second = await capture(session.call('hwp_doc_fields', { docId: 'doc-1' }));
    expect(second).toBeInstanceOf(RhwpTimeoutError);

    // 타임아웃 이후 서버가 실제로 보낸 응답(약 40ms 뒤)이 늦게 도착해도, 이미
    // 정리된 id 라 무시돼야 한다 — 다음 요청과 뒤섞이면 "A 의 답이 B 에게 간다".
    await sleep(80);
  });

  it('제한 시간을 넉넉히 주면 평소처럼 동작한다', async () => {
    const session = new Session({ cwd: fake.dir, timeoutMs: 5000 });
    live.push(session);
    const result = await session.call('hwp_doc_info', { docId: 'doc-1' });
    expect(result.get<string>('tool')).toBe('hwp_doc_info');
  });
});

// ── 봉투 해석 ───────────────────────────────────────────────────────────────

describe('Session — 결과 봉투 해석', () => {
  it('structuredContent 가 있으면 그것을 쓴다', async () => {
    // 텍스트를 다시 파싱하는 것보다 정확하다. 구조화 결과를 두고 텍스트를 읽으면
    // 직렬화 과정의 표현 손실을 덤으로 얻는다.
    const session = spawnSession();
    const result = await session.call('hwp_doc_info', { docId: 'doc-1' });
    expect(result).toBeInstanceOf(Envelope);
    expect(result.schemaVersion).toBe('1.0');
    expect(result.get<string>('tool')).toBe('hwp_doc_info');
  });

  it('structuredContent 가 없으면 content[0].text 를 JSON 으로 읽는다', async () => {
    const session = spawnSession('text-json');
    const result = await session.call('hwp_doc_info', { docId: 'doc-1' });
    expect(result.get<string>('tool')).toBe('hwp_doc_info');
    expect(argsOf(result)).toEqual({ docId: 'doc-1' });
  });

  it('텍스트가 JSON 이 아니면 { text } 로 감싼다 — 내용을 버리지 않는다', async () => {
    // 모든 도구가 JSON 을 내는 것은 아니다. 파싱에 실패했다고 결과를 버리면
    // 호출자는 "성공했는데 아무것도 없다"는 가장 해석하기 어려운 상태를 받는다.
    const session = spawnSession('text-plain');
    const result = await session.call('hwp_doc_info', { docId: 'doc-1' });
    expect(result.get<string>('text')).toBe('평문 결과: hwp_doc_info');
  });

  it('isError 는 UsageError 이고 nextCall 교정 단서를 꺼낼 수 있다', async () => {
    // 도구가 거부한 것은 **우리 쪽 호출 조립 문제**다. 서버가 실어 보낸 교정 호출을
    // 예외에 담아 두면 기계가 그대로 다음 수를 둘 수 있다.
    const session = spawnSession('reject');
    const error = await capture(session.call('hwp_doc_save', { docId: 'doc-1', output: 'a.hwp' }));

    expect(error).toBeInstanceOf(UsageError);
    const usage = error as UsageError;
    expect(usage.exitCode).toBe(2);
    expect(usage.nextCall?.name).toBe('hwp_doc_fields');
    expect(usage.nextCall?.why).toContain('누름틀');
    // 판정 근거(봉투)까지 남는지 본다 — 예외만 남기면 왜 거부됐는지가 사라진다.
    expect(String(usage.envelope?.['error'])).toContain('hwp_doc_save');
  });

  it('JSON-RPC error 프레임은 ProtocolError 이고 도구 이름을 붙인다', async () => {
    const session = spawnSession('rpc-error');
    const error = await capture(session.call('hwp_doc_info', { docId: 'doc-1' }));

    expect(error).toBeInstanceOf(ProtocolError);
    expect((error as ProtocolError).message).toContain('hwp_doc_info');
    expect((error as ProtocolError).message).toContain('알 수 없는 도구');
  });
});

// ── 고장 경로 ───────────────────────────────────────────────────────────────

describe('Session — 서버가 죽었을 때', () => {
  it('응답 없이 종료하면 ProtocolError 이고 stderr 와 명령줄을 싣는다', async () => {
    // 진단은 stdout 이 아니라 stderr 에 있다. 예외가 stderr 를 버리면 사용자에게는
    // "실패했다"만 남고, 원인은 이미 사라진 자식과 함께 없어진다.
    const session = spawnSession('crash');
    const error = await capture(session.call('hwp_open', { path: '문서.hwp' }));

    expect(error).toBeInstanceOf(ProtocolError);
    expect((error as ProtocolError).stderr).toContain('응답 전에 죽었습니다');
    // 재현 가능한 명령줄이 남아야 버그 리포트가 성립한다.
    expect((error as ProtocolError).argv).toContain('mcp-serve');
  });
});

// ── 닫힘 계약 ───────────────────────────────────────────────────────────────

describe('Session·Document — 닫힘', () => {
  it('닫힌 세션에 호출하면 SessionClosedError — 조용히 새 서버를 띄우지 않는다', async () => {
    const session = spawnSession();
    await session.call('hwp_doc_info', { docId: 'doc-1' });
    await session.close();

    const error = await capture(session.call('hwp_doc_info', { docId: 'doc-1' }));
    expect(error).toBeInstanceOf(SessionClosedError);
    // 어떤 도구를 부르다 막혔는지 알려줘야 호출부를 찾을 수 있다.
    expect((error as SessionClosedError).message).toContain('hwp_doc_info');
  });

  it('닫힌 문서 핸들은 모든 메서드가 SessionClosedError', async () => {
    // 하나만 막고 나머지를 열어 두면, 닫힌 핸들로 편집이 들어가 **다른 문서를
    // 만지는** 최악의 경우가 생긴다.
    const doc = await openDocument('문서.hwp', { cwd: fake.dir });
    await doc.close();

    for (const call of [
      () => doc.info(),
      () => doc.text(),
      () => doc.fields(),
      () => doc.tables(),
      () => doc.search('예산'),
      () => doc.renderPage(0, '쪽0.svg'),
      () => doc.fillFields({ 성명: '홍길동' }),
      () => doc.replaceText('가', '나'),
      () => doc.setCell(0, 0, 0, '값'),
      () => doc.save('산출.hwp'),
    ]) {
      await expect(call()).rejects.toBeInstanceOf(SessionClosedError);
    }
  });

  it('close() 는 멱등이다 — 정리 예외가 원래 예외를 가리지 않는다', async () => {
    const session = spawnSession();
    await session.close();
    await session.close();

    const doc = await openDocument('문서.hwp', { cwd: fake.dir });
    await doc.close();
    // finally 의 close 와 명시적 close 가 겹칠 때 두 번째가 던지면, 진짜 원인은
    // 정리 예외에 가려져 영영 보이지 않는다.
    await doc.close();
  });

  it('문서를 닫아도 넘겨받은 세션은 살아 있다', async () => {
    // 세션을 공유하는 이유가 여기 있다. 문서 하나를 닫았다고 서버가 내려가면
    // 공유는 이득이 아니라 새로운 결합이 된다.
    const session = spawnSession();
    const doc = await openDocument('문서.hwp', { session });
    await doc.close();

    await expect(doc.info()).rejects.toBeInstanceOf(SessionClosedError);
    const alive = await session.call('hwp_doc_info', { docId: doc.docId });
    expect(alive.get<string>('tool')).toBe('hwp_doc_info');
  });
});

// ── 누수 ────────────────────────────────────────────────────────────────────

describe('openDocument — 실패 경로의 정리', () => {
  it('자기가 만든 세션은 실패해도 정리한다', async () => {
    // 여기서 새면 서버가 남아 문서 파일을 잡고, 다음 작업이 이유 없이 막힌다.
    // 자식은 400ms 뒤 마커를 쓴다 — 정리됐다면 그 시점이 오지 않는다.
    fake.clearMarker();
    const error = await capture(openDocument('문서.hwp', { cwd: fake.dir, profile: 'no-docid' }));
    expect(error).toBeInstanceOf(ProtocolError);

    await sleep(900);
    expect(existsSync(fake.markerPath), '자식이 살아남았다 — 세션이 샜다').toBe(false);
  });

  it('넘겨받은 세션은 실패해도 건드리지 않는다 (대조군)', async () => {
    // 위 테스트의 짝이다. 마커가 원래 안 만들어지는 픽스처였다면 위 단언은
    // 아무것도 증명하지 않고 통과한다. 그리고 남의 세션을 실패 경로에서 닫아
    // 버리면, 같은 서버에 열려 있던 다른 문서까지 함께 죽는다.
    fake.clearMarker();
    const session = spawnSession('no-docid');
    const error = await capture(openDocument('문서.hwp', { session }));
    expect(error).toBeInstanceOf(ProtocolError);

    await sleep(900);
    expect(existsSync(fake.markerPath), '남의 세션을 닫아 버렸다').toBe(true);
  });
});

// ── 도구별 인자 조립 ────────────────────────────────────────────────────────

describe('Document — 도구 인자 조립', () => {
  let opened: Document | undefined;

  beforeAll(async () => {
    opened = await openDocument('문서.hwp', { cwd: fake.dir });
  });

  afterAll(async () => {
    await opened?.close();
  });

  /** 열린 핸들. `beforeAll` 이 실패했으면 조용히 통과하지 않고 여기서 멈춘다. */
  function doc(): Document {
    if (opened === undefined) throw new Error('문서 핸들을 열지 못했습니다');
    return opened;
  }

  it('text() 는 page 를 주지 않으면 인자에 넣지 않는다', async () => {
    // `page: undefined` 가 그대로 나가면 서버는 "0쪽"이나 "잘못된 쪽"으로 해석할 수
    // 있다. 없는 옵션은 키 자체가 없어야 전체 텍스트 요청으로 읽힌다.
    const all = await doc().text();
    expect(argsOf(all)).toEqual({ docId: doc().docId });
    expect('page' in argsOf(all)).toBe(false);

    const one = await doc().text({ page: 2 });
    expect(argsOf(one)).toEqual({ docId: doc().docId, page: 2 });
  });

  it('renderPage 는 쪽과 출력 경로를 그대로 보낸다', async () => {
    // `output` 은 선택 인자가 아니다. 어디에 그렸는지 모르는 렌더는 눈검증 루프를
    // 닫지 못하고, 바인딩이 임시 경로를 임의로 정하면 그 파일을 지울 책임이
    // 호출자에게 떠넘겨진다.
    expect(doc().renderPage.length).toBe(2);
    const rendered = await doc().renderPage(3, '산출/쪽 3.svg');
    expect(argsOf(rendered)).toEqual({ docId: doc().docId, page: 3, output: '산출/쪽 3.svg' });
  });

  it('search 는 대소문자 구분을 기본 참으로 명시해 보낸다', async () => {
    // 기본값을 클라이언트가 명시하지 않으면 서버 기본값이 바뀔 때 조용히 결과가
    // 달라진다. 계약을 양쪽에 적어 두면 바뀌는 순간 테스트가 먼저 깨진다.
    expect(argsOf(await doc().search('예산'))).toEqual({
      docId: doc().docId,
      query: '예산',
      caseSensitive: true,
    });
    expect(argsOf(await doc().search('예산', { caseSensitive: false }))['caseSensitive']).toBe(
      false,
    );
  });

  it('replaceText·setCell·fillFields 가 계약 이름 그대로 보낸다', async () => {
    // 키 이름이 하나라도 camel/snake 로 어긋나면 서버는 "알 수 없는 인자"가 아니라
    // "기본값"으로 처리한다 — 조용히 다른 일을 한다.
    expect(argsOf(await doc().replaceText('2025년', '2026년'))).toEqual({
      docId: doc().docId,
      find: '2025년',
      replace: '2026년',
      caseSensitive: true,
    });
    expect(argsOf(await doc().setCell(1, 2, 3, '값'))).toEqual({
      docId: doc().docId,
      table: 1,
      row: 2,
      col: 3,
      text: '값',
    });
    expect(argsOf(await doc().fillFields({ 성명: '홍길동' }))).toEqual({
      docId: doc().docId,
      data: { 성명: '홍길동' },
    });
  });

  it('save 는 verify 를 기본 거짓으로 명시한다 — 검증 안 한 저장을 통과로 읽지 않게', async () => {
    expect(argsOf(await doc().save('산출.hwp'))).toEqual({
      docId: doc().docId,
      output: '산출.hwp',
      verify: false,
    });
    expect(argsOf(await doc().save('산출.hwp', { verify: true }))['verify']).toBe(true);
  });
});
