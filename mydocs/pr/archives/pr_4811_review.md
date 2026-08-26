---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-15
---

# PR #4811 검토 - Gym 트라젝토리 필요성 감사

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4811](https://github.com/edwardkim/rhwp/pull/4811) · @kevin9327 |
| 원 head | `d3af5637e57b2813dd6e3b4e320a551ab105660e` |
| 기준선 | `upstream/devel@4cf8a5898` |
| 누적 적용 | `d3af5637e` → `d31003b71` |
| 메인터너 보정 | `f674ac7c5` |
| 원 CI | 작성 시점 참고값: CI·CodeQL 성공, mergeable `MERGEABLE` |
| 자동 검토 | [P2 #4811 경고](https://github.com/edwardkim/rhwp/pull/4811#discussion_r3789123243) 재현 후 보정 |

## 변경과 보정 이유

원 PR은 다단계 기준풀이의 마지막 step을 제거해도 제출이 통과하면 무의미한 경로라고 판정한다. 하지만
마지막 `answer` 또는 `keyring_from`은 결과를 모으는 수집 단계다. 이를 제거하면 `core-cli/T12` 같은
과제는 마지막 실제 동작의 필요성을 검증하지 못하고 answer 누락으로만 실패한다.

보정은 trailing 수집 단계를 남긴 채 마지막 외부 의미 step만 제거하도록 바꾸고, 이 경계를 회귀 테스트로
고정했다. 실제 감사에서 `objects-media/OM03`의 중복 thumbnail 실행이 무의미한 선행 step으로 드러나
reference에서 제거했다. 결과적으로 기준 풀이는 같은 산출을 한 번만 생성한다.

## 완료 검증

- `python3 -m unittest ... test_gym_trajectory.py ... test_workflow_contract_wiring.py`: 총 89건 통과, 의도된 1건 skip.
- `python3 gym/tools/trajectory.py --bin target/pr-review/release-test/rhwp --json`: 25개 다단계 과제 모두 마지막 실제 동작이 load-bearing, theater 0건.
- `gym/packs/objects-media/reference/OM03.json` 기준풀이: 중복 단계 제거 뒤 정상 채점 통과.
- `git diff --check`: 통과.

**자동 경고를 보정한 뒤 수용 후보.** merge 직전에는 원 PR 최신 head와 required check를 다시 확인한다.
