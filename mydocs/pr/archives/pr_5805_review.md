---
kind: pr-review
status: review-complete-pending-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5805 검토 - Studio 문서 이동 키 처리

| 항목 | 내용 |
| --- | --- |
| PR / 작성자 | [#5805](https://github.com/edwardkim/rhwp/pull/5805) / `planet6897` |
| source head / 적용 commit | `d54129334a3bf86e8c8839336a54ee51549bfc7b` / `eac99d2c8` |
| 관련 issue | [#5803](https://github.com/edwardkim/rhwp/issues/5803) |
| GitHub 상태 | Open, non-draft, `MERGEABLE`; source CI 성공 |
| 라우팅 | `maintainer_general` + `intake_and_review` + `local_validation` + `multi_pr_update_branch` |

PgUp, PgDn, Home, End가 toolbar/combo 포커스와 header/footer/footnote 편집 상태에서 무동작하거나, 큰
페이지의 중간을 건너뛰고 캐럿을 화면 밖으로 보내던 문제를 고친다. 전역 navigation key 전달, 한 화면과
페이지 경계 중 가까운 거리로의 이동, 본문 caret 유지가 핵심이다.

통합 candidate에서 Studio `npm test` **1,058 passed, 1 skipped**, production build,
`e2e:page-key-scroll`, `e2e:home-end-key`를 재실행해 통과했다. `e2e:manifest-check`의 세
미등재 파일은 기준 `devel`에도 존재하는 선행 상태이므로 이 PR 범위에서 변경하지 않았다.

**수용 권고.** Rust 런타임 변경은 없으나 Studio 변경이므로 최신 candidate의 Frontend package gate와
Canvas visual diff 성공도 함께 확인했다.
