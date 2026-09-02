---
kind: pr-review
status: review-complete-pending-trailing-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5858 검토 - 마지막 줄 dash leader 자연 폭

## 접수 메타데이터

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR / 작성자 | [#5858](https://github.com/edwardkim/rhwp/pull/5858) / [@kevin9327](https://github.com/kevin9327) |
| 관련 issue | [#5830](https://github.com/edwardkim/rhwp/issues/5830) |
| base / source head | `devel` / `3d49882284e7b8fb54af6177b16eba3bbe0ac0aa` |
| 변경 규모 | 5 files, +346 / -0 |
| 작성 시점 상태 | non-draft, `MERGEABLE`, `CLEAN`, maintainerCanModify=true |
| 통합 반영 | `9a07c8ea2` (`review/open-prs-20260822`) |

## 범위와 검토

- 양쪽 정렬이 아닌 마지막 줄에서도 dash leader가 자연 폭까지 회복되도록 한다.
- 자연 폭 상한을 둬 남은 슬랙 전체를 dash에 배분하는 과확장을 막는다.
- #5830 계약은 p34의 여백 도달과 p35의 여백 미도달을 함께 고정한다.

## 검증과 시각 범위

- 통합 candidate `4b28259bb`에서 전체 nextest **8,160 passed, 39 skipped**, clippy, native-Skia, WASM build를 통과했다.
- 같은 head의 GitHub Build & Test, archive build/shard, Lint, Native Skia, CodeQL Rust/JavaScript/Python, Canvas visual diff, Adapter inter-diff, Proptest roundtrip도 성공했다. WASM과 frontend unit은 변경 범위 정책에 따른 정상 skip이다.
- `samples/issue1891/86712_regulatory_analysis.hwpx`와 `pdf/issue1921/86712_regulatory_analysis-2024.pdf`를 p34-35에서 sweep했다. 임시 결과는 `output/visual_open_prs_20260822_5830_a`에 있다.
- 대표 asset은 `mydocs/pr/assets/pr_5858_issue5830_p034_review.png`(SHA-256 `8f54ffaa624977909d82020850750cab70ffdb0a8bf166ab9ee84b6c6a453969`)와 `pr_5858_issue5830_p035_review.png`(SHA-256 `50a2557d2d0bfde849841c04c90c147d9734b22ea6c2340d8ea38b218f9bf3c1`)다.
- p35에는 자동 후보가 없었다. p34에는 기존 전체 page-flow/font 차이 후보가 남지만, dash leader의 계약 범위 밖 넓은 fidelity 차이이며 이 수정으로 완전 해소됐다고 주장하지 않는다.

## 최종 판정

**수용 권고.** 자연 폭 상한과 p34/p35 상반 조건의 회귀 계약이 있어 좁은 dash leader 보정으로 수용 가능하다. merge 조건은 PR #5889 최신 head CI와 작업지시자 승인이다.
