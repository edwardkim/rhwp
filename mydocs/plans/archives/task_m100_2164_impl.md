# Task #2164 구현 계획서

## 1. 수정 계약

표 셀 안 문단을 Enter로 분할하거나 Backspace로 병합한 뒤에는 다음 계약을 만족해야 한다.

1. 편집 시작 문단의 첫 줄 vpos 기준점은 유지한다.
2. 각 후속 문단의 첫 줄 vpos는 앞 문단의 마지막 줄 끝과 문단 간격 뒤에 온다.
3. 한 문단 안 여러 줄의 상대 vpos는 reflow 결과를 그대로 유지한다.
4. 신규 문단을 삽입하면 기존 후속 문단 전체가 신규 문단 높이만큼 전진한다.
5. 문단을 병합하면 기존 후속 문단 전체가 제거된 문단 높이만큼 당겨진다.

## 2. 구현 범위

- `src/document_core/commands/text_editing.rs`
  - 셀 문단 배열의 `LINE_SEG.vertical_pos`를 순서대로 재연결하는 내부 helper를 추가한다.
  - `split_paragraph_in_cell_native`에서 분할 전 기준 vpos를 보존하고 양쪽 reflow 뒤
    해당 문단부터 후속 문단까지 재계산한다.
  - `merge_paragraph_in_cell_native`에서도 병합 문단 reflow 뒤 같은 재계산을 수행한다.
- 문단 높이는 reflow된 `line_height + line_spacing`을 사용하고, 문단 경계에는 해석된
  `spacing_after`와 다음 문단의 `spacing_before`를 HWPUNIT으로 환산해 포함한다.
- 샘플명, 셀 번호, 입력 문자열에 따른 분기는 추가하지 않는다.

## 3. 회귀 테스트

`tests/issue_2164_cell_enter_overlap.rs`에서 실제 제보 원본을 사용한다.

- `의견제출 요지` 다음 큰 셀을 문서 구조로 찾는다.
- 첫 셀 문단에 텍스트를 넣고 Enter를 수행한다.
- 신규 문단의 cursor y가 이전 문단보다 아래인지 검증한다.
- 기존 후속 문단도 신규 문단보다 아래인지 검증한다.
- Backspace 병합 뒤 셀 문단 수와 후속 문단 y가 원래 순서로 복구되는지 검증한다.
- 문단부호는 코어 문서 데이터가 아닌 뷰 설정이므로 브라우저에서 ON/OFF 좌표 동등성을
  별도 확인한다.

## 4. 회귀 위험과 방어

- 저장 `LINE_SEG`를 사용하는 정적 표 배치는 건드리지 않는다.
- 일반 본문 vpos 재계산과 페이지네이션 알고리즘은 변경하지 않는다.
- 셀 구조 편집 직후에만 재계산한다.
- focused 테스트 뒤 전체 Rust 테스트, clippy, WASM build, 실제 브라우저 Enter/Backspace,
  Chrome 확장 개발 빌드를 순서대로 검증한다.

## 5. 완료 기준

- 실제 제보 원본에서 Enter 후 앞 문단, 신규 문단, 기존 후속 문단 y가 단조 증가한다.
- 문단부호 ON/OFF 모두 텍스트와 조판부호가 겹치지 않는다.
- Backspace 후 다시 Enter해도 첫 Enter와 같은 배치가 재현된다.
- 저장 후 재열기 및 Chrome 확장 개발 빌드에서도 같은 결과를 확인한다.
