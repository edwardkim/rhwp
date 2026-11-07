/** 글꼴을 고르는 UI의 공통 오름차순이다. 실제 프로그램 선택 우선순위와 구분한다. */
const fontNameCollator = new Intl.Collator('ko', { numeric: true });
function languageOrder(name: string): number {
  const firstLetter = name.normalize('NFC').match(/\p{Letter}/u)?.[0] ?? '';
  if (/\p{Script=Hangul}/u.test(firstLetter)) return 0;
  if (/\p{Script=Latin}/u.test(firstLetter)) return 1;
  return 2;
}
export function compareFontNames(left: string, right: string): number {
  return languageOrder(left) - languageOrder(right)
    || fontNameCollator.compare(left.normalize('NFC'), right.normalize('NFC'));
}
