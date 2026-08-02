/**
 * 브라우저 어댑터 — WASM 위에서 같은 봉투를 낸다.
 *
 * 이 어댑터의 값어치는 "소비자 코드가 환경에 독립적"이라는 약속 하나다. 그 약속은
 * 두 가지가 지켜질 때만 성립한다:
 *
 * 1. **같은 모양의 봉투가 나온다.** 브라우저에서만 필드 이름이 다르면 공유 로직이
 *    한쪽에서 조용히 undefined 를 읽는다.
 * 2. **없는 기능은 시끄럽게 없다고 말한다.** WASM 빌드마다 선택 메서드가 다른데,
 *    없는 것을 빈 결과로 돌려주면 "표가 0개인 문서"와 구분되지 않는다.
 *
 * 그리고 브라우저 고유의 위험이 하나 더 있다 — **핸들 누수**다. 탭은 오래 살아서
 * 새어 나간 WASM 메모리가 쌓이고, 증상은 "쓰다 보면 느려진다"로만 나타난다.
 * 그래서 성공 경로뿐 아니라 **예외 경로에서도** 해제를 확인한다.
 */

import { describe, expect, it } from 'vitest';

import {
  createBrowserClient,
  createNodeClient,
  type RhwpWasmDocument,
  type RhwpWasmModule,
} from '../src/browser.js';
import { Envelope } from '../src/envelope.js';
import { RhwpError } from '../src/errors.js';

// ── 가짜 WASM 모듈 ──────────────────────────────────────────────────────────

interface StubOptions {
  readonly pageCount?: number;
  /** 문서 전체 평문. */
  readonly text?: string;
  /** 주면 쪽별 추출을 **지원하는** 빌드가 된다. 없으면 지원하지 않는 빌드. */
  readonly pageTexts?: readonly string[];
  readonly structureJson?: string;
  readonly tablesJson?: string;
  readonly fieldsJson?: string;
  readonly searchJson?: string;
  /** `free()` 를 아예 내놓지 않는 빌드. */
  readonly omitFree?: boolean;
}

interface Stub {
  readonly module: RhwpWasmModule;
  /** `fromBytes` 가 받은 바이트 — 복사 없이 그대로 넘어갔는지 본다. */
  readonly received: Uint8Array[];
  /** 해제 횟수. 성공이든 예외든 연 만큼 해제돼야 한다. */
  readonly freed: number;
  /** `searchJson` 이 받은 `[질의, 대소문자 구분]`. */
  readonly searchCalls: [string, boolean][];
}

function createStub(options: StubOptions = {}): Stub {
  const received: Uint8Array[] = [];
  const searchCalls: [string, boolean][] = [];
  const counter = { freed: 0 };

  const pageCount = options.pageCount ?? 3;
  const fullText = options.text ?? '문서 전체 평문';
  const { pageTexts, structureJson, tablesJson, fieldsJson, searchJson } = options;

  const wasm: RhwpWasmModule = {
    fromBytes(source: Uint8Array): RhwpWasmDocument {
      received.push(source);

      // 필수 표면만 먼저 만든다. 선택 메서드는 "그 빌드에 있을 때만" 붙인다 —
      // 언제나 붙여 두면 "없는 빌드"를 이 테스트가 흉내 낼 수 없다.
      const doc: RhwpWasmDocument = {
        pageCount: () => pageCount,
        extractText: () => fullText,
        renderPageSvg: (page: number) => `<svg data-page="${page}">쪽 ${page}</svg>`,
      };
      if (pageTexts !== undefined) doc.extractPageText = (page) => pageTexts[page] ?? '';
      if (structureJson !== undefined) doc.structureJson = () => structureJson;
      if (tablesJson !== undefined) doc.tablesJson = () => tablesJson;
      if (fieldsJson !== undefined) doc.fieldsJson = () => fieldsJson;
      if (searchJson !== undefined) {
        doc.searchJson = (query, caseSensitive) => {
          searchCalls.push([query, caseSensitive]);
          return searchJson;
        };
      }
      if (options.omitFree !== true) {
        doc.free = () => {
          counter.freed += 1;
        };
      }
      return doc;
    },
  };

  return {
    module: wasm,
    received,
    searchCalls,
    get freed(): number {
      return counter.freed;
    },
  };
}

/** 문서 바이트 한 줌. */
function bytes(...values: number[]): Uint8Array {
  return new Uint8Array(values);
}

/** 같은 내용의 `ArrayBuffer` — 브라우저에서 `File.arrayBuffer()` 가 주는 형태. */
function buffer(...values: number[]): ArrayBuffer {
  const result = new ArrayBuffer(values.length);
  new Uint8Array(result).set(values);
  return result;
}

/** 거부를 값으로 잡는다 — 메시지까지 봐야 "친절한가"를 판정할 수 있다. */
async function capture(promise: Promise<unknown>): Promise<unknown> {
  try {
    await promise;
  } catch (error) {
    return error;
  }
  throw new Error('거부되어야 하는데 정상 이행했습니다');
}

// ── 조회 축 ─────────────────────────────────────────────────────────────────

describe('createBrowserClient — 조회 결과가 봉투로 나온다', () => {
  it('info 는 쪽수와 출처를 담은 봉투를 낸다', async () => {
    const stub = createStub({ pageCount: 7 });
    const result = await createBrowserClient(stub.module).info(bytes(1, 2, 3));

    expect(result).toBeInstanceOf(Envelope);
    // 스키마 버전을 빼먹으면 소비자가 봉투 세대를 판별할 수 없다.
    expect(result.schemaVersion).toBe('1.0');
    expect(result.get<number>('pageCount')).toBe(7);
    // 브라우저에는 파일 경로가 없다. 없는 것을 지어내지 않고 그렇다고 말한다.
    expect(result.get<string>('source')).toBe('(bytes)');
  });

  it('exportText 는 쪽마다 한 항목씩 낸다', async () => {
    const stub = createStub({ pageCount: 3, pageTexts: ['첫 쪽', '둘째 쪽', '셋째 쪽'] });
    const result = await createBrowserClient(stub.module).exportText(bytes(1));

    expect(result.get<number>('pageCount')).toBe(3);
    expect(result.get<unknown[]>('pages')).toEqual([
      { page: 0, text: '첫 쪽' },
      { page: 1, text: '둘째 쪽' },
      { page: 2, text: '셋째 쪽' },
    ]);
  });

  it('쪽별 추출이 없는 빌드에서는 전체 텍스트를 첫 쪽에만 담는다', async () => {
    // 쪽마다 같은 내용을 복제하면 소비자는 중복을 **진짜 데이터**로 오해한다 —
    // "3쪽짜리 문서에 같은 문장이 세 번 있다"는 결론은 원문에 대한 거짓말이다.
    const stub = createStub({ pageCount: 3, text: '한 덩어리 평문' });
    const result = await createBrowserClient(stub.module).exportText(bytes(1));

    expect(result.get<unknown[]>('pages')).toEqual([
      { page: 0, text: '한 덩어리 평문' },
      { page: 1, text: '' },
      { page: 2, text: '' },
    ]);
  });

  it('구조·표·누름틀은 WASM 이 준 JSON 을 그대로 봉투로 만든다', async () => {
    const stub = createStub({
      structureJson: '{"schemaVersion":"1.0","sections":[{"index":0}]}',
      tablesJson: '{"schemaVersion":"1.0","tables":[]}',
      fieldsJson: '{"schemaVersion":"1.0","fields":[{"name":"성명"}]}',
    });
    const client = createBrowserClient(stub.module);

    expect((await client.exportStructure(bytes(1))).children('sections')).toHaveLength(1);
    expect((await client.exportTables(bytes(1))).get<unknown[]>('tables')).toEqual([]);
    expect((await client.fields(bytes(1))).children('fields')[0]?.get('name')).toBe('성명');
  });

  it('search 는 대소문자 구분을 기본 참으로 넘긴다', async () => {
    // 기본값을 어댑터가 명시하지 않으면 WASM 기본값이 바뀔 때 브라우저에서만
    // 결과가 달라진다 — Node 와 브라우저가 같은 답을 준다는 약속이 깨진다.
    const stub = createStub({ searchJson: '{"schemaVersion":"1.0","matches":[]}' });
    const client = createBrowserClient(stub.module);

    await client.search(bytes(1), '예산');
    await client.search(bytes(1), '예산', { caseSensitive: false });

    expect(stub.searchCalls).toEqual([
      ['예산', true],
      ['예산', false],
    ]);
  });

  it('renderPage 는 SVG 문자열을 그대로 돌려준다', async () => {
    const stub = createStub({ pageCount: 4 });
    const svg = await createBrowserClient(stub.module).renderPage(bytes(1), 2);
    expect(svg).toContain('data-page="2"');
  });
});

// ── 입력 정규화 ─────────────────────────────────────────────────────────────

describe('createBrowserClient — 문서 입력', () => {
  it('Uint8Array 는 복사 없이 그대로 넘어간다', async () => {
    // 문서 바이트는 수십 MB 가 될 수 있다. 어댑터가 한 겹 복사하면 브라우저에서
    // 최대 사용량이 두 배가 되고, 그 비용은 아무 대가도 사지 못한다.
    const stub = createStub();
    const source = bytes(1, 2, 3, 4);
    await createBrowserClient(stub.module).info(source);
    expect(stub.received[0]).toBe(source);
  });

  it('ArrayBuffer 는 Uint8Array 로 감싸서 넘긴다', async () => {
    // `<input type="file">` 과 `fetch(...).arrayBuffer()` 가 주는 것이 이 형태다.
    // 여기서 받지 않으면 소비자가 매번 변환 코드를 쓴다.
    const stub = createStub();
    await createBrowserClient(stub.module).info(buffer(9, 8, 7));
    expect(stub.received[0]).toBeInstanceOf(Uint8Array);
    expect(Array.from(stub.received[0] ?? bytes())).toEqual([9, 8, 7]);
  });

  it('경로 문자열을 주면 해결책까지 담아 거절한다', async () => {
    // 브라우저는 파일을 열 수 없다. 여기서 조용히 실패하면 사용자는 "빈 문서"를
    // 받아 들고 원인을 문서 쪽에서 찾는다.
    const stub = createStub();
    const error = await capture(createBrowserClient(stub.module).info('문서.hwp'));

    expect(error).toBeInstanceOf(RhwpError);
    expect((error as RhwpError).message).toContain('파일 경로를 열 수 없습니다');
    expect((error as RhwpError).message).toContain('arrayBuffer');
    // 문서를 열지도 못했으니 해제할 핸들도 없다.
    expect(stub.received).toHaveLength(0);
    expect(stub.freed).toBe(0);
  });
});

// ── 핸들 수명 ───────────────────────────────────────────────────────────────

describe('createBrowserClient — 핸들 해제', () => {
  it('성공하면 연 만큼 해제한다', async () => {
    const stub = createStub({ pageCount: 2 });
    const client = createBrowserClient(stub.module);

    await client.info(bytes(1));
    await client.exportText(bytes(1));
    await client.renderPage(bytes(1), 0);

    expect(stub.received).toHaveLength(3);
    expect(stub.freed).toBe(3);
  });

  it('예외로 빠져나가도 해제한다 — 여기서 새면 탭이 무거워지고 원인을 못 찾는다', async () => {
    // 두 갈래 모두 본다: 없는 기능(파싱 전 실패)과 범위 초과(문서를 연 뒤 실패).
    const stub = createStub({ pageCount: 2 });
    const client = createBrowserClient(stub.module);

    await expect(client.exportStructure(bytes(1))).rejects.toBeInstanceOf(RhwpError);
    await expect(client.renderPage(bytes(1), 99)).rejects.toBeInstanceOf(RhwpError);

    expect(stub.received).toHaveLength(2);
    expect(stub.freed).toBe(2);
  });

  it('free 를 내놓지 않는 빌드에서도 터지지 않는다', async () => {
    // 해제 메서드는 선택 표면이다. 없다고 어댑터가 죽으면 그 빌드는 통째로 못 쓴다.
    const stub = createStub({ omitFree: true });
    const result = await createBrowserClient(stub.module).info(bytes(1));
    expect(result.get<number>('pageCount')).toBe(3);
    expect(stub.freed).toBe(0);
  });
});

// ── 없는 기능·잘못된 결과 ───────────────────────────────────────────────────

describe('createBrowserClient — 이 빌드가 못 하는 일', () => {
  it('선택 메서드가 없으면 무엇이 없는지와 어디를 볼지 알려준다', async () => {
    // 빈 봉투를 돌려주면 "표가 없는 문서"와 구분되지 않는다. 그 혼동은 사용자가
    // 문서를 의심하게 만들고, 정작 원인은 WASM 빌드 버전이다.
    const stub = createStub();
    const client = createBrowserClient(stub.module);

    const cases: [string, () => Promise<unknown>][] = [
      ['구조 추출', () => client.exportStructure(bytes(1))],
      ['표 추출', () => client.exportTables(bytes(1))],
      ['누름틀 조회', () => client.fields(bytes(1))],
      ['검색', () => client.search(bytes(1), '예산')],
    ];

    for (const [label, call] of cases) {
      const error = await capture(call());
      expect(error).toBeInstanceOf(RhwpError);
      expect((error as RhwpError).message).toContain(label);
      expect((error as RhwpError).message).toContain('@rhwp/editor');
    }
  });

  it('결과가 JSON 이 아니거나 객체가 아니면 사유를 나눠서 말한다', async () => {
    // "파싱 실패"와 "모양이 다름"은 고치는 방법이 다르다. 뭉뚱그리면 사용자가
    // 엉뚱한 곳을 본다.
    const notJson = createStub({ structureJson: '이건 JSON 이 아니다' });
    const notObject = createStub({ structureJson: '[1, 2, 3]' });

    const first = await capture(createBrowserClient(notJson.module).exportStructure(bytes(1)));
    expect((first as RhwpError).message).toContain('JSON 이 아닙니다');

    const second = await capture(createBrowserClient(notObject.module).exportStructure(bytes(1)));
    expect((second as RhwpError).message).toContain('객체가 아닙니다');

    // 어느 쪽이든 핸들은 해제돼야 한다.
    expect(notJson.freed).toBe(1);
    expect(notObject.freed).toBe(1);
  });

  it('renderPage 는 범위 밖을 거절하고 가능한 범위를 알려준다', async () => {
    const stub = createStub({ pageCount: 3 });
    const client = createBrowserClient(stub.module);

    const error = await capture(client.renderPage(bytes(1), 3));
    expect(error).toBeInstanceOf(RhwpError);
    // 범위를 함께 말해야 호출자가 한 번에 고친다.
    expect((error as RhwpError).message).toContain('0..2');

    await expect(client.renderPage(bytes(1), -1)).rejects.toBeInstanceOf(RhwpError);
    // 정수가 아닌 쪽 번호는 조용히 잘라내지 않는다 — 1.5쪽은 존재하지 않는다.
    await expect(client.renderPage(bytes(1), 1.5)).rejects.toBeInstanceOf(RhwpError);
    // 경계값은 통과해야 한다 (범위 검사가 한 칸 어긋나지 않았는지).
    expect(await client.renderPage(bytes(1), 2)).toContain('data-page="2"');
  });
});

// ── Node 쪽 짝 ──────────────────────────────────────────────────────────────

describe('createNodeClient — 같은 인터페이스, 반대 입력', () => {
  it('바이트를 주면 해결책까지 담아 거절한다', async () => {
    // Node 경로는 서브프로세스에 **경로**를 넘긴다. 바이트를 받아 몰래 임시 파일로
    // 쓰면 그 파일을 지울 책임이 아무에게도 없다 — 그래서 받지 않는다고 말한다.
    const node = createNodeClient();
    const source = bytes(1, 2, 3);

    const error = await capture(node.info(source));
    expect(error).toBeInstanceOf(RhwpError);
    expect((error as RhwpError).message).toContain('파일 경로를 넘기세요');
    expect((error as RhwpError).message).toContain('임시 파일');

    for (const call of [
      () => node.exportText(source),
      () => node.exportStructure(source),
      () => node.exportTables(source),
      () => node.fields(source),
      () => node.search(source, '예산'),
      () => node.search(source, '예산', { caseSensitive: false }),
      () => node.renderPage(source, 0),
    ]) {
      await expect(call()).rejects.toBeInstanceOf(RhwpError);
    }
  });
});
