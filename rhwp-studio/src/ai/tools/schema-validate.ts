/**
 * 도구 입력 검증 — 도구 schema 가 쓰는 부분집합만 안다.
 *
 * 서버가 strict schema 로 이미 검증하지만, 실행 전에 한 번 더 본다. 모델 출력은 신뢰할 수 없는
 * 값이고, 다른 제공자는 서버 검증이 없을 수 있다.
 *
 * 지원: `type: object`(properties·required·additionalProperties: false),
 * `integer`(minimum·maximum), `string`(minLength·maxLength), `boolean`.
 */
export type ToolSchema = {
  type: 'object';
  properties: Record<string, PropertySchema>;
  required: readonly string[];
  additionalProperties: false;
};

export type PropertySchema =
  | { type: 'integer'; description: string; minimum?: number; maximum?: number }
  | { type: 'string'; description: string; minLength?: number; maxLength?: number }
  | { type: 'boolean'; description: string };

export type ValidationResult =
  | { ok: true; value: Record<string, unknown> }
  | { ok: false; error: string };

export function validateToolInput(schema: ToolSchema, input: unknown): ValidationResult {
  if (typeof input !== 'object' || input === null || Array.isArray(input)) {
    return { ok: false, error: '입력은 객체여야 합니다.' };
  }
  const value = input as Record<string, unknown>;
  for (const key of Object.keys(value)) {
    if (!(key in schema.properties)) return { ok: false, error: `알 수 없는 필드: ${key}` };
  }
  for (const key of schema.required) {
    if (!(key in value)) return { ok: false, error: `필수 필드 없음: ${key}` };
  }
  for (const [key, prop] of Object.entries(schema.properties)) {
    if (!(key in value)) continue;
    const field = value[key];
    switch (prop.type) {
      case 'integer':
        if (!Number.isSafeInteger(field)) return { ok: false, error: `${key} 는 정수여야 합니다.` };
        if (prop.minimum !== undefined && (field as number) < prop.minimum) {
          return { ok: false, error: `${key} 는 ${prop.minimum} 이상이어야 합니다.` };
        }
        if (prop.maximum !== undefined && (field as number) > prop.maximum) {
          return { ok: false, error: `${key} 는 ${prop.maximum} 이하여야 합니다.` };
        }
        break;
      case 'string':
        if (typeof field !== 'string') return { ok: false, error: `${key} 는 문자열이어야 합니다.` };
        if (prop.minLength !== undefined && field.length < prop.minLength) {
          return { ok: false, error: `${key} 는 ${prop.minLength}자 이상이어야 합니다.` };
        }
        if (prop.maxLength !== undefined && field.length > prop.maxLength) {
          return { ok: false, error: `${key} 는 ${prop.maxLength}자 이하여야 합니다.` };
        }
        break;
      case 'boolean':
        if (typeof field !== 'boolean') return { ok: false, error: `${key} 는 true/false 여야 합니다.` };
        break;
    }
  }
  return { ok: true, value };
}
