# PR #1828 리뷰 — 쪽 상단 spacing_before vpos 증거 게이트

## 메타

| 항목 | 내용 |
|------|------|
| PR | https://github.com/edwardkim/rhwp/pull/1828 |
| 작성자 | @planet6897 |
| base / head | `devel` / `pr/devel-1827` |
| 작성 시점 참고 head | `69afc43631b7764a4456c90c4715dffc7cda9f9b` |
| 작성 시점 참고 상태 | `MERGEABLE`, GitHub Actions 통과 |
| reviewer assign | @jangster77 지정 완료 |

## 변경 범위

- 쪽 상단 `spacing_before` 트림을 저장 vpos 증거가 있을 때로 제한한다.
- `paragraph_layout.rs` 와 `tools/compare_line_baselines.py` 를 수정한다.
- PR 목적은 페이지 상단에서 발생한 5pt급 상수 오프셋 해소다.

## 로컬 검증

- 체리픽 커밋: `69afc43631b7` -> `f8e7fef9b`
- 충돌: 없음
- 누적 검증: focused Rust test, `svg_snapshot`, release-test integration, Clippy 통과.
- visual sweep:
  - 기준 샘플: `samples/task1750/split_guard_spacing_before.hwp`
  - 기준 PDF: `samples/task1750/split_guard_spacing_before-2024.pdf`
  - 페이지: p2
  - 임시 review PNG: `output/pr1823_1840_review_visual/pr1828-split-guard/review/review_002.png`
  - asset: `mydocs/pr/assets/pr_1828_visual_split_guard_p002.png`
  - `visual_accuracy_proxy_percent`: 약 `12.84%`
  - 자동 후보: `flagged=0/1`

## 판단

대표 p2 visual sweep 에서 PR 목적이었던 쪽 상단 spacing_before 상수 오프셋은 재현되지 않았다. overlay 의 차이는
글꼴/래스터 차이 중심이며, 페이지 흐름과 줄 위치는 merge 후보로 볼 수 있는 수준이다.

## 결론

merge 후보. #1832 는 이 PR 위에 쌓인 후속 도구 보정이므로 #1828 을 먼저 merge 하는 순서가 맞다.
