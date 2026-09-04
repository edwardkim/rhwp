# Task #1811 Stage 5 — sample16 HWP3-origin 회귀 분리

## 배경

Stage 4 완료 후 전체 테스트를 실행했을 때 `issue_1035_alignment` 가 실패했다.

- `samples/hwp3-sample16-hwp5.hwp`: 66쪽 vs 기대 64쪽
- `samples/hwp3-sample16-hwp5-2022.hwp`: 66쪽 vs 기대 64쪽

Stage 4 tracked diff 를 임시 제거한 상태에서도 동일 실패가 재현되었으므로, Stage 4 변경이
직접 원인은 아니다. PR 준비 전 현재 브랜치에서 이 회귀를 해소한다.

## 검토 방향

- Stage 3 의 HWPX RowBreak 합성 lineSeg 보강이 HWP3-origin HWP5 변환본까지 적용되는지 확인한다.
- 특정 샘플명 기준 분기는 사용하지 않는다.
- 보정은 입력 문서 속성, source format, RowBreak/Cell/LineSeg/ParaShape 속성에 근거한다.

## 목표

1. `issue_1035_alignment` 를 다시 통과시킨다.
2. Stage 3/4 의 focused regression 은 유지한다.
3. 전체 테스트를 재시도한다.

## 원인

Stage 3 에서 `recompose_for_cell_width` 가 missing lineSeg 와 synthetic lineSeg 를 함께 다루도록
확장되면서, 셀 폭에서 paragraph margin/indent 를 차감한 `text_width_px` 가 두 경로 모두에
적용되었다.

- HWPX synthetic lineSeg: 합성 줄 경계가 paragraph style 을 반영해야 하므로 차감 폭이 필요하다.
- HWP/HWP3-origin missing lineSeg: 기존 Task #671/#1042 fallback 은 셀 inner width 기준이며,
  legacy bullet tolerance `1.04` 로 보정한다.

HWP3-origin HWP5 sample16 계열은 두 번째 경로인데 폭이 과도하게 좁아져 64쪽 문서가 66쪽으로
over-split 되었다.

## 수정

- `has_synthetic_line_segs` 를 별도 계산했다.
- paragraph margin/indent 차감 폭은 synthetic lineSeg 경로에만 적용했다.
- lineSeg 가 비어 있는 HWP/HWP3-origin fallback 은 기존처럼 `cell_inner_width_px` 를 사용한다.

## 검증

- `cargo fmt`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1035_alignment -- --nocapture`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1749_saved_bounds_page_break -- --nocapture`: 통과
- `env CARGO_INCREMENTAL=0 cargo build --profile release-test --bin rhwp`: 통과
- visual sweep:
  - p4: `flagged=0/1`
    - `/Users/tsjang/rhwp/output/issue1811-stage5-p4-visual/issue1811-page-break-p4/review/review_004.png`
  - p5: `flagged=0/1`
    - `/Users/tsjang/rhwp/output/issue1811-stage5-p5-visual/issue1811-page-break-p5/review/review_005.png`
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `git diff --check`: 통과

## 전체 테스트 메모

- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 재시도 결과,
  `issue_1035_alignment` 는 통과했다.
- 이후 `tests/issue_1139_inline_picture_duplicate.rs` 에서 endnote PDF 위치 회귀 31건이 실패했다.
- 원인 분리를 위해 다음 코드를 각각/일괄로 임시 제거해 대표 실패
  `issue_1284_2024_between20_page19_question24_continues_from_pdf_top` 를 재검증했다.
  - Stage 5 composer 변경 제거: 동일 실패
  - Stage 3 composer 변경 전체 제거: 동일 실패
  - Stage 3 table_layout 변경 전체 제거: 동일 실패
  - Stage 3 document.rs 변경 전체 제거: 동일 실패
  - Stage 3/4 관련 코드 변경 전체 제거: 동일 실패
- 따라서 현재 #1811 Stage 3~5 변경이 만든 실패로 보지 않고, 별도 기존 로컬/브랜치 상태
  이슈로 분리한다.
