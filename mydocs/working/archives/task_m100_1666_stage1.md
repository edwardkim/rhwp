# Task M100 #1666 Stage 1 완료 보고서

## 단계 목표

PR `Build & Test`의 일반 build/test step을 `release-test` 중심으로 전환하되, trusted event에서는 release-grade
검증을 유지한다.

## 변경 파일

- `.github/workflows/ci.yml`
- `mydocs/orders/20260701.md`
- `mydocs/plans/task_m100_1666.md`
- `mydocs/plans/task_m100_1666_impl.md`
- `mydocs/working/task_m100_1666_stage1.md`

변경하지 않은 파일:

- `Cargo.toml`
- `tests/**`
- `tests/golden_svg/**`

## workflow 변경 내용

`Build & Test` job의 step 이름과 순서는 유지하고, cargo command 내부에서 `GITHUB_EVENT_NAME`으로 profile을
분기했다.

| step | pull_request | non-PR trusted/manual event |
|------|--------------|-----------------------------|
| Build | `cargo build --profile release-test --verbose` | `cargo build --release --verbose` |
| Native Skia tests | `cargo test --profile release-test --features native-skia skia --lib --verbose` | `cargo test --release --features native-skia skia --lib --verbose` |
| Run lib tests | `cargo test --profile release-test --lib --verbose` | `cargo test --release --lib --verbose` |
| Run integration tests | `cargo test --profile release-test --tests --verbose` | `cargo test --release --tests --verbose` |

각 step에는 다음 로그를 추가했다.

- PR: `profile=release-test event=pull_request`
- non-PR: `profile=release event={event}`

## 의도

- PR run에서는 `release-test` profile로 `Build`, `Run lib tests`, `Run integration tests` 축을 맞춘다.
- `release` profile의 LTO/link/codegen 비용을 PR 피드백 루프에서 제거한다.
- `Native Skia tests`는 feature set 차이 때문에 별도 compile이 남을 수 있음을 정상 비용으로 분리한다.
- `devel` / `main` push, tag, `workflow_dispatch`에서는 release-grade 검증을 유지한다.

## 메인테이너 결정사항 반영

- PR은 `release-test` 중심으로 전환했다.
- release-grade 검증은 trusted event에서 유지했다.
- `Build & Test` job 이름은 유지했다.
- job 병렬화는 하지 않았다.
- `Swatinem/rust-cache` 또는 nextest는 도입하지 않았다.
- 회귀 가드 162개 / issue 계열 131개 구조는 건드리지 않았다.

## 로컬 정적 검증

| 항목 | 결과 | 비고 |
|------|------|------|
| `git diff --check` | 통과 | whitespace 문제 없음 |
| `actionlint .github/workflows/ci.yml` | 통과 | workflow 문법 검증 통과 |
| YAML 파싱 | 통과 | `yaml.safe_load` 기준 구조 파싱 가능 |
| 변경 파일 범위 | 통과 | workflow와 #1666 절차 문서만 변경 |
| `Cargo.toml` 변경 여부 | 통과 | 변경 없음 |
| `tests/**`, `tests/golden_svg/**` 변경 여부 | 통과 | 변경 없음 |

## after 관측 필요 항목

PR run 완료 후 다음 값을 PR body 또는 #1666 이슈 코멘트에 먼저 기록한다.

- PR checks 완료 시간
- `CI / Build & Test` job 시간
- `Build`, `Native Skia tests`, `Run lib tests`, `Run integration tests` step 시간
- `profile=release-test` 로그 노출 여부
- `Compiling rhwp`, `Fresh`, `Dirty rhwp ... has changed` 로그 비교
- `Finished release ...`가 PR 일반 build/test에서 사라졌는지
- `Finished release-test ...` 중심으로 바뀌었는지
- cache restore hit/miss, cache size, save skipped
- `Cache reservation failed`, `Failed to save`, `##[error]` 여부
- 회귀 가드 162개와 issue 계열 131개 실행 여부
- `Build & Test` required check 표면 변경 여부

장기 measurement 원천 문서는 CI 관측 후 후속 문서 PR에서 갱신한다.

## 다음 단계

1. 코드 PR을 생성한다.
2. PR CI 완료 후 raw measurement를 PR body 또는 #1666 이슈 코멘트에 기록한다.
3. PR merge 후 `devel` push run의 trusted release 검증을 확인한다.
