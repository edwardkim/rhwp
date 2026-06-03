# 구현계획서 — Task #1086: k-water-rfp.hwp +2 over-split 재검토

- 이슈: [edwardkim/rhwp#1086](https://github.com/edwardkim/rhwp/issues/1086)
- 수행계획서: `mydocs/plans/task_m100_1086.md`
- 기준: `upstream/devel` `dae5c94f`

## Stage 1 — 원점 진단

### 1.1 목적

기존 폐기 진행분의 가설을 배제하고, 최신 `upstream/devel` 기준으로 +2 over-split 의 실제 기여 지점을 다시 특정한다.

### 1.2 절차

1. 한컴 정답지 `pdf/k-water-rfp-2022.pdf` 와 rhwp `dump-pages` 결과를 page-by-page 로 매칭한다.
2. `pi=52`, `pi=180`, `pi=255` 각각에 대해 다음을 기록한다.
   - 표 구조와 page break 정책
   - 현재 `PartialTable` fragment 범위
   - 각 fragment 의 `used`, `hwp_used`, diff
   - row/rowspan/cell 내부 cut 여부
3. 다음 코드 경로를 읽고 실제 분할 결정 지점을 기록한다.
   - `src/renderer/typeset.rs::typeset_block_table`
   - `src/renderer/layout/table_layout.rs::advance_row_cut`
   - `src/renderer/layout/table_layout.rs::advance_row_block_cut`
   - `src/renderer/layout/table_layout.rs::row_cut_content_height`
   - `src/renderer/layout/table_partial.rs::layout_partial_table`
4. 필요 시 진단 테스트 또는 임시 로그를 추가한다. 이 단계의 진단 코드는 구현 fix 와 분리한다.

### 1.3 판정 기준

| 결과 | 다음 단계 |
|------|-----------|
| 특정 표 하나가 +1/+2 를 대부분 설명 | Stage 2 에서 해당 경로만 좁게 수정 |
| `pi=52` 와 `pi=180` 이 각각 +1 | Stage 2/3 분할 구현 |
| 기존 issue body 의 cell 내부 paragraph overflow 가 맞음 | 셀 내부 cut 경로 중심으로 구현 |
| 다른 열린 PR 의 변경과 중복 가능 | 작업지시자에게 base/stack 여부 재확인 |

## Stage 2 — 1순위 후보 최소 구현

Stage 1 결과에 따라 하나만 선택한다.

### 후보 A — `pi=52` 셀 내부 paragraph cut 정합

검토 위치:

- `cell_units()` 의 vpos reset hard break 처리
- `advance_row_cut()` 의 예산 비교
- `row_cut_content_height()` 의 whole row vs split row cell.height 적용
- `layout_partial_table()` 의 `start_cut/end_cut` → line ranges 변환

목표:

- page 5/6 split 이 한컴 정답과 같은 위치에서 발생
- 셀 내부 문단 누락/중복 없음
- 기존 중첩 표/rowspan split 회귀 없음

### 후보 B — `pi=180` orphan row fragment 제거

검토 위치:

- `typeset_block_table()` 의 `avail_for_rows`
- `header_overhead`, `host_spacing`, `cell_spacing` 계산
- `row_cut_content_height()` 와 `MeasuredTable.row_heights` 차이
- 마지막 행 `rows=31..32` 가 page 16 에 들어갈 수 있는지

목표:

- 마지막 한 행만 단독 page 로 떨어지는 현상 제거
- row 14..32 또는 동등한 한컴 정합 split 확보

### 후보 C — `pi=255` cut 경로 잔여 보정

검토 위치:

- `end_cut=[1,21]`, `start_cut=[1,21]` 의 의미
- split start/end 에서 content height 와 visible height 정합
- 중첩 표 포함 셀의 atomic 가시성

목표:

- 기존 cut 기능을 보존하면서 실제 추가 페이지 기여 여부만 정정

## Stage 3 — hwpspec.hwp 한컴 오피스 대조 재분석

Stage 2 후 작업지시자가 `samples/hwpspec.hwp` 의 한컴 오피스 스크린샷을 추가 기준으로 제공했다. 이 기준에서는 rhwp 와 한컴 오피스가 다르므로, `k-water-rfp.hwp` 정정은 유지하면서 `hwpspec.hwp` 의 20쪽 경계와 전체 178쪽 정합을 별도 처리한다.

금지:

- vpos 전역 보정 같은 광범위 변경으로 우회
- `k-water-rfp` 파일명 기반 특수 처리
- 한 후보가 설명하지 못하는 회귀를 다른 후보에 섞어 임의 보정

구현 방향:

1. HWP3-origin page tolerance 대상 문서에서만 `vpos=200/500 HU` near-top reset 을 허용한다.
2. 표 host 문단은 near-top reset 에서 제외해 partial table 정상 흐름을 유지한다.
3. 표지성 구역 시작 패턴이 모두 맞을 때만 선행 빈 쪽을 삽입해 한컴의 홀수쪽 제목 정렬을 맞춘다.

## Stage 4 — 검증과 보고

자동 검증:

```bash
cargo fmt --all -- --check
cargo test --release --lib
cargo test --release --tests
cargo clippy --release --lib -- -D warnings
cargo build --release
```

필수 재현 검증:

```bash
target/release/rhwp dump-pages samples/k-water-rfp.hwp | rg -c '^=== 페이지'
```

목표:

```text
27
```

회귀 sweep 후보:

- `samples/exam_kor.hwp`
- `samples/exam_math.hwp`
- `samples/aift.hwp`
- `samples/biz_plan.hwp`
- `samples/hwp3-sample10-hwp5.hwp`
- 표 분할/중첩 표 관련 기존 테스트: `issue_418`, `issue_nested_table_border`

시각 검증:

- rhwp-studio/WASM 판정용 빌드 준비
- 작업지시자 한컴/PDF 기준 시각 판정 요청

## 구현 결과

작업지시자 승인 후 Stage 1 진단과 Stage 2 구현을 완료했다.

최종 구현은 Stage 2 의 세 변경과 Stage 3 의 두 변경으로 구성된다.

1. `pi=52`: RowBreak 표의 rowspan-touched 블록도 셀 내부 hard break 가 있을 때만 `advance_row_block_cut` 경로를 허용한다.
2. `pi=180`: block RowBreak 표에 한해 셀 마지막 줄 trailing line spacing 을 fit 높이에서 제외한다. TAC 표는 기존 geometry 를 보존한다.
3. `pi=66`: 단일 컬럼 paragraph 에서 다음 line segment 가 `vpos=0` 으로 reset 되더라도 첫 줄이 현재 페이지에 들어가면 paragraph 전체를 선행 page advance 하지 않는다.
4. `hwpspec.hwp page 20`: HWP3-origin page tolerance 대상에서만 `vpos=200/500 HU` near-top reset 을 인정해 `pi=89` 확장 컨트롤 그림을 다음 쪽으로 보낸다.
5. `hwpspec.hwp Part II`: 표지성 구역 시작 패턴에 한해 빈 쪽을 삽입해 전체 178쪽과 목차 쪽번호를 맞춘다.

검증 결과:

```text
samples/k-water-rfp.hwp = 27 pages
samples/aift.hwp = 74 pages
samples/hwpspec.hwp = 178 pages
samples/2022년 국립국어원 업무계획.hwp = 35 pages
cargo fmt --all -- --check = pass
cargo clippy --release --lib -- -D warnings = pass
cargo test --release --lib = pass
cargo test --release --test issue_1086 --test issue_554 = pass
cargo test --release --test issue_nested_table_border --test svg_snapshot = pass
```
