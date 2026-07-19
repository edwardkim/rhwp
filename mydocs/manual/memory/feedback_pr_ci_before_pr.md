---
kind: memory
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-17
---

# PR 전 로컬 CI급 검증 필수

- 내부 타스크 PR을 만들거나 PR 브랜치에 최종 푸시하기 전에는 먼저 로컬에서 CI에 준하는 검증을 실행한다.
- 최소 기준은 해당 변경 범위의 핵심 테스트, `cargo fmt --all -- --check`, `git diff --check`, 그리고 가능하면 `cargo test` 또는 CI 실패 가능성이 큰 통합/스냅샷 테스트다.
- SVG/browser 출력, golden, 렌더링 정책을 바꾼 경우에는 PR 생성 전에 관련 스냅샷 테스트를 반드시 실행하고 golden 갱신 여부를 확인한다.
- 검증이 실패했거나 아직 실행하지 않은 상태에서는 "PR 준비 완료" 또는 "PR 생성"으로 진행하지 않는다.
- 시간이 오래 걸리는 검증은 실행 중인 명령과 현재 단계를 사용자에게 짧게 보고한다.
