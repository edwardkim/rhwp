# Task M100 #1579 구현 계획서

## Stage 1

순수 review 문서 PR fast-pass 조건을 보강한다.

- `ci.yml`
  - PR commit을 뒤에서부터 훑는 기존 구조는 유지한다.
  - 모든 PR commit이 review 문서 전용이면 `candidateSha = pr.base.sha` 기록 후 즉시
    `fast_pass=true`를 출력한다.
  - mixed PR은 기존 `Build & Test` check 조회를 유지한다.

- `codeql.yml`
  - 모든 PR commit이 review 문서 전용이면 base CodeQL matrix 조회 없이 즉시 fast-pass한다.
  - mixed PR은 기존 required CodeQL check 조회를 유지한다.

## Stage 2

검증과 정리를 수행한다.

- `actionlint`로 workflow 문법 확인
- `git diff --check` 확인
- 문서 전용이 아니므로 cargo 빌드/테스트는 생략하고 사유를 PR 본문에 기록

## PR 후 확인

테스트용 문서 전용 PR에서 기대 상태는 다음과 같다.

- `CI preflight`: pass
- `Build & Test`: skipped
- `CodeQL preflight`: pass
- `Analyze (*)`: skipped 또는 미실행
- `Render Diff preflight`: pass
- `Canvas visual diff`: skipped
