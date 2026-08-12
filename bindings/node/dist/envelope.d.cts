/**
 * 봉투 → TypeScript 객체 매핑.
 *
 * 설계 판단: 명령마다 클래스를 손으로 쓰지 않는다. 봉투는 "필드 추가 허용" 계약이라
 * 수기 타입은 rhwp 가 필드를 더할 때마다 뒤처지고, 뒤처졌다는 사실조차 드러나지
 * 않는다. 대신 {@link Envelope} 하나가 봉투를 감싸고, **타입 파라미터**로 정적 모양을
 * 받는다. 그 타입은 `tools/gen-types.ts` 가 `capabilities` 에서 생성한다.
 *
 * 파이썬판(M18)은 동적 매핑으로 충분했지만 TypeScript 는 다르다 — 타입이 계약을
 * **강제**해야 값어치가 있다. 그래서 `.raw` 가 제네릭으로 좁혀지고, 동시에
 * {@link Envelope.get} 이 "없는 필드는 조용한 undefined 가 아니라 예외"를 지킨다.
 *
 * @packageDocumentation
 */
/** 봉투의 기본 모양 — 생성 타입이 없을 때의 안전한 상한. */
export type RawEnvelope = Record<string, unknown>;
/** `verify` 하위 봉투 원문. */
export interface RawVerifyReport {
    readonly identical?: boolean;
    readonly diffCount?: number | null;
    readonly reparseError?: string;
}
/**
 * 저장 직후 자기검증 결과.
 *
 * `null`(검증 안 함)과 실패는 다르다 — 이 둘을 섞으면 **검증하지 않은 저장을 통과로
 * 읽는다.** 그래서 이 클래스는 "검증을 요청했을 때만" 존재한다.
 */
export declare class VerifyReport {
    private readonly report;
    constructor(report: RawVerifyReport);
    /** 저장본이 메모리 IR 과 동일한가. 이 값이 판정의 전부다. */
    get identical(): boolean;
    /** 차이 개수. 재파싱 자체가 실패했으면 null. */
    get diffCount(): number | null;
    /** 저장본을 다시 읽지 못했을 때의 사유. 정상이면 undefined. */
    get reparseError(): string | undefined;
    /** 원문. */
    get raw(): RawVerifyReport;
    toString(): string;
}
/**
 * `--json` 봉투 하나를 감싸는 읽기 전용 뷰.
 *
 * @typeParam T - 생성된 봉투 타입. 주지 않으면 {@link RawEnvelope}.
 *
 * 세 가지 방식으로 같은 값에 닿는다:
 *
 * ```ts
 * env.raw.pageCount        // 정적 타입 (생성 타입이 있을 때)
 * env.get('pageCount')     // 없으면 예외 — 오타를 조용히 넘기지 않는다
 * env.getPath('verify.identical')  // 점 경로
 * ```
 */
export declare class Envelope<T extends RawEnvelope = RawEnvelope> {
    private readonly source;
    /** snake_case 로 물었을 때 원문 키를 찾기 위한 색인. */
    private readonly snakeIndex;
    constructor(source: T);
    /**
     * 원문 봉투 (**사본**).
     *
     * 생성 타입을 주면 여기서 정적으로 좁혀진다 — `env.raw.pageCount` 가 `number`.
     */
    get raw(): T;
    /** 봉투에 있는 키 목록. */
    keys(): string[];
    /** 필드가 있는지. */
    has(key: string): boolean;
    /**
     * 필드 하나를 꺼낸다. 원문 키·snake_case·camelCase 를 모두 받는다.
     *
     * @throws {Error} 없는 필드일 때. **조용한 undefined 를 돌려주지 않는다** —
     *   오타가 "값 없음"으로 둔갑하면 그게 가장 찾기 어려운 버그가 된다. 메시지에
     *   있는 필드를 함께 담아 즉시 고칠 수 있게 한다.
     */
    get<V = unknown>(key: string): V;
    /**
     * 없으면 기본값을 돌려준다 — "없어도 되는" 선택 필드를 읽을 때만 쓴다.
     *
     * 필수 필드에 이걸 쓰면 {@link get} 의 보호를 스스로 버리는 것이다.
     */
    getOr<V>(key: string, fallback: V): V;
    /**
     * `"verify.identical"` 처럼 점 경로로 꺼낸다. 없으면 `undefined`.
     */
    getPath<V = unknown>(dotted: string): V | undefined;
    /** 봉투 스키마 버전. */
    get schemaVersion(): string | undefined;
    /**
     * `--verify` 보고가 있으면 {@link VerifyReport}, **미요청이면 `null`**.
     *
     * `null` 은 "검증 안 함"이지 "검증 실패"가 아니다.
     */
    get verify(): VerifyReport | null;
    /**
     * 편집이 바꾼 쪽 목록(0 기준). **확정 불가·무산출이면 `null`.**
     *
     * `null`(모른다)과 `[]`(바뀐 쪽이 없다)는 다른 결론이다. 둘을 falsy 로
     * 뭉뚱그리면 "확인할 게 없다"고 잘못 판단한다.
     */
    get changedPages(): number[] | null;
    /** 하위 객체를 봉투로 감싸 돌려준다. */
    child<C extends RawEnvelope = RawEnvelope>(key: string): Envelope<C> | null;
    /**
     * 배열 필드를 봉투 배열로.
     *
     * 배열 항목이 다시 배열인 경우(`[[1,2]]`)를 걸러낸다 — `Envelope` 생성자가
     * 배열을 거부하므로, 안 거르면 조회 한 번이 `TypeError` 로 터진다.
     * 객체가 아닌 항목은 봉투가 될 수 없으니 조용히 제외하는 것이 맞다.
     */
    children<C extends RawEnvelope = RawEnvelope>(key: string): Envelope<C>[];
    /** JSON 직렬화 시 원문을 그대로 내보낸다. */
    toJSON(): T;
    toString(): string;
}
/** dict 를 {@link Envelope} 로 (이미 봉투면 그대로). */
export declare function asEnvelope<T extends RawEnvelope = RawEnvelope>(value: T | Envelope<T>): Envelope<T>;
/**
 * batch 계열이 내는 NDJSON 한 줄.
 *
 * 봉투로 감싸지 않는다 — 부분 실패를 `error` 필드로 판별하는 것이 이 축의 계약인데,
 * {@link Envelope.get} 은 없는 필드에 예외를 던지므로 `error` 유무 검사가 번거로워진다.
 */
export interface BatchRecord {
    readonly schemaVersion?: string;
    readonly source?: string;
    /** 있으면 이 항목은 실패다. 스트림에서 사라지지 않고 레코드로 남는다. */
    readonly error?: string;
    readonly [key: string]: unknown;
}
