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
| receipt trailing review | `4d4359e46fdb2686c9f028e7a6e1c69be5a5bc39` — `docs(pr): record #4356 receipt review` |
| aggregation identity correction | `51e069bd131ff14f901ede1c49f9aaf41b38f5be` — `docs(roadmap): complete #4356 aggregation identity` |
| final trailing review | 이 문서를 포함한 `docs(pr): finalize #4356 aggregation review` commit |

## 실행 내용

1. 원 프로토콜과 공개 Git history를 대조해 private rubric을 재현할 수 없음을 확인했다.
2. 1차 보정 `540b2aea...`을 보존하고 source open-book cohort, content-addressed
   environment, run-package manifest와 submission timestamp 계약을 후속 docs commit에
   추가했다.
3. #4355와 PR body의 legacy wording을 외부 mutation 없이 residual blocker로 기록했다.
4. 기존 `7c306013...` history를 보존하고 canonical environment/package hash, 동일
   timestamp authority receipt, task variant와 집계 규약을 별도 docs commit에 추가했다.
5. 기존 `e3b4a993...`와 `4d4359e4...` history를 보존하고 participant-visible
   `required_artifact_contract`를 variant hash에 결속했다. 회차별
   `submission_destination`은 package field로 유지하되 variant에서는 제외했다.
6. 결과·ledger의 `guidance_class`, 5-tuple 집계 key, guidance class별 preregistered
   task mix·weight와 overall 규약을 `51e069bd...`에 추가했다.
7. artifact-bound variant, storage-locator exclusion, guidance class·5-tuple assertion과
   Markdown 링크·metadata·whitespace·linear history를 검사했다.
8. 이 review update를 aggregation identity correction 뒤의 별도 single-parent commit으로
   추가했다.

## rollback

최신 집계 identity 보정에 문제가 생기면 `docs(pr): finalize #4356 aggregation review`와
`51e069bd...`를 역순으로 revert한다. receipt 보정까지 제거해야 할 때만
`4d4359e4...`, `e3b4a993...`을 이어서 역순으로 revert하고, open-book 보정까지
제거해야 할 때만 `7c306013...`, `e0393528...`, `540b2aea...`를 다시 역순으로
revert한다. contributor source는 amend, rebase, reset 또는 force-push하지 않는다.
