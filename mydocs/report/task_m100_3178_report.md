---
kind: report
status: done
last_verified: 2026-07-23
---

# task_m100_3178_report

Issue: #3178

## 요약

`#2707`(PR #2711)·`#3169`(PR #3171)이 명시적으로 범위 밖에 남긴 `dump`(dump_controls)·
`diag`(diag_document) 두 명령에도 동일 클래스의 종료 코드 미전파 버그가 있음을 실측으로
확인하고, 동일 계약(EXIT_OK/EXIT_RUNTIME/EXIT_USAGE)으로 최소 수정했다.

## 재현 (수정 전)

```
$ rhwp dump nonexistent.hwp; echo $?
오류: 파일을 읽을 수 없습니다 - nonexistent.hwp: ...
0

$ rhwp diag nonexistent.hwp; echo $?
오류: 파일을 읽을 수 없습니다 - nonexistent.hwp: ...
0

$ rhwp dump; echo $?      # 인자 없음
0

$ rhwp diag; echo $?      # 인자 없음
0
```

## 원인

`dump_controls`(src/main.rs, ~1200줄)·`diag_document`(~530줄) 모두 `fn(&[String])`
(반환값 없음) 시그니처였고, 세 오류 경로(인자 없음/파일 읽기 실패/파싱 실패) 각각
`eprintln!` 후 `return;`으로 끝났다. `main()` 디스패치도 `exit_with(...)`로 감싸지
않고 직접 호출했다.

몸통 크기와 무관하게 오류 반환 지점은 각 함수당 3곳뿐이라 계약 적용 자체는 국소적이었다.

## 수정

- `dump_controls`, `diag_document`: `fn(&[String])` → `fn(&[String]) -> i32`.
  - 인자 없음 → `EXIT_USAGE`
  - 파일 읽기 실패 / 문서 파싱 실패 → `EXIT_RUNTIME`
  - 정상 종료 → `EXIT_OK`
- `main()` 디스패치의 `"dump"`·`"diag"` 분기를 `exit_with(...)`로 감쌈.
- `tests/cli_exit_codes_dump_diag.rs` 신규 — 인자 없음/읽기 실패/파싱 실패/성공 경로 4개 테스트.

## 검증

```
RUSTFLAGS="-C linker=rust-lld" cargo build --bin rhwp
RUSTFLAGS="-C linker=rust-lld" cargo test --test cli_exit_codes_dump_diag   # 신규 4개, 통과
RUSTFLAGS="-C linker=rust-lld" cargo test --test cli_exit_codes             # 기존 10개 회귀, 통과
cargo fmt --check -- src/main.rs tests/cli_exit_codes_dump_diag.rs         # 변경 파일 포맷 이슈 없음
                                                                             # (기존 examples/ 의 CRLF 경고는
                                                                             #  본 변경과 무관한 사전 존재 이슈)
RUSTFLAGS="-C linker=rust-lld" cargo clippy --bin rhwp --tests             # 신규 경고 없음
                                                                             # (johab.rs/byte_writer.rs 사전 존재
                                                                             #  경고 2건은 본 변경과 무관)
```

수정 전/후 실측:

```
$ rhwp dump nonexistent.hwp; echo $?
...
1   → (수정 전 0)

$ rhwp diag nonexistent.hwp; echo $?
...
1   → (수정 전 0)

$ rhwp dump; echo $?
...
2   → (수정 전 0)

$ rhwp diag; echo $?
...
2   → (수정 전 0)
```

성공 경로 회귀 확인 (samples/hwp3-sample.hwp):

```
$ rhwp dump samples/hwp3-sample.hwp; echo $?
0
$ rhwp diag samples/hwp3-sample.hwp; echo $?
0
```

## 범위

이번 수정은 `dump`·`diag` 두 명령만 다룬다. `ir-diff`/`test-*`/`gen-*` 계열과
`rhwp::diagnostics::*::run` 계열도 동일 클래스 버그가 있음을 확인했으나(실측:
모두 실패 시 exit 0), 함수 개수와 각각의 사용법/오류 경로가 많아 이번 범위에서는
제외했다. 후속 이슈로 분리 권장.
