---
kind: pr-review
status: review-complete-pending-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5810 검토 - Track 4 성능 probe 출처와 표본 수

| 항목 | 내용 |
| --- | --- |
| PR / 작성자 | [#5810](https://github.com/edwardkim/rhwp/pull/5810) / `lpaiu-cs` |
| source head / 적용 commit | `94d9ac756627c804e336745843f3b02e686d3d5a` / `9f7b92790` |
| 관련 issue | [#3315](https://github.com/edwardkim/rhwp/issues/3315) |
| GitHub 상태 | Open, non-draft, `MERGEABLE`; source CI 성공 |
| 라우팅 | `maintainer_general` + `intake_and_review` + `local_validation` + `multi_pr_update_branch` |

Track 4 probe 출력에 served origin과 Track 3 API 존재 여부를 기록하고, `--keys`, `--refreshes`,
`--warmup` 표본 수를 인자로 만든다. 잘못된 worktree를 서빙하거나 sub-ms의 짧은 표본이 배율을 왜곡해도
측정 대상과 표본 조건을 산출물에서 판별할 수 있게 한다.

통합 후보에서 `npm run e2e:issue-3315-perf`가 통과했고 Track 3 API provenance가 출력됐다. 기본 짧은
표본은 baseline 0.76ms, JPEG 3.06ms로 4.00x여서 자체 x2 기준을 넘지만, 이는 probe의 기본 표본이
짧아 생기는 진단값이다. source가 기록한 `--keys=300 --refreshes=300 --warmup=50`의 1.62x와 같은
수치를 이 통합 검증에서 재현했다고 주장하지 않는다.

**수용 권고, #3315는 계속 open.** 이 PR은 성능 개선이나 CI gate가 아니라 이후 측정의 provenance와
표본 제어를 추가한 것이다.
