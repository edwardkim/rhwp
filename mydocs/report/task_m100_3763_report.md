---
kind: report
status: active
canonical: mydocs/report/task_m100_3763_report.md
last_verified: 2026-08-02
---

# #3763 처리 기록 — CLI stdin 계약 테스트의 BrokenPipe 플레이키 제거

- Issue: [#3763](https://github.com/edwardkim/rhwp/issues/3763) — CLI stdin 계약 테스트가
  `BrokenPipe` 로 간헐 실패해 CI 를 막는다
- 브랜치 `task/3763-stdin-broken-pipe`

## 증상

2026-08-01 하루에 서로 다른 두 PR 이 같은 원인으로 막혔다. 매번 샤드 1개가 실패하고
fail-fast 로 나머지 7개가 취소되어, 실패 1건이 잡 8개를 통째로 날렸다.

| PR | 샤드 | 터진 테스트 |
|---|---|---|
| #3752 | 3/8 | `batch_global_auth_options_are_rejected_before_consuming_path_stdin` (34행) |
| #3753 | 6/8 | `batch_convert_rejects_flag_as_out_dir_before_any_write` (53행) |

```
stdin 쓰기 실패: Os { code: 32, kind: BrokenPipe, message: "Broken pipe" }
```

두 PR 모두 `tests/batch_axes_contract.rs` 를 건드리지 않았다. #3752 는 `src/main.rs` 의
진단 서브커맨드 종료 코드, #3753 은 `src/serializer/hwpx/utils.rs` 의 테스트 추가였다.
즉 변경 내용과 무관한 기존 플레이키다.

## 근인 — 검증하려는 동작이 곧 실패 원인

헬퍼는 자식을 띄우고 stdin 에 경로 목록을 쓴 뒤 결과를 받는다.

```rust
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(stdin_body.as_bytes())
        .expect("stdin 쓰기 실패");   // <- EPIPE 면 패닉
```

그런데 이 헬퍼를 쓰는 테스트들의 계약이 **"자식이 stdin 을 읽기 전에 거부하고 종료한다"**
이다. 테스트 이름이 그대로 말한다 — `*_rejected_before_consuming_path_stdin`,
`*_rejects_flag_as_out_dir_before_any_write`.

자식이 인자 검증 단계에서 usage 오류로 즉시 죽으면 파이프의 읽기 끝이 닫히고, 부모의
`write_all` 은 `EPIPE` 를 받는다. 파이프 버퍼(리눅스 기본 64KiB)가 흡수해 주면 통과하고,
자식이 조금 더 빨리 죽으면 실패한다 — 순수한 경합이다.

**기능이 의도대로(=더 일찍) 거부할수록 테스트가 더 잘 깨진다.** 부하가 큰 CI 러너에서
특히 잘 드러나고, 로컬에서는 거의 재현되지 않는다.

`EPIPE` 는 여기서 오류가 아니라 검증 대상 동작의 정상적인 부산물이다.

## 수정

`write_all` 이 `ErrorKind::BrokenPipe` 를 돌려주면 넘어가고, 그 밖의 오류는 지금처럼
패닉한다.

- `tests/batch_axes_contract.rs` — 호출부가 2곳이라 `write_stdin_tolerating_broken_pipe`
  헬퍼로 묶고 `run_with_stdin`·`run_with_stdin_in_dir` 이 함께 쓴다.
- `tests/cli_json_contract.rs` — 같은 형태의 헬퍼 1곳. `batch_mode_flag_rejected_for_other_subcommands`
  등 같은 성격의 조기 거부 테스트를 갖고 있어 잠재 위험이 동일하다.
- `tests/info_title_contract.rs` — 같은 형태의 헬퍼 1곳.

검증력은 줄지 않는다. 실제 계약은 `wait_with_output()` 이 돌려주는 종료 코드·stdout·
부분 산출물 유무로 확인하고, 그 단언은 그대로다. 자식이 정말로 stdin 내용을 필요로 했다면
바로 그 단언에서 잡힌다.

제품 코드 변경 없음. 테스트 하네스만 고친다.

## 검증

- `cargo fmt` 기준 3개 파일 포맷 확인.
- 저장소 전체 빌드·테스트는 CI 에 맡긴다 — 작업 PC 의 MSVC 링커(`dbghelp.lib`)가 손상되어
  있고 GNU 툴체인에는 `dlltool` 이 없어 로컬 `cargo test` 가 아예 돌지 않는다.
- 플레이키 특성상 초록 1회가 수정의 증명은 아니다. 다만 실패 경로가 `write_all` 의 `EPIPE`
  단 하나였고 그 경로를 제거했으므로, 이후 같은 메시지로 재발하면 이 수정과 다른 원인이다.
