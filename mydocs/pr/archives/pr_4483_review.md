---
kind: pr-review
status: local-accept-recommendation
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4483 리뷰 - 웹한글 3자 차등 대조 하네스

| 항목 | 검토 기록 |
| --- | --- |
| 원 PR | [#4483](https://github.com/edwardkim/rhwp/pull/4483) · @planet6897 |
| base / 원 head | `devel` / `e4b6b86c753faaa047e86402ef2dbbbd5340ab80` |
| 규모 | #4418을 선행으로 포함하는 stacked PR, 5개 고유 기능 commit |
| 작성 시점 상태 | OPEN, `MERGEABLE`, `CLEAN` (merge 직전 재확인 필요) |

## 범위와 충돌 해소

웹한글·COM·rhwp 결과를 같은 scenario에서 대조하고, Windows Chrome 탐색과 runner 재시도 규약을 추가한다.
검토 체리픽에서는 #4418의 20 commit을 먼저 적용하고 #4483의 merge commit은 제외했다.
`tools/hwpctrl_compat/run_gate.py` 충돌은 원 PR 최신본을 채택했다. 선언된 scenario 오류는 즉시 실패하고,
프로세스 충돌만 1회 재시도하는 구분을 보존한다.

mock WebHwp 환경에서 `p4-setmutate`가 20개 호출을 성공했고, HWPCTRL package gate도 모든 scenario를
성공으로 마쳤다. Python harness contract 28건도 통과했다.

## 판정

원 PR의 COM/웹한글 차등 대조 목적과 runner 실패 의미가 충돌 해소 뒤에도 유지된다. #4418의 선행
provenance를 중복 적용하지 않은 점도 확인했다.

**최종 권고: 최신 통합 head의 CI와 작업지시자 승인 후 수용.**
