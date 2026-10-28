---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_6766_stage2.md
last_verified: 2026-09-20
---

# #6766 2단계: 정식 integration test 경로 정렬

## 분석과 계획

PR #7285 생성 뒤 collaborator self-PR 절차와 Rust test 제출 규칙을 대조했다. 새 integration
test source는 `tests/cases/`에 두고 generated suite는 검증 산출물로만 남겨야 한다. 1단계에서
추가한 회귀의 내용은 적절하지만 경로가 `tests/` 루트여서 이 제출 규칙을 만족하지 않았다.
동작을 바꾸지 않고 source만 `tests/cases/`로 옮긴 뒤, 새 경로가 suite에 포함되는지와 같은
회귀를 다시 확인한다.

## 결과보고

- `issue_6766_clickhere_caret_alignment.rs`를 `tests/cases/`로 이동했다. 테스트 assertion과
  제품 소스는 바꾸지 않았다.
- test-suite prepare/check가 1,384 sources와 48 integration target 일치로 통과했다.
- 이동된 source를 포함한 `regression_suite_004` 회귀는 1 passed / 198 skipped로 통과했다.
- `CARGO_BUILD_JOBS=8`로 fmt, 네이티브 clippy, WASM clippy, workspace build, workspace
  all-target clippy를 다시 실행해 모두 통과했다.
- 1단계에서 완료한 전체 release-test 10,098개는 source 내용이 같아 다시 실행하지 않았다.
  최종 PR head의 GitHub CI가 전체 회귀를 다시 검증해야 한다.

이 단계는 분석 → 코드 수정·검증 → 결과보고 순서를 완료했다. generated suite·manifest와
빌드 산출물은 커밋하지 않는다.
