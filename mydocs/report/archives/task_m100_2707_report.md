# task_m100_2707 처리결과 보고서 — CLI 종료 코드 계약

- **이슈**: [#2707](https://github.com/edwardkim/rhwp/issues/2707)
- **브랜치**: `task/m100-2707-cli-exit-codes` (base `devel` @ `c4e6faa3`)
- **범위**: `src/main.rs`, `tests/cli_exit_codes.rs`(신규), `mydocs/manual/cli_commands.md`
- **분류**: 결함 수정 (CLI 계약 불일치)

## 1. 문제

`rhwp` CLI 는 거의 모든 명령이 **실패해도 종료 코드 0** 을 반환했다. `src/main.rs` 의
디스패치에서 종료 코드를 전파하는 arm 은 `export-pdf` 하나뿐이었고, 나머지는 전부
`fn(&[String]) -> ()` 라 모든 치명 경로가 `eprintln!(...); return;` 으로 끝나
`main` 이 정상 종료했다.

증상은 세 갈래다.

1. **치명 실패가 0** — 파일을 못 읽어도, 파싱이 실패해도, 페이지 범위를 벗어나도,
   출력 파일을 한 장도 못 써도 0이었다.
2. **성공 메시지가 사실과 다름** — 페이지 루프의 "N개 … 완료" 메시지가 요청 페이지 수
   (`pages.len()`)를 그대로 찍어, 전 페이지 저장이 실패해도 전부 성공한 것처럼 보고했다.
3. **알 수 없는 옵션·명령이 조용히 통과** — 옵션 오타는 경고만 찍고 렌더를 계속했고,
   알 수 없는 명령은 사용법을 **stdout** 으로 출력하고 0으로 끝났다.

이 때문에 `set -e`, `if ! rhwp ...`, CI step 판정, Makefile 체인이 모두 무력화된다.
종료 코드로 성공을 판정하는 AI 에이전트는 실패 위에 다음 단계를 쌓는다.

## 2. 분석

**이것은 미설계 영역이 아니라 일관성 결여다.** 저장소 안에 이미 올바른 선례가 있다.

- `export-hml` — 사용법 오류 `process::exit(2)`, 런타임 실패 `process::exit(1)` 로 완전히 올바름.
- `export-pdf` — `fn(&[String]) -> i32` 이고 디스패치가 코드를 전파. 알 수 없는 옵션도
  `return 2`. 테스트(`tests/render_p37_direct_pdf_export.rs:140,154`)가 사용법 오류 2를,
  `tests/render_p37_pdf_backend_cli.rs:60` 이 런타임 실패 1을 이미 단언하고 있다.
- `dump-pages` — 알 수 없는 옵션에서 파일 IO 전에 멈추는 계약이
  `tests/dump_pages_cli.rs` 로 이미 고정돼 있다.

반면 `convert` / `export-hwpx` 는 **절반만 올바르다**. 검증 실패에는 3/4 를 쓰면서
읽기·파싱·저장 실패는 0이었다. 즉 "출력이 아예 안 만들어진 쪽이 0, 만들어졌는데 미세
차이가 있는 쪽이 3"이라는 역전이 성립했다. 일관되게 틀린 것보다 이 부분적 정확성이 더
위험하다 — 사용자가 "rhwp 는 종료 코드를 쓴다"고 학습한 뒤 배신당하기 때문이다.

따라서 **새 규약을 발명하지 않고 `export-hml`/`export-pdf` 의 기존 관례를 확장**했다.

| 코드 | 의미 |
|---:|---|
| 0 | 성공 |
| 1 | 런타임 실패 (읽기·파싱·렌더·쓰기) |
| 2 | 사용법 오류 (인자 없음, 알 수 없는 옵션/명령, 페이지 범위 초과) |
| 3 | `--verify` IR 차이 — **기존 계약, 미변경** |
| 4 | `--verify-pages` 페이지 수 불일치 — **기존 계약, 미변경** |

3/4 는 `mydocs/manual/cli_commands.md` 에 문서화된 공개 계약이므로 기존
`process::exit(3)`/`process::exit(4)` 호출부 6곳을 **한 줄도 건드리지 않았다.**

## 3. 변경

### 3.1 종료 코드 상수와 전파 헬퍼 (`src/main.rs`)

`EXIT_OK`/`EXIT_RUNTIME`/`EXIT_USAGE` 상수와 `exit_with(exit_code: i32)` 헬퍼를 추가했다.
3/4 는 이미 문서화된 계약이라 상수화 대상에서 제외하고 기존 호출부를 그대로 뒀다.

디스패치의 `export-pdf` arm 이 쓰던 인라인 `if exit_code != 0 { process::exit(...) }` 을
같은 헬퍼로 통일해, 전파 방식이 명령마다 갈라지지 않게 했다.

### 3.2 `-> i32` 로 전환한 명령 (9개 함수)

`export-svg`, `export-render-tree`, `export-structure`, `export-png`(양쪽 `cfg` 변종),
`export-text`, `export-markdown`, `convert`, `export-hwpx`.

치명 경로 76곳을 분류해 `return EXIT_USAGE`(51곳) / `return EXIT_RUNTIME`(25곳)로 바꿨다.
분류 기준은 위 표 그대로다 — 인자 누락·옵션 값 오류·페이지 범위 초과는 2,
읽기·파싱·출력 폴더 생성·직렬화·저장 실패는 1.

### 3.3 실제로 쓴 페이지 수 집계

`export-svg` / `export-render-tree` / `export-text` / `export-markdown` 의 페이지 루프에
`written` 카운터를 넣어 **저장에 성공한 개수**를 세고, 완료 메시지에 그 값을 찍는다.
`written != pages.len()` 이면 `EXIT_RUNTIME`.

- `export-png` 은 이미 `success` 를 정확히 세고 있었으므로 종료 코드만 붙였다.
- `export-markdown` 의 이미지 실패는 경고로 남기고 MD 자체는 저장되므로 **페이지 실패로
  세지 않는다** — 이미지 카운터(`written_image_count`)와 페이지 카운터를 분리했다.

### 3.4 알 수 없는 옵션·명령

- 알 수 없는 옵션 5곳(`export-svg`/`export-render-tree`/`export-png`/`export-text`/
  `export-markdown`)이 `i += 1` 로 넘어가던 것을 `return EXIT_USAGE` 로 바꿨다.
  `export-structure`·`export-pdf`·`dump-pages` 는 이미 치명이었다.
- 알 수 없는 명령·명령 누락 폴백을 **stdout → stderr** 로 옮기고 `process::exit(2)`.
  어떤 명령이 문제인지 이름을 함께 출력한다.

### 3.5 `native-skia` 미포함 빌드의 `export-png`

기능이 아예 빌드되지 않았는데 0으로 끝나 스크립트가 성공으로 읽던 경로를 2로 바꿨다.

### 3.6 문서

`mydocs/manual/cli_commands.md` 에 "종료 코드 (#2707)" 절과 표를 추가했다.
3/4 의 기존 서술(`§3 convert`/`export-hwpx`)은 그대로 두고 표에 함께 등재만 했다.

## 4. 검증

### 4.1 신규 테스트 — `tests/cli_exit_codes.rs` (10개)

기존 프로세스 수준 CLI 테스트 관례(`env!("CARGO_BIN_EXE_rhwp")` + `std::process::Command`,
`tests/hml_cli.rs` 등)를 그대로 따랐다. `export-png` 케이스는 `#[cfg(feature = "native-skia")]`
로 양방향 게이트했다.

| 테스트 | 단언 |
|---|---|
| `missing_arguments_report_usage_error` | 7개 명령 인자 없음 → 2 |
| `unknown_command_writes_usage_to_stderr_and_fails` | 오타 명령 → 2, stderr 출력, **stdout 비어 있음** |
| `missing_command_reports_usage_error` | 인자 전무 → 2 |
| `unknown_option_is_fatal_instead_of_silently_ignored` | `--fontpath` 오타 → 2, 산출물 미생성 |
| `page_out_of_range_reports_usage_error` | `-p 9999` → 2 |
| `unreadable_input_reports_runtime_failure` | 7개 명령 존재하지 않는 입력 → 1 |
| `page_write_failure_is_counted_and_reported` | 전 페이지 저장 실패 → 1, "**0개** TXT 파일" 보고 |
| `help_and_version_still_succeed` | `--help`/`--version`/`-h`/`-V` → 0 |
| `successful_export_returns_zero` | 정상 내보내기 → 0 |
| `export_png_*` | feature 유무별 계약 |

`page_write_failure_is_counted_and_reported` 는 출력 폴더 자리에 일반 파일을 두어
모든 페이지 저장을 실패시킨다 — OS 무관하고 fixture 추가가 필요 없다.

### 4.2 red→green 실증 (실제 실행 캡처)

`export-text` 수정을 되돌렸다 — 디스패치를 `exit_with(export_text(...))` → 값 버림으로,
완료 메시지를 `written` → `pages.len()` 로.

**RED** (`cargo test --profile release-test --test cli_exit_codes`):

```
---- missing_arguments_report_usage_error stdout ----
assertion `left == right` failed: 종료 코드 2 를 기대했다
명령: rhwp export-text
stderr:
오류: HWP 파일 경로를 지정해주세요.
사용법: rhwp export-text <파일.hwp> [옵션] (rhwp --help 참조)
  left: Some(0)
 right: Some(2)

---- page_write_failure_is_counted_and_reported stdout ----
assertion `left == right` failed: 종료 코드 1 를 기대했다
명령: rhwp export-text ...\samples/hwp3-sample.hwp -o ...\rhwp-exit-codes-blocker-not-a-dir-...
stdout:
문서 로드 완료: ...\samples/hwp3-sample.hwp (16페이지)
텍스트 내보내기 완료: 16개 TXT 파일 → ...\rhwp-exit-codes-blocker-not-a-dir-.../
stderr:
오류: TXT 저장 실패 - ...\hwp3-sample_001.txt: 지정된 경로를 찾을 수 없습니다. (os error 3)
... (16회 반복) ...
  left: Some(0)
 right: Some(1)

failures:
    missing_arguments_report_usage_error
    page_out_of_range_reports_usage_error
    page_write_failure_is_counted_and_reported
    unreadable_input_reports_runtime_failure

test result: FAILED. 6 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out
```

**16쪽 문서에서 16장 전부 저장에 실패했는데 "16개 TXT 파일" 을 찍고 0으로 끝난다** —
이슈 §3 이 서술한 증상이 그대로 재현됐다.

**GREEN** (수정 복원 후 동일 명령):

```
running 10 tests
test export_png_without_native_skia_reports_usage_error ... ok
test unknown_option_is_fatal_instead_of_silently_ignored ... ok
test unknown_command_writes_usage_to_stderr_and_fails ... ok
test page_out_of_range_reports_usage_error ... ok
test missing_command_reports_usage_error ... ok
test successful_export_returns_zero ... ok
test page_write_failure_is_counted_and_reported ... ok
test help_and_version_still_succeed ... ok
test missing_arguments_report_usage_error ... ok
test unreadable_input_reports_runtime_failure ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
```

### 4.3 CI 3종 (전부 통과)

```
cargo fmt --all -- --check              → Diff in ... 0건
  (이 Windows 체크아웃은 core.autocrlf=true 라 손대지 않은 파일 다수가
   "Incorrect newline style" 로 보고된다. 실제 포맷 차이인 `Diff in ...` 만 판정 대상.)

cargo clippy --all-targets -- -D warnings → Finished (경고 0)

cargo test --profile release-test --tests → (4.4 참조)
```

### 4.4 전체 테스트 결과

`cargo test --profile release-test --tests` (`--lib` 이 아니라 `tests/` 전체를 도는 경로):

```
CARGO EXIT CODE = 0

테스트 바이너리 292개
합계: 3471 passed; 0 failed; 23 ignored
"test result: FAILED" / "panicked at" / "error[" / "failures:" 일치 0건

     Running tests\cli_exit_codes.rs
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

기존 CLI 테스트(`tests/hml_cli.rs`, `tests/issue_1638_convert_verify_gate.rs`,
`tests/dump_pages_cli.rs`, `tests/render_p37_*.rs`, `tests/issue_2225_*.rs`)는 모두
정상 인자만 사용하므로 이번 변경의 영향을 받지 않았고 전부 통과했다.

## 5. 미실행 항목 (투명 고지)

- **`native-skia` feature 로는 `cargo check` 만 수행** — `export-png` 의
  `#[cfg(feature = "native-skia")]` 변종이 컴파일되는지는 확인했으나(통과), 해당
  feature 로 테스트 스위트를 돌리지는 않았다. `tests/cli_exit_codes.rs` 의
  `export_png_follows_the_same_contract` 는 미실행이며,
  `export_png_without_native_skia_reports_usage_error` 만 실제로 돌았다.
- **실제 셸/CI 파이프라인 통합 확인 없음** — `set -e` 스크립트나 GitHub Actions step 에서
  실패가 실제로 전파되는지는 테스트 하네스로만 간접 검증했다.

## 6. 잔여 (별개 이슈 권장)

### 6.1 아직 종료 코드 0인 명령

이슈 §1 이 함께 지목했으나 이번 범위에서 제외했다 — 전부 진단·개발 보조 명령이고,
내보내기·변환 계열(스크립트·CI 가 실제로 체이닝하는 축)을 먼저 정확히 고치는 편이
PR 위험을 낮춘다고 판단했다.

`info`, `dump`, `dump-note-shape`, `dump-endnote-lines`, `dump-pages`, `diag`,
`build-from-ingest`, `dump-records`, `ir-diff`, `test-*`, `gen-*`, 그리고
`rhwp::diagnostics::*::run` 계열.

(`export-hml`, `export-pdf`, `thumbnail` 은 이미 종료 코드를 쓴다.)

### 6.2 이슈 §8 이 이미 분리한 항목

- 단일 페이지 파일명 규칙 불일치(`export-png` 만 `total_pages == 1`, 나머지는 `page_count == 1`)
- `export-svg --font-path` 가 일부 렌더 경로에서 무시됨
- 문서 누락(`measure-width`/`core-pages` 미등재, `export-pdf --backend` 미등재,
  `cli_commands.md` 의 direct backend 서술이 실제와 다름)
