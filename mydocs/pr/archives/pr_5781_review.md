---
kind: pr-review
status: superseded-by-integrated-fix
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5781 검토 - flow 그림 페이지 배경 누락

## 접수와 판정

| 항목 | 내용 |
| --- | --- |
| PR / 작성자 | [#5781](https://github.com/edwardkim/rhwp/pull/5781) / `planet6897` |
| source head | `4207ebaf7b48d2f78d9a3c7bff3a604e45b387f2` |
| 기준 | `upstream/devel@fb434269eea237cc12053914560a2dbaf16270bf` |
| GitHub 상태 | Open, non-draft, `MERGEABLE`; source CI 성공 |
| 라우팅 | `maintainer_general` + `intake_and_review` + `multi_pr_update_branch` |

`flowImages.length > 0`인 Studio DOM 경로에서 페이지 배경 plane이 없어지는 결함을 고친다. 다만 같은
#5780 결함을 포함한 더 넓은 버그헌트 r4인 [#5786](https://github.com/edwardkim/rhwp/pull/5786)를 통합
후보에 적용했다. 두 변경을 중복 적용하면 Studio 배경 layer 경로가 겹치므로 #5781의 code head는
직접 cherry-pick하지 않았다.

## 검증과 최종 권고

- #5786에 포함된 #5780 구현과 HWP 2020 기준 PDF의 visual sweep을 통합 후보에서 다시 확인했다.
- `samples/issue5780/flow_image_page_background.hwpx`의 1쪽은 HWP 2020 PDF와 비교해 candidate 0건,
  pixel match 98.5481%, visual accuracy proxy 98.50499%였다.
- 검토 기준 PNG와 PDF는 각각
  `mydocs/pr/assets/pr_5786_issue5780_flow_image_page_background_review.png`,
  `pdf/pr_5786/hancom2020/issue5780_flow_image_page_background_hancom2020.pdf`에 보존했다.

**직접 수용은 비권고, #5786으로 supersede close 권고.** 통합 PR merge 후 #5781에는 중복 적용을 피한
이유와 #5786의 검증 근거를 comment로 남기고 닫는다.
