---
kind: report
status: active
canonical: mydocs/report/task_m100_3884_report.md
last_verified: 2026-08-04
---

# #3884 처리 기록 — 진단 명령이 미지 플래그를 조용히 무시한다 (G1·G2)

- Issue: [#3884](https://github.com/edwardkim/rhwp/issues/3884) — [#3880 L1] 봉투 규약 위반
- 브랜치 `task/3884-unknown-flag-rejection`

## 증상

이슈가 실측으로 등록한 위반이다. devel 에서 코드로 재확인했다.

| 명령 | 종전 동작 | 어긴 규약 |
|---|---|---|
| `dump <문서> --bogus-flag` | exit 0, stdout 18,643 B | #3349 — 미지 플래그 즉시 exit 2 |
| `dump <문서> --json` | exit 0, 사람용 텍스트 | 같음 (`--json` 도 미지 플래그로 취급됨) |
| `diag <문서> --json` | exit 0, 사람용 텍스트 | 같음 |
| `bench <문서> --json` | exit 1, stdout 518 B | `jsonContract.failure` — 실패 시 stdout 0바이트 |

## 근인 — 세 곳이 서로 다른 방식으로 삼켰다

- `dump_controls` (`src/main.rs`) — 인자 루프의 `_ => { i += 1; }` 가 미지 인자를 **건너뛴다**.
- `diag_document` (`src/main.rs`) — 플래그를 **아예 파싱하지 않고** `args[0]` 만 쓴다.
  뒤에 뭘 붙이든 통째로 무시된다.
- `bench::run` (`src/diagnostics/bench.rs`) — `other => files.push(...)` 로 미지 인자를
  **파일 경로로 삼킨다.**

`bench` 가 G1(실패 경로 stdout 유출)까지 어긴 것은 이 삼킴의 결과다. `--json` 이 "파일"이
되어 읽기에 실패하는데, 그 시점엔 이미 헤더 두 줄을 stdout 에 찍은 뒤다.

조용히 무시하는 쪽이 오류보다 나쁘다. 오류라면 호출자가 고칠 수 있지만, 성공으로 돌아오면
**자기가 요청한 것과 다른 것을 받고도 알 수 없다.** `--json` 을 준 에이전트가 사람용 텍스트를
JSON 으로 파싱하다 깨지는 경로가 정확히 이것이다.

## 수정

세 곳 모두 #3349 규약대로 **`-` 로 시작하는 미지 인자를 즉시 거부**한다(exit 2, stdout 비움,
사유는 stderr).

`-` 로 시작하지 않는 인자는 종전 취급을 유지한다 — `bench` 의 파일 목록처럼 위치 인자를
여럿 받는 명령이 있어서, 모든 미지 인자를 거부하면 정상 호출이 깨진다.

`--json` 을 이 세 명령에 **구현하지는 않았다.** 이슈가 지적한 것은 "선언과 실행이 어긋난다"가
아니라 "조용히 무시한다"이고, JSON 봉투를 새로 만드는 것은 별도 계약 설계다. 지금은 명확한
거부로 어긋남을 없앤다.

## 검증

- `tests/issue_3884_unknown_flag_rejection.rs` — 네 조합(`dump --bogus-flag`,
  `dump --json`, `diag --json`, `bench --json`)이 exit 2 · stdout 0바이트 · stderr 비어 있지
  않음을 고정한다. 반대 방향으로 정상 호출(`dump`, `dump --section 0`, `diag`)이 그대로
  성공하는지도 함께 본다 — 거부 규칙이 멀쩡한 호출을 깨면 안 된다.
- `rustfmt` 로 변경 파일 포맷 확인.
- 이 PC 는 MSVC 링커(`dbghelp.lib`) 손상으로 `cargo test` 가 돌지 않는다. CI 가 판정한다.
