# Task #685 Stage 1 단계 보고서 — `getPageLeftResolved` 헬퍼 추가 + verbose 패턴 정리

- **이슈**: [#685](https://github.com/edwardkim/rhwp/issues/685)
- **수행계획서**: [task_m100_685.md](../plans/task_m100_685.md)
- **구현 계획서**: [task_m100_685_impl.md](../plans/task_m100_685_impl.md)
- **단계 위치**: 3 단계 중 1/3
- **변경 성격**: 동치 refactor (동작 변경 없음)
- **작성일**: 2026-05-08

---

## 변경 요약

| 파일 | 변경 | 비고 |
|------|------|------|
| `rhwp-studio/src/view/virtual-scroll.ts` | +12 LOC | `getPageLeftResolved(pageIdx, containerWidth)` 헬퍼 추가 |
| `rhwp-studio/src/engine/input-handler.ts` | -3 LOC, +1 LOC | `formBboxToOverlayRect` 내 verbose sentinel 패턴을 헬퍼 호출로 단순화 |

총 코드 변경: ~+10 LOC. 기능적 동작 변경 없음 (헬퍼 동치성 검증).

---

## 1. virtual-scroll.ts: 헬퍼 추가

[`rhwp-studio/src/view/virtual-scroll.ts`](../../rhwp-studio/src/view/virtual-scroll.ts) 의 `getPageLeft(pageIdx)` 직후 다음 메서드 추가:

```ts
/**
 * 페이지의 X 좌표를 그리드/단일 컬럼 모드 통합으로 반환.
 * 그리드 모드: pageLefts[i] 그대로.
 * 단일 컬럼 모드(sentinel −1): (containerWidth - pageWidth) / 2 fallback.
 */
getPageLeftResolved(pageIdx: number, containerWidth: number): number {
  const pl = this.pageLefts[pageIdx] ?? -1;
  if (pl >= 0) return pl;
  const pw = this.pageWidths[pageIdx] ?? 0;
  return (containerWidth - pw) / 2;
}
```

기존 [`getPageLeft(pageIdx)`](../../rhwp-studio/src/view/virtual-scroll.ts#L155-L157) 는 그대로 보존 — `canvas-view.ts`, `field-marker-renderer.ts`, `caret-renderer.ts` 의 4 호출자 무회귀.

## 2. input-handler.ts: formBboxToOverlayRect 단순화

[`rhwp-studio/src/engine/input-handler.ts`](../../rhwp-studio/src/engine/input-handler.ts) 의 `formBboxToOverlayRect` 메서드 (양식 개체 bbox → scroll-content 절대 좌표 변환):

**Before**:
```ts
const scrollContent = this.container.querySelector('#scroll-content');
const contentWidth = scrollContent?.clientWidth ?? 0;
const pageDisplayWidth = this.virtualScroll.getPageWidth(pageIdx);
const pageLeft = this.virtualScroll.getPageLeft(pageIdx) >= 0
  ? this.virtualScroll.getPageLeft(pageIdx)
  : (contentWidth - pageDisplayWidth) / 2;
```

**After**:
```ts
const scrollContent = this.container.querySelector('#scroll-content');
const contentWidth = scrollContent?.clientWidth ?? 0;
const pageLeft = this.virtualScroll.getPageLeftResolved(pageIdx, contentWidth);
```

`pageDisplayWidth` 변수는 이 메서드 내 다른 사용처가 없어 제거 (`bbox.w * zoom` 가 width 직접 사용 — Read 검증).

→ 헬퍼의 두 모드(sentinel/grid) 동작 동치성을 자연스럽게 검증.

---

## 검증 결과

### 1. typecheck

```
$ cd rhwp-studio && npx tsc --noEmit
(무에러 — exit 0)
```

### 2. vite build

```
$ npx vite build
✓ 85 modules transformed.
✓ built in 725ms
PWA v1.2.0
mode      generateSW
precache  52 entries (23195.46 KiB)
```

### 3. 동치성 e2e 무회귀 sanity (`body-outside-click-fallback.test.mjs`, headless)

```
$ node e2e/body-outside-click-fallback.test.mjs --mode=headless
exit=0
```

핵심 측정 결과 (samples/exam_kor.hwp 16p, viewport 기본):
- page 15 꼬리말 영역: `buggyPageX=561.3, correctPageX=561.3` (zoom=1.0 단일 컬럼 → 두 공식 동치)
- page 0 본문 / page 1 꼬리말 click 모두 가설 (a)/(b)/(c) 모두 negative — 회귀 없음.

→ 본 변경은 양식 오버레이 좌표 산출에서 **동작 비트 단위로 동일** 함을 확인 (단일 컬럼 모드 sentinel fallback 경로).

---

## 회귀 위험 점검

| 영역 | 위험 | 결과 |
|------|------|------|
| `formBboxToOverlayRect` 단일 컬럼 모드 | 헬퍼 fallback 식이 기존과 다르면 양식 오버레이 위치 어긋남 | OK — 식 동일 (`(contentWidth - pageWidth) / 2`) |
| `formBboxToOverlayRect` 그리드 모드 | 헬퍼 grid 경로가 `getPageLeft(pageIdx) >= 0` 분기와 다르면 변경 발생 | OK — 양쪽 모두 `pageLefts[i]` 반환 |
| `pageDisplayWidth` 변수 제거 | 다른 곳에서 변수 참조 시 컴파일 실패 | OK — typecheck 통과 |
| 기존 `getPageLeft` 호출자 4곳 (canvas-view 등) | 헬퍼 추가가 기존 메서드 변경 시 영향 | OK — 기존 메서드 미변경 |

---

## 다음 단계

Stage 2 — `input-handler-mouse.ts` 14곳 헬퍼 일괄 치환 (구현계획서 Stage 2 절차 따름).

승인 요청 → 승인 시 Stage 2 진행.
