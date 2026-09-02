---
kind: pr-review
status: review-complete-pending-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5814 검토 - 테두리 없는 글자겹침 원문자 표시

| 항목 | 내용 |
| --- | --- |
| PR / 작성자 | [#5814](https://github.com/edwardkim/rhwp/pull/5814) / `kevin9327` |
| source head / 통합 적용 commits | `7163d463dc42b276a381616bdc33f512e308f924` / `db72f871b`, `f0dab865c` |
| 관련 issue | [#5790](https://github.com/edwardkim/rhwp/issues/5790) |
| 기준 | `upstream/devel@7df17a0ca9b8070192a230878fc9f56313ecae83` |
| GitHub 상태 | Open, non-draft, `CLEAN`; 최신 source CI 성공 |
| 통합 후보 | `review/green-ci-20260821-r2` |

`border_type=0`인 `CharOverlap`은 한컴이 테두리를 그리지 않으므로 원문자 `③`/`④`를 그대로 paint한다.
테두리를 그리는 경우에만 안쪽 숫자로 푼다. SVG, Canvas, Skia가 단일 `char_overlap_display_text` 규칙을
공유하도록 중복 구현도 제거했다.

## 검증과 시각 증적

- focused `issue_5790_charoverlap_circled`: 3 passed. parser IR에는 `③`/`④`가 유지되고 SVG에는 bare
  `3`/`4`가 나타나지 않는다.
- 통합 후보 전체 Rust nextest 8,068 passed, 0 failed; native-Skia와 CI native fixture도 통과했다.
- 한컴 2020 MCP PDF 13/13쪽과 SVG 13/13쪽을 visual sweep으로 비교했다. PDF는
  `pdf/pr_5814/hancom2020/issue1880_takeplace_oracle_p13-hancom2020.pdf`에 보관했다.
- page 5의 frame-tail 후보 1건은 bottom `여야 한다.`의 기존 layout 후보이며, 변경 대상 `④`는 같은 쪽
  y=723의 절대 positioned overlap이다. line flow를 바꾸지 않는 paint-time 변경이므로 차단 결함으로
  분류하지 않았다. 검토 PNG는 `mydocs/pr/assets/pr_5814_issue5790_p005_review.png`에 보관했다.

**통합 후보로 수용 권고.** 원문자 보존과 두 display mode의 분기가 명시적으로 회귀 고정됐고, sweep의
잔여 page 5 후보는 변경 범위 밖이다.
