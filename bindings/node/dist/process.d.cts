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
import type { BatchRecord, RawEnvelope } from './envelope.js';
/**
 * 기본 제한 시간(ms). 대형 문서 렌더가 수십 초 걸릴 수 있어 넉넉히 잡는다.
 * `null` 을 넘기면 무제한.
 */
export declare const DEFAULT_TIMEOUT_MS = 300000;
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
    /**
     * 예외에 실을 봉투 (호출자가 이미 파싱해 뒀을 때).
     *
     * {@link runJson} 은 stdout 을 직접 파싱해 자동으로 실어 보내지만,
     * {@link runRaw} 는 원문만 돌려주므로 호출자가 미리 파싱한 봉투가 있으면
     * 여기로 넘겨야 판정 근거가 예외에서 빠지지 않는다(D-20, 파이썬
     * `_process.py` 의 `envelope_hint` 와 대칭).
     */
    readonly envelopeHint?: RawEnvelope | undefined;
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
/**
 * rhwp 를 실행하고 원문 결과를 돌려준다.
 *
 * @param args - 실행 인자 (프로그램 이름 제외).
 * @param options - 실행 옵션. `check` 는 없다 — 검사는 호출자가
 *   {@link raiseForExit} 로 명시한다.
 */
export declare function runRaw(args: readonly Argument[], options?: RunOptions & {
    readonly check?: boolean;
}): Promise<CompletedRun>;
/**
 * `--json` 명령을 실행하고 봉투를 돌려준다.
 *
 * 종료 코드 검사는 **파싱 뒤**에 한다 — exit 3(판정 실패)일 때도 봉투가 나오고,
 * 그 봉투에 판정 근거가 들어 있기 때문이다. 순서를 뒤집으면 가장 중요한 정보를
 * 버리게 된다.
 *
 * @throws {ProtocolError} stdout 이 JSON 이 아니거나, 성공했는데 비어 있을 때.
 */
export declare function runJson<T extends RawEnvelope = RawEnvelope>(args: readonly Argument[], options?: RunOptions): Promise<T>;
/**
 * batch 계열을 실행하고 NDJSON 레코드 목록을 돌려준다.
 *
 * batch 는 **부분 실패도 실패**다 — 성공 레코드는 스트림에 남고 종료 코드가
 * 신호한다. 그래서 여기서는 exit 1 을 예외로 올리지 않고, 레코드에 담긴 `error`
 * 필드를 호출자가 보게 한다. 스트림을 통째로 버리면 성공분까지 잃는다.
 */
export declare function runNdjson<T extends BatchRecord = BatchRecord>(args: readonly Argument[], options?: RunOptions): Promise<T[]>;
/**
 * NDJSON 을 **스트리밍**으로 읽는다 — 대량 배치에서 메모리를 아낀다.
 *
 * 전량을 모으는 {@link runNdjson} 과 달리 레코드가 나오는 대로 넘긴다.
 * 소비자가 중간에 멈추면(`break`) 자식 프로세스도 정리한다 — 남으면 파일을
 * 잡고 있어 다음 작업이 막힌다.
 */
export declare function iterNdjson<T extends BatchRecord = BatchRecord>(args: readonly Argument[], options?: RunOptions): AsyncIterableIterator<T>;
