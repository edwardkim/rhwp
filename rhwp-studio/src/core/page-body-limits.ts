import type { PageDef } from '@/core/types';

/**
 * 쪽 설정이 본문을 남겨 두는지 판정한다.
 *
 * 마주보는 여백의 합이 용지를 넘으면 본문 영역이 소멸하는데, 렌더러는 그때 용지의 5% 기본
 * 여백으로 폴백한다(Rust model/page.rs 의 [Task #1583]). 화면은 그럴듯하게 나오고 저장되는
 * PageDef 만 쓸 수 없는 값이 되므로, 사용자가 값을 넣는 자리에서 막는다.
 *
 * 눈금자 핀 드래그와 편집 용지 대화상자가 같은 한도를 쓴다 — 한쪽만 막으면 같은 문서를
 * 다른 입력으로 만들 수 있다.
 */

/** 본문이 남아 있어야 하는 최소 크기 (mm) */
export const MIN_BODY_MM = 10;

const HWPUNIT_PER_MM = 7200 / 25.4;

/** 세로 방향이면 용지 크기를 뒤바꾼다 — PageDef 는 원본(세로) 크기로 저장한다 */
function paperSize(def: PageDef): { width: number; height: number } {
  return def.landscape
    ? { width: def.height, height: def.width }
    : { width: def.width, height: def.height };
}

/** 본문 가로/세로 크기 (HWPUNIT). 음수면 여백이 용지를 넘었다는 뜻이다. */
export function pageBodySize(def: PageDef): { width: number; height: number } {
  const paper = paperSize(def);
  return {
    width: paper.width - def.marginLeft - def.marginRight - def.marginGutter,
    height: paper.height - def.marginTop - def.marginBottom - def.marginHeader - def.marginFooter,
  };
}

/**
 * 본문이 최소 크기만큼 남지 않으면 사람이 읽을 수 있는 이유를, 남으면 null 을 돌려준다.
 */
export function pageBodyViolation(def: PageDef): string | null {
  const min = MIN_BODY_MM * HWPUNIT_PER_MM;
  const body = pageBodySize(def);
  const mm = (hwpunit: number) => (hwpunit / HWPUNIT_PER_MM).toFixed(1);

  if (body.width < min) {
    return `좌우 여백이 너무 커서 본문이 남지 않습니다 (본문 너비 ${mm(body.width)}mm, `
      + `최소 ${MIN_BODY_MM}mm). 왼쪽·오른쪽·제본 여백을 줄여주세요.`;
  }
  if (body.height < min) {
    return `위아래 여백이 너무 커서 본문이 남지 않습니다 (본문 높이 ${mm(body.height)}mm, `
      + `최소 ${MIN_BODY_MM}mm). 위쪽·아래쪽·머리말·꼬리말 여백을 줄여주세요.`;
  }
  return null;
}
