---
kind: review
status: accepted-local
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-04
---

# PR #3898 검토 - 개인정보 마스킹 레시피

- 원 head: `4c9942939742f9f8b2dac7e130461cc0bdd4c0b8`
- 범위: 마스킹 레시피와 문서 지도·LLM 진입점 연결.
- 시각 검증: 불필요. 문서 전용 변경이다.

## 결과

변경 Markdown 상대 링크 검사와 `git diff --check`가 통과했다. 레시피는
`redact`→`sanitize`→재검사 순서, `--no-raw` 로그 노출 방지, 출력 경로 명시를
구체적으로 다룬다. 관련 `redact_sanitize_contract` 15건과 전체 release-test도
누적 후보에서 통과했다. 별도 결함은 찾지 못했다.

## 후속 기록

수용 판단은 완료된 로컬 검증을 기준으로 한다. #3889의 가이드 수치 보정 시 이
레시피 링크와 표면 설명도 함께 대조한다.
