---
kind: report
status: completed
canonical: mydocs/plans/task_m100_5955.md
last_verified: 2026-08-24
---

# Task M100 #5955 — Stage W7.5-2C source boundary 계약 정정

## 판정

Stage W7.5-2C 최소 정정을 완료했다. v2 rule과 change-set payload에 단일 `sourceBoundaryId`를 필수로
추가하고 immutable selection tuple에 포함했다. 초기 v1→v2 이행의 830개 boundary mismatch와 tuple
before/after mismatch는 모두 0이다.

## 원인과 정정

W7 projection generator는 backend row 하나에 정확히 한 source boundary를 요구한다. Stage W7.5-2의
carry-forward rule은 legacy evidence에 이 값이 남아 있어 동작했지만, `add-rule`과 replacement는 legacy
evidence가 `null`이므로 projection route를 표현할 수 없었다.

정정 후 역할은 다음처럼 분리된다.

- `sourceBoundaryId`: 현재 selection과 projection lookup의 semantic 입력
- `evidence.sourceBoundaryIds`: v1에서 이행한 역사 evidence, 신규 rule에서는 `null`
- `evidenceIds`: 현재 change-set을 정당화하는 일반 evidence graph 참조

## 검증

| 항목 | 결과 |
| --- | --- |
| v1 boundary 배열이 정확히 한 개인 rule | 830/830 |
| v1 evidence/v2 semantic boundary mismatch | 0 |
| ruleId·selection tuple before/after mismatch | 0 |
| v2 active/retired | 830/0 |
| focused lifecycle·migration contract | 22/22 통과 |
| JSON schema와 canonical artifact validation | 통과 |
| v1 봉인 artifact 변경 | 0 |

missing boundary와 같은 ruleId의 boundary mutation은 focused negative contract에서 거부된다. 실제 font
mapping, projection output과 runtime consumer는 이 정정에서 바꾸지 않았다.

## 다음 경계

Stage W7.5-3은 projection generator가 `rule.sourceBoundaryId`를 사용하도록 전환해야 한다. 전환 전후
projection row SHA-256과 population이 같아야 하며, lifecycle resolver와 W8 mapping correction은 여전히
범위 밖이다.
