import type { PageInfo } from '../core/types';

export interface PageSpaceRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export const PAGE_MARGIN_GUIDE_COLOR = '#C0C0C0';
export const PAGE_MARGIN_GUIDE_LINE_WIDTH = 0.6;
export const PAGE_MARGIN_GUIDE_LENGTH = 15;

export type PageMarginGuideEdges = 'both' | 'top' | 'bottom';

/**
 * 페이지 공간 사각형의 선택한 모서리에 한컴형 바깥 꺾쇠를 그린다.
 *
 * 일반 본문 여백과 머리말/꼬리말 편집 경계가 같은 모양·확대 계약을 공유하도록
 * 좌표 계산과 페인트 속성을 이 함수 하나에서 관리한다.
 */
export function drawPageMarginGuideCorners(
  rect: PageSpaceRect,
  canvas: HTMLCanvasElement,
  scale: number,
  edges: PageMarginGuideEdges = 'both',
  clip?: PageSpaceRect,
): void {
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const left = rect.x;
  const top = rect.y;
  const right = rect.x + rect.width;
  const bottom = rect.y + rect.height;
  const L = PAGE_MARGIN_GUIDE_LENGTH;

  ctx.save();
  // WASM 렌더링 후 ctx transform 상태가 불확실하므로 명시적으로 설정
  ctx.setTransform(scale, 0, 0, scale, 0, 0);
  if (clip) {
    // partial replay가 지운 영역만 가이드를 복구한다. 이 clip이 없으면 patch 밖의
    // subpixel stroke가 타건마다 누적되어 점점 진해진다.
    ctx.beginPath();
    ctx.rect(clip.x, clip.y, clip.width, clip.height);
    ctx.clip();
  }
  ctx.strokeStyle = PAGE_MARGIN_GUIDE_COLOR;
  ctx.lineWidth = PAGE_MARGIN_GUIDE_LINE_WIDTH;
  ctx.beginPath();

  if (edges !== 'bottom') {
    // 좌상 코너
    ctx.moveTo(left, top - L);
    ctx.lineTo(left, top);
    ctx.lineTo(left - L, top);

    // 우상 코너
    ctx.moveTo(right + L, top);
    ctx.lineTo(right, top);
    ctx.lineTo(right, top - L);
  }

  if (edges !== 'top') {
    // 좌하 코너
    ctx.moveTo(left - L, bottom);
    ctx.lineTo(left, bottom);
    ctx.lineTo(left, bottom + L);

    // 우하 코너
    ctx.moveTo(right, bottom + L);
    ctx.lineTo(right, bottom);
    ctx.lineTo(right + L, bottom);
  }

  ctx.stroke();
  ctx.restore();
}

/** 편집 용지 여백 가이드라인을 캔버스에 그린다 (4모서리 L자 표시). */
export function drawPageMarginGuides(
  pageInfo: PageInfo,
  canvas: HTMLCanvasElement,
  scale: number,
  clip?: PageSpaceRect,
): void {
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const {
    width,
    height,
    marginLeft,
    marginRight,
    marginTop,
    marginBottom,
    marginHeader,
    marginFooter,
  } = pageInfo;
  // 한컴 HWP 기준: 본문 시작 = marginHeader + marginTop
  const top = marginHeader + marginTop;
  const right = width - marginRight;
  // 한컴 HWP 기준: 본문 끝 = height - marginFooter - marginBottom
  const bottom = height - marginFooter - marginBottom;
  drawPageMarginGuideCorners(
    {
      x: marginLeft,
      y: top,
      width: right - marginLeft,
      height: bottom - top,
    },
    canvas,
    scale,
    'both',
    clip,
  );
}
