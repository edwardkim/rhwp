# Task M100 #955 최종 보고서 — rustfmt 정규화 및 cargo fmt 적용 정책 수립

## 1. 이슈

```text
#955 [tooling] rustfmt 정규화 및 cargo fmt 적용 정책 수립
```

## 2. 목표

이번 작업의 목표는 `cargo fmt`가 기능 변경과 무관한 대량 diff를 만드는 문제를 정책과
기술 양쪽에서 정리하는 것이다.

구체적인 목표는 다음과 같다.

```text
1. rustfmt 기준을 저장소에 명시한다.
2. 기능 변경과 포맷 정규화 변경을 분리하는 규칙을 문서화한다.
3. 저장소 전체 Rust 소스를 한 번 정규화한다.
4. CI에서 cargo fmt --all -- --check를 적용한다.
5. floating stable toolchain으로 인해 Clippy/format 기준이 예고 없이 변하는 문제를 줄인다.
```

## 3. 작업 브랜치

```text
local/task955
```

기준 브랜치:

```text
local/devel
origin/devel
```

기준 커밋:

```text
c2d2157d test: update expectations for current layout
```

## 4. 주요 커밋

```text
60ff1901 Task #955: define rustfmt policy
ea564999 Task #955: normalize rustfmt output
cb024171 Task #955: enforce fmt check in CI
```

## 5. 변경 사항

### 5.1 rust-toolchain.toml 추가

추가 파일:

```text
rust-toolchain.toml
```

정책:

```text
channel = "1.93.1"
components = ["clippy", "rustfmt"]
targets = ["wasm32-unknown-unknown"]
profile = "minimal"
```

### 5.2 rustfmt.toml 추가

추가 파일:

```text
rustfmt.toml
```

정책:

```text
edition = "2021"
max_width = 100
newline_style = "Unix"
use_small_heuristics = "Default"
```

stable rustfmt 옵션만 사용하도록 했다.

### 5.3 문서 정책 보강

수정 파일:

```text
CONTRIBUTING.md
CLAUDE.md
```

핵심 규칙:

```text
- 기능 변경과 전체 포맷 정규화를 같은 커밋에 섞지 않는다.
- 전체 cargo fmt --all은 포맷 전용 이슈/브랜치/커밋에서만 수행한다.
- 신규 PR은 cargo fmt --all -- --check를 통과해야 한다.
- rustfmt/toolchain 정책 변경은 별도 이슈로 분리한다.
```

### 5.4 전체 rustfmt 정규화

실행:

```bash
cargo fmt --all
```

정규화 결과:

```text
405 Rust files changed
96082 insertions(+), 36162 deletions(-)
```

대상 범위:

```text
examples/
src/
tests/
```

기능 변경 없이 포맷 전용 커밋으로 분리했다.

### 5.5 CI fmt check 적용

수정 파일:

```text
.github/workflows/ci.yml
```

추가 단계:

```yaml
- name: Format check
  run: cargo fmt --all -- --check
```

위치는 build/test보다 앞에 두어 포맷 실패가 빠르게 드러나도록 했다.

### 5.6 CI toolchain 기준 고정

`CI` workflow의 `Build & Test`, `WASM Build`에 다음 기준을 명시했다.

```yaml
toolchain: 1.93.1
```

이는 저장소의 `rust-toolchain.toml`과 같은 기준이다.

## 6. CI 실패 대응

작업 중 원격 `devel` CI 실패를 확인했다.

대상 run:

```text
26007032301
https://github.com/edwardkim/rhwp/actions/runs/26007032301
```

실패 원인:

```text
src/diagnostics/hwp5_contract_probe.rs:366
clippy::unnecessary_unwrap
```

원인 분석:

```text
- 기존 CI가 dtolnay/rust-toolchain@stable을 사용하고 있었다.
- 최신 stable Clippy 기준에서 is_some() 확인 후 unwrap() 호출이 -D warnings에 걸렸다.
- #955의 문제의식인 "도구 기준 floating으로 인한 예고 없는 CI 실패"가 실제로 발생한 사례다.
```

대응:

```text
- CI toolchain을 1.93.1로 고정했다.
- 해당 코드를 if let Some(...) 형태로 정리했다.
```

## 7. 검증

실행:

```bash
cargo test --quiet
cargo clippy -- -D warnings
cargo fmt --all -- --check
git diff --check
```

결과:

```text
cargo test --quiet 통과
cargo clippy -- -D warnings 통과
cargo fmt --all -- --check 통과
git diff --check 통과
```

참고:

```text
cargo test --quiet 실행 중 기존 test 코드 warning이 일부 출력되었지만, 실패는 없었다.
이번 #955의 완료 조건은 fmt 정책/정규화/CI check 적용이며 warning 정리는 별도 이슈로 분리할 수 있다.
```

## 8. 산출 문서

```text
mydocs/plans/task_m100_955.md
mydocs/working/task_m100_955_stage1.md
mydocs/working/task_m100_955_stage2.md
mydocs/working/task_m100_955_stage3.md
mydocs/report/task_m100_955_report.md
```

## 9. 남은 작업

이번 이슈 범위에서는 완료로 판단한다.

후속 후보:

```text
1. 다른 GitHub Actions workflow도 rust-toolchain.toml 기준으로 통일할지 검토
2. cargo test에서 출력되는 non-fatal warning 정리
3. CI에 cargo fmt check가 들어간 뒤 contributor 문서 안내 강화
```

## 10. 결론

#955는 완료 조건을 충족했다.

```text
- rustfmt 기준이 저장소에 명시되었다.
- 전체 Rust 소스가 단일 포맷 전용 커밋으로 정규화되었다.
- CI에서 cargo fmt --all -- --check가 강제된다.
- floating stable로 인한 Clippy 실패 사례를 해결하고, CI 기준을 1.93.1로 고정했다.
```
