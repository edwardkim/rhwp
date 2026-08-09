---
kind: pr_review_impl
status: active
canonical: mydocs/pr/pr_4356_review.md
last_verified: 2026-08-10
---

# PR #4356 메인터너 보정 실행 기록

## 커밋 경계

| 구분 | SHA / 내용 |
| --- | --- |
| contributor source | `125176f6eb1b7b78fc1b8a7bf5e58cc63c7322d3` |
| first maintainer correction | `540b2aea8ea03d44f5c6250fb1305fb3f7c85486` — `docs(roadmap): make #4356 experiment start reproducible` |
| follow-up docs correction | `e03935281a9450f529f6fd818ccfb47b256c16d2` — `docs(roadmap): define #4356 open-book cohort` |
| first trailing review | `7c306013da14eef6b20524345bccdfe1c335ee15` — `docs(pr): update #4356 open-book review` |
| receipt docs correction | `e3b4a993c923b60ec6678dadcebc9a08d2522e21` — `docs(roadmap): bind #4356 experiment receipts` |
| trailing review update | 이 문서를 포함한 `docs(pr): record #4356 receipt review` commit |

## 실행 내용

1. 원 프로토콜과 공개 Git history를 대조해 private rubric을 재현할 수 없음을 확인했다.
2. 1차 보정 `540b2aea...`을 보존하고 source open-book cohort, content-addressed
   environment, run-package manifest와 submission timestamp 계약을 후속 docs commit에
   추가했다.
3. #4355와 PR body의 legacy wording을 외부 mutation 없이 residual blocker로 기록했다.
4. 기존 `7c306013...` history를 보존하고 canonical environment/package hash, 동일
   timestamp authority receipt, task variant와 집계 규약을 별도 docs commit에 추가했다.
5. receipt/environment/variant assertion, Markdown 링크·metadata·whitespace와 linear
   history를 검사했다.
6. 이 review update를 receipt docs correction 뒤의 별도 single-parent commit으로 추가했다.

## rollback

receipt 보정에 문제가 생기면 최신 trailing review update와 `e3b4a993...`을 역순으로
revert한다. open-book 보정까지 제거해야 할 때만 `7c306013...`, `e0393528...`,
`540b2aea...`를 다시 역순으로 revert한다. contributor source는 amend, rebase,
reset 또는 force-push하지 않는다.
