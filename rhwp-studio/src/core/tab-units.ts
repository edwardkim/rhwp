/**
 * 탭 위치와 구역 기본 탭 간격의 pt 환산.
 *
 * 둘 다 문단 여백처럼 HWPUNIT 의 2배로 저장된다(1pt = 200). 조판(style_resolver.rs)은 탭 위치를
 * 저장값의 절반으로 놓고, 한컴 HWPX 는 기본 탭 40pt 를 tabStop="8000" tabStopVal="4000" 으로 적는다.
 */
const TAB_UNITS_PER_PT = 200;

/** 저장값 → pt */
export function tabUnitsToPt(units: number): number {
  return units / TAB_UNITS_PER_PT;
}

/** pt → 저장값 */
export function ptToTabUnits(pt: number): number {
  return Math.round(pt * TAB_UNITS_PER_PT);
}
