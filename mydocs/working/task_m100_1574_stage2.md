# Task M100 #1574 Stage 2 완료보고서 — PR 검증 중 CI 디스크 부족 보정

- 이슈: #1574
- 브랜치: `task_m100_1574_ci_fast_pass_trigger`
- 작성일: 2026-06-26
- 단계: Stage 2 — PR 검증과 CI 보정

## 1. PR 생성과 1차 검증

PR #1575를 생성해 Stage 1 수정이 실제 GitHub Actions에서 동작하는지 확인했다.

1차 head `170713e375f2242f88444ea52cfc14e1c28ab234`에서 확인한 결과:

| 체크 | 결과 |
|---|---|
| `CI preflight` | pass |
| `CodeQL preflight` | pass |
| `Render Diff preflight` | pass |
| `Analyze (javascript-typescript)` | pass |
| `Analyze (python)` | pass |
| `Analyze (rust)` | pass |
| `Canvas visual diff` | pass |
| `Build & Test` | pass |
| `CodeQL` | pass |

이 상태에서 #1575는 최신 `devel`보다 뒤처진 상태라 merge가 막혔다.

## 2. 최신 devel 반영 후 실패

최신 `upstream/devel`의 #1540 merge commit `4c67c7f4`를 #1575 브랜치에 반영했다.
충돌은 없었으나, 최신 head `417b21b886bac3bba841b7357d2e543902275d78`에서 `Build & Test`가 실패했다.

실패 지점:

- step: `Run tests`
- 명령: `cargo test --verbose`
- 원인: 통합 테스트 binary link 중 runner 디스크 부족

로그 핵심:

```text
collect2: fatal error: ld terminated with signal 7 [Bus error], core dumped
error: could not compile `rhwp` (test "issue_630") due to 1 previous error
Caused by:
  No space left on device (os error 28)
```

이는 #1574 fast-pass 로직 실패가 아니라 CI의 debug 전체 테스트 링크 병목이다. 저장소 매뉴얼은 통합 테스트에
`release-test` 프로필 사용을 권장하므로 CI 명령을 같은 흐름으로 맞추는 것이 맞다.

## 3. CI 테스트 명령 보정

`.github/workflows/ci.yml`의 `Build & Test` job을 다음처럼 조정했다.

- `Build`: `cargo build --verbose` -> `cargo build --release --verbose`
- `Native Skia tests`: `cargo test --release --features native-skia skia --lib --verbose`
- `Run tests`: 제거
- `Run lib tests`: `cargo test --release --lib --verbose`
- `Run integration tests`: `cargo test --profile release-test --tests --verbose`

의도:

- debug profile 통합 테스트 binary 대량 링크를 피한다.
- 저장소 매뉴얼의 PR 전 검증 흐름과 CI를 맞춘다.
- LTO release 통합 테스트 병목을 피하기 위해 `release-test` 프로필을 사용한다.

## 4. 로컬 검증

| 명령 | 결과 |
|---|---|
| `actionlint .github/workflows/ci.yml .github/workflows/codeql.yml .github/workflows/render-diff.yml` | 통과 |
| `ruby -e 'require "yaml"; ... YAML.load_file(...)'` | 통과 |
| `git diff --check` | 통과 |

## 5. 다음 확인

- CI test 명령 보정 commit을 #1575에 push한다.
- #1575 최신 head 기준 GitHub Actions가 모두 통과하는지 확인한다.
- #1575 merge 후 #1572에 review 문서 전용 후속 커밋을 추가해 preflight fast-pass가 `pr_review_workflow.md`
  9.3.1/9.4 규칙대로 동작하는지 확인한다.
