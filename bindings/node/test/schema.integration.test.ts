/**
 * IR 스키마 통합 — 문서 모델의 자기서술을 바인딩이 그대로 읽는가.
 *
 * 이 스키마는 두 가지의 원천이다: 생성 타입(`src/ir.ts`)과 그 타입에 붙는 설명이다.
 * 그래서 여기서 보는 것은 "JSON 이 파싱되나"가 아니라 **생성기가 쓸 수 있는 상태인가**다.
 * 끊어진 `$ref` 하나, 설명 없는 정의 하나가 그대로 바인딩 사용자에게 전달된다.
 *
 * 이 rhwp 에 `export-ir-schema` 가 없으면(#3762 머지 전) 전부 건너뛴다 — 없는 명령을
 * 시험한 실패는 계약 위반과 구분되지 않아 게이트의 신호를 흐린다.
 */

import { beforeAll, describe, expect, it } from 'vitest';

import { irSchema, type IrSchema } from '../src/index.js';
import { supportsIrSchema } from './helpers/integration.js';

const schemaSupported = await supportsIrSchema();

/** 루트 표기가 이름 문자열이든 정의 객체든 같은 방식으로 읽는다. */
function typeName(value: unknown): string {
  if (typeof value === 'string') return value;
  if (value !== null && typeof value === 'object' && 'name' in value) {
    return String((value as { name: unknown }).name);
  }
  return String(value);
}

describe.skipIf(!schemaSupported)('IR 스키마 — 생성기가 쓸 수 있는 상태인가', () => {
  let loaded: IrSchema | undefined;

  beforeAll(async () => {
    loaded = await irSchema();
  });

  function schema(): IrSchema {
    if (loaded === undefined) throw new Error('IR 스키마를 아직 읽지 못했습니다');
    return loaded;
  }

  it('정의가 충분히 많고, 루트가 Document 이며, 끊어진 참조가 없다', () => {
    const names = schema().names();

    // 문서 모델이 25개도 안 되는 타입으로 표현될 리 없다. 크게 모자라면 스키마
    // 산출이 중간에 끊긴 것이다 — 그 상태로 타입을 생성하면 조용히 빈 IR 이 나온다.
    expect(names.length).toBeGreaterThanOrEqual(25);
    expect(new Set(names).size, '정의 이름이 중복된다').toBe(names.length);

    expect(typeName(schema().root)).toBe('Document');
    expect(schema().has('Document')).toBe(true);
    expect(schema().version).toBeTruthy();

    // 끊어진 `$ref` 는 "그 타입이 있다"고 약속해 놓고 없는 상태다. 생성기는 그 이름으로
    // 코드를 뽑고, 컴파일이 깨지는 것은 사용자 쪽이다.
    const dangling = schema().danglingReferences();
    expect(dangling, `끊어진 참조: ${JSON.stringify(dangling)}`).toHaveLength(0);
  });

  it('편집 API 가 다루는 타입이 스키마에 실재한다', () => {
    // 이 다섯은 바인딩의 편집 축(누름틀·표·본문)이 좌표로 지목하는 대상이다.
    // 스키마에 없으면 "무엇을 고치는지" 설명할 어휘가 사라진다.
    for (const required of ['Section', 'Paragraph', 'TableControl', 'TableCell', 'FieldRange']) {
      expect(schema().has(required), `${required} 정의가 스키마에 없다`).toBe(true);
      expect(typeName(schema().get(required))).toBe(required);
    }
  });

  it('모든 정의에 설명이 붙어 있다 — 생성 바인딩의 docstring 원천이다', () => {
    const undocumented: string[] = [];
    for (const typeDef of schema()) {
      const description = typeDef.description ?? '';
      if (description.trim().length === 0) undocumented.push(typeDef.name);
    }

    // 설명 없는 정의는 생성된 타입에 주석 없이 나타난다. 사용자는 이름만 보고
    // 무엇인지 추측하게 되고, 추측은 대개 틀린다.
    expect(undocumented, `설명 없는 정의: ${undocumented.join(', ')}`).toHaveLength(0);
  });

  it('모든 필드가 유효한 TypeScript 타입으로 환산되고, 참조 대상이 실재한다', () => {
    const problems: string[] = [];

    for (const typeDef of schema()) {
      for (const field of typeDef.fields) {
        const tsType = field.tsType;
        if (typeof tsType !== 'string' || tsType.trim().length === 0) {
          problems.push(`${typeDef.name}.${field.name}: tsType 이 비었다`);
          continue;
        }
        // `any` 로 환산되면 타입이 계약을 강제하지 못한다 — 없는 것과 같다.
        if (/\bany\b/.test(tsType)) {
          problems.push(`${typeDef.name}.${field.name}: tsType 이 any 다 (${tsType})`);
        }
        for (const reference of [field.ref, field.itemRef]) {
          if (reference && !schema().has(reference)) {
            problems.push(`${typeDef.name}.${field.name}: 없는 타입 참조 (${reference})`);
          }
        }
      }
    }

    expect(problems, problems.join('\n')).toHaveLength(0);
  });

  it('유니온 정의는 변형 목록을, 객체 정의는 필드 목록을 준다', () => {
    const problems: string[] = [];

    for (const typeDef of schema()) {
      if (typeDef.isUnion && typeDef.variants.length === 0) {
        problems.push(`${typeDef.name}: 유니온인데 변형이 없다`);
      }
      if (typeDef.isUnion && typeDef.isObject) {
        // 둘 다 참이면 생성기가 어느 형태로 뽑을지 결정할 수 없다.
        problems.push(`${typeDef.name}: 객체이면서 유니온으로 표기됐다`);
      }
      for (const field of typeDef.fields) {
        if (!typeDef.field(field.name)) {
          problems.push(`${typeDef.name}: fields 에 있는 ${field.name} 을 field() 가 못 찾는다`);
        }
      }
    }

    expect(problems, problems.join('\n')).toHaveLength(0);
  });
});
