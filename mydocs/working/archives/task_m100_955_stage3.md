# Task M100 #955 Stage 3 완료보고서 — CI fmt check 적용 및 Clippy 기준 고정

## 1. 목표

이번 단계의 목표는 Stage 1, Stage 2에서 확정한 fmt 정책을 CI에 반영하고, 원격 `devel`
CI 실패 원인을 함께 제거하는 것이다.

## 2. 확인한 CI 실패

대상 run:

```text
26007032301
https://github.com/edwardkim/rhwp/actions/runs/26007032301
```

실패 위치:

```text
Workflow: CI
Job: Build & Test
Step: Clippy
```

실패 원인:

```text
src/diagnostics/hwp5_contract_probe.rs:366
clippy::unnecessary_unwrap
```

원격 CI는 `dtolnay/rust-toolchain@stable`을 사용하고 있었고, 최신 stable Clippy 기준에서
`is_some()` 확인 후 `unwrap()`을 호출하는 패턴이 `-D warnings`에 의해 실패했다.

## 3. 변경 사항

### 3.1 CI toolchain 기준 고정

수정 파일:

```text
.github/workflows/ci.yml
```

`Build & Test`와 `WASM Build`의 Rust toolchain 설치 단계에 다음을 명시했다.

```yaml
toolchain: 1.93.1
```

이 값은 Stage 1에서 추가한 `rust-toolchain.toml`의 `channel = "1.93.1"`과 같은 기준이다.

### 3.2 CI fmt check 추가

`Build & Test` job의 build/test 이전에 fmt check를 추가했다.

```yaml
- name: Format check
  run: cargo fmt --all -- --check
```

의도:

```text
- 빌드와 테스트보다 먼저 포맷 불일치를 빠르게 실패시킨다.
- 컨트리뷰터가 기능 변경과 포맷 변경을 섞는 것을 CI에서 조기에 발견한다.
- Stage 2의 전체 rustfmt 정규화 커밋 이후 상태를 CI 계약으로 만든다.
```

### 3.3 Clippy 실패 패턴 수정

수정 파일:

```text
src/diagnostics/hwp5_contract_probe.rs
```

변경 전:

```text
oracle_id_mappings.is_some() 확인 후 oracle_id_mappings.unwrap() 호출
```

변경 후:

```text
Option<&RecordMeta>를 replacement 후보로 만들고 if let Some(...)으로 처리
```

기능 의미는 유지하되, 최신 Clippy에서도 경고 없이 통과하도록 정리했다.

## 4. 검증

실행:

```bash
cargo fmt --all -- --check
git diff --check
cargo clippy -- -D warnings
```

결과:

```text
cargo fmt --all -- --check 통과
git diff --check 통과
cargo clippy -- -D warnings 통과
```

## 5. 판단

Stage 3 변경은 CI 정책 적용과 Clippy 기준 대응으로 한정된다.

```text
- CI가 더 이상 floating stable 기준에만 의존하지 않는다.
- cargo fmt --all -- --check가 CI의 명시 계약이 되었다.
- run 26007032301의 직접 실패 원인이었던 unnecessary_unwrap 패턴을 제거했다.
```

## 6. 다음 단계

Stage 4에서는 전체 검증과 최종 보고를 진행한다.

```text
1. cargo test --quiet
2. cargo clippy -- -D warnings 재확인
3. 필요 시 git diff --check 재확인
4. 최종 보고서 작성
```

## 7. 승인 요청

Stage 3은 CI fmt check 적용과 Clippy 실패 원인 제거까지 완료했다.

작업지시자 승인 후 Stage 4의 전체 검증과 최종 보고를 진행한다.
