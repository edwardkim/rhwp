---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #5112 검토 - taiki-e/install-action 2.85.13

## 접수와 범위

| 항목 | 기록 |
| --- | --- |
| PR | [#5112](https://github.com/edwardkim/rhwp/pull/5112) |
| 작성자 / source | `app/dependabot` / `dependabot/github_actions/devel/taiki-e/install-action-2.85.13` |
| 원 source head | `8e7ac27bce22c4e813f5caf8b554bd74d6634448` |
| 기준 / 규모 | `devel`, 2 files, +2 / -2 |
| 원 PR 상태 | 작성 시점 `MERGEABLE` / `CLEAN` |
| 통합 PR | [#5186](https://github.com/edwardkim/rhwp/pull/5186) |

고정 SHA의 `taiki-e/install-action`을 2.85.13으로 갱신한다.

## 통합 적용과 검증

원 SHA를 `c26bab366b1ee37b5f47a31e5bd054901d505f73`로 적용했다. workflow pin의 정적 diff를 확인했고, cargo-nextest 설치를
포함하는 #5186 archive CI가 성공했다.

## 판단

실제 archive runner에서 새 action pin이 성공했다. **통합 수용 권고.**
