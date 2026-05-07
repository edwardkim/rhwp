# Task M100 #658 단계 2 완료보고서

## 단계명

native selection rect 정합화

## 작업 범위

`src/document_core/queries/cursor_nav.rs::get_selection_rects_native()`에서 줄 경계 offset을 처리하는 방식을 정정했다. 같은 `charOffset`이 이전 `TextRun`의 끝이면서 다음 `TextRun`의 시작인 경우, 선택 rect의 시작점과 끝점이 서로 다른 의도로 cursor hit를 찾아야 하는데 기존 코드는 render tree 순회에서 먼저 발견된 run을 그대로 사용했다.

## 원인

단계 1 진단에서 `section=1`, `parent_para=16`, `control=0`, `cell=0`의 셀 내부 선택 시 다음 패턴이 확인됐다.

- 다음 줄 시작 offset이 이전 줄 끝 `TextRun`에 매칭됨
- 그 결과 다음 줄 선택 rect의 `x`가 이전 줄 끝 좌표에서 시작함
- 이후 `right_hit`과 조합되면서 `x + width`가 페이지 폭 `1028.0px`을 넘어 최대 `1207.9px`까지 확장됨

본문 다중 줄 선택에서도 같은 경계 모호성 때문에 두 번째 줄 rect가 첫 번째 줄 y 좌표를 재사용하는 문제가 있었다.

## 변경 내용

### cursor hit bias 추가

선택 rect 계산 내부에 `CursorBias`를 추가했다.

- `Leading`: 선택 rect의 왼쪽/시작점. offset이 run 경계에 걸리면 다음 run의 시작을 우선한다.
- `Trailing`: 선택 rect의 오른쪽/끝점. offset이 run 경계에 걸리면 이전 run의 끝을 우선한다.

이에 맞춰 `find_body_cursor()`와 `find_cell_cursor()`는 첫 hit를 즉시 반환하지 않고, 후보 cursor hit를 점수화해 가장 적합한 run을 선택한다.

### 셀 경로 방어 보강

셀 내부 cursor 검색에서 `ctx.path[0]` 직접 접근 대신 `ctx.path.first()`를 사용해 비정상 cell path에서도 panic이 나지 않도록 했다.

### 회귀 테스트 추가

- `tests/issue_658_text_selection_rects.rs`

테스트 항목:

- `exam_social.hwp` 페이지 2 오른쪽 자료 박스 셀 내부 전체 선택 rect가 페이지 폭을 넘지 않는지 확인
- 같은 셀 첫 문단 3줄 선택에서 두 번째/세 번째 줄이 이전 줄 끝 y 좌표를 재사용하지 않는지 확인
- 본문 문단 `section=1`, `pi=15` 다중 줄 선택에서 두 번째 줄이 첫 줄 y 좌표를 재사용하지 않는지 확인

## 결과

단계 1 진단 예제 재실행 결과, 모든 관찰 케이스의 `overflow_count`가 0으로 변경됐다.

핵심 결과:

```text
--- data table p16 c0 paragraph 0 ---
overflow_count=0

--- data table p16 c0 paragraphs 0..6 ---
overflow_count=0

--- data table p16 c0 lower dialog ---
overflow_count=0
```

대표 rect도 다음처럼 페이지 폭 안으로 유지된다.

```text
#01 p=1 x=589.7 y=239.0 w=362.7 h=12.7 right=952.4
#02 p=1 x=589.7 y=254.2 w=107.2 h=12.7 right=696.9
```

## 검증

```bash
cargo test --test issue_658_text_selection_rects
```

결과: 2개 통과

```bash
cargo check --example inspect_658_selection
```

결과: 통과

```bash
cargo run --example inspect_658_selection
```

결과: 관찰 대상 selection rect 모두 `overflow_count=0`

```bash
cargo test --test issue_598_footnote_marker_nav
```

결과: 4개 통과

```bash
cargo test --lib --release
```

결과: 1141개 통과, 0개 실패, 2개 ignored. 기존 warning 5개는 이번 변경과 무관하다.

## 남은 작업

native rect가 페이지 바깥으로 튀는 문제는 정정됐다. 단계 3에서는 `rhwp-studio`의 선택 하이라이트 렌더링 경로를 점검해 드래그 중 DOM 삭제/재생성 비용을 줄인다.
