# PR: 중첩 표 안쪽 셀 문단 시작 Backspace 무반응 수정

## 배경

Issue #2717 — rhwp-studio 에서 중첩 표(nested table) 안쪽 셀의 문단 시작 위치에서 Backspace 를 누르면 아무 반응이 없는 버그.

## 원인 분석

`input-handler-text.ts` 의 `handleBackspace` 함수에서 셀 문단 병합 여부를 판정할 때 `pos.cellParaIndex! > 0` 조건을 사용한다.

그러나 `DocumentPosition.cellParaIndex` 는 flat 필드로 `cellPath[0]` (최외곽 표) 의 문단 인덱스를 가리킨다. 중첩 표의 최내곽 셀 문단 인덱스는 `cellPath[last].cellParaIndex` 에 있다.

## 수정 내용

**파일**: `rhwp-studio/src/engine/input-handler-text.ts`

`handleDelete` 함수는 이미 `useCellPath` 변수로 cellPath 존재 시 최내곽 cellParaIndex 를 올바르게 읽는다. `handleBackspace` 에도 동일한 로직을 적용했다.

## 영향 범위
- 중첩 표 안쪽 셀에서만 동작 변화 (단일 표 / 본문 영향 없음)
