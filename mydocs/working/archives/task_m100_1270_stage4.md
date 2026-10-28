# Task #1270 Stage 4 완료보고서 — 회귀 검증

## 범위

- 신규 #1270 회귀 테스트 실행
- 인라인 picture 중복 방지 회귀 테스트 실행
- 표 셀 TAC picture 및 TopAndBottom picture 흐름 회귀 테스트 실행
- lib 전체 테스트 실행
- clippy warning 게이트 확인
- 표/캡션 추가 회귀 테스트 실행

## 검증 결과 요약

| 검증 | 결과 |
|------|------|
| `cargo test --test issue_1270_caption_inline_image` | 통과 — 2 passed |
| `cargo test --test issue_1139_inline_picture_duplicate` | 통과 — 85 passed |
| `cargo test --test issue_1352_table_cell_tac_picture_text` | 통과 — 1 passed |
| `cargo test --test issue_1459_topbottom_picture_reflow` | 통과 — 3 passed |
| `cargo test --lib` | 통과 — 1959 passed, 6 ignored |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `cargo test --test issue_530` | 통과 — 1 passed |
| `cargo test --test issue_1486_hwpx_partial_tac_table` | 통과 — 6 passed |

## 상세 결과

### 신규 #1270 테스트

```bash
cargo test --test issue_1270_caption_inline_image
```

```text
running 2 tests
test table_caption_inline_tac_picture_emits_image_node ... ok
test table_caption_picture_only_tac_paragraph_emits_image_node ... ok

test result: ok. 2 passed; 0 failed
```

### 인라인 picture 중복 방지

```bash
cargo test --test issue_1139_inline_picture_duplicate
```

```text
running 85 tests
...
test result: ok. 85 passed; 0 failed
```

### 표 셀 TAC picture

```bash
cargo test --test issue_1352_table_cell_tac_picture_text
```

```text
running 1 test
test hy001_first_cell_tac_picture_stays_inside_center_aligned_cell ... ok

test result: ok. 1 passed; 0 failed
```

### TopAndBottom picture 흐름

```bash
cargo test --test issue_1459_topbottom_picture_reflow
```

```text
running 3 tests
test turning_first_picture_off_reflows_remaining_tac_picture ... ok
test topbottom_second_picture_flows_before_tac_picture ... ok
test non_tac_topbottom_picture_is_not_caret_stop ... ok

test result: ok. 3 passed; 0 failed
```

### lib 전체 테스트

```bash
cargo test --lib
```

```text
test result: ok. 1959 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out
```

### clippy

```bash
cargo clippy --lib -- -D warnings
```

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.64s
```

### 표/캡션 추가 회귀

```bash
cargo test --test issue_530
```

```text
running 1 test
test issue_530_tac_top_caption_does_not_overlap_header_row ... ok

test result: ok. 1 passed; 0 failed
```

```bash
cargo test --test issue_1486_hwpx_partial_tac_table
```

```text
running 6 tests
test issue_1486_page29_tac_logo_aligns_with_text_line ... ok
test issue_1486_page13_page_number_keeps_footer_gap ... ok
test issue_1486_rowspan_block_tail_stays_on_pdf_page14 ... ok
test issue_1486_partial_table_tac_nested_table_stays_inside_page_body ... ok
test issue_1486_page19_nested_square_picture_is_not_page_clipped ... ok
test issue_1486_terminal_rowbreak_sliver_does_not_push_pdf_page22_content ... ok

test result: ok. 6 passed; 0 failed
```

## 판정

Stage 4 회귀 검증은 통과로 판정한다.

- 신규 테스트가 텍스트 포함 캡션 및 picture-only 캡션의 인라인 TAC picture `ImageNode` 방출을 검증한다.
- 기존 인라인 picture 중복 방지 테스트가 통과했다.
- 기존 표 셀 TAC picture와 TopAndBottom picture 흐름 테스트가 통과했다.
- 표 Top caption 및 partial TAC table 경로 테스트도 통과했다.
- lib 전체 테스트와 clippy warning 게이트가 통과했다.

## 잔여 범위

메인테이너 지시대로 다음은 후속 범위로 남긴다.

- 캡션 내 플로팅 picture 배치 일반화
- 첨부 샘플의 `image2` 완전 시각 해소
- 캡션 속 그림의 caption 재귀 렌더링
- 이슈 #1270 close

## 다음 단계

작업지시자 승인 후 Stage 5 최종 보고서를 작성하고 오늘 할일 문서를 완료 상태로 갱신한다.
