# #5571 기여자·검토자 integration test 제출 절차 정리

## 목표

새 integration test의 기여자 제출 경로와 review worktree·CI의 파생 suite 준비 경로를
문서에서 일관되게 분리한다.

## 관측 근거

- PR #5550은 `tests/agent_cli_pack_contract.rs`를 최상위 `tests/`에 제출해 CI 정책에 의해 거부됐다.
- `CONTRIBUTING.md`의 회귀 테스트 절은 원본을 `tests/cases/`에만 두고 일반 기여자가
  manifest 준비를 하지 않는다고 설명한다.
- 같은 문서의 앞부분과 PR 템플릿은 모든 PR에 manifest `--check`를 요구해, 준비되지 않은 source
  checkout에서도 파생 결과를 다루도록 유도했다.

## 결정

1. 기여자는 새 원본을 `tests/cases/*.rs`에만 추가하고 generated suite·manifest·Cargo generated target을
   커밋하지 않는다.
2. `rust-test-suite-manifest.mjs --prepare`와 manifest `--check`는 review worktree·CI만 수행한다.
3. `rust-unit-test-tiers.mjs --check`는 source-side `#[cfg(test)]` 변경에서만 실행할 수 있는 무생성
   정책 검사로 별도 표기한다.
4. PR 템플릿, 기여 안내, 에이전트 지침, 개발·검토 안내를 같은 역할 구분으로 갱신한다.

## 범위 제외

- test 정책, 자동 배정 스크립트, CI workflow의 동작은 변경하지 않는다.
- PR #5550의 test 파일 이동은 해당 source PR의 메인터너 보정 범위다.
