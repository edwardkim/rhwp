# task 1293 stage80 - PR CI 승인 조건 명시

## 목적

미주 기능 완성 목표는 자동 승인으로 계속 진행하되, PR 직전 전체 CI 검증은 작업지시자
승인 없이는 실행하지 않도록 검증 계획을 명확히 한다.

## 배경

- `/Goal` 진행 중 focused test, 재현 테스트, visual sweep은 계속 자동으로 수행한다.
- `cargo test --verbose`, `cargo clippy -- -D warnings`처럼 PR CI 전체 테스트 성격의 긴
  검증은 작업지시자의 명시 승인 이후에만 수행한다.
- 자동 승인 지시가 있어도 PR CI 전체 테스트 승인을 대체하지 않는다.

## 수정 대상

- `mydocs/manual/codex/docs_and_git_workflow.md`
- `mydocs/plans/task_m100_1293_impl.md`

## 검증

- 문서 문구 확인
- 작업 트리 상태 확인
