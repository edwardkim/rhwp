---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-15
---

# PR #4793 검토 - batch-ops Gym pack

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4793](https://github.com/edwardkim/rhwp/pull/4793) · @kevin9327 |
| 원 head | `deb17b289c9cd5b64a0f9a89360242d41f3b3fd8` |
| 통합 기준선 | `upstream/devel@44bcba400072128bdc4e4d6c05bf822e3ff60996` |
| 누적 적용 | `deb17b289` → `4a49c1d2e` |
| 원 CI | `Validate gym scorer contracts` 실패로 Lint·Build & Test 실패; CodeQL 성공 |

## 변경과 CI 실패 원인

다문서 대량 처리 `batch-ops` pack과 BO01 task/reference/asset을 추가한다. 원 head는 새 pack을
`gym/profiles/maintainer.json`에 등록하지 않아 `test_maintainer_profile_covers_every_pack` 계약을
위반했다. 이는 pack 내용이나 Rust build 결함이 아니라 profile 완전성 누락이며, Lint 실패가
Build & Test의 실행을 막았다.

## 메인터너 보정과 검증

통합 branch의 `335fcac05`에서 `batch-ops`를 알파벳 순서로 maintainer profile에 등록했다.

- `test_gym_packs.py`를 포함한 Gym Python 계약 58건 통과, 의도된 1건 skip.
- `git diff --check upstream/devel...HEAD`: 통과.

**보정 후 통합 후보 수용.** 원격 source branch는 변경하지 않았으며, 통합 PR에서는 이 보정과 함께
Full CI를 다시 확인해야 한다.
