---
kind: pr-review-implementation
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4937 메인터너 보정 기록

| 순서 | commit | 역할 |
| --- | --- | --- |
| 1 | `886832732` | RenderTree 기반 `layout-anomaly` 구현과 단위 회귀 적용 |
| 2 | `0bf52c088` | 설계 문서·CLI 레퍼런스 추가 |
| 3 | `e218a34b1` | MCP 도구 등록 추가 |
| 4 | `396afddf0` | agent profile·생성 매뉴얼 등록, 단위 시험 분리 |
| 5 | `2656276fc` | provenance map·계약 recipe·knowledge map 필드 사전 완결 |

명령을 구현·CLI·MCP만으로 끝내면 agent가 발견하지 못하거나, 공개 JSON command 전수 계약과
provenance 정책에서 빠진다. 보정은 이 세 계약면을 구현과 동기화했다. `layout_anomaly.rs`는 998줄로
유지하고 행동 회귀는 하위 test module에 분리했다. focused 13건, agent 10건, provenance 2건,
knowledge map 2건, 전체 nextest 6,479건이 통과했다.
