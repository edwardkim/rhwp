---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-15
---

# PR #4804 검토 - 전 pack 정합 감사

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4804](https://github.com/edwardkim/rhwp/pull/4804) · @kevin9327 |
| 원 head | `03c1705866dac9c70b8d88049496e2e1333d8071` |
| 기준선 | `upstream/devel@bcb65ed68` |
| 누적 적용 | `03c170586` → `3edf841e5` |
| 메인터너 보정 | `f674ac7c5` |
| 원 CI | 작성 시점 참고값: CI·CodeQL 성공, mergeable `MERGEABLE` |

## 변경과 판단

전 Gym pack의 기준풀이 짝, ID 전역 고유성, 기본 metadata 정합을 검사하는 `gym/tools/audit.py`와
단위 계약을 추가한다. renderer·Rust 동작은 바꾸지 않으므로 시각 검증은 대상이 아니다.

원 PR에는 정합 감사의 단위 테스트가 있었지만 CI의 `Validate gym scorer contracts` 단계에 연결되지 않아
새 검사 자체가 변경 뒤 실행되지 않는 경로가 남았다. 통합 후보에서 `test_gym_audit.py`를 같은 경량 Python
계약 단계에 연결했다.

## 완료 검증

- `python3 -m unittest ... test_gym_audit.py ... test_workflow_contract_wiring.py`: 총 89건 통과, 의도된 1건 skip.
- `python3 gym/tools/audit.py --json`: 17개 pack, 정합 위반 0건.
- `git diff --check`: 통과.

**로컬 보정 후 수용 후보.** merge 직전에는 원 PR 최신 head와 required check를 다시 확인한다.
