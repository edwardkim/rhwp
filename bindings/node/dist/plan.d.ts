/**
 * API 3층 — 계획 실행기.
 *
 * 도구를 체이닝하는 대신 계획서 하나를 만든다. rhwp 가 **정적 선검증(실행 0)** →
 * **원자 실행**(전 step 인메모리 적용) → **사후 단언 통과 시에만 단 한 번 저장**
 * 순으로 처리하므로, 중간 실패가 반쪽 편집 문서를 남기지 않는다.
 *
 * ```ts
 * const plan = new Plan('서식.hwp', '제출본.hwp')
 *   .fillFields({ 성명: '홍길동' })
 *   .setCheckbox(1)
 *   .verify();
 *
 * const preview = await plan.check();   // 디스크 무변경
 * if (preview.ok) await plan.run();
 * ```
 *
 * @packageDocumentation
 */
import { Envelope } from './envelope.js';
import { type RunOptions } from './process.js';
/** 계획 step 하나 (직렬화된 형태). */
export type PlanStep = Readonly<Record<string, unknown>> & {
    readonly action: string;
};
/** 계획서 전체. */
export interface PlanDocument {
    readonly planVersion: '1.0';
    readonly input: string;
    readonly output: string;
    readonly steps: readonly PlanStep[];
    readonly assertions?: Readonly<Record<string, boolean>>;
    /** 참이면 선검증만 하고 디스크를 건드리지 않는다. */
    readonly dryRun?: boolean;
}
/** 계획 실행/검사 결과 저널. */
export declare class PlanResult extends Envelope {
    /** 위반 없이 통과했는가 (검사·실행 공통). */
    get ok(): boolean;
    /** 선검증 위반 목록. 통과했으면 빈 배열. */
    get violations(): Envelope[];
    /** 검사 전용 실행이었는가 (디스크 무변경). */
    get isDryRun(): boolean;
    /** 검사 모드의 step 별 미리보기. 실행 모드면 빈 배열. */
    get preview(): Envelope[];
    /** 실행 모드의 step 별 결과. 검사 모드면 빈 배열. */
    get steps(): Envelope[];
    /**
     * 위반을 사람이 읽을 여러 줄로 — 로그·오류 메시지에 그대로 쓴다.
     */
    describeViolations(): string;
}
/** {@link Plan.replaceText} 옵션. */
export interface PlanReplaceOptions {
    /** 이 순번 하나만 (0 기준). */
    readonly occurrence?: number | undefined;
    /** 대소문자를 구분할지. 기본 참. */
    readonly caseSensitive?: boolean | undefined;
}
/** {@link Plan.setCell} 옵션. */
export interface PlanSetCellOptions {
    /** 기존 글자 모양을 유지할지. */
    readonly keepStyle?: boolean | undefined;
}
/**
 * 계획서 빌더 — 체이닝으로 step 을 쌓는다.
 *
 * 빌더는 **문법만** 검사한다(값 타입·필수 인자). 실제 실행 가능성은 rhwp 의
 * 선검증이 판정한다 — 판정자를 두 곳에 두면 반드시 어긋난다.
 */
export declare class Plan {
    private readonly input;
    private readonly output;
    private readonly steps;
    private readonly assertions;
    constructor(input: string, output: string);
    /** 누름틀 채우기. `{ "이름#1": "값" }` 으로 동명 순번 지정. */
    fillFields(data: Readonly<Record<string, unknown>>): this;
    /** 문자열 치환. `occurrence` 를 주면 그 순번 하나만. */
    replaceText(find: string, replace: string, options?: PlanReplaceOptions): this;
    /** 표 셀 기록. 좌표는 `exportTables` 로 확인한다. */
    setCell(table: number, row: number, col: number, text: string, options?: PlanSetCellOptions): this;
    /** 빈 체크박스(□) 중 `occurrence` 번째를 표시(☑)한다. */
    setCheckbox(occurrence: number): this;
    /** 저장 직후 자기검증을 요구한다 (실패 시 저장 없이 exit 3). */
    verify(enabled?: boolean): this;
    /** 채우지 못한 필드가 하나도 없어야 한다고 단언한다. */
    requireAllFieldsFound(enabled?: boolean): this;
    /** 계획서 JSON 구조를 돌려준다 (검토·저장·전송용). */
    toJSON(options?: {
        readonly dryRun?: boolean;
    }): PlanDocument;
    /**
     * **실행하지 않고** 검사만 한다 — 디스크 무변경, step 별 미리보기 반환.
     *
     * 위반이 있으면 예외가 아니라 `result.violations` 로 돌려준다. 계획을 고쳐서
     * 다시 검사하는 것이 정상 흐름이기 때문이다.
     *
     * @throws {RhwpError} rhwp 가 계획 `--dry-run` 을 지원하지 않을 때.
     *   **조용히 실제 실행으로 내려가지 않는다** — "검사"인 줄 알고 불렀는데
     *   문서가 편집·저장되면 그보다 나쁜 배신은 없다.
     */
    check(options?: RunOptions): Promise<PlanResult>;
    /** 실행한다. 단언이 실패하면 **저장 없이** 판정이 담긴 저널을 돌려준다. */
    run(options?: RunOptions): Promise<PlanResult>;
    toString(): string;
}
/** 테스트에서 지원 여부 캐시를 비운다. */
export declare function clearPlanCapabilityCache(): void;
/**
 * 이미 만들어 둔 계획서(객체)를 그대로 실행한다.
 *
 * 빌더를 쓰지 않고 JSON 파일에서 읽어온 계획을 돌릴 때 쓴다. 선검증 위반은
 * {@link execute} 와 같은 규약으로 예외가 아니라 결과로 돌아온다.
 */
export declare function runPlan(plan: PlanDocument | Readonly<Record<string, unknown>>, options?: RunOptions): Promise<PlanResult>;
