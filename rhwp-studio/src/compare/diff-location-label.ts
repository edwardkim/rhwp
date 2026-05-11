import type { DiffItem } from './types';

/** 미리보기에 쓰이는 문자열이 “변경 내용 없음”으로 간주되는지 */
export function isComparePreviewAbsent(raw: string | undefined): boolean {
  const t = (raw ?? '').trim();
  return t === '' || t === '(없음)';
}

/**
 * 한쪽 문서 기준 사람이 읽는 구역·쪽 (annotateDiffSectionPages 결과 + 스냅샷 직후 앵커).
 * - 오른쪽 `added`: path가 오른쪽 문단이라 왼쪽에는 `contextOnLeft` 구역을 씀.
 * - 왼쪽 `removed`: 오른쪽에는 `contextOnRight` 구역을 씀.
 */
export function formatParagraphLocationForSide(item: DiffItem, side: 'left' | 'right'): string | null {
  let sec: number;
  if (side === 'left' && item.severity === 'added') {
    if (!item.contextOnLeft) return null;
    sec = item.contextOnLeft.section;
  } else if (side === 'right' && item.severity === 'removed') {
    if (!item.contextOnRight) return null;
    sec = item.contextOnRight.section;
  } else {
    sec = item.path.section;
  }

  if (sec < 0) return null;

  const sectionPage = side === 'left' ? item.leftSectionPage : item.rightSectionPage;
  if (sectionPage !== undefined && sectionPage > 0) {
    return `제 ${sec + 1}구역, ${sectionPage}쪽`;
  }

  const anchor = side === 'left' ? item.leftAnchor : item.rightAnchor;
  if (anchor) return `제 ${sec + 1}구역, ${anchor.pageIndex + 1}쪽`;

  return `제 ${sec + 1}구역`;
}

/** 비교 결과 목록·요약 줄 (좌/우 각각 채워지면 둘 다 표시) */
export function formatDiffLocationCombined(item: DiffItem): string | null {
  const L = formatParagraphLocationForSide(item, 'left');
  const R = formatParagraphLocationForSide(item, 'right');
  if (!L && !R) return null;
  if (L && R) return `좌 ${L}, 우 ${R}`;
  return L ?? R;
}
