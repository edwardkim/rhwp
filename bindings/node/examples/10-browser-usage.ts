/*
 * 브라우저 어댑터 — 같은 코드가 Node 와 브라우저에서 돈다.
 *
 * Node 에서는 rhwp 실행 파일을 자식 프로세스로 띄운다. 브라우저에는 자식
 * 프로세스가 없으므로 `@rhwp/editor` 의 WASM 을 대신 쓴다. 두 경로에서 다른
 * 것은 **실행 수단**뿐이고, 봉투 타입과 판정 규약은 같다. 그래서 읽기 코드는
 * 한 벌만 있으면 된다 — 이 파일이 보이려는 것이 그것이다.
 *
 * 이 파일은 "돌려 보는" 예제가 아니라 "이식 가능한 모양"을 보이는 예제다.
 * 아래 `main` 은 Node 경로만 실행한다. 브라우저 경로는 번들러가 WASM 을 실어
 * 줘야 실제로 동작한다.
 *
 *   npx tsx examples/10-browser-usage.ts 문서.hwp
 *
 * ── 번들러 설정 (브라우저 경로를 실제로 쓸 때) ──────────────────────────────
 *
 * 1) `@rhwp/editor` 를 의존성에 추가한다. WASM 바이너리가 함께 들어 있다.
 * 2) `.wasm` 을 자산으로 내보내게 한다.
 *      - Vite   : 기본 지원. `?url` 로 받거나 `assetsInclude: ['**\/*.wasm']`.
 *      - webpack: `experiments.asyncWebAssembly = true`.
 *      - esbuild: `--loader:.wasm=file`.
 * 3) `@rhwp/node` 를 브라우저 번들에 넣지 않는다. 이 패키지는 `node:child_process`
 *    를 참조하므로 번들러가 폴리필을 끼워 넣으려 하고, 그 폴리필은 조용히 실패한다.
 *    브라우저 진입점에서는 `createBrowserClient` 만 가져오도록 코드 분할하거나,
 *    아래처럼 클라이언트를 **주입**받아 공유 코드가 실행 수단을 모르게 한다.
 * ──────────────────────────────────────────────────────────────────────────
 */

import process from 'node:process';

import * as rhwp from '../src/index.js';

/**
 * 공유 코드가 실제로 쓰는 표면만 뽑는다.
 *
 * `RhwpClient` 전체를 요구하면 읽기만 하는 함수가 편집 능력까지 끌고 다니게 되고,
 * 테스트에서 가짜를 만들 때도 쓰지도 않는 메서드를 전부 채워야 한다.
 */
type ReaderClient = Pick<rhwp.RhwpClient, 'info' | 'exportText'>;

/**
 * 실행 수단을 모르는 공유 코드.
 *
 * Node 든 브라우저든 이 함수는 그대로다. 봉투가 같은 타입이기 때문에 판정을
 * 읽는 방식도 바뀌지 않는다 — 이게 어댑터를 두는 이유다.
 */
export async function summarize(client: ReaderClient, path: string): Promise<string> {
  const meta = await client.info(path);
  // `RhwpClient` 는 **양쪽에서 같은 의미로 동작하는 것만** 약속한다. 1층의
  // `exportText(path, { page })` 같은 옵션은 여기 없다 — WASM 쪽에 같은 옵션이
  // 없는데 인터페이스에 넣으면, 공유 코드가 브라우저에서만 조용히 다르게 동작한다.
  const text = await client.exportText(path);
  const [firstPage] = text.children('pages');
  const first = (firstPage?.getOr<string>('text', '') ?? '').slice(0, 80).replace(/\n/g, ' ');
  // 브라우저 어댑터의 `info` 봉투에는 `format` 이 없다 — 없는 필드에 `.get()` 을
  // 쓰면 예외이므로, 두 경로에서 함께 도는 코드는 `.getOr()` 로 읽어야 한다.
  return `${meta.getOr<string>('format', '(bytes)')} · ${meta.get<number>('pageCount')}쪽 · ${first}…`;
}

/**
 * `createBrowserClient` 가 받는 WASM 모듈 타입.
 *
 * 손으로 적지 않고 함수 시그니처에서 끌어온다. 어댑터가 요구하는 모양이 바뀌면
 * 이 별칭도 같이 바뀌고, 예제가 조용히 낡지 않는다.
 */
type RhwpWasm = Parameters<typeof rhwp.createBrowserClient>[0];

/**
 * 브라우저 경로.
 *
 * WASM 모듈은 호스트 앱이 로드해서 넘긴다. 여기서 `import('@rhwp/editor')` 를
 * 직접 하지 않는 이유는, 그 순간 이 파일이 브라우저 전용이 되어 Node 에서
 * 타입 검사조차 못 하게 되기 때문이다.
 */
export function browserReader(wasm: RhwpWasm): ReaderClient {
  return rhwp.createBrowserClient(wasm);
}

/**
 * Node 경로.
 *
 * 1층 무상태 명령이 곧 클라이언트 표면이다. 별도의 팩토리가 필요 없다.
 */
export const nodeReader: ReaderClient = {
  info: rhwp.info,
  exportText: rhwp.exportText,
};

async function main(path: string): Promise<number> {
  // 호출부는 어느 쪽 클라이언트인지 신경 쓰지 않는다. 브라우저에서는 이 한 줄만
  //   const client = browserReader(await loadRhwpEditorWasm());
  // 로 바뀐다.
  const client: ReaderClient = nodeReader;
  console.log(await summarize(client, path));
  return 0;
}

const argv = process.argv.slice(2);
const [docPath] = argv;
if (argv.length !== 1 || docPath === undefined) {
  console.error('사용법: npx tsx examples/10-browser-usage.ts 문서.hwp');
  process.exit(2);
}

process.exit(await main(docPath));
