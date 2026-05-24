# Task #1086 Stage 2 완료 보고서 — 최소 구현과 회귀 가드

- 이슈: [edwardkim/rhwp#1086](https://github.com/edwardkim/rhwp/issues/1086)
- 브랜치: `local/task1086`
- 기준: `upstream/devel` `dae5c94f`
- 입력: Stage 1 진단 결과와 작업지시자 Stage 2 승인

## 1. 구현 요약

Stage 1 에서 확인한 +2 원인은 `pi=52` 와 `pi=180` 이다. 구현 중 `pi=66` 의 vpos reset paragraph 선행 page advance 도 page count 에 관여하는 것을 확인해 함께 정정했다.

### 1.1 `pi=52` — RowBreak + rowspan block cut

변경 파일:

- `src/renderer/typeset.rs`
- `src/renderer/layout/table_layout.rs`

정정:

- RowBreak 표에서 `rowspan_touched` 행을 무조건 atomic 으로 취급하지 않는다.
- 다만 blast radius 를 줄이기 위해, row block 내부 셀 유닛에 `hard_break_before` 가 있는 경우만 `advance_row_block_cut` 을 허용한다.
- 이 가드는 `LayoutEngine::row_block_has_internal_hard_break` 로 분리했다.

결과:

```text
PartialTable pi=52 rows=0..4 cont=false end_cut=[3, 4, 2, 4, 4, 2, 20]
PartialTable pi=52 rows=2..4 cont=true  start_cut=[3, 4, 2, 4, 4, 2, 20]
```

회귀 가드:

- `samples/aift.hwp` 의 RowBreak 표 중 단순 rowspan label 케이스는 기존 행 경계 분할을 유지한다.
- 확인 지점:

```text
pi=767 rows=0..20 / rows=20..30
pi=901 rows=0..13 / rows=13..17
```

### 1.2 `pi=180` — block RowBreak 마지막 줄 trailing 높이 정합

변경 파일:

- `src/renderer/layout/table_layout.rs`
- `src/renderer/height_measurer.rs`

정정:

- block RowBreak 표의 셀 마지막 줄은 line spacing trailing 을 fit 높이에서 제외한다.
- CellBreak 와 TAC(`treat_as_char`) 표는 기존 trailing geometry 를 보존한다.

결과:

```text
PartialTable pi=180 rows=0..15 cont=false
PartialTable pi=180 rows=15..32 cont=true
```

회귀 가드:

- KTX TOC TAC 표는 기존 SVG snapshot geometry 를 보존한다.
- `tests/svg_snapshot.rs::issue_267_ktx_toc_page` 통과.

### 1.3 `pi=66` — vpos reset paragraph first-line guard

변경 파일:

- `src/renderer/typeset.rs`

정정:

- 단일 컬럼 paragraph 에서 다음 line segment 가 `vpos=0` 으로 reset 되더라도, 첫 줄이 현재 페이지의 base available height 안에 들어가면 paragraph 전체를 선행 page advance 하지 않는다.

결과:

```text
PartialParagraph pi=66 lines=0..1 vpos=67370..0 [vpos-reset@line1]
PartialParagraph pi=66 lines=1..2 vpos=0 [vpos-reset@line1]
```

## 2. 테스트 변경

신규:

- `tests/issue_1086.rs`
  - `samples/k-water-rfp.hwp` page count 가 한컴 PDF 정답인 27 pages 와 일치하는지 검증한다.

수정:

- `tests/issue_nested_table_border.rs`
  - k-water-rfp pagination 변경으로 페이지 번호가 이동하므로, 특정 page index 대신 전체 페이지에서 nested border outline 패턴을 찾도록 변경했다.

## 3. 검증 결과

직접 재현:

```text
target/release/rhwp dump-pages samples/k-water-rfp.hwp | rg -c '^=== 페이지' = 27
target/release/rhwp dump-pages samples/aift.hwp | rg -c '^=== 페이지' = 74
```

자동 검증:

```text
cargo fmt --all -- --check = pass
cargo build --release --bin rhwp = pass
cargo clippy --release --lib -- -D warnings = pass
cargo test --release --lib = pass (1352 passed, 6 ignored)
cargo test --release --tests = pass
```

기존 경고:

- `cargo test --release --lib` 와 `cargo test --release --tests` 에서 기존 테스트 코드의 warning 6건이 출력된다.
- `cargo clippy --release --lib -- -D warnings` 는 통과했다.

## 4. 결론

`samples/k-water-rfp.hwp` 는 `upstream/devel` 기준 29 pages 에서 한컴 PDF 정답과 같은 27 pages 로 정합되었다. 핵심 회귀 샘플인 `samples/aift.hwp` 는 74 pages 를 유지했고, KTX TOC SVG snapshot 도 보존했다.
