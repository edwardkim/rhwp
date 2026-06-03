# Task M100 #955 Stage 1 완료보고서 — rustfmt 정책과 도구 기준 확정

## 1. 목표

이번 단계의 목표는 전체 `cargo fmt`를 실행하기 전에 저장소 차원의 fmt 정책과 도구 기준을
명확히 하는 것이다.

## 2. 변경 사항

### 2.1 Rust toolchain 고정

추가 파일:

```text
rust-toolchain.toml
```

내용:

```text
channel = "1.93.1"
components = ["clippy", "rustfmt"]
targets = ["wasm32-unknown-unknown"]
profile = "minimal"
```

의도:

```text
- 컨트리뷰터와 CI가 같은 rustc/rustfmt/clippy 기준을 사용한다.
- wasm32-unknown-unknown target을 저장소 기본 toolchain에 포함한다.
- stable floating 기준으로 인한 rustfmt diff 변동을 막는다.
```

### 2.2 rustfmt 정책 파일 추가

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

원칙:

```text
- stable rustfmt 옵션만 사용한다.
- rustfmt 옵션 변경은 기능 브랜치와 분리한다.
- 전체 정규화는 포맷 전용 단계에서만 수행한다.
```

### 2.3 CONTRIBUTING.md 보강

PR 전 체크리스트에 다음을 추가했다.

```bash
cargo fmt --all -- --check
```

그리고 포맷 정책 섹션을 추가했다.

핵심 내용:

```text
- 기능 변경과 전체 포맷 정규화를 같은 커밋에 섞지 않는다.
- PR에서 무관한 대량 fmt diff가 생기면 devel 기준으로 rebase 후 다시 확인한다.
- 전체 cargo fmt --all은 포맷 전용 이슈/브랜치에서만 수행한다.
- rustfmt 옵션 또는 Rust toolchain 변경은 별도 이슈로 분리한다.
```

### 2.4 CLAUDE.md 보강

작업 규칙에 fmt 정책을 추가했다.

핵심 내용:

```text
- 기능 변경과 포맷 변경을 같은 커밋에 섞지 않는다.
- 전체 cargo fmt --all은 포맷 전용 이슈/브랜치에서만 실행한다.
- 기능/조사 브랜치에서는 무관한 rustfmt diff를 만들지 않는다.
- formatter 기준은 rust-toolchain.toml과 rustfmt.toml을 따른다.
```

## 3. 검증

실행:

```bash
rustc --version
cargo fmt --version
git diff --check
```

결과:

```text
rustc 1.93.1 (01f6ddf75 2026-02-11)
rustfmt 1.8.0-stable (01f6ddf758 2026-02-11)
git diff --check 통과
```

주의:

```text
- rust-toolchain.toml 추가 직후 첫 rustc/cargo fmt 확인은 샌드박스에서 rustup 임시 파일 생성이 막혀 실패했다.
- 권한 상승 후 toolchain 설치/확인이 완료되었고, 이후 cargo fmt --version은 정상 동작했다.
- Stage 1에서는 의도적으로 cargo fmt --all을 실행하지 않았다.
```

## 4. 다음 단계

Stage 2에서는 포맷 정규화 전용 작업만 수행한다.

```text
1. cargo fmt --all 실행
2. diff가 포맷 변경만 포함하는지 확인
3. 기능 변경이 섞이지 않았는지 검토
4. 포맷 전용 커밋 작성
```

## 5. 승인 요청

Stage 1은 정책과 도구 기준 확정까지 완료했다.

작업지시자 승인 후 Stage 2의 전체 포맷 정규화 전용 작업을 진행한다.
