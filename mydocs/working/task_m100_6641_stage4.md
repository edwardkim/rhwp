# Task M100 #6641 Stage 4 — 제품 전체 검증

- **실행일**: 2026-09-03 KST
- **exact source head**: `beaa64c5ef04a712edcce4b0bd0881ec9d1d4113`
- **최신 devel 기준**: `upstream/devel@8d4c25d014dc42992aad6fa92c8eb761254c6bfc`
- **최신 devel 보존 merge**: `17dbbef65`
- **결론**: Rust 필수 lint와 전체 test가 실패 없이 통과했으며 제출 diff에 파생물이 없다.

## 1. 검증 전제

Stage 3 이후 원격 devel 전진분을 정상 merge하고 focused 56/56을 다시 통과한 clean head를
검증 대상으로 고정했다. `target/pr-review` 기존 경로만 재사용했으며 별도 대형 target을 만들거나
기존 target을 삭제하지 않았다.

`node scripts/rust-test-suite-manifest.mjs --prepare` 결과는 다음과 같다.

- 1,129 test sources
- 4,821 static test attributes
- 28 generated suites + 20 exceptions = 48/48 integration targets
- nextest 최소 6,559 cases
- weight 953004..958110

## 2. Rust 필수 게이트

AGENTS.md의 순서를 바꾸거나 단계를 생략하지 않았다.

| 순서 | 명령 | 결과 |
| ---: | --- | --- |
| 1 | `node scripts/rust-test-suite-manifest.mjs --prepare` | PASS |
| 2 | `cargo fmt --all` | PASS, source 변경 없음 |
| 3 | `cargo fmt --all -- --check` | PASS |
| 4 | `cargo clippy --locked --target-dir target/pr-review -- -D warnings` | PASS, 44.29초 |
| 5 | `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings` | PASS, 39.47초 |
| 6 | `cargo build --locked --workspace --target-dir target/pr-review` | PASS, 1분 15초 |
| 7 | `cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings` | PASS, 1분 05초 |

`-D warnings`를 적용한 네이티브·WASM·workspace all-targets Clippy에서 경고를 허용하거나
우회한 항목은 없다.

## 3. 전체 회귀

다음 정본 명령을 `--no-fail-fast`로 실행했다.

```bash
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review --tests --no-fail-fast
```

결과는 `8,969 tests run: 8,969 passed, 46 skipped`이며 실행 구간은 321.131초였다. release-test
컴파일은 3분 45초였다. nextest가 slow로 분류한 3건도 모두 통과했다.

- `issue_2063::huge_cellbreak_table_paginates_without_quadratic_blowup`: 210.850초
- `ir_field_sweep_baseline::ir_field_sweep_does_not_regress`: 78.655초
- `hwp5_roundtrip_baseline::baseline_all_samples_roundtrip_partition_14`: 76.486초

설치된 nextest `0.9.137`이 저장소 권고 `0.9.140`보다 낮다는 경고와
`.config/nextest.toml`의 `junit.report-skipped` 미인식 경고가 있었으나, 테스트 수집·실행·종료
코드는 정상이다. 버전 검사를 우회하는 옵션은 사용하지 않았다.

## 4. 제출 경계와 판정

전체 실행 뒤 manifest `--check`는 준비 시점과 같은 1,129 sources, 4,821 attributes,
48/48 targets, 최소 6,559 cases를 확인했다. `git diff --check`가 통과했고 worktree는 clean이었다.
generated suite·manifest, Gym task/reference/oracle, 임시 로그는 제품 diff에 추가하지 않았다.

따라서 Stage 4 종료 게이트는 충족됐다. 다음 Stage 5는 이 exact candidate 계보에서 BO05·BO15를
canary로 먼저 실행한 뒤에만 #6628 정상 입력 전수·discrimination·trajectory를 각각 한 번 수행한다.
Stage 5 실행과 원격 push·PR·comment는 이번 승인 범위에 포함하지 않는다.
