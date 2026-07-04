# PR #1826 리뷰 — HWPX saved_bounds p5 drift 원인 1 방어

## 메타

| 항목 | 내용 |
|------|------|
| PR | https://github.com/edwardkim/rhwp/pull/1826 |
| 작성자 | @planet6897 |
| base / head | `devel` / `pr/devel-1811` |
| 작성 시점 참고 head | `9d69668a670cd466a7e1484bbd5d74a7f5ba9a19` |
| 작성 시점 참고 상태 | `MERGEABLE`, GitHub Actions 통과 |
| reviewer assign | @jangster77 지정 완료 |

## 변경 범위

- 합성 LINE_SEG 를 원본 linesegarray 증거처럼 오인해 vpos 소폭 전방보정을 적용하는 경로를 차단한다.
- saved_bounds 누적 페이지 drift 의 원인 1 을 방어하고, 원인 2 잔존을 분리한다.
- PR 작성자의 최신 코멘트 기준으로 v2 head 는 seoul_1006 회귀를 다시 통과하도록 보정된 상태다.

## 로컬 검증

- 체리픽 커밋: `9d69668a670c` -> `6cfdc12d6`
- 충돌: 없음
- 누적 검증: focused Rust test, release-test integration, Clippy 통과.
- visual sweep:
  - 기준 샘플: `samples/task1749/saved_bounds_cumulative_page_break.hwpx`
  - 기준 PDF: `samples/task1749/saved_bounds_cumulative_page_break-2024.pdf`
  - 페이지: p5
  - 임시 review PNG: `output/pr1823_1840_review_visual/pr1826-saved-bounds/review/review_005.png`
  - asset: `mydocs/pr/assets/pr_1826_visual_saved_bounds_p005.png`
  - `visual_accuracy_proxy_percent`: 약 `5.70%`
  - 자동 후보: `flagged=1/1`

## 판단

시각 검증 p5 에서는 기준 PDF 대비 잔존 drift/overflow 후보가 남는다. 다만 이는 PR 이 설명한 원인 2 잔존과
일치하며, 이 PR 의 merge 목적은 원인 1 방어와 회귀 분리다. 따라서 #1811 을 완전 close 하는 merge 가 아니라
후속 원인 2 추적을 유지하는 조건의 merge 후보로 본다.

## 결론

merge 후보. merge 후에도 #1811 잔여 원인 2 추적 여부를 확인한다.
