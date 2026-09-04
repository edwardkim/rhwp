---
kind: working
status: active
issue: 5596
---

# #5596 review-only trailing tail 보조 workflow fast-pass

## 관찰

- [#5569](https://github.com/edwardkim/rhwp/pull/5569)의 review 기록 trailing head는 `mydocs/**`만
  바꿨지만, PR 전체 diff에 코드 후보가 남아 `Proptest roundtrip`과 `Adapter inter-diff` worker가
  재실행됐다.
- 기본 CI는 검증 완료 코드 후보 뒤의 review-only tail을 식별하지만, 두 보조 workflow는 단순
  PR 전체 diff만 확인한다.

## 설계

- 기존 전체 `mydocs/**` PR skip과 stable check 이름을 유지한다.
- PR head의 마지막 연속 `mydocs/**` commit들을 찾고, 선형 parent 연결과 최대 20개 tail을 확인한다.
- tail 직전 후보 SHA에서 같은 PR 번호의 동일 workflow 최신 run이 `success`일 때만 worker를 skip한다.
- GitHub API, 파일 목록, 계보, 후보 run 중 하나라도 확인하지 못하면 worker를 실행한다.
- 이 workflow 변경은 `devel` 대상 PR에서 즉시 적용되며 `main` 반영을 기다리지 않는다.

## 검증 계획

1. Proptest와 Adapter workflow contract test를 실행한다.
2. CI impact workflow contract test를 실행해 workflow 배선의 fail-closed 기대값을 확인한다.
3. 최신 head PR의 Actions에서 preflight는 성공하고 두 worker가 `skipped`로 남는지 관찰한다.

## 검증 결과

- `python3 -m unittest scripts/tests/test_proptest_roundtrip_workflow.py scripts/tests/test_adapter_diff_workflow.py scripts/tests/test_ci_impact_workflow.py`: 48 passed.
- `git diff --check`: passed.
