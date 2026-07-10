# Task M100 #2164 Stage 2 완료보고서

- 이슈: #2164
- 브랜치: `codex/task_m100_2164`
- 재현 원본: `samples/issue2164/의견제출서(양식).hwp`
- 작성일: 2026-07-10

## 1. 이번 스테이지 목표

표 셀에서 Enter로 문단을 분할한 뒤 신규 문단의 `LINE_SEG.vertical_pos`가 앞 문단과
같아져 텍스트와 문단부호가 겹치는 코어 레이아웃 결함을 해소한다.

## 2. 구현

### 셀 문단 vpos 재연결

`src/document_core/commands/text_editing.rs`에 셀 문단 배열의 vpos를 순서대로 다시
연결하는 공통 helper를 추가했다.

- 텍스트 삽입/삭제 뒤 변경 문단부터 후속 문단까지 재계산
- Enter 분할과 Backspace 병합 전 문단의 vpos 원점 보존
- 일반 셀과 중첩 셀 path 기반 분할/병합에 같은 계약 적용
- 문단 `spacing_after`와 다음 문단 `spacing_before`를 경계 간격에 포함
- 저장 vpos가 뒤로 감소하는 RowBreak 로컬 원점 경계에서는 재계산을 중단해 페이지
  분할 신호를 보존

### 빈 문단 줄 높이

`src/renderer/composer/line_breaking.rs`에서 빈 문단은 고정 9pt fallback이나 앞 줄
치수 복사가 아니라 자신의 활성 `char_shape` 글자 크기로 줄 높이를 만든다. 기존 vpos
원점만 보존한다.

이 규칙으로 제보 원본의 13pt 빈 문단 높이는 유지하면서 TAC 그림 높이를 새 빈 문단이
상속하는 #1452 회귀는 피했다.

## 3. 회귀 테스트

`tests/issue_2164_cell_enter_overlap.rs`를 추가했다.

- 실제 제보 원본에서 대상 셀을 헤더 텍스트와 행 구조로 탐색
- `1212121212121212121` 입력 후 Enter
- 세 문단의 모델 vpos와 캐럿 y 단조 증가 검증
- Backspace 병합 후 재Enter 좌표 재검증
- HWP 저장 후 재로드한 문서에서도 문단 수, vpos, 캐럿 y 보존 검증

수정 전 RED:

```text
vpos = [0, 0, 2080]
caret y = [390.7, 386.2, 418.5]
```

수정 후 실제 브라우저:

```text
caret y = [390.7, 418.5, 446.2]
height = [17.3, 17.3, 17.3]
```

문단부호 ON/OFF에서 좌표는 동일했고, `1111`, Enter, `2222` 입력 시 텍스트 줄과
문단부호 겹침은 해소됐다.

## 4. 검증

- `cargo test --test issue_2164_cell_enter_overlap`: 2 passed
- `cargo test --lib issue1452_enter_after_dropped_inline_picture_keeps_next_para_below_picture`: passed
- `cargo test --lib test_merge_paragraph_in_cell_preserves_controls`: passed
- `cargo test --lib renderer::composer::tests`: 43 passed
- `cargo test --profile release-test --lib`: 2190 passed, 0 failed, 7 ignored
- `wasm-pack build --target web --out-dir pkg`: passed
- `rhwp-studio npm run build`: passed
- `rhwp-chrome npm run build`: passed
- `git diff --check`: passed

## 5. 다음 스테이지 잔존 문제

작업지시자 실제 화면 판정에서 `1111` 입력 후 Enter를 누르면 검은 캐럿이 신규 빈 문단으로
즉시 이동하지 않고 이전 `1111` 줄에 남아 보이는 문제가 확인됐다. 문서 코어의 신규 문단
좌표와 문단부호는 정상화됐으므로, 다음 스테이지에서는 다음 순서를 분리 계측한다.

1. `SplitParagraphInCellCommand` 반환 위치
2. `CursorState.moveTo`가 계산한 신규 문단 rect
3. `document-changed` 렌더 갱신 시점
4. `CaretRenderer` DOM 좌표 갱신 시점

현재 스테이지는 문단 겹침 해소까지만 커밋하고 캐럿 refresh 정정은 다음 스테이지에서
별도 수행한다.
