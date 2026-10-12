/**
 * `insert:field` 커맨드 매개변수 — 대화상자 없이 누름틀을 넣는 자동화 경로.
 *
 * 메뉴·툴바는 `{ anchorEl }` 만 실어 보내므로 이 키들이 하나도 없으면 기존처럼 대화상자를 연다.
 * 하나라도 있으면 대화상자 입력과 같은 값(`ClickHereProps`)을 매개변수에서 만든다. 길이 상한은
 * 대화상자와 같은 상수를 호출자가 넘긴다(#2851 직렬화 랩어라운드 방지와 같은 근거).
 */
import type { ClickHereProps } from '../ui/field-edit-dialog.ts';

/** 대화상자 대신 쓰는 매개변수 키. `type` 은 현재 누름틀(`clickhere`)만 받는다. */
export const FIELD_INSERT_PARAM_KEYS = ['name', 'guide', 'memo', 'editable', 'type'] as const;

/** 메뉴·툴바 dispatch 가 싣는 UI 전용 키 — 매개변수 검사에서 제외한다. */
const UI_ONLY_KEYS = new Set(['anchorEl']);

export interface FieldInsertLimits {
  name: number;
  guide: number;
  memo: number;
}

/** 대화상자 대신 매개변수 경로를 탈 호출인가. 형식 검사는 `parseFieldInsertParams` 가 한다. */
export function hasFieldInsertParams(params: Record<string, unknown> | undefined): boolean {
  if (!params) return false;
  return Object.keys(params).some((key) => (FIELD_INSERT_PARAM_KEYS as readonly string[]).includes(key));
}

function stringParam(params: Record<string, unknown>, key: string, max: number): string {
  const value = params[key];
  if (value === undefined) return '';
  if (typeof value !== 'string') throw new Error(`insert:field ${key} must be a string`);
  if (value.length > max) throw new Error(`insert:field ${key} exceeds ${max} characters`);
  return value;
}

/**
 * 매개변수로 누름틀 속성을 만든다.
 *
 * - 필드 키가 하나도 없으면 `null` — 호출자는 대화상자를 연다.
 * - 알 수 없는 키·잘못된 형식·상한 초과는 던진다. 자동화 호출자는 `threw` 사유와 메시지를 받는다.
 */
export function parseFieldInsertParams(
  params: Record<string, unknown> | undefined,
  limits: FieldInsertLimits,
): ClickHereProps | null {
  if (!params || !hasFieldInsertParams(params)) return null;
  const keys = Object.keys(params).filter((key) => !UI_ONLY_KEYS.has(key));
  const unknown = keys.filter((key) => !(FIELD_INSERT_PARAM_KEYS as readonly string[]).includes(key));
  if (unknown.length > 0) throw new Error(`insert:field unknown param: ${unknown.sort()[0]}`);

  if (params.type !== undefined && params.type !== 'clickhere') {
    throw new Error('insert:field type must be "clickhere"');
  }
  if (params.editable !== undefined && typeof params.editable !== 'boolean') {
    throw new Error('insert:field editable must be a boolean');
  }

  return {
    name: stringParam(params, 'name', limits.name),
    guide: stringParam(params, 'guide', limits.guide),
    memo: stringParam(params, 'memo', limits.memo),
    editable: params.editable ?? true,
  };
}
