/**
 * IR 스키마 소비 — `export-ir-schema` 를 읽어 타입 정보를 노출한다.
 *
 * 바인딩이 IR 모양을 **하드코딩하지 않는** 이유가 여기 있다. rhwp 가 IR 에 필드를
 * 더하면 스키마가 먼저 알려주고, 코드 생성기(`tools/gen-types.ts`)가 그걸 읽어
 * 타입을 다시 만든다. 수기 목록을 두면 반드시 뒤처진다.
 *
 * @packageDocumentation
 */

import { exportCapabilitiesSchema, exportIrSchema, type CommandOptions } from './commands.js';
import type { RawEnvelope } from './envelope.js';

/** JSON Schema 조각. */
export type SchemaNode = Readonly<Record<string, unknown>>;

/** `{ "$ref": "#/$defs/X" }` 에서 `X` 를 꺼낸다. */
function refName(spec: unknown): string | undefined {
  if (spec === null || typeof spec !== 'object') return undefined;
  const ref = (spec as Record<string, unknown>)['$ref'];
  if (typeof ref === 'string' && ref.startsWith('#/$defs/')) {
    return ref.slice('#/$defs/'.length);
  }
  return undefined;
}

/** 원시 타입의 TS 표기. */
const PRIMITIVE_TS: Readonly<Record<string, string>> = {
  string: 'string',
  integer: 'number',
  number: 'number',
  boolean: 'boolean',
  object: 'Record<string, unknown>',
};

/** 배열 항목이 원시 타입일 때의 표기. */
function scalarHint(spec: unknown): string {
  if (spec === null || typeof spec !== 'object') return 'unknown';
  const jsonType = (spec as Record<string, unknown>)['type'];
  if (typeof jsonType === 'string' && jsonType in PRIMITIVE_TS) {
    return PRIMITIVE_TS[jsonType] as string;
  }
  return 'unknown';
}

/** 스키마가 서술하는 필드 하나. */
export class FieldDef {
  constructor(
    /** 봉투에서의 원문 키. */
    readonly name: string,
    /** 원문 스키마 조각. */
    readonly raw: SchemaNode,
    /** 필수 필드인지. */
    readonly required: boolean,
  ) {}

  /** 설명 — 생성된 바인딩의 JSDoc 원천. */
  get description(): string {
    const value = this.raw['description'];
    return typeof value === 'string' ? value : '';
  }

  /** JSON 타입 (`object`/`array`/`string`/`integer`/`boolean`). */
  get jsonType(): string | undefined {
    const value = this.raw['type'];
    return typeof value === 'string' ? value : undefined;
  }

  /** 다른 정의를 가리키면 그 이름. */
  get ref(): string | undefined {
    return refName(this.raw);
  }

  /** 배열이면 항목이 가리키는 정의 이름. */
  get itemRef(): string | undefined {
    return refName(this.raw['items']);
  }

  /** 열거형이면 허용 값 목록. */
  get enumValues(): string[] | undefined {
    const values = this.raw['enum'];
    return Array.isArray(values) ? values.map((v) => String(v)) : undefined;
  }

  /**
   * TypeScript 타입 표기 — 코드 생성기가 그대로 쓴다.
   *
   * 열거형은 리터럴 유니온으로 낸다. TS 에서는 이게 `string` 보다 훨씬 유용하다
   * (오타를 컴파일러가 잡는다).
   */
  get tsType(): string {
    const ref = this.ref;
    if (ref) return ref;

    const enumValues = this.enumValues;
    if (enumValues && enumValues.length > 0) {
      return enumValues.map((v) => JSON.stringify(v)).join(' | ');
    }

    const jsonType = this.jsonType;
    if (jsonType === 'array') {
      const inner = this.itemRef ?? scalarHint(this.raw['items']);
      return `readonly ${inner}[]`;
    }
    if (jsonType !== undefined && jsonType in PRIMITIVE_TS) {
      return PRIMITIVE_TS[jsonType] as string;
    }

    // `oneOf` 로 null 을 허용하는 형태.
    const oneOf = this.raw['oneOf'];
    if (Array.isArray(oneOf)) {
      const names = oneOf.map((o) => refName(o)).filter((n): n is string => Boolean(n));
      const first = names[0];
      if (first !== undefined) return `${first} | null`;
    }
    return 'unknown';
  }

  toString(): string {
    return `FieldDef(${this.name}${this.required ? '' : '?'}: ${this.tsType})`;
  }
}

/** 스키마 정의(`$defs` 항목) 하나. */
export class TypeDef {
  constructor(
    /** 정의 이름. */
    readonly name: string,
    /** 원문 스키마 조각. */
    readonly raw: SchemaNode,
  ) {}

  /** 설명. */
  get description(): string {
    const value = this.raw['description'];
    return typeof value === 'string' ? value : '';
  }

  /** 객체 타입인지. */
  get isObject(): boolean {
    return this.raw['type'] === 'object';
  }

  /** `oneOf` 태그 유니온인지 (예: `Control`). */
  get isUnion(): boolean {
    return Array.isArray(this.raw['oneOf']);
  }

  /** 유니온이면 변형 정의 이름 목록. */
  get variants(): string[] {
    const oneOf = this.raw['oneOf'];
    if (!Array.isArray(oneOf)) return [];
    return oneOf.map((o) => refName(o)).filter((n): n is string => Boolean(n));
  }

  /** 필드 목록 (필수가 앞, 그 안에서 이름순). */
  get fields(): FieldDef[] {
    const props = this.raw['properties'];
    if (props === null || typeof props !== 'object') return [];

    const requiredRaw = this.raw['required'];
    const required = new Set(
      Array.isArray(requiredRaw) ? requiredRaw.map((r) => String(r)) : [],
    );

    return Object.entries(props as Record<string, unknown>)
      .map(([name, spec]) => new FieldDef(name, (spec ?? {}) as SchemaNode, required.has(name)))
      .sort((a, b) => {
        if (a.required !== b.required) return a.required ? -1 : 1;
        return a.name.localeCompare(b.name);
      });
  }

  /**
   * 이름으로 필드 하나.
   *
   * @throws {Error} 없으면. 있는 필드를 함께 알려준다.
   */
  field(name: string): FieldDef {
    const found = this.fields.find((f) => f.name === name);
    if (!found) {
      throw new Error(
        `${this.name} 에 '${name}' 필드가 없습니다. 있는 필드: ${this.fields
          .map((f) => f.name)
          .join(', ')}`,
      );
    }
    return found;
  }

  toString(): string {
    return `TypeDef(${this.name}, ${this.fields.length} fields)`;
  }
}

/** `export-ir-schema`/`export-capabilities-schema` 결과를 순회 가능한 형태로. */
export class IrSchema implements Iterable<TypeDef> {
  private readonly body: Record<string, unknown>;
  private readonly defs: Record<string, unknown>;

  constructor(private readonly envelope: RawEnvelope) {
    const schema = envelope['schema'] ?? envelope;
    if (schema === null || typeof schema !== 'object' || Array.isArray(schema)) {
      throw new TypeError('스키마 본문이 객체가 아닙니다');
    }
    this.body = schema as Record<string, unknown>;
    const defs = this.body['$defs'];
    this.defs =
      defs !== null && typeof defs === 'object' && !Array.isArray(defs)
        ? (defs as Record<string, unknown>)
        : {};
  }

  /** 스키마 버전 — 봉투 `schemaVersion` 과 별개다. */
  get version(): string {
    const fromEnvelope =
      this.envelope['irSchemaVersion'] ?? this.envelope['capabilitiesSchemaVersion'];
    const fromBody = this.body['irSchemaVersion'] ?? this.body['capabilitiesSchemaVersion'];
    const value = fromEnvelope ?? fromBody;
    return typeof value === 'string' ? value : 'unknown';
  }

  /** JSON Schema 방언 URI. */
  get dialect(): string {
    const value = this.envelope['dialect'] ?? this.body['$schema'];
    return typeof value === 'string' ? value : '';
  }

  /** 루트 타입 (보통 `Document`). */
  get root(): TypeDef {
    return this.get(refName(this.body) ?? 'Document');
  }

  /** 정의 이름 목록 (정렬). */
  names(): string[] {
    return Object.keys(this.defs).sort();
  }

  /** 정의가 있는지. */
  has(name: string): boolean {
    return name in this.defs;
  }

  /**
   * 이름으로 정의 하나.
   *
   * @throws {Error} 없으면. 있는 정의를 함께 알려준다.
   */
  get(name: string): TypeDef {
    if (!(name in this.defs)) {
      throw new Error(
        `스키마에 '${name}' 정의가 없습니다. 있는 정의: ${this.names().join(', ')}`,
      );
    }
    return new TypeDef(name, (this.defs[name] ?? {}) as SchemaNode);
  }

  /** 정의 개수. */
  get size(): number {
    return Object.keys(this.defs).length;
  }

  [Symbol.iterator](): Iterator<TypeDef> {
    const names = this.names();
    let index = 0;
    const self = this;
    return {
      next(): IteratorResult<TypeDef> {
        if (index >= names.length) return { done: true, value: undefined };
        const name = names[index] as string;
        index += 1;
        return { done: false, value: self.get(name) };
      },
    };
  }

  /**
   * 끊어진 `$ref` 를 `[참조한 곳, 없는 이름]` 으로 돌려준다.
   *
   * 코드 생성 전에 이걸 확인하면 생성기가 절반쯤 만들다 죽는 일을 막는다.
   */
  danglingReferences(): [string, string][] {
    const broken: [string, string][] = [];
    for (const typeDef of this) {
      for (const field of typeDef.fields) {
        for (const target of [field.ref, field.itemRef]) {
          if (target && !this.has(target)) {
            broken.push([`${typeDef.name}.${field.name}`, target]);
          }
        }
      }
      for (const variant of typeDef.variants) {
        if (!this.has(variant)) broken.push([typeDef.name, variant]);
      }
    }
    return broken;
  }

  /** 원문 스키마 본문. */
  get raw(): Record<string, unknown> {
    return { ...this.body };
  }

  toString(): string {
    return `IrSchema(v${this.version}, ${this.size} defs)`;
  }
}

/**
 * rhwp 에서 IR JSON Schema 를 읽어 온다.
 *
 * 문서를 입력으로 받지 않는다 — 스키마는 **타입의 자기서술**이지 특정 문서의
 * 속성이 아니다.
 */
export async function irSchema(options: CommandOptions = {}): Promise<IrSchema> {
  const envelope = await exportIrSchema(options);
  return new IrSchema(envelope.raw);
}

/** 명령 표면(capabilities)의 JSON Schema — 타입 생성기가 봉투 모양을 읽는다. */
export async function capabilitiesSchema(options: CommandOptions = {}): Promise<IrSchema> {
  const envelope = await exportCapabilitiesSchema(options);
  return new IrSchema(envelope.raw);
}
