/**
 * 한글 클립보드 문서 모델(`[data-hwpjson]`)을 코어가 받을 수 있는지 미리 본다.
 *
 * 코어 `pasteHwpJson` 은 본문 제어문자 중 구역 정의·단·쪽 번호·표·그리기 개체·누름틀(`%clk`·필드 끝)만 안다
 * (`src/document_core/hwpjson/body.rs` 의 `CI_*`). 날짜·하이퍼링크 같은 다른 필드 제어문자가
 * 하나라도 있으면 코어가 오류를 내고, 그 실패가 편집 명령 오류 경계를 거치는 동안에는
 * 뒤이은 HTML 붙여넣기까지 거절돼 아무것도 붙지 않는다. 그래서 모르는 제어문자가 보이면
 * 코어를 부르기 전에 HTML 경로로 돌린다.
 */

/** 코어가 다루는 본문 제어문자 ID(4글자 ctrlId 를 big-endian 정수로). */
export const SUPPORTED_HWPJSON_CONTROL_IDS: ReadonlySet<number> = new Set([
  1_936_024_420, // secd
  1_668_246_628, // cold
  1_885_826_672, // pgnp
  1_952_607_264, // tbl
  1_735_618_336, // gso
  627_272_811, // %clk — 누름틀 시작
  157_510_763, // 	clk — 필드 끝
]);

/** 코어가 다루는 글자 안 특수 문자 코드(`ch` 의 `{cc}` 중 `co` 없는 것). 9 = 탭. */
export const SUPPORTED_HWPJSON_INLINE_CODES: ReadonlySet<number> = new Set([9]);

/**
 * 문서 모델 JSON 이 코어가 다룰 수 없는 본문 제어문자를 담고 있으면 그 ctrlId 글자들을,
 * 없으면 빈 배열을 돌려준다. JSON 이 아니면 `null`(호출한 쪽이 HTML 경로로 간다).
 * 본문 제어문자는 `{cc, ci, co}` 세 키를 함께 가진 객체다.
 */
export function unsupportedHwpJsonControls(model: string): string[] | null {
  let root: unknown;
  try {
    root = JSON.parse(model);
  } catch {
    return null;
  }
  const found = new Set<string>();
  const stack: unknown[] = [root];
  while (stack.length > 0) {
    const v = stack.pop();
    if (Array.isArray(v)) {
      for (const x of v) stack.push(x);
      continue;
    }
    if (!v || typeof v !== 'object') continue;
    const o = v as Record<string, unknown>;
    if ('cc' in o && 'co' in o && typeof o.ci === 'number' && !SUPPORTED_HWPJSON_CONTROL_IDS.has(o.ci)) {
      found.add(ctrlIdText(o.ci));
    }
    // 글자 안 특수 문자(`ch` 배열 안의 `{cc, …}` — 제어 표 참조 `co` 가 없다)는 코어가 탭(cc=9)만 안다.
    if (Array.isArray(o.ch)) {
      for (const c of o.ch as unknown[]) {
        if (c && typeof c === 'object' && !('t' in c) && !('co' in c)) {
          const cc = (c as Record<string, unknown>).cc;
          if (typeof cc === 'number' && !SUPPORTED_HWPJSON_INLINE_CODES.has(cc)) found.add(`cc:${cc}`);
        }
      }
    }
    for (const x of Object.values(o)) if (x && typeof x === 'object') stack.push(x);
  }
  return [...found];
}

function ctrlIdText(ci: number): string {
  const n = ci >>> 0;
  return String.fromCharCode((n >>> 24) & 0xff, (n >>> 16) & 0xff, (n >>> 8) & 0xff, n & 0xff);
}
