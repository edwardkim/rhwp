/**
 * API 2층 — 세션(핸들) 클라이언트.
 *
 * `mcp-serve` 를 stdio JSON-RPC 로 띄우고 `hwp_doc_*` 도구를 그대로 노출한다.
 * 1층(무상태)이 호출마다 문서를 재파싱하는 반면, 2층은 한 번 열어 두고 여러 번
 * 만진다 — 대형 문서 반복 작업에서 차이가 크다.
 *
 * ```ts
 * const doc = await openDocument('서식.hwp');
 * try {
 *   await doc.fillFields({ 성명: '홍길동' });
 *   const saved = await doc.save('제출본.hwp', { verify: true });
 * } finally {
 *   await doc.close();   // 서버가 남으면 다음 작업이 파일을 못 연다
 * }
 * ```
 *
 * @packageDocumentation
 */
import { Envelope, type RawEnvelope } from './envelope.js';
/** 세션 호출 기본 제한 시간(ms). `null` 을 주면 무제한. */
export declare const DEFAULT_SESSION_TIMEOUT_MS = 300000;
/** {@link Session} 생성 옵션. */
export interface SessionOptions {
    /** 역할 프로필 — 도구 노출 범위를 제한한다. */
    readonly profile?: string | undefined;
    /** 작업 디렉터리. */
    readonly cwd?: string | undefined;
    /**
     * 호출 하나당 제한 시간(ms). 기본 {@link DEFAULT_SESSION_TIMEOUT_MS}, `null` 이면 무제한.
     *
     * 파이썬판(`Session(timeout=300.0)`)엔 있었지만 Node 엔 없어 응답이 영원히
     * 안 와도 끊지 못했다(D-14). stdio 가 이벤트 기반이라 파이썬처럼 블로킹
     * `readline` 을 건드릴 필요 없이 대기 중인 요청 하나만 타이머로 정리하면 된다.
     */
    readonly timeoutMs?: number | null | undefined;
}
/**
 * `mcp-serve` 자식 프로세스 하나를 감싼 JSON-RPC 클라이언트.
 *
 * 보통은 {@link openDocument} 가 만들어 주는 {@link Document} 를 쓰면 되고,
 * 여러 문서를 한 서버에서 열고 싶을 때만 직접 만든다.
 */
export declare class Session {
    private readonly child;
    private readonly argv;
    private nextId;
    private closed;
    /** 요청을 직렬화한다 — 응답 id 대조가 성립하려면 한 번에 하나만 보내야 한다. */
    private queue;
    private buffer;
    private readonly pending;
    private stderrText;
    private readonly timeoutMs;
    constructor(options?: SessionOptions);
    /** 줄 단위로 프레임을 잘라 대기 중인 요청에 넘긴다. */
    private onStdout;
    private dispatch;
    private failAllPending;
    /**
     * 도구 하나를 호출하고 결과 봉투를 돌려준다.
     *
     * @throws {SessionClosedError} 이미 닫힌 세션.
     * @throws {UsageError} 도구가 `isError` 를 세운 경우. 서버가 `didYouMean`·
     *   `nextCall` 교정 단서를 실어 보내면 예외의 `envelope` 에 담긴다.
     * @throws {ProtocolError} 응답이 JSON-RPC 계약을 어긴 경우.
     */
    call<T extends RawEnvelope = RawEnvelope>(name: string, args: Readonly<Record<string, unknown>>): Promise<Envelope<T>>;
    private send;
    /** JSON-RPC 응답에서 도구 결과 봉투를 꺼낸다. */
    private unwrap;
    /** 서버를 정리한다. 여러 번 불러도 안전하다. */
    close(): Promise<void>;
    /** `await using` 지원 (TS 5.2+ / Node 20+). */
    [Symbol.asyncDispose](): Promise<void>;
}
/** 열린 문서 핸들 — 세션 위의 얇은 편의 계층. */
export declare class Document {
    private readonly session;
    /** 서버가 발급한 핸들 식별자. */
    readonly docId: string;
    private readonly ownsSession;
    private closed;
    constructor(session: Session, 
    /** 서버가 발급한 핸들 식별자. */
    docId: string, ownsSession: boolean);
    private callTool;
    /** 문서 요약 (재파싱 없음). */
    info(): Promise<Envelope>;
    /** 평문. `page` 를 주면 그 쪽만. */
    text(options?: {
        readonly page?: number | undefined;
    }): Promise<Envelope>;
    /** 누름틀 목록. */
    fields(): Promise<Envelope>;
    /** 표 전량. */
    tables(): Promise<Envelope>;
    /** 주소가 붙은 검색. */
    search(query: string, options?: {
        readonly caseSensitive?: boolean | undefined;
    }): Promise<Envelope>;
    /**
     * 한 쪽을 SVG 파일로 — 편집 직후 눈검증 루프를 닫는 도구.
     *
     * @param page - 0 기준 쪽 번호. 편집 봉투의 `changedPages` 를 그대로 넘기면
     *   바뀐 쪽만 상수 비용으로 확인할 수 있다.
     * @param output - SVG 를 쓸 경로. **도구 계약상 필수**다.
     */
    renderPage(page: number, output: string): Promise<Envelope>;
    /** 누름틀 채우기. */
    fillFields(data: Readonly<Record<string, unknown>>): Promise<Envelope>;
    /** 문자열 치환. */
    replaceText(find: string, replace: string, options?: {
        readonly caseSensitive?: boolean | undefined;
    }): Promise<Envelope>;
    /** 표 셀 기록. 좌표는 {@link Document.tables} 로 확인한다. */
    setCell(table: number, row: number, col: number, text: string): Promise<Envelope>;
    /** 저장. `verify: true` 면 저장 직후 자기검증 보고가 봉투에 담긴다. */
    save(output: string, options?: {
        readonly verify?: boolean | undefined;
    }): Promise<Envelope>;
    /** 핸들을 닫는다 (세션을 소유하면 서버도 함께 정리). */
    close(): Promise<void>;
    /** `await using` 지원. */
    [Symbol.asyncDispose](): Promise<void>;
    toString(): string;
}
/** {@link openDocument} 옵션. */
export interface OpenOptions {
    /** 보호 문서 암호. */
    readonly password?: string | undefined;
    /** 이미 만든 세션에 얹는다. 주면 문서를 닫아도 세션은 남는다. */
    readonly session?: Session | undefined;
    /** 새 세션을 만들 때의 역할 프로필. */
    readonly profile?: string | undefined;
    /** 작업 디렉터리. */
    readonly cwd?: string | undefined;
}
/**
 * 문서를 열어 핸들을 돌려준다.
 *
 * `session` 을 주지 않으면 전용 서버를 띄우고, 문서를 닫을 때 함께 정리한다.
 */
export declare function openDocument(path: string, options?: OpenOptions): Promise<Document>;
