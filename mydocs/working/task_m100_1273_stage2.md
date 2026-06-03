# Task M100 #1273 Stage 2 완료 보고서

이동 커맨드 by-path 지원 — 중첩 그림 이동의 undo/redo 정상화.

## 1. 배경

Stage 1로 드래그 상태 ref에 `cellPath`가 보존되어 **이동 실시간 드래그**는 해결되었으나,
이동 종료 시 기록되는 `MovePictureCommand`/`MoveShapeCommand`가 scalar
`get/setPictureProperties`(`get/setShapeProperties`)만 호출하여, **undo/redo 시점에 중첩 그림을
본문으로 오해석** → 동일 오류 재현 위험이 남아 있었다.

## 2. 변경 내용

### `rhwp-studio/src/engine/command.ts`
- 모듈 헬퍼 3종 추가(파일의 `do*` 헬퍼 idiom과 일치):
  - `sameCellPath(a, b)`: `JSON.stringify(?? [])` 비교 — undefined/빈배열은 본문 동일 취급.
  - `moveGetProps(wasm, kind, sec, ppi, ci, cellPath?)`: cellPath 존재 시
    `getCell{Picture,Shape}PropertiesByPath`, 없으면 scalar getter.
  - `moveSetProps(wasm, kind, sec, ppi, ci, cellPath, props)`: 동일 분기로 setter.
- `MovePictureCommand`/`MoveShapeCommand`:
  - 생성자에 선택적 `cellPath?: CellPathLike` 추가(8번째, `timestamp?` 앞 — 기존 7-인자 호출 호환).
  - `execute`/`undo`를 `moveGetProps`/`moveSetProps`(kind `'image'`/`'shape'`)로 교체.
  - `mergeWith` 가드에 `sameCellPath` 비교 추가 + 재생성 시 `this.cellPath` 전달.

### `rhwp-studio/src/engine/input-handler-picture.ts`
- `finishPictureMoveDrag`(L778): `new CmdClass(...)` 8번째 인자로 `r.cellPath` 전달.
  단일 경로(`pictureMoveState.ref`, Stage 1)·다중 경로(`multiMoveRefs` spread) 모두 `r.cellPath` 보유.

## 3. 효과

- 중첩 그림 이동의 **undo/redo가 by-path API로 정상 동작**. body-level 그림은 기존 scalar 경로 유지(무변경).
- `mergeWith`가 cellPath까지 구분 → 서로 다른 위치 개체의 이동이 잘못 병합되지 않음.

## 4. 검증

- `npx tsc --noEmit`: 편집 파일(`command.ts`, `input-handler-picture.ts`) 신규 오류 **0건**.
  전체 출력이 base와 **완전 동일**(기존 무관 오류 3건 `canvaskit-renderer.ts`만 존재).
- `MovePictureCommand`/`MoveShapeCommand` 호출처는 내부 `mergeWith` 2곳 + `finishPictureMoveDrag`
  1곳뿐 — 모두 갱신 완료. WASM by-path getter/setter는 기존 번들에 존재(무변경).

## 5. 다음 단계

Stage 3 — E2E lifecycle 테스트(select→resize/rotate/move(+undo)) + 수동 시각 확인 +
`nested_picture_selection_ref_consumers_1171.md`에 "드래그 상태 staging ref 소비처 누락" 사례 추가.
