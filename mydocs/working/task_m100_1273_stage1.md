# Task M100 #1273 Stage 1 완료 보고서

드래그 상태 ref 타입 확장 + `cellPath`/`headerFooter` 보존 — 리사이즈·회전·이동 실시간 경로 정정.

## 1. 변경 내용

### (1) `rhwp-studio/src/engine/input-handler.ts`
- L20: `@/core/types`에서 `CellPathLike` import 추가.
- 단일 드래그 상태 3종(`pictureResizeState`·`pictureMoveState`·`pictureRotateState`)의 `ref` 필드
  타입에 선택 필드 추가(가산적·안전, 기존 동작 영향 없음):
  ```ts
  ref: { sec; ppi; ci; type: 'image'|'shape'|'equation'|'group';
         cellPath?: CellPathLike;
         headerFooter?: { kind: 'header'|'footer'; outerParaIdx: number; outerControlIdx: number } };
  ```
  (`lineEndpointState.ref`(`type: string`)는 변경 대상 아님 — 범위 외.)

### (2) `rhwp-studio/src/engine/input-handler-mouse.ts`
- L307(회전)·L323(리사이즈)·L369(이동) 드래그 상태 생성 시 ref 리터럴에
  `cellPath: ref.cellPath, headerFooter: ref.headerFooter`를 추가로 복사.
  (다중 선택 경로의 spread `{ ...r }` 와 동일한 path 보존 의미.)

## 2. 효과

- 드래그 시작 시 선택 ref의 `cellPath`/`headerFooter`가 staging 객체로 보존됨 →
  `updatePictureResizeDrag`/`updatePictureRotateDrag`/`updatePictureMoveDrag` 및
  `finishPictureResizeDrag`가 `setObjectProperties`에서 **by-path WASM API**로 정상 분기.
- 이로써 **리사이즈·회전은 완전 해결**(회전은 undo 커맨드 없음), **이동 실시간 드래그 해결**.
- 이동 undo/redo는 Move 커맨드가 아직 scalar이므로 Stage 2에서 처리.

## 3. 검증

- `npx tsc --noEmit`: 편집 파일에서 신규 오류 **0건**. 기존 3건은 무관 파일
  (`src/view/canvaskit-renderer.ts`, 누락 모듈 `canvaskit-wasm`)로 base와 동일(diff 없음).
- rhwp-studio에는 lint 스크립트/eslint 설정이 없어 정적 검사는 `tsc`가 기준.
- 수동/E2E 시각 검증은 Stage 3에서 일괄 수행.

## 4. 다음 단계

Stage 2 — `MovePictureCommand`/`MoveShapeCommand` by-path 지원 + `finishPictureMoveDrag` cellPath 전달.
