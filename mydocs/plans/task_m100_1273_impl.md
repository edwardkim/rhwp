# 구현계획서 — 글상자/셀 중첩 그림 마우스 드래그 조작 실패 수정 (M100 #1273)

- **이슈**: #1273 / **브랜치**: `local/task1273`
- **수행계획서**: `mydocs/plans/task_m100_1273.md`
- **원칙**: TypeScript 전용, 기존 by-path WASM API 재사용, 다중 선택 spread 패턴과 의미 일치

## Stage 1 — 드래그 상태 ref 타입 확장 + path 보존

**목표**: 리사이즈·회전 완전 해결 + 이동 실시간 드래그 해결.

1. `rhwp-studio/src/engine/input-handler.ts`
   - `@/core/types`에서 `CellPathLike` import 추가.
   - `pictureResizeState`(L175)·`pictureMoveState`(L191)·`pictureRotateState`(L208)의 `ref` 필드
     타입에 선택 필드 `cellPath?: CellPathLike`, `headerFooter?: { kind: 'header'|'footer';
     outerParaIdx: number; outerControlIdx: number }` 추가(가산적).
2. `rhwp-studio/src/engine/input-handler-mouse.ts`
   - L307(회전)·L323(리사이즈)·L369(이동)의 `ref: { sec, ppi, ci, type }` 리터럴에
     `cellPath: ref.cellPath, headerFooter: ref.headerFooter` 복사 추가.

**검증**: `npx tsc --noEmit`, `npm run lint`. (리사이즈·회전은 이 단계로 종결)

## Stage 2 — 이동 커맨드 by-path 지원

**목표**: 중첩 그림 이동의 undo/redo 정상화.

1. `rhwp-studio/src/engine/command.ts`
   - `MovePictureCommand`/`MoveShapeCommand` 생성자에 선택적 `cellPath?: CellPathLike` 추가.
   - `execute`/`undo`에서 `cellPath` 존재 시 `getCell{Picture,Shape}PropertiesByPath` +
     `setCell{Picture,Shape}PropertiesByPath`로 분기, 없으면 기존 scalar 유지.
   - `mergeWith` 가드에 `cellPath` 동일성 비교 추가.
   - 선례: `ResizeObjectCommand.setProps`(L830-844).
2. `rhwp-studio/src/engine/input-handler-picture.ts`
   - `finishPictureMoveDrag`(L774-783)에서 커맨드 생성 시 `r.cellPath` 전달.

**검증**: `npx tsc --noEmit`, `npm run lint`.

## Stage 3 — E2E lifecycle 테스트 + 수동 확인 + 문서 보완

**목표**: 사용자 연산 표면 검증 + 재발 방지 기록.

1. `rhwp-studio/e2e/`에 글상자 그림 대상 "select→resize", "select→rotate",
   "select→move(+undo)" 시나리오 추가(기존 `text-flow.test.mjs` 패턴). 단일·다중 선택.
2. 수동 시각 확인(글상자 + 머리말/꼬리말 그림) — 콘솔 오류 0건.
3. `mydocs/troubleshootings/nested_picture_selection_ref_consumers_1171.md`에
   "드래그 상태 staging ref 소비처 누락" 사례 추가(교훈 #2 확장).

## 단계 게이트

각 Stage 완료 후 `mydocs/working/task_m100_1273_stage{N}.md` 보고서 작성 + 소스와 함께 커밋,
승인 후 다음 Stage 진행.
