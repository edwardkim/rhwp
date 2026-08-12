---
kind: pr-review
status: local-accept-recommendation
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4526 리뷰 - 저장 사다리 drift 오라클

| 항목 | 검토 기록 |
| --- | --- |
| 원 PR | [#4526](https://github.com/edwardkim/rhwp/pull/4526) · @planet6897 |
| base / 원 head | `devel` / `f28234d14f922e89e731b85dd4d195e150ba046d` |
| 규모 | 2 files, `+288/-0`, 2 commits |
| 작성 시점 상태 | OPEN, `MERGEABLE`, `CLEAN` (merge 직전 재확인 필요) |

## 범위와 검토

`verify_ladder_drift.py`로 같은 쪽 안의 저장 line-seg과 렌더 y 편차를 찾아내고,
`verify_pi_line_vs_hangul.py`에 병렬·COM 교차 검증 경로를 보강한다. 진단 도구이므로 renderer의
판정을 바꾸지 않으며, candidate와 확정 결함을 분리한다.

Python `compileall`과 HWPCTRL Python harness contract 28건을 통과했다. #4490/#4491 fixture의
focused renderer 검증과 HWP 2020 기준 비교도 누적 branch에서 함께 성공했다.

## 판정

동일 페이지 내부 drift라는 기존 PI 오라클의 사각을 실제 재현 fixture로 보완한다. 도구 출력의
candidate를 자동 결함 또는 자동 merge gate로 승격하지 않는 설명도 유지한다.

**최종 권고: 최신 통합 head의 CI와 작업지시자 승인 후 수용.**
