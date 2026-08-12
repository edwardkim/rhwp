/**
 * IR 스키마 소비 — `export-ir-schema` 를 읽어 타입 정보를 노출한다.
 *
 * 바인딩이 IR 모양을 **하드코딩하지 않는** 이유가 여기 있다. rhwp 가 IR 에 필드를
 * 더하면 스키마가 먼저 알려주고, 코드 생성기(`tools/gen-types.ts`)가 그걸 읽어
 * 타입을 다시 만든다. 수기 목록을 두면 반드시 뒤처진다.
 *
 * @packageDocumentation
 */
import { type CommandOptions } from './commands.js';
import type { RawEnvelope } from './envelope.js';
/** JSON Schema 조각. */
export type SchemaNode = Readonly<Record<string, unknown>>;
/** 스키마가 서술하는 필드 하나. */
export declare class FieldDef {
    /** 봉투에서의 원문 키. */
    readonly name: string;
    /** 원문 스키마 조각. */
    readonly raw: SchemaNode;
    /** 필수 필드인지. */
    readonly required: boolean;
    constructor(
    /** 봉투에서의 원문 키. */
    name: string, 
    /** 원문 스키마 조각. */
    raw: SchemaNode, 
    /** 필수 필드인지. */
    required: boolean);
    /** 설명 — 생성된 바인딩의 JSDoc 원천. */
    get description(): string;
    /** JSON 타입 (`object`/`array`/`string`/`integer`/`boolean`). */
    get jsonType(): string | undefined;
    /** 다른 정의를 가리키면 그 이름. */
    get ref(): string | undefined;
    /** 배열이면 항목이 가리키는 정의 이름. */
    get itemRef(): string | undefined;
    /** 열거형이면 허용 값 목록. */
    get enumValues(): string[] | undefined;
    /**
     * TypeScript 타입 표기 — 코드 생성기가 그대로 쓴다.
     *
     * 열거형은 리터럴 유니온으로 낸다. TS 에서는 이게 `string` 보다 훨씬 유용하다
     * (오타를 컴파일러가 잡는다).
     */
    get tsType(): string;
    toString(): string;
}
/** 스키마 정의(`$defs` 항목) 하나. */
export declare class TypeDef {
    /** 정의 이름. */
    readonly name: string;
    /** 원문 스키마 조각. */
    readonly raw: SchemaNode;
    constructor(
    /** 정의 이름. */
    name: string, 
    /** 원문 스키마 조각. */
    raw: SchemaNode);
    /** 설명. */
    get description(): string;
    /** 객체 타입인지. */
    get isObject(): boolean;
    /** `oneOf` 태그 유니온인지 (예: `Control`). */
    get isUnion(): boolean;
    /** 유니온이면 변형 정의 이름 목록. */
    get variants(): string[];
    /** 필드 목록 (필수가 앞, 그 안에서 이름순). */
    get fields(): FieldDef[];
    /**
     * 이름으로 필드 하나.
     *
     * @throws {Error} 없으면. 있는 필드를 함께 알려준다.
     */
    field(name: string): FieldDef;
    toString(): string;
}
/** `export-ir-schema`/`export-capabilities-schema` 결과를 순회 가능한 형태로. */
export declare class IrSchema implements Iterable<TypeDef> {
    private readonly envelope;
    private readonly body;
    private readonly defs;
    constructor(envelope: RawEnvelope);
    /** 스키마 버전 — 봉투 `schemaVersion` 과 별개다. */
    get version(): string;
    /** JSON Schema 방언 URI. */
    get dialect(): string;
    /** 루트 타입 (보통 `Document`). */
    get root(): TypeDef;
    /** 정의 이름 목록 (정렬). */
    names(): string[];
    /** 정의가 있는지. */
    has(name: string): boolean;
    /**
     * 이름으로 정의 하나.
     *
     * @throws {Error} 없으면. 있는 정의를 함께 알려준다.
     */
    get(name: string): TypeDef;
    /** 정의 개수. */
    get size(): number;
    [Symbol.iterator](): Iterator<TypeDef>;
    /**
     * 끊어진 `$ref` 를 `[참조한 곳, 없는 이름]` 으로 돌려준다.
     *
     * 코드 생성 전에 이걸 확인하면 생성기가 절반쯤 만들다 죽는 일을 막는다.
     */
    danglingReferences(): [string, string][];
    /** 원문 스키마 본문. */
    get raw(): Record<string, unknown>;
    toString(): string;
}
/**
 * rhwp 에서 IR JSON Schema 를 읽어 온다.
 *
 * 문서를 입력으로 받지 않는다 — 스키마는 **타입의 자기서술**이지 특정 문서의
 * 속성이 아니다.
 */
export declare function irSchema(options?: CommandOptions): Promise<IrSchema>;
/** 명령 표면(capabilities)의 JSON Schema — 타입 생성기가 봉투 모양을 읽는다. */
export declare function capabilitiesSchema(options?: CommandOptions): Promise<IrSchema>;
