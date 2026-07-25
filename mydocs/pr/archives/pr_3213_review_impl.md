---
kind: reference
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-24
---

# PR #3213 통합 실행 계획 — HF 필드 undo 모델 좌표 보정

원 기능 커밋 `8c34d63dd`를 #3208이 포함된 최신 `devel` 위에 적용한 뒤, maintainer 보정
`0cab08c80`으로 화면 표시 offset과 모델 mutation offset의 경계를 명시한다. 보정은 field marker
삽입 결과를 실제 모델 위치로 반환하고 history가 그 위치만 기록하도록 하며, UI caret의 표시·모델
좌표 전체 변환을 별도 리팩터로 넓히지 않는다.

검증 단계는 Rust offset 재현 테스트, Studio TypeScript/unit test, 전체 Rust test 및 WASM build다.
회귀 시 `0cab08c80`과 원 기능 커밋을 역순으로 revert할 수 있다. 통합 PR CI 성공과 작업지시자 승인
후 원 PR #3213에 통합 완료 안내를 남긴다.
