/**
 * rhwp 종료 코드 → TypeScript 예외 매핑.
 *
 * 핵심 규약 (`mydocs/tech/bindings_foundation.md` §3, M18 과 동형):
 *
 * - exit 1 = 런타임 실패 → {@link RhwpRuntimeError}
 * - exit 2 = 사용법 오류 → {@link UsageError} (호출을 조립한 **우리 쪽** 버그)
 * - exit 3/4 = **검증 단언 실패 — 예외가 아니라 반환값의 판정 필드**
 *
 * exit 3/4 를 던지지 않는 이유가 이 모듈의 존재 이유다. `--verify` 가 불일치를
 * 보고하거나 `render-diff` 가 회귀를 검출한 것은 **도구가 정상 동작한 결과**다.
 * 예외로 만들면 호출자가 `try/catch` 로 "고장"처럼 다루게 되고, 정작 봉투에 담긴
 * 판정 근거(`diffCount`·`status`·`pages`)를 읽지 않는다.
 *
 * @packageDocumentation
 */
/** 성공. */
export declare const EXIT_OK = 0;
/** 런타임 실패 (읽기·파싱·렌더·쓰기). */
export declare const EXIT_RUNTIME = 1;
/** 사용법 오류 (인자 없음, 알 수 없는 옵션/명령, 범위 초과, 계획 선검증 위반). */
export declare const EXIT_USAGE = 2;
/** 검증 단언 실패 (convert/export-hwpx --verify, edit --verify, run assertions). */
export declare const EXIT_VERIFY = 3;
/** --verify-pages 페이지 수 불일치. */
export declare const EXIT_VERIFY_PAGES = 4;
/** 종료 코드 유니온 — 알려진 코드만. */
export type KnownExitCode = typeof EXIT_OK | typeof EXIT_RUNTIME | typeof EXIT_USAGE | typeof EXIT_VERIFY | typeof EXIT_VERIFY_PAGES;
/** 예외 생성에 필요한 맥락. */
export interface ErrorContext {
    /** 실행한 명령줄 (재현용). */
    readonly argv?: readonly string[] | undefined;
    /** 프로세스 종료 코드. 프로세스를 못 띄웠으면 undefined. */
    readonly exitCode?: number | undefined;
    /** 도구가 남긴 진단 원문. 진단은 stdout 이 아니라 stderr 에 있다. */
    readonly stderr?: string | undefined;
    /** 파싱에 성공한 봉투가 있으면 그대로 (판정 근거 보존). */
    readonly envelope?: Record<string, unknown> | undefined;
    /** 원인 예외. */
    readonly cause?: unknown;
}
/**
 * 모든 rhwp 예외의 기반.
 *
 * `instanceof` 가 트랜스파일 이후에도 동작하도록 프로토타입을 명시 복원한다
 * (TS 가 ES5 로 내려갈 때 내장 Error 상속이 깨지는 알려진 함정).
 */
export declare class RhwpError extends Error {
    /** 실행한 명령줄. */
    readonly argv?: readonly string[] | undefined;
    /** 종료 코드. */
    readonly exitCode?: number | undefined;
    /** 도구 진단 원문. */
    readonly stderr: string;
    /** 판정 근거가 담긴 봉투. */
    readonly envelope?: Record<string, unknown> | undefined;
    constructor(message: string, context?: ErrorContext);
    /**
     * 재현 가능한 명령 문자열. 버그 리포트에 그대로 붙일 수 있게 공백을 감싼다.
     */
    get command(): string;
    /** 가장 구체적인 진단 (stderr 마지막 줄). */
    get lastDiagnostic(): string;
    toString(): string;
}
/**
 * rhwp 실행 파일을 찾지 못했다.
 *
 * 탐색 순서(`RHWP_BIN` → 패키지 동봉 → `PATH`)를 모두 시도한 뒤에만 발생한다.
 * 메시지에 시도한 경로를 모두 담아, 사용자가 어디에 두면 되는지 알 수 있게 한다.
 */
export declare class BinaryNotFoundError extends RhwpError {
}
/**
 * exit 2 — 호출 조립이 틀렸다.
 *
 * 이건 **우리 쪽(바인딩 또는 호출자) 버그**다. 재시도해도 같은 결과가 나오므로
 * 호출자는 인자를 고쳐야 한다.
 */
export declare class UsageError extends RhwpError {
    /**
     * stderr 의 `힌트:` 줄에서 did-you-mean 교정 제안을 추출한다.
     *
     * @returns 제안 문구. 없으면 undefined.
     */
    get suggestion(): string | undefined;
    /**
     * 서버가 실어 보낸 교정 호출(`nextCall`). 기계가 그대로 따라할 수 있는 형태다.
     */
    get nextCall(): {
        name: string;
        arguments?: Record<string, unknown>;
        why?: string;
    } | undefined;
}
/**
 * exit 1 — 읽기·파싱·렌더·쓰기가 실패했다.
 *
 * 파일이 없거나 손상됐거나 디스크에 쓸 수 없는 경우다. 인자를 고쳐도 해결되지
 * 않으며, 입력 자체를 봐야 한다.
 */
export declare class RhwpRuntimeError extends RhwpError {
}
/**
 * exit 3/4 — 검증 단언이 실패했다. **기본적으로는 발생하지 않는다.**
 *
 * `throwOnVerdict: true` 를 명시했을 때만 던져진다. 기본 경로는 판정을 반환값으로
 * 돌려준다 — 도구는 정상 동작했고, 실패한 것은 *문서에 대한 단언*이기 때문이다.
 */
export declare class VerdictFailed extends RhwpError {
    /** exit 4 (페이지 수 불일치)인지. */
    get isPageCountMismatch(): boolean;
}
/**
 * stdout 이 계약을 어겼다 — JSON 이 아니거나, 기대한 프레임이 아니다.
 *
 * `--json` 모드의 stdout 은 순수 JSON(배치는 NDJSON)이고 실패 경로는 0바이트다.
 * 그 계약이 깨졌다는 뜻이므로 도구 버그이거나 버전 불일치다.
 */
export declare class ProtocolError extends RhwpError {
}
/** 이미 닫힌 세션 핸들을 다시 썼다. */
export declare class SessionClosedError extends RhwpError {
}
/**
 * 봉투에 없는 필드를 물었다 ({@link module:envelope.Envelope.get}).
 *
 * 파이썬판은 `KeyError`/`AttributeError` — 표준 예외 계열이지만 최소한 하나의
 * 계열이다. Node 의 `Envelope.get` 은 예전엔 일반 `Error` 를 던져 `catch (e) {
 * if (e instanceof RhwpError) … }` 로 거르는 코드가 이 예외를 놓쳤다(D-17).
 */
export declare class EnvelopeKeyError extends RhwpError {
}
/** 제한 시간 안에 끝나지 않았다. 자식 프로세스는 종료를 시도한 뒤 던져진다. */
export declare class RhwpTimeoutError extends RhwpError {
}
/** {@link raiseForExit} 옵션. */
export interface RaiseForExitOptions extends ErrorContext {
    /**
     * 참이면 exit 3/4 도 {@link VerdictFailed} 로 던진다.
     *
     * 기본값 거짓 — 판정은 반환값으로 다루는 것이 이 바인딩의 규약이다.
     */
    readonly throwOnVerdict?: boolean;
}
/**
 * 종료 코드를 검사해 필요하면 예외를 던진다.
 *
 * @param exitCode - 프로세스 종료 코드.
 * @param options - 예외에 담을 맥락과 판정 처리 방식.
 *
 * @throws {UsageError} exit 2.
 * @throws {RhwpRuntimeError} exit 1, 또는 사전에 없는 0 아닌 코드.
 * @throws {VerdictFailed} exit 3/4 이면서 `throwOnVerdict` 가 참일 때.
 */
export declare function raiseForExit(exitCode: number, options?: RaiseForExitOptions): void;
/**
 * 알려진 종료 코드인지.
 *
 * 타입 좁히기에 쓴다 — 알려지지 않은 코드를 분기에서 놓치지 않도록.
 */
export declare function isKnownExitCode(code: number): code is KnownExitCode;
