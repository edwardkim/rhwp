---
kind: review
status: rework-required
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-04
---

# PR #3889 검토 - 에이전트 가이드 실측 수치

- 원 head: `a8f9b2cb29cdd067995f06b333823c91fdcb0f63`
- 범위: agent knowledge map·surface/task/troubleshooting guide 4편 대폭 확충.
- 예외 절차: 4,141줄 문서 대형 PR로 `rework_and_exceptions` 절차를 적용했다.
- 시각 검증: 불필요. 문서 전용 변경이다.

## 차단 사항

active 가이드가 현행 실측이라고 적은 수치가 누적 release-test binary의 실제
`capabilities`·MCP 응답과 다르다.

| 항목 | 문서 | 실제 |
| --- | ---: | ---: |
| CLI 명령 | 61 | 64 |
| JSON 계약 명령 | 31 | 34 |
| `recordFields` 합집합 | 148 | 159 |
| `capabilities --mcp` 도구 | 39 | 43 |
| MCP `tools/list` 도구 | 51 | 55 |

또한 `agent_knowledge_map.md`는 #3903가 보완한 `edit redact`·`edit sanitize`·schema
봉투 등의 출처 표지 누락 6종을 현행 예외로 서술한다. 이 상태로 active canonical
가이드를 병합하면 사용자가 이미 보정된 동작을 예외로 오해한다.

## 요청 사항

수용할 관련 변경을 기준으로 실측을 다시 수행해 모든 수치, 전수 표, 출처 표지 예외를
갱신해야 한다. 과거 기준을 보존할 필요가 있으면 active 가이드가 아니라 날짜·commit이
명시된 historical snapshot으로 분리한다. 보정 뒤에는 같은 로컬 자기서술 검사를 다시
실행한다.
