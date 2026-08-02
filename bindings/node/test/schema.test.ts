/**
 * IR 스키마 소비 계층 — 하드코딩 없이 타입 정보를 읽는다.
 *
 * 바인딩이 IR 모양을 손으로 적지 않는 이유를 지키는 테스트다. 스키마가 먼저 알려주고
 * 생성기(`tools/gen-types.ts`)가 따라간다. 여기서는 **실물 rhwp 없이** 최소 스키마
 * 하나로 그 읽기 규칙을 고정한다 (파이썬판 `tests/test_schema.py` 의 TS 판).
 *
 * 이 파일의 중심은 {@link FieldDef.tsType} 이다. 파이썬은 타입이 틀려도 실행은 되지만
 * TypeScript 는 **타입이 계약을 강제해야** 값어치가 있다. 특히 열거형을 `string` 으로
 * 환산해 버리면 오타가 컴파일러를 그대로 통과해, 타입을 생성한 의미가 사라진다.
 */

import { describe, expect, it } from 'vitest';

import type { RawEnvelope } from '../src/envelope.js';
import { FieldDef, IrSchema, TypeDef } from '../src/schema.js';

const DIALECT = 'https://json-schema.org/draft/2020-12/schema';

/** 실제 rhwp 없이도 읽기 규칙을 전부 밟을 수 있게 만든 최소 정의 묶음. */
const DEFS: Record<string, unknown> = {
  Document: {
    type: 'object',
    description: '문서 루트',
    properties: {
      // 참조 배열 — 생성기가 `readonly Section[]` 을 뽑아야 하는 자리.
      sections: { type: 'array', items: { $ref: '#/$defs/Section' }, description: '구역 목록' },
      pageCount: { type: 'integer', description: '쪽수' },
      title: { type: 'string', description: '제목' },
      ratio: { type: 'number', description: '배율' },
      draft: { type: 'boolean', description: '초안 여부' },
      meta: { type: 'object', description: '부가 정보' },
      cover: { $ref: '#/$defs/Preview', description: '표지' },
      preview: { oneOf: [{ $ref: '#/$defs/Preview' }, { type: 'null' }], description: '미리보기' },
      tags: { type: 'array', items: { type: 'string' }, description: '태그' },
      // 항목 모양을 알 수 없는 배열.
      notes: { type: 'array', items: {}, description: '메모' },
      // 아무 단서도 없는 필드 — 여기서 추측하면 틀린 타입이 생성된다.
      origin: { description: '출처(모양 미상)' },
    },
    required: ['sections', 'pageCount'],
    additionalProperties: true,
  },
  Section: {
    type: 'object',
    description: '구역',
    properties: {
      index: { type: 'integer', description: '구역 번호' },
      kind: { type: 'string', enum: ['header', 'body', 'footer'], description: '구역 종류' },
    },
    required: ['index'],
    additionalProperties: true,
  },
  Preview: {
    type: 'object',
    description: '미리보기',
    properties: { hasImage: { type: 'boolean', description: '이미지 여부' } },
    required: [],
    additionalProperties: true,
  },
  Control: {
    description: '컨트롤 유니온',
    oneOf: [{ $ref: '#/$defs/Section' }, { $ref: '#/$defs/Preview' }],
  },
};

/** 정의 묶음을 `export-ir-schema` 봉투 모양으로 감싼다. */
function envelopeOf(defs: Record<string, unknown>): RawEnvelope {
  return {
    schemaVersion: '1.0',
    irSchemaVersion: '1.0',
    dialect: DIALECT,
    definitionCount: Object.keys(defs).length,
    schema: {
      $schema: DIALECT,
      irSchemaVersion: '1.0',
      $ref: '#/$defs/Document',
      $defs: defs,
    },
  };
}

function fixture(): IrSchema {
  return new IrSchema(envelopeOf(DEFS));
}

/** 이름으로 필드 하나 — 없으면 `field()` 가 던지므로 조용한 통과가 없다. */
function fieldOf(schema: IrSchema, typeName: string, fieldName: string): FieldDef {
  return schema.get(typeName).field(fieldName);
}

// ── 스키마 전체 ─────────────────────────────────────────────────────────────

describe('IrSchema — 스키마 본문 읽기', () => {
  it('버전과 방언을 봉투에서 꺼낸다', () => {
    const schema = fixture();
    expect(schema.version).toBe('1.0');
    expect(schema.dialect).toContain('json-schema.org');
  });

  it('봉투에 없으면 본문에서 찾고, 그래도 없으면 unknown 이라고 말한다', () => {
    // 버전을 모르는 것과 "1.0 인 것 같다"는 다르다. 추측한 버전으로 생성기를 돌리면
    // 호환되지 않는 스키마를 호환된다고 믿는 상태가 된다.
    const fromBody = new IrSchema({ schema: { irSchemaVersion: '2.0', $schema: DIALECT, $defs: {} } });
    expect(fromBody.version).toBe('2.0');
    expect(fromBody.dialect).toBe(DIALECT);

    // capabilities 쪽 스키마는 다른 키로 버전을 싣는다.
    expect(new IrSchema({ capabilitiesSchemaVersion: '9.9', schema: { $defs: {} } }).version).toBe(
      '9.9',
    );
    expect(new IrSchema({ $defs: {} }).version).toBe('unknown');
    expect(new IrSchema({ $defs: {} }).dialect).toBe('');
  });

  it('봉투에 schema 키가 없으면 봉투 자체를 본문으로 본다', () => {
    // 명령에 따라 스키마가 봉투 안에 한 겹 더 들어가기도, 그대로 오기도 한다.
    // 둘 중 하나만 받으면 나머지 한쪽에서 "정의가 0개"라는 조용한 결과가 나온다.
    const flat = new IrSchema({ $ref: '#/$defs/Document', $defs: DEFS });
    expect(flat.size).toBe(4);
    expect(flat.root.name).toBe('Document');
  });

  it('루트를 $ref 로 찾고, 표기가 없으면 Document 로 본다', () => {
    expect(fixture().root).toBeInstanceOf(TypeDef);
    expect(fixture().root.name).toBe('Document');
    expect(fixture().root.description).toBe('문서 루트');
    expect(new IrSchema({ schema: { $defs: DEFS } }).root.name).toBe('Document');
  });

  it('정의 목록·개수·순회가 모두 같은 것을 가리키고 정렬돼 있다', () => {
    const schema = fixture();
    const names = schema.names();

    // 정렬이 계약인 이유: 생성기 출력이 순회 순서를 그대로 따르므로, 순서가 흔들리면
    // 내용이 같은데도 생성 파일 diff 가 매번 달라진다.
    expect(names).toEqual(['Control', 'Document', 'Preview', 'Section']);
    expect([...names].sort()).toEqual(names);
    expect(schema.size).toBe(4);
    expect([...schema].map((t) => t.name)).toEqual(names);
    expect([...schema].every((t) => t instanceof TypeDef)).toBe(true);
  });

  it('has 는 있고 없음을 그대로 답한다', () => {
    expect(fixture().has('Section')).toBe(true);
    expect(fixture().has('없는타입')).toBe(false);
  });

  it('없는 정의를 물으면 있는 정의를 함께 알려준다', () => {
    // "없다"만 알려주면 사용자는 이름을 다시 추측한다. 목록을 붙이면 대개 오타이고,
    // 오타는 목록을 보는 순간 끝난다.
    expect(() => fixture().get('없는타입')).toThrow(/없는타입/);
    expect(() => fixture().get('없는타입')).toThrow(/Control, Document, Preview, Section/);
  });

  it('$defs 가 없으면 빈 스키마로 읽는다 — 터지지 않고 0개라고 말한다', () => {
    const empty = new IrSchema({ schema: { type: 'object' } });
    expect(empty.size).toBe(0);
    expect(empty.names()).toEqual([]);
  });

  it('본문이 객체가 아니면 TypeError', () => {
    // 문자열·배열을 스키마로 받아들이면 이후 모든 조회가 조용히 빈 결과를 낸다.
    expect(() => new IrSchema({ schema: '객체가 아님' })).toThrow(TypeError);
    expect(() => new IrSchema({ schema: [1, 2, 3] })).toThrow(TypeError);
    expect(() => new IrSchema({ schema: 42 })).toThrow(TypeError);
  });

  it('raw 는 사본이라 밖에서 고쳐도 스키마가 오염되지 않는다', () => {
    const schema = fixture();
    const raw = schema.raw;
    raw['$defs'] = {};
    expect(schema.size).toBe(4);
  });

  it('toString 이 버전과 정의 수를 요약한다', () => {
    expect(fixture().toString()).toBe('IrSchema(v1.0, 4 defs)');
  });
});

// ── 정의 하나 ───────────────────────────────────────────────────────────────

describe('TypeDef — 정의 하나 읽기', () => {
  it('필수 필드가 앞, 그 안에서 이름순이다', () => {
    // 이 순서가 곧 생성된 인터페이스의 필드 순서다. 필수를 앞에 두면 타입을 읽는
    // 사람이 "무엇을 반드시 채워야 하는가"를 먼저 본다.
    const fields = fixture().get('Document').fields;
    const names = fields.map((f) => f.name);

    expect(fields.filter((f) => f.required).map((f) => f.name)).toEqual([
      'pageCount',
      'sections',
    ]);
    expect(names.slice(0, 2)).toEqual(['pageCount', 'sections']);

    const optional = names.slice(2);
    expect(optional).toEqual([...optional].sort((a, b) => a.localeCompare(b)));
    expect(optional).toContain('origin');
  });

  it('객체와 유니온을 구분하고, 유니온은 변형 이름을 준다', () => {
    // 둘 다 참이거나 둘 다 거짓이면 생성기가 어느 형태로 뽑을지 결정할 수 없다.
    const document = fixture().get('Document');
    const control = fixture().get('Control');

    expect(document.isObject).toBe(true);
    expect(document.isUnion).toBe(false);
    expect(control.isUnion).toBe(true);
    expect(control.isObject).toBe(false);
    expect(new Set(control.variants)).toEqual(new Set(['Section', 'Preview']));
    // 유니온은 properties 가 없다 — 없는 것을 예외로 만들지 않는다.
    expect(control.fields).toEqual([]);
    expect(fixture().get('Document').variants).toEqual([]);
  });

  it('없는 필드를 물으면 있는 필드를 함께 알려준다', () => {
    expect(() => fixture().get('Document').field('없는필드')).toThrow(/Document/);
    expect(() => fixture().get('Document').field('없는필드')).toThrow(/sections/);
  });

  it('설명이 없으면 빈 문자열이다 — undefined 를 흘리지 않는다', () => {
    // 설명은 생성된 바인딩의 JSDoc 원천이다. undefined 가 흘러들면 문서에
    // "undefined" 라는 글자가 그대로 박힌다.
    expect(new TypeDef('X', {}).description).toBe('');
    expect(new FieldDef('y', {}, false).description).toBe('');
  });

  it('toString 이 정의 이름과 필드 수를 요약한다', () => {
    expect(fixture().get('Preview').toString()).toBe('TypeDef(Preview, 1 fields)');
  });
});

// ── 타입 환산 ───────────────────────────────────────────────────────────────

describe('FieldDef.tsType — 스키마를 TypeScript 로 환산한다', () => {
  it('$ref 는 정의 이름 그대로', () => {
    const cover = fieldOf(fixture(), 'Document', 'cover');
    expect(cover.ref).toBe('Preview');
    expect(cover.tsType).toBe('Preview');
  });

  it('열거형은 리터럴 유니온이다 — 오타를 컴파일러가 잡는 지점', () => {
    // 여기서 `string` 을 내면 타입 생성의 가장 큰 이득이 사라진다. `kind: 'heder'`
    // 같은 오타가 런타임까지 살아남아, 조용히 아무것도 매치되지 않는다.
    const kind = fieldOf(fixture(), 'Section', 'kind');
    expect(kind.enumValues).toEqual(['header', 'body', 'footer']);
    expect(kind.tsType).toBe('"header" | "body" | "footer"');
    // `type: 'string'` 이 함께 있어도 열거형이 이긴다.
    expect(kind.jsonType).toBe('string');
  });

  it('배열은 항목 타입을 붙여 readonly 로 낸다', () => {
    const schema = fixture();
    const sections = fieldOf(schema, 'Document', 'sections');
    expect(sections.itemRef).toBe('Section');
    // 봉투는 읽기 전용 뷰다. 가변 배열로 내면 소비자가 응답을 고쳐 놓고 "고쳤다"고
    // 믿게 된다 — 원본 문서에는 아무 일도 일어나지 않는다.
    expect(sections.tsType).toBe('readonly Section[]');
    expect(fieldOf(schema, 'Document', 'tags').tsType).toBe('readonly string[]');
    expect(fieldOf(schema, 'Document', 'notes').tsType).toBe('readonly unknown[]');
  });

  it('원시 타입을 TS 이름으로 옮긴다', () => {
    const schema = fixture();
    expect(fieldOf(schema, 'Document', 'title').tsType).toBe('string');
    expect(fieldOf(schema, 'Document', 'pageCount').tsType).toBe('number');
    expect(fieldOf(schema, 'Document', 'ratio').tsType).toBe('number');
    expect(fieldOf(schema, 'Document', 'draft').tsType).toBe('boolean');
    expect(fieldOf(schema, 'Document', 'meta').tsType).toBe('Record<string, unknown>');
    expect(fieldOf(schema, 'Preview', 'hasImage').tsType).toBe('boolean');
  });

  it('oneOf 로 null 을 허용하는 모양은 `X | null` 이다', () => {
    // `null`(값 없음)과 필드 자체가 없는 것을 타입에서 섞으면, 소비자가 둘을 같은
    // falsy 로 다루고 "미리보기가 없다"와 "아직 안 만들었다"를 구분하지 못한다.
    expect(fieldOf(fixture(), 'Document', 'preview').tsType).toBe('Preview | null');
  });

  it('단서가 없으면 unknown 이다 — 추측하지 않는다', () => {
    // 추측한 타입은 틀렸을 때 컴파일러가 편을 들어 주므로 가장 늦게 발견된다.
    expect(fieldOf(fixture(), 'Document', 'origin').tsType).toBe('unknown');
    expect(new FieldDef('x', { type: '알수없는타입' }, false).tsType).toBe('unknown');
    expect(new FieldDef('x', { oneOf: [{ type: 'null' }] }, false).tsType).toBe('unknown');
  });

  it('toString 이 선택 필드를 물음표로 표시한다', () => {
    const required = fieldOf(fixture(), 'Document', 'sections');
    const optional = fieldOf(fixture(), 'Document', 'title');
    expect(required.toString()).toBe('FieldDef(sections: readonly Section[])');
    expect(optional.toString()).toBe('FieldDef(title?: string)');
  });
});

// ── 끊어진 참조 ─────────────────────────────────────────────────────────────

describe('IrSchema.danglingReferences — 생성 전에 잡는다', () => {
  it('건강한 스키마는 빈 배열을 준다', () => {
    // 아래 검출 테스트의 대조군이다. 이게 없으면 "항상 무언가를 찾는" 구현도
    // 검출 테스트를 통과한다.
    expect(fixture().danglingReferences()).toEqual([]);
  });

  it('필드의 $ref·items.$ref 가 끊어지면 [출처, 대상] 으로 잡는다', () => {
    // 생성기는 이 이름으로 코드를 뽑고, 컴파일이 깨지는 것은 사용자 쪽이다.
    // 절반쯤 만들다 죽는 것보다 시작 전에 목록으로 보여 주는 편이 낫다.
    const broken = new IrSchema(
      envelopeOf({
        ...DEFS,
        Document: {
          type: 'object',
          description: '문서',
          properties: {
            ghost: { $ref: '#/$defs/없는타입' },
            ghosts: { type: 'array', items: { $ref: '#/$defs/없는항목' } },
            title: { type: 'string' },
          },
          required: [],
        },
      }),
    );

    const dangling = broken.danglingReferences();
    expect(dangling).toContainEqual(['Document.ghost', '없는타입']);
    expect(dangling).toContainEqual(['Document.ghosts', '없는항목']);
    // 멀쩡한 필드까지 끌고 들어오지 않는다.
    expect(dangling.map(([source]) => source)).not.toContain('Document.title');
  });

  it('유니온 변형이 끊어져도 잡는다', () => {
    // 변형은 필드가 아니라서 필드만 훑는 구현이 통째로 놓치는 자리다.
    const broken = new IrSchema(
      envelopeOf({
        ...DEFS,
        Control: {
          description: '컨트롤',
          oneOf: [{ $ref: '#/$defs/Section' }, { $ref: '#/$defs/없는변형' }],
        },
      }),
    );
    expect(broken.danglingReferences()).toContainEqual(['Control', '없는변형']);
  });
});
