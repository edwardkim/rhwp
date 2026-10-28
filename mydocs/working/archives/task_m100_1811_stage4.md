# Task #1811 Stage 4 — mixed text/table RowBreak partial 순서 보정

## 배경

Stage 3 에서 HWPX/HWP 내부 page count, pi52 cut parity, pi57 TAC RowBreak 합성 lineSeg
보강은 통과했다. 그러나 한글 2024 PDF 기준 visual sweep 에서는 p4/p5 차이가 남았다.

- p4 PDF: pi52 본문 `사회기여` 텍스트가 같은 페이지에서 먼저 소비되고, 이어서 같은 문단의
  `사회기여 봉사활동 아이디어` 표가 페이지 하단에 표시된다.
- 현재 rhwp: 같은 문단 안의 mixed nested table 조각이 먼저 표시되고, pi52 본문 일부가 p5 로 넘어간다.

## 근거

이 보정은 특정 샘플명/페이지 번호가 아니라 다음 문서 구조에 근거한다.

- RowBreak 표 셀의 cut unit 순서
- 같은 셀 문단 안의 `TextRun` line units 와 `mixed_nested_fragment` units
- 문단 control 위치와 split cut 범위

## 목표

1. `cell_units` 가 생성한 unit 순서를 `layout_partial_table` 의 렌더 순서가 보존한다.
2. mixed text + nested table 문단에서 현재 cut 범위가 텍스트 line unit 과 nested fragment unit 을 함께 포함하면,
   텍스트가 먼저 그려지고 이어서 nested table fragment 가 그려진다.
3. focused regression 과 visual sweep 으로 p4/p5 잔여 차이를 다시 확인한다.

## 구현

- `prefill_before_deferred_table` 에 묻혀 있던 visible-host RowBreak host text pre-emit 을
  `pre_emit_visible_rowbreak_host_text` 로 분리했다.
- HWPX RowBreak 자리차지 표가 부분 분할 루프로 바로 진입하는 경우에도 host text 를 먼저
  `PartialParagraph` 로 소비하도록 했다.
- HWP 저장 LINE_SEG 경로는 기존 cut 을 유지해야 하므로, partial split 직전 추가 pre-emit 은
  HWPX 입력에서만 적용했다. 기존 지연 이월 prefill 경로는 그대로 둔다.

## 검증 결과

- `cargo fmt`: 통과
- `env CARGO_INCREMENTAL=0 cargo build --profile release-test --bin rhwp`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1749_saved_bounds_page_break -- --nocapture`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `git diff --check`: 통과
- `dump-pages`:
  - HWP: 5쪽 유지, p4 `pi=52` `end_cut=[3]` 유지
  - HWPX: 5쪽 유지, p4 `pi=52` host `PartialParagraph` 가 `PartialTable` 보다 먼저 배치,
    첫 fragment `end_cut=[1]`
- visual sweep:
  - p4: `flagged=0/1`
    - `/Users/tsjang/rhwp/output/issue1811-stage4-p4-visual/issue1811-page-break-p4/review/review_004.png`
  - p5: `flagged=0/1`
    - `/Users/tsjang/rhwp/output/issue1811-stage4-p5-visual/issue1811-page-break-p5/review/review_005.png`

## 전체 테스트 메모

- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 는 `issue_1035_alignment` 에서 실패했다.
  - 실패: `samples/hwp3-sample16-hwp5.hwp`, `samples/hwp3-sample16-hwp5-2022.hwp` page count 66 vs 기대 64
  - Stage 4 tracked diff 를 임시 제거한 상태에서도 동일 실패가 재현되어, 이번 Stage 4 변경 원인은 아니다.
