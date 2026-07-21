## 개요

Issue #2717: 중첩 표(nested table) 안쪽 셀의 첫 번째 문단 시작에서 Backspace 키를 눌렀을 때 아무 반응이 없는 버그를 수정한다.

## 원인

`handleBackspace` 함수는 커서가 셀 내부(`inCell === true`)에 있을 때, 문단 시작 위치(`charOffset === 0`)에서 Backspace를 누르면 `pos.cellParaIndex` 값을 확인하여 이전 문단과의 병합 여부를 결정한다.

그러나 중첩 표 구조에서는 `pos.cellPath`가 존재하며, 실제로 참조해야 하는 셀 문단 인덱스는 `cellPath`의 마지막(최내곽) 엔트리에 저장되어 있다. 기존 코드는 `pos.cellParaIndex`만을 사용했기 때문에 중첩 표 안쪽 셀에서는 이 값이 0 또는 undefined로 평가되어 아무 동작도 수행하지 않았다.

## 수정 내용

- `else if (pos.cellParaIndex! > 0)` 조건을 `else` 블록으로 변경
- `cellPath`가 존재하는 경우 마지막 엔트리의 `cellParaIndex`를, 그렇지 않은 경우 기존 `pos.cellParaIndex`를 사용하도록 개선
- 추출한 `cpi` 값이 0보다 클 때만 `MergeParagraphInCellCommand`를 실행

## 영향 범위

- `rhwp-studio/src/engine/input-handler-text.ts` 파일의 `handleBackspace` 함수만 수정
- 중첩 표 안쪽의 일반 표 쪽에는 영향 없음 (기존 동작 유지)
- 중첩되지 않은 일반 표 셀에서의 동작에도 영향 없음 (`cellPath`가 없으면 기존과 동일하게 `pos.cellParaIndex` 사용)
