# Task M100-1071 Stage 3 완료 보고서

## 1. 목표

Stage 3의 목표는 `shape-001.hwpx` 의 TAC 도형 컨트롤이 rhwp-studio 편집 커서에서
한 글자 단위처럼 이동되는지 확인하고, 구조 컨트롤이 커서 이동 단위에 섞이는 문제를 정정하는 것이다.

## 2. 핵심 판단

Stage 2 이후 HWPX 첫 문단 control stream은 HWP 정답지와 같아졌다.

```text
[SectionDef, ColumnDef, Shape, Shape]
```

하지만 커서 이동에서 이 배열을 그대로 사용하면 `SectionDef`, `ColumnDef` 같은 구조 컨트롤까지
글자 이동 단위 후보가 된다. 이 둘은 HWP5 record stream에는 필요한 컨트롤이지만, 사용자가
좌우 방향키로 통과해야 하는 글자는 아니다.

따라서 Stage 3에서는 control position을 두 층으로 분리했다.

```text
1. raw control text position
   - HWP/HWPX record stream과 char_offsets의 원래 control slot을 보존한다.
   - public debug API인 getControlTextPositions()는 이 층을 유지한다.

2. logical control position
   - 커서/편집 이동용 위치다.
   - SectionDef, ColumnDef 같은 구조 컨트롤은 제외한다.
   - Shape, Table, Picture, Equation, Footnote, Endnote 만 한 글자 폭으로 센다.
```

## 3. 구현

수정 파일:

```text
src/document_core/helpers.rs
src/document_core/queries/doc_tree_nav.rs
src/document_core/queries/cursor_rect.rs
src/renderer/layout/paragraph_layout.rs
```

구현 내용:

```text
1. find_logical_control_positions(para) 추가
   - raw find_control_text_positions() 결과를 기반으로 한다.
   - 구조 컨트롤은 logical advance를 증가시키지 않는다.
   - TAC 도형/표/그림/수식/각주/미주는 logical advance를 1 증가시킨다.

2. navigable_text_len(), logical_paragraph_length() 정리
   - 텍스트 길이 + logical inline control 수를 기준으로 문단 탐색 길이를 계산한다.
   - CharOverlap 보정은 기존처럼 유지한다.

3. doc_tree_nav cursor 이동 경로 변경
   - navigate_next_editable(), context 복원, 컨트롤 진입/탈출 경로가 logical position을 사용한다.

4. cursor_rect/hit-test 변경
   - 인라인 도형/각주 marker의 커서 좌표와 hit-test offset도 logical position을 사용한다.

5. 조판부호 marker 변경
   - show-control-codes 모드에서 도형/그림/표 marker의 field marker 위치도 logical position으로 맞춘다.
```

## 4. 회귀 가드

신규 파일:

```text
tests/issue_1071_tac_cursor_nav.rs
```

검증 내용:

```text
1. HWPX shape-001 첫 문단에서 forward 이동:
   0 → 1 → 2 → 3 → 4

2. backward 이동:
   4 → 3 → 2 → 1 → 0

3. offset 0..4 의 cursor rect x 좌표가 역행하지 않는다.
```

`src/document_core/helpers.rs`에도 구조 컨트롤 제외 단위 테스트를 추가했다.

```text
raw positions:
  [SectionDef, ColumnDef, Footnote, Footnote] => [0, 0, 0, 2]

logical positions:
  [SectionDef, ColumnDef, Footnote, Footnote] => [0, 0, 0, 3]
```

## 5. 검증 결과

```text
cargo test --test issue_1071_tac_cursor_nav
cargo test --test issue_1067_shape_rotation
cargo test --lib logical_positions_ignore_section_and_column_controls
cargo test --test issue_598_footnote_marker_nav
cargo test --test hwpx_to_hwp_adapter stage4_section_def
cargo fmt --check
cargo check
```

결과:

```text
issue_1071_tac_cursor_nav: 2 passed, 0 failed
issue_1067_shape_rotation: 7 passed, 0 failed
logical_positions_ignore_section_and_column_controls: passed
issue_598_footnote_marker_nav: 4 passed, 0 failed
hwpx_to_hwp_adapter stage4_section_def: 2 passed, 0 failed
cargo fmt --check: success
cargo check: success
```

## 6. 결론

Stage 3는 TAC 도형 커서 이동 문제를 raw record position과 편집용 logical position의 혼동으로
정의하고 정정했다.

이제 `shape-001.hwpx` 첫 문단은:

```text
[TAC 도형][공백][공백][TAC 도형]
```

흐름으로 0부터 4까지 이동하며, 구조 컨트롤인 `SectionDef`, `ColumnDef`는 커서가 통과하는
글자 단위에 포함되지 않는다.

다음 단계는 WASM 빌드 후 rhwp-studio에서 작업지시자 시각/상호작용 판정을 받는 것이다.
