# 완료 보고서 — Task M100-2823

- 이슈: #2823
- 제목: 다중 선택 코너 리사이즈 확정 시 세로 배율이 가로 배율로 잘못 적용됨
- 작성일: 2026-07-22
- 브랜치: `task/m100-1-multiselect-corner-resize-scale-mismatch`

## 1. 완료 내용

`rhwp-studio/src/engine/input-handler-picture.ts` 의 `finishPictureResizeDrag`
다중 선택 리사이즈 확정 분기에서, 코너 핸들(`nw`/`ne`/`sw`/`se`) 드래그를 확정할 때
세로 배율(`sy`) 계산식이 `scaleY` 가 아니라 `scaleX` 를 재사용하고 있었다.

같은 파일의 라이브 프리뷰 담당 함수 `updatePictureResizeDrag` 는 코너에서
`sx = scaleX`, `sy = scaleY` 를 독립적으로 적용하는데, `finishPictureResizeDrag`
만 `sy` 에도 `scaleX` 를 대입해 두 함수가 어긋나 있었다. 그 결과 다중 선택 상태에서
가로/세로 비율이 다르게 코너를 드래그하면, 드래그 중에는 정상적으로 독립 배율로
커지다가 마우스를 놓는 순간 세로 크기가 가로 배율로 재계산되어 개체가 순간적으로
튀는 버그가 발생했다.

`git log -S` 로 확인한 결과 이 비대칭은 회전 리사이즈 개선 커밋(`e8e551e9`)에서
도입된 이후 지금까지 남아 있었고, 오늘 수정된 #2717/#2720/#2756/#2759/#2766/#2776
범위와는 겹치지 않는다.

## 2. 주요 변경

- `rhwp-studio/src/engine/input-handler-picture.ts`
  - `finishPictureResizeDrag` 다중 선택 코너 분기의 `sy` 계산식을
    `isCorner ? scaleX : ...` 에서 `isCorner ? scaleY : ...` 로 수정 (1줄).
- `rhwp-studio/tests/multiselect-corner-resize-scale.test.ts` (신규)
  - 소스 가드 테스트: `finishPictureResizeDrag` 의 `sy` 계산식이 코너에서
    `scaleY` 를 사용하는지 정적으로 확인해 재발(scaleX 복붙 실수)을 차단한다.

## 3. 검증 결과

통과:

- `npm test` (500개 중 499 통과, 신규 테스트 포함 전부 통과. 유일한 실패는
  기존에 알려진 pre-existing 실패 `tests/cell-flow-boundary.test.ts` — 이번
  변경과 무관)
- `npx tsc --noEmit` (baseline과 동일하게 `@wasm/rhwp.js` 관련 TS2307 2건만
  존재, 신규 타입 에러 0건)

## 4. 범위 및 제약

- TypeScript 파일만 수정했다 (`.rs` 파일 변경 없음).
- 단일 개체 리사이즈(`calcResizedBboxRotated` 경로)와 다중 선택의 코너가 아닌
  측면(`n`/`s`/`e`/`w`) 리사이즈는 이번 변경의 영향을 받지 않는다.
