# PR #1902 리뷰

## 메타

| 항목 | 값 |
|------|----|
| PR | #1902 |
| 제목 | task 1638: convert/export-hwpx 검증 게이트 추가 |
| 작성자 | jangster77 |
| base | devel |
| head | task/m100-1638-convert-verify-gate |
| 관련 이슈 | #1638 |
| 규모 | +364 / -20, 5 files |
| 문서 작성 시점 참고값 | mergeable: MERGEABLE, mergeStateStatus: BLOCKED |

최종 merge 조건은 PR head 최신 커밋 기준 GitHub Actions 통과와 작업지시자 승인이다.

## 변경 범위

- `convert` 명령에 `--verify`, `--verify-pages` 옵션을 추가했다.
- `export-hwpx` 명령에 `--verify`, `--verify-pages` 옵션을 추가했다.
- 검증 실패 시 산출물은 남기고 종료 코드로 실패를 전달한다.
  - `--verify`: 재파싱 실패 또는 IR diff 발생 시 exit 3
  - `--verify-pages`: 페이지 수 불일치 또는 페이지 검증용 재파싱 실패 시 exit 4
- `rhwp --help`, `mydocs/manual/cli_commands.md`를 갱신했다.
- `tests/issue_1638_convert_verify_gate.rs`로 CLI 바이너리 경로를 검증한다.

## 이슈 대응 판단

#1638은 PR #1366의 `hwp2hwpx --verify` / `--verify-pages` 아이디어를 현재 devel CLI 구조로 재구성하는 작업이다.
현재 devel에서 HWPX 출력은 `export-hwpx`, HWP 출력은 `convert`가 담당하므로 신규 `hwp2hwpx` 명령을 만들지
않고 기존 명령 옵션으로 구현한 판단이 적절하다.

`convert --verify`는 HWPX 입력에서 어댑터 정규화가 들어갈 수 있으므로, 원본 IR 직접 비교가 아니라 HWP 저장
직전의 어댑터 적용 후 IR과 재파싱 IR을 비교한다. 이 기준은 false positive를 줄이면서 저장 산출물 자기정합을
검증하는 목적에 맞다.

## 렌더 영향 및 visual sweep 판정

visual sweep 대상이 아니다.

- 렌더러, 레이아웃, 페인팅, pagination 알고리즘을 바꾸지 않는다.
- 새 샘플, 기준 PDF, golden, visual regression fixture를 추가하지 않는다.
- 변경은 CLI 변환 후 검증 게이트와 문서/테스트 추가에 한정된다.
- 페이지 수 검증은 `--verify-pages` 옵션 동작 자체를 CLI 테스트와 full CI로 확인했다.

## 로컬 검증

Focused 검증:

- `cargo fmt`
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1638_convert_verify_gate`
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1868_export_hwpx_cli`
- `env CARGO_INCREMENTAL=0 cargo run --quiet --bin rhwp -- convert samples/hwp3-sample.hwp <tmp>.hwp --verify --verify-pages`
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`
- `git diff --check`

Self PR full CI equivalent:

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
  - sparse checkout + `CARGO_TARGET_DIR=/dev/shm/rhwp-target-1638`
  - `cargo test --profile release-test --test issue_1638_convert_verify_gate` 통과
  - release-test binary로 `export-hwpx ... --verify --verify-pages` 통과
  - release-test binary로 `convert ... --verify --verify-pages` 통과
- Windows `win10-ted`
  - 기존 작업트리를 건드리지 않고 별도 worktree에서 검증
  - `cargo test --test issue_1638_convert_verify_gate` 통과
  - SSH 기본 셸, `cmd.exe /C`, PowerShell에서 `export-hwpx`/`convert` 검증 옵션 실행 통과

## 리스크

- `--verify`는 strict IR diff이므로 향후 serializer/parser에서 의미 없는 순서 차이가 생기면 민감하게 실패할 수 있다.
  다만 검증 옵션은 opt-in이고 실패 시 산출물을 남기므로 운영상 진단 가능하다.
- `convert --verify`의 비교 기준은 원본 IR이 아니라 저장 직전 IR이다. PR 본문과 CLI 문서에 이 의미를 명확히
  남겨 혼동을 줄였다.

## 결론

merge 후보로 판단한다. 최종 merge 전에는 PR head 최신 커밋 기준 GitHub Actions 통과 여부를 확인한다.
