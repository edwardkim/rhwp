# TASK-M100-2717: 중첩 표 안쪽 셀 문단 시작 Backspace 무반응 수정

## 개요
중첩 표(nested table) 안쪽 셀에서 문단 시작 위치(커서 charOffset=0)에서 Backspace 키를 누르면
이전 셀 문단과 병합되어야 하나, 아무 반응이 없는 버그를 수정한다.

## 원인 분석
- `input-handler-text.ts`의 `handleBackspace` 함수에서 셀 문단 시작 Backspace 처리 시
  `pos.cellParaIndex`를 사용하여 이전 문단 존재 여부를 판단한다.
- `DocumentPosition.cellParaIndex`는 **외부 표(outer table) 기준 레거시 flat 필드** 로,
  중첩 표 안쪽 셀에서는 외부 셀의 cellParaIndex를 가리킨다.
- 이로 인해 안쪽 셀의 실제 문단 인덱스가 아닌 엉뚱한 값을 참조하여 조건문(`cpi > 0`)이
  실패하고, MergeParagraphInCellCommand가 실행되지 않는다.
- 동일 파일의 `handleDelete`는 이미 `cellPath`의 마지막 엔트리(최내곽 셀)에서
  `cellParaIndex`를 읽어 올바르게 동작하지만, `handleBackspace`는 이 처리가 누락되었다.

## 수정 내용
- `handleBackspace`의 `inCell` 브랜치에서 `cellPath` 존재 시
  `cellPath[cellPath.length - 1].cellParaIndex`(최내곽 셀의 문단 인덱스)를 우선 사용하고,
  없을 때만 flat `pos.cellParaIndex`로 폴백하도록 수정.
- `handleDelete`의 동일 패턴과 일관성을 유지.

## 변경 파일
- `rhwp-studio/src/engine/input-handler-text.ts`

## 영향 범위
- 중첩 표 안쪽 셀 문단 시작 Backspace → 이전 문단 병합 정상 동작
- 단일 표(non-nested)는 flat 필드 폴백으로 기존 동작 유지
- MergeParagraphInCellCommand는 내부적으로 `cellParaIndexOf()`를 통해 이미 중첩 셀을
  올바르게 처리하므로, 호출부의 조건문만 수정하면 충분

Closes #2717
