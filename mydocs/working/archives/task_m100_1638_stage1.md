# Task M100 #1638 Stage 1 완료 보고

## 변경 요약

- `export-hwpx`에 `--verify`, `--verify-pages` 옵션을 추가했다.
- `convert`에 `--verify`, `--verify-pages` 옵션을 추가했다.
- 검증 실패 시 산출물은 남기고 종료 코드로 실패를 전달한다.
  - `--verify`: IR 차이 또는 재파싱 실패 시 exit 3
  - `--verify-pages`: 페이지 수 불일치 또는 페이지 검증용 재파싱 실패 시 exit 4
- `rhwp --help`와 `mydocs/manual/cli_commands.md`를 새 옵션에 맞게 갱신했다.
- `tests/issue_1638_convert_verify_gate.rs`를 추가해 CLI 바이너리 경로를 검증했다.

## 구현 판단

현재 `devel` 기준에서 HWPX 출력은 `export-hwpx`, HWP 출력은 `convert`가 담당한다. 따라서 옛
`hwp2hwpx` 명령은 추가하지 않고, 두 기존 명령에 검증 게이트를 붙였다.

`convert --verify`는 HWPX 입력에서 어댑터가 문서를 정규화할 수 있으므로, 변환 전 원본이 아니라
`export_hwp_with_adapter()` 호출 후의 IR과 재파싱 IR을 비교한다.

## 검증

- `cargo fmt`
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1638_convert_verify_gate`
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1868_export_hwpx_cli`
- `env CARGO_INCREMENTAL=0 cargo run --quiet --bin rhwp -- convert samples/hwp3-sample.hwp <tmp>.hwp --verify --verify-pages`
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`

모두 통과했다.

## Self PR full CI 검증

사용자가 self PR 기준 full CI 검증을 요청해 GitHub Actions `ci.yml`의 pull_request 경로와 맞춰 순차 실행했다.

- `cargo fmt --all -- --check`
- `env CARGO_INCREMENTAL=0 cargo build --profile release-test --verbose`
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests --verbose`
- `env CARGO_INCREMENTAL=0 cargo check --target wasm32-unknown-unknown --lib`
- `env CARGO_INCREMENTAL=0 cargo clippy -- -D warnings`
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --features native-skia skia --lib --verbose`
- `wasm-pack build --target web --release`

모두 통과했다.

## 교차 환경 검증

- Linux `ubuntu-ted`
  - 검증은 sparse checkout + `CARGO_TARGET_DIR=/dev/shm/rhwp-target-1638`로 수행했다.
  - `env CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/dev/shm/rhwp-target-1638 cargo test --profile release-test --test issue_1638_convert_verify_gate`
  - `/dev/shm/rhwp-target-1638/release-test/rhwp export-hwpx ... --verify --verify-pages`
  - `/dev/shm/rhwp-target-1638/release-test/rhwp convert ... --verify --verify-pages`
- Windows `win10-ted`
  - 기존 `C:\Users\admin\Desktop\rhwp\rhwp` 작업트리는 미커밋 변경이 있어 건드리지 않고,
    `GIT_LFS_SKIP_SMUDGE=1`로 별도 worktree `C:\Users\admin\Desktop\rhwp\rhwp-verify-1638`를 만들었다.
  - `cargo test --test issue_1638_convert_verify_gate`
  - SSH 기본 셸에서 `target\debug\rhwp.exe export-hwpx/convert ... --verify --verify-pages`
  - `cmd.exe /C`에서 같은 명령 확인
  - `powershell -NoProfile -Command`에서 같은 명령 확인

교차 환경 검증도 모두 통과했다.
