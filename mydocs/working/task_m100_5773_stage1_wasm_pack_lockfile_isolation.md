# #5773 로컬 WASM 검증 Cargo lockfile 격리

- 이슈: [#5773](https://github.com/edwardkim/rhwp/issues/5773)
- 브랜치: `codex/wasm-pack-locked-metadata`
- 기준: `upstream/devel@f26c2e7ca`

## 배경

`wasm-pack build --locked`는 Cargo build 호출에만 `--locked`를 전달한다. 앞선 `cargo metadata`가
lockfile을 갱신할 수 있어, 의존성을 변경하지 않은 로컬 WASM 검증 뒤에도 루트 `Cargo.lock`이 dirty로
남는다. 이는 의도적인 의존성 갱신 diff와 검증 산출물을 혼동하게 만든다.

## 범위

- POSIX shell, Windows PowerShell, `cmd.exe`용 native wrapper를 제공한다.
- wrapper는 metadata 호출에도 `--locked`를 적용하고, 일반 Cargo 호출은 원래 인수대로 위임한다.
- 개발·PR 검토 문서, 로컬 source bind-mount Docker Compose WASM 경로, WASM pkg 누락 진단을 wrapper
  명령으로 갱신한다.
- GitHub Actions workflow와 의존성 갱신 정책은 범위 밖이다.

## 사전 재현·분리 결과

- raw `wasm-pack build --target web --out-dir pkg --locked`는 root `Cargo.lock`을 변경했다.
- `node --test scripts/tests/rust-test-suite-manifest.test.mjs` 18건, suite `--prepare`/`--check`,
  unit-tier `--check`, focused `issue_1035_alignment`은 통과했고, 이후 `Cargo.toml`·`Cargo.lock` diff는
  없었다.
- `--prepare`가 만드는 32개 generated harness와 9개 exception은 ignored 검증 산출물이며, source PR에
  포함하지 않는다.

## 완료 기준

- POSIX wrapper 실행 뒤 `git diff --exit-code -- Cargo.toml Cargo.lock`가 통과한다.
- Windows PowerShell과 `cmd.exe` wrapper도 같은 검사를 통과한다.
- raw `wasm-pack build`를 권장하는 로컬 개발·검토 문서와 진단 메시지가 남지 않는다.
