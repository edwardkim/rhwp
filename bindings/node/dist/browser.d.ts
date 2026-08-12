/**
 * 브라우저 어댑터 — 서브프로세스가 불가능한 환경.
 *
 * Node 는 rhwp 실행 파일을 띄우지만 브라우저는 그럴 수 없다. 그래서 같은 봉투 타입을
 * `@rhwp/editor` 의 WASM 위에서 구현하고, 양쪽이 같은 {@link RhwpClient} 인터페이스를
 * 만족하게 한다 — 소비자 코드가 환경에 독립적이 된다.
 *
 * ```ts
 * // 환경별로 클라이언트만 갈아 끼우고, 아래 로직은 그대로 쓴다.
 * const client: RhwpClient = isBrowser
 *   ? createBrowserClient(wasm)
 *   : createNodeClient();
 *
 * const meta = await client.info(source);
 * ```
 *
 * ## 무엇을 못 하나 (정직하게)
 *
 * WASM 경로는 **CLI 전용 기능을 흉내 내지 않는다**. 파일을 쓰는 산출 명령
 * (`exportPdf`·`thumbnail`·`extractPages`), 프로세스를 띄우는 세션·계획 계층,
 * 배치 스트림은 브라우저에 없다. {@link RhwpClient} 가 조회 축만 담는 이유다 —
 * 없는 기능을 인터페이스에 넣고 런타임에 던지는 것보다, 타입이 처음부터 말하는 편이 낫다.
 *
 * @packageDocumentation
 */
import { Envelope } from './envelope.js';
/**
 * Node·브라우저가 공유하는 최소 표면.
 *
 * 조회 축만 담는다 — 양쪽에서 **실제로 같은 의미로 동작하는** 것만 약속한다.
 */
export interface RhwpClient {
    /** 문서 요약. */
    info(source: DocumentSource): Promise<Envelope>;
    /** 쪽별 평문. */
    exportText(source: DocumentSource): Promise<Envelope>;
    /** 문서 구조. */
    exportStructure(source: DocumentSource): Promise<Envelope>;
    /** 표 전량. */
    exportTables(source: DocumentSource): Promise<Envelope>;
    /** 누름틀 목록. */
    fields(source: DocumentSource): Promise<Envelope>;
    /** 주소가 붙은 검색. */
    search(source: DocumentSource, query: string, options?: {
        readonly caseSensitive?: boolean;
    }): Promise<Envelope>;
    /** 한 쪽을 SVG 문자열로. */
    renderPage(source: DocumentSource, page: number): Promise<string>;
}
/**
 * 문서 입력.
 *
 * Node 는 경로(`string`)를, 브라우저는 바이트(`Uint8Array`)를 준다. 같은 인터페이스가
 * 둘 다 받는 이유는, 소비자가 "어디서 왔는지"를 몰라도 되게 하기 위함이다.
 */
export type DocumentSource = string | Uint8Array | ArrayBuffer;
/**
 * `@rhwp/editor` WASM 모듈이 제공해야 하는 최소 표면.
 *
 * 전체 WASM API 를 요구하지 않는다 — 여기 적힌 것만 있으면 어댑터가 성립한다.
 * 그래야 WASM 쪽이 진화해도 이 어댑터가 덜 깨진다.
 */
export interface RhwpWasmModule {
    /** 바이트에서 문서를 연다. */
    fromBytes(bytes: Uint8Array): RhwpWasmDocument;
}
/** WASM 문서 핸들이 제공해야 하는 표면. */
export interface RhwpWasmDocument {
    pageCount(): number;
    /** 문서 전체 평문. 쪽 구분은 어댑터가 맞춘다. */
    extractText(): string;
    /** 한 쪽 평문. 없으면 어댑터가 전체 텍스트로 대체한다. */
    extractPageText?(page: number): string;
    /** 한 쪽 SVG. */
    renderPageSvg(page: number): string;
    /** 구조·표·필드를 JSON 문자열로 (있는 것만). */
    structureJson?(): string;
    tablesJson?(): string;
    fieldsJson?(): string;
    searchJson?(query: string, caseSensitive: boolean): string;
    /** 자원 해제. */
    free?(): void;
}
/**
 * WASM 모듈을 {@link RhwpClient} 로 감싼다.
 *
 * 문서를 열 때마다 WASM 핸들을 만들고 **반드시 해제한다** — 브라우저에서 누수가 나면
 * 탭이 무거워지고 원인을 찾기 어렵다.
 */
export declare function createBrowserClient(wasm: RhwpWasmModule): RhwpClient;
/**
 * Node 쪽 {@link RhwpClient} 구현 — 같은 인터페이스를 서브프로세스로.
 *
 * 브라우저 코드와 공유하는 로직을 Node 에서도 그대로 돌릴 때 쓴다.
 */
export declare function createNodeClient(): RhwpClient;
