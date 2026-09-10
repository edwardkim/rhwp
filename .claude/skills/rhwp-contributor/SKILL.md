---
name: rhwp-contributor
description: rhwp 저장소의 기여 구현과 PR 제출 절차를 안내합니다. "rhwp에 기여", "PR 올려", "이슈 만들고 수정", "버그 고쳐서 제출", "기여 절차" 요청에 사용합니다. CONTRIBUTING.md의 현재 변경 범위별 검증과 제출 규칙을 따르며, 메인터너 검토·merge 요청은 PR review workflow로 연결합니다.
---

# rhwp-contributor

## 역할과 권위

이슈 분석, 구현, 검증과 PR 제출을 돕는 기존 기여 절차 스킬이다.
새로운 제출 규약이나 메인터너의 수용/merge 판단을 대신하지 않는다.

- 기여자의 공개 정본: [CONTRIBUTING.md](../../../CONTRIBUTING.md).
- 에이전트 공통 경계: [AGENTS.md](../../../AGENTS.md).
- 검토/merge 요청의 경로: [PR review workflow](../../../mydocs/manual/pr_review_workflow.md)와
  [선택표](../../../mydocs/manual/pr_review/README.md).
- 실제 제출 항목: [현재 PR 템플릿](../../../.github/pull_request_template.md).

프로젝트 문서 간 충돌은 현재 역할의 canonical 문서로 해결한다.
이 스킬의 과거 예제, fixture와 레시피는 현행 명령이나 필수 게이트의 정본이 아니다.
아래 활성 자식 문서와 정본으로 절차를 결정하며, 과거 스킬 고도화 작업의 비범위를 모든 기여에 적용하지 않는다.

## 활성 자식 문서

| 작업 | 먼저 읽을 문서 |
| --- | --- |
| 진행 순서와 제출 경계 | [procedure-order.md](references/procedure-order.md) |
| 원인과 독립적인 기대값 | [analyze-canonical.md](references/analyze-canonical.md) |
| 구현 범위와 책임 계층 | [implement-scope.md](references/implement-scope.md) |
| 변경 범위별 검증 | [clippy-and-tests.md](references/clippy-and-tests.md) |
| 실제 출력 비교 | [visual-evidence.md](references/visual-evidence.md) |
| PR 본문과 완료 표시 | [pr-template-checkboxes.md](references/pr-template-checkboxes.md) |
| 환경 차이와 미실행 | [exceptions.md](references/exceptions.md) |
| 반복 오류 방지 | [pitfalls.md](references/pitfalls.md) |

## 실행 순서

1. 기존 이슈와 관련 열린 PR을 확인하고 원인, 기대 결과, 완료 조건과 비범위를 정한다.
2. 현재 정본과 수정 계층의 계약을 읽고 실제 실패 사례 및 적용되지 않아야 할 반례를 정한다.
3. CONTRIBUTING의 브랜치/격리 절차를 따른다. 기존 PR의 보정이면 새 중복 PR을 기본으로 만들지 않는다.
4. 승인된 범위에서 원인을 수정하고 관련 회귀 테스트를 작성한다. 사용자와 다른 작업의 변경은 보존한다.
5. CONTRIBUTING의 범위표와 검증 절차로 검증한다. 필수 게이트를 이 스킬의 짧은 예제로 대체하지 않는다.
6. 제출 SHA와 검증 결과, 남은 실패와 비범위를 기록한다. 필요한 문서는 현재 역할의 절차에 따른다.
7. 경로를 지정해 stage하고 승인받은 push/PR 작업만 수행한다. PR 제목과 본문은 가능한 한 한국어로 쓴다.

## 증빙과 완료

문서 편집의 캡슐 경로는 AGENTS의 권장 기능이며 모든 기여의 강제 제출 조건이 아니다.
사용하는 경우 기존 명령 `rhwp replay --plan-json <계획> --capsule work.capsule.json --json`,
`rhwp audit <폴더> --json`, `rhwp lineage <머리캡슐> --json`의 현재 CLI 계약을 따른다.

미실행 검사를 통과로 표시하지 않는다. 검증 실패를 숨기려고 경고를 억제하거나 기대값을 출력에 맞추지 않는다.
필수 게이트가 미완이면 준비 완료로 선언하지 않으며, 명시적인 검증 제한이 있으면 그 제한과 미완료 상태를 기록한다.
지침 자체는 게시나 merge 권한을 부여하지 않는다.
