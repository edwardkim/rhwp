---
kind: report
status: final
task: m100-3169
issue: 3169
---

# Task #3169 처리 결과 — 진단 명령 6종 종료 코드 계약 확장

## 1. 요약

`#2707`(PR #2711)은 `export-*`/`convert`/`export-hwpx` 계열만 고쳤고, 자체 보고서
§6.1(`mydocs/report/task_m100_2707_report.md:243-252`)에서 같은 결함이
`info`, `dump`, `dump-note-shape`, `dump-endnote-lines`, `dump-pages`,
`diag`, `build-from-ingest`, `dump-records`, `ir-diff`, `test-*`, `gen-*`,
`rhwp::diagnostics::*::run` 계열에 남아 있다고 명시했다.

이번 작업은 그중 사용자가 직접 호출하는 6개 명령 — `info`, `dump-note-shape`,
`dump-endnote-lines`, `dump-pages`, `dump-records`, `build-from-ingest` — 에
`#2707` 관례(`EXIT_OK`/`EXIT_RUNTIME`/`EXIT_USAGE`)를 그대로 확장했다.

## 2. 재현 (수정 전)

```
$ ./target/debug/rhwp info nonexistent.hwp
오류: 파일을 읽을 수 없습니다 - nonexistent.hwp: ... (os error 2)
$ echo $?
0
```

`dump-note-shape`, `dump-endnote-lines`, `dump-pages`, `dump-records`,
`build-from-ingest` 모두 동일 — 인자 없음/파일 읽기 실패/파싱 실패/범위 초과
경로 전부 종료 코드 0.

`dump-pages` 는 `#2551` 로 인자 검증 로직(파싱 실패·범위 초과 감지)은 이미
형제 명령과 정합되어 있었으나, 오류 감지 후 종료 코드 전파만 빠져 있었다 —
`return;` 은 있었지만 이 함수가 `main` 의 `match` 에서 `exit_with(...)` 로
감싸이지 않아 항상 프로세스 종료 코드 0으로 끝났다.

## 3. 수정

`src/main.rs`:

- `show_info`, `dump_note_shape`, `dump_endnote_lines`, `dump_pages`,
  `dump_raw_records`, `build_from_ingest` 시그니처를 `fn(&[String])` →
  `fn(&[String]) -> i32` 로 변경.
- 각 함수의 치명 경로(인자 없음/누락 → `EXIT_USAGE`, 파일 읽기·파싱·직렬화·저장
  실패 → `EXIT_RUNTIME`, 페이지/섹션/문단/컨트롤 인덱스 범위 초과 →
  `EXIT_USAGE`)에서 `return;` 대신 해당 상수를 반환하도록 수정.
  성공 경로 끝에 `EXIT_OK` 추가.
- `main()` 의 디스패치에서 이 6개 명령을 `exit_with(...)` 로 감쌈.

3/4(`--verify`/`--verify-pages`)는 건드리지 않았다 — 해당 없음(이 6개 명령에는
그 옵션이 없음).

## 4. 검증

```
cargo build --bin rhwp
cargo test --test cli_exit_codes                        # 기존 회귀, 통과
cargo test --test cli_exit_codes_diagnostic_commands     # 신규 회귀
cargo test --test dump_pages_cli                         # #2551 회귀, 통과
cargo fmt --check -- src/main.rs tests/cli_exit_codes_diagnostic_commands.rs
cargo clippy --bin rhwp --tests
```

신규 테스트 `tests/cli_exit_codes_diagnostic_commands.rs` (5개, 전부 통과):

- `missing_arguments_report_usage_error` — 인자 없음 → 2
- `unreadable_input_reports_runtime_failure` — 존재하지 않는 입력 → 1
- `dump_pages_out_of_range_reports_usage_error` — 페이지 범위 초과 → 2
- `build_from_ingest_missing_output_reports_usage_error` — `-o` 누락 → 2
- `successful_diagnostic_commands_return_zero` — 성공 경로는 여전히 0
  (회귀 방지; `dump-records` 는 HWP5 CFB 샘플 `samples/2010-01-06.hwp` 사용 —
  HWP3 샘플은 CFB 컨테이너가 아니라 이 명령이 지원하지 않음)

수정 후 실측:

```
$ ./target/release/rhwp info nonexistent.hwp; echo $?
오류: 파일을 읽을 수 없습니다 - ...
1
$ ./target/release/rhwp info samples/2010-01-06.hwp >/dev/null; echo $?
0
```

## 5. 범위 밖 (잔여)

`dump`(`dump_controls`, ~1200줄), `diag`(`diag_document`, ~530줄), `ir-diff`,
`test-*`, `gen-*`, `rhwp::diagnostics::*::run` 계열은 함수 규모가 커
별도 이슈로 남긴다 — `#2707` 도 동일한 이유(진단·개발 보조 명령, PR 위험 최소화)로
같은 판단을 내렸다.
