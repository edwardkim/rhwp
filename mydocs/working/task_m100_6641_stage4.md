# Task M100 #6641 Stage 4 — 제품 전체 검증

- **실행일**: 2026-09-03 KST
- **exact source/test candidate**: `7f1174f1d59bc020aaa38ceb7e148a8ae77b2784`
- **candidate tree**: `1a32c5cd3f5bab1e720b974434d469504d9a8272`
- **최신 devel 기준**: `upstream/devel@900b56edcaff3c1f84567c3f7c9e398a0dd9e8bb`
- **최신 devel 보존 merge**: `7f1174f1d59bc020aaa38ceb7e148a8ae77b2784`
- **결론**: Rust 필수 lint와 전체 test가 실패 없이 통과했으며 제출 diff에 파생물이 없다.

## 1. 검증 전제

Stage 3 이후 원격 devel 전진분을 두 차례 정상 merge했다. 마지막 merge에는 #6632 셀 줄높이,
#6648 중첩 표 outer margin, #6651 글상자 text offset 변경이 포함되어 field reflow와 의미상 인접했다.
따라서 기존 focused 56건과 새 layout 계약 4건을 합친 60/60을 다시 통과한 clean candidate를
검증 대상으로 고정했다. `target/pr-review` 기존 경로만 재사용했으며 별도 대형 target을 만들거나
기존 target을 삭제하지 않았다.

`node scripts/rust-test-suite-manifest.mjs --prepare` 결과는 다음과 같다.

- 1,132 test sources
- 4,825 static test attributes
- 28 generated suites + 20 exceptions = 48/48 integration targets
- nextest 최소 6,559 cases
- weight 953485..958746

## 2. Rust 필수 게이트

AGENTS.md의 순서를 바꾸거나 단계를 생략하지 않았다.

| 순서 | 명령 | 결과 |
| ---: | --- | --- |
| 1 | `node scripts/rust-test-suite-manifest.mjs --prepare` | PASS |
| 2 | `cargo fmt --all` | PASS, source 변경 없음 |
| 3 | `cargo fmt --all -- --check` | PASS |
| 4 | `cargo clippy --locked --target-dir target/pr-review -- -D warnings` | PASS |
| 5 | `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings` | PASS |
| 6 | `cargo build --locked --workspace --target-dir target/pr-review` | PASS |
| 7 | `cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings` | PASS |
| 8 | `node scripts/rust-unit-test-tiers.mjs --check` | PASS, 4,221 tests / 299 modules |

`-D warnings`를 적용한 네이티브·WASM·workspace all-targets Clippy에서 경고를 허용하거나
우회한 항목은 없다.

## 3. 전체 회귀

다음 정본 명령을 `--no-fail-fast`로 실행했다.

```bash
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review --tests --no-fail-fast \
  --status-level fail --final-status-level fail
```

결과는 종료 코드 0이다. 목록 9,019건에서 정책상 ignored 46건을 제외한 `8,973/8,973`이
실행·통과했다. release-test 컴파일은 3분 33초였다. 출력 옵션은 통과 개별 행만 억제했을 뿐
선택 범위·실패 판정·`--no-fail-fast` 의미를 바꾸지 않는다.

설치된 nextest `0.9.137`이 저장소 권고 `0.9.140`보다 낮다는 경고와
`.config/nextest.toml`의 `junit.report-skipped` 미인식 경고가 있었으나, 테스트 수집·실행·종료
코드는 정상이다. 버전 검사를 우회하는 옵션은 사용하지 않았다.

## 4. 제출 경계와 판정

전체 실행 뒤 manifest `--check`는 준비 시점과 같은 1,132 sources, 4,825 attributes,
48/48 targets, 최소 6,559 cases를 확인했다. `git diff --check`가 통과했고 worktree는 clean이었다.
generated suite·manifest, Gym task/reference/oracle, 임시 로그는 제품 diff에 추가하지 않았다.

따라서 최신 devel 재동기화 뒤 Stage 4 종료 게이트도 충족됐다. 이 exact candidate 바이너리로
BO05·BO15 canary를 먼저 통과한 뒤 #6628 정상 입력 전수·discrimination·trajectory도 각각 한 번
재실행했다. 결과는 최종 보고서 5절에 기록하며, 원격 push·PR·comment는 별도 승인 범위다.
