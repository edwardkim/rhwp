---
kind: pr-review-implementation
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4929 메인터너 보정 기록

| 순서 | commit | 역할 |
| --- | --- | --- |
| 1 | `717f698e2` | 원 PR의 SWS 감사기와 `engagement.py --validate` 연계 적용 |
| 2 | `396afddf0` | 상대 corpus 출력 경로를 절대 경로로 고정하고 회귀 추가 |

상대 `corpus_root`에서는 이미 base와 결합한 결과를 다시 base와 결합해 읽을 수 있었다. 보정은 경로
결합 후 `resolve()`만 수행하므로 감사 규칙과 산출물 형식은 바꾸지 않는다. 자동화 계약 시험 33건과
`sws_audit.py --self-check` 문제 0건으로 완료했다.
