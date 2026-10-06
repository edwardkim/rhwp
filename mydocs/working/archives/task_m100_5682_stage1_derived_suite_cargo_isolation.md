# Task M100-5682 Stage 1 - 파생 suite Cargo 격리

- Issue: #5682
- 기준: `upstream/devel` `2d897ca04dc80819a9833cf96f2a971c3ae792a1`
- 범위: integration suite 준비, Cargo registry 동기화, Rust test lockfile 보호, 관련 개발·검토 문서

## 문제

`rust-test-suite-manifest --prepare`가 review/CI용 harness와 manifest를 준비하면서 루트
Cargo test-target registry도 함께 갱신하면, 검토자는 자신의 변경과 관계없이 `Cargo.toml`이
dirty 상태가 된다. lockfile을 갱신할 수 있는 테스트 실행까지 겹치면 `Cargo.lock`도 검토
산출물로 오인될 수 있다.

## 설계

1. 기본 `--prepare`는 ignored generated harness와 manifest만 준비한다.
2. Cargo test-target registry 갱신은 `--sync-cargo-targets`를 명시한 maintainer 작업으로만
   허용하며 marker block 밖 변경을 거부한다.
3. `run-rust-test.mjs`가 nextest와 `cargo test`에 `--locked`를 한 번만 전달한다.
4. 실제 module 호환성을 확인한 소수의 source는 `moduleIntegrationOverrides`로 generated suite에
   편입하고, 남은 독립 target은 현재 Cargo registry와 일치시킨다.
5. 기여자는 `tests/cases/` 원본만 제출하고, 파생 파일과 registry 동기화의 책임 경계를 문서에
   명확히 기록한다.

## 검증 근거

- manifest/Cargo 격리 및 `--locked` 계약 테스트 22건 통과
- `--prepare`, manifest `--check`, 자동 배정된 regression target 3건 실행 통과
- 전후 SHA-256 비교로 루트 `Cargo.toml`과 `Cargo.lock` 불변 확인
- `cargo fmt --all -- --check`, `git diff --check` 통과

## 완료 기준

- 일반 review worktree에서 파생 suite를 준비해도 루트 Cargo 파일이 변경되지 않는다.
- Cargo registry 갱신이 필요할 때만 별도 명령과 marker 검증을 사용한다.
- source PR과 파생 검토 산출물의 제출 경계가 문서와 PR 템플릿에서 일치한다.
