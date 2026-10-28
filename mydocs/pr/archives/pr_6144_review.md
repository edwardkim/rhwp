---
kind: pr-review
status: accepted-with-visual-warning
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6144 review - #6121 셀 안 anchored 개체 페인트 순서

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6144
- 작성자: `planet6897`
- 원 PR head: `d27ad1cab980`
- 통합 검토 브랜치: `review/open-prs-20260826-r1`
- 기준: `upstream/devel@1011a89475c9`
- 원 PR 상태: non-draft, source CI 녹색, comments/reviews 0건

## 검토 판단

**수용 가능, 단 visual_sweep 경고 코멘트 필요**. 셀 안 anchored 개체가 셀 본문 텍스트 아래/위 어느
layer에 배치되어야 하는지 명확히 해, 헤더 영역에서 object가 텍스트를 덮거나 가려지는 회귀를 고정한다.
대표 헤더 crop에서는 target 개선을 확인했다. 다만 직접 visual_sweep의 2단 본문 분석에서
`line_order_overlap`, `column_line_band_drift` flag가 남으므로 원 PR/issue 코멘트에는 이 경고가
target paint-order 개선과 별도 관찰값임을 명시한다.

## 증적과 검증

- 대표 시각 증적: `mydocs/report/cell-anchored-paint-order-6121/before_header_crop.png`,
  `mydocs/report/cell-anchored-paint-order-6121/after_header_crop.png`
- focused 검증: #6121 셀 anchored object paint order 테스트 1 pass
- 통합 후보 전체 검증: rustfmt, clippy, 전체 nextest 8,399 pass, WASM, native-Skia 통과

- 직접 visual_sweep 증적:
  - `rhwp info --json`: `mydocs/pr/assets/sample_issue6121_156531618_police_press_header_info.json`
  - local Hancom 2020 PDF: `pdf/pr_6088_6144/local_hancom2020/pr_6144_issue6121_police_press_header-local2020.pdf`
  - 대표 review PNG: `mydocs/pr/assets/pr_6144_issue6121_visual_review_001.png`
  - metrics: `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_metrics.tsv`, `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_flags.tsv` (`page=1`, pixel match `88.78936%`)
  - 주의: `analysis/metrics.json`에는 `line_order_overlap`, `column_line_band_drift` flag가 남는다. 헤더 crop의 target object paint-order 개선은 확인되지만, 2단 본문 band drift 경고는 PR/issue 코멘트에서 별도 확인 사항으로 남긴다.

## 후속

원 PR과 #6121 이슈에는 헤더 crop 증적과 함께 visual_sweep flag가 남은 사실을 같이 기록한다. 통합
전 최종 판단에서는 이 flag를 별도 follow-up으로 둘지, 추가 메인터너 보정이 필요한지 확인한다.
