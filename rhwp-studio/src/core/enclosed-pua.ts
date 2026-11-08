// Rust `renderer::enclosed_pua`와 같은 판정이다. 렌더러가 테두리와 안쪽 글자로 합성하는
// 한컴 PUA만 다루며, 메트릭은 Rust와 같은 동봉 em 데이터를 소비한다.
import metrics from '../../../assets/fonts/enclosed-pua-metrics.json' with { type: 'json' };

export type EnclosedPua =
  | { kind: 'boxedNumber'; number: number }
  | { kind: 'circledSeal' };

export const enclosedPuaMetrics = metrics;

export function enclosedPua(codePoint: number): EnclosedPua | null {
  // [#8201] 서명·날인 기호 — 타원 안 '인'.
  if (codePoint === 0xf012b) return { kind: 'circledSeal' };
  if (codePoint >= 0xf02b0 && codePoint <= 0xf02c4) {
    return { kind: 'boxedNumber', number: codePoint - 0xf02b0 };
  }
  return null;
}

/** 테두리 안에 그리는 일반 글자. */
export function enclosedPuaInnerText(glyph: EnclosedPua): string {
  return glyph.kind === 'circledSeal' ? metrics.circledSeal.text : String(glyph.number);
}

/** Rust inner_faces와 같은 메트릭 기반 안쪽 글꼴 우선순위. */
export function enclosedPuaInnerFaces(glyph: EnclosedPua): readonly string[] {
  return glyph.kind === 'circledSeal'
    ? [metrics.innerFace, metrics.circledSeal.publicFace]
    : [metrics.innerFace];
}

/** 동봉 em 메트릭이 있는 기호. 다자리 네모 숫자는 기존 폴백을 쓴다. */
export function enclosedPuaHasMetrics(glyph: EnclosedPua): boolean {
  return glyph.kind === 'circledSeal' || glyph.number < metrics.boxedDigit.digits.length;
}
