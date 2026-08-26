---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-15
---

# PR #4795 검토 - render-tree Gym pack

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4795](https://github.com/edwardkim/rhwp/pull/4795) · @kevin9327 |
| 원 head | `17f99b187749d8fd2a5fbdcfe249ecead53ad6e8` |
| 통합 기준선 | `upstream/devel@44bcba400072128bdc4e4d6c05bf822e3ff60996` |
| 누적 적용 | `17f99b187` → `a5b6fe7bd` |
| 원 CI | `Validate gym scorer contracts` 실패로 Lint·Build & Test 실패; CodeQL 성공 |

## 변경과 CI 실패 원인

`export-render-tree` 능력을 측정하는 `render-tree` pack과 RT01 task/reference를 추가한다. 원 head는
새 pack의 `maintainer` profile 등록을 빠뜨려 #4793과 같은 완전성 계약을 위반했다. 따라서 Lint가
실패하고 Build & Test는 종속 실패로 종료됐다.

## 메인터너 보정과 검증

통합 branch의 `335fcac05`에서 `render-tree`를 알파벳 순서로 maintainer profile에 등록했다.

- `test_gym_packs.py`를 포함한 Gym Python 계약 58건 통과, 의도된 1건 skip.
- `git diff --check upstream/devel...HEAD`: 통과.

**보정 후 통합 후보 수용.** 이 pack은 task metadata와 기준 JSON만 추가하며 renderer 구현이나
시각 기준선을 직접 변경하지 않는다.
