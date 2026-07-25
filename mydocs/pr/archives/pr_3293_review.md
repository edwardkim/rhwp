# PR #3293 검토·통합 기록 — self-hosted Rustup settings 레이스

## 메타

| 항목 | 값 |
|---|---|
| PR | [#3293](https://github.com/edwardkim/rhwp/pull/3293) |
| 작성자 | `edwardkim` |
| base / merge commit | `devel` / `5242c1fad39585e37177a853b56e78deb1871c5f` (squash merge, 2026-07-25 07:14:53 UTC) |
| 원 PR head | `1178c68de7f57e8ff90dc63a317e9b8e2cbfb512` |
| 관련 이슈 | [#3289](https://github.com/edwardkim/rhwp/issues/3289) — 이미 CLOSED (PR #3286 통합 시 auto-close) |
| 규모 | 4 files, +76/-9 — CI workflow/composite action 및 #3289 운영 보고서 |
| 유형 | self-hosted CI runner 공유 홈 상태 레이스 보정; 렌더 출력 변경 없음 |

## 배경과 범위 판단

[#3270](https://github.com/edwardkim/rhwp/pull/3270)의 CI run `30147842082`에서
`runner-lxc-06`이 테스트 시작 전에 `~/.rustup/settings.toml`을 읽지 못했다. 오류는
`missing field version`이었으며, `dtolnay/rust-toolchain`이 병렬 job마다 공유
`~/.rustup/settings.toml`을 재작성하는 경로가 원인이다. shard 5가 즉시 실패했고,
나머지 shard는 matrix `fail-fast`로 취소됐다.

PR은 self-hosted runner에서는 `dtolnay/rust-toolchain`을 실행하지 않고 호스트에 미리 설치된
Rust toolchain과 `rust-toolchain.toml` directory override를 사용하게 했다. 같은 공유 홈 위험을
가진 wasm-pack 전개, cache 저장, `apt-get` 설치 경로도 hosted 전용으로 gate했다.

CI/workflow와 운영 문서만 바꾸며 PDF/SVG/Canvas 출력·fixture·renderer 코드는 바꾸지 않으므로,
2.6절의 visual sweep 대상은 아니다.

## 검증과 통합 판단

PR head 기준 GitHub Actions가 모두 성공했다.

- CI run `30148354306`: preflight, lint, frontend gate, Build test archive, Native Skia,
  Default-feature test 8 shard, Build & Test 모두 SUCCESS. `WASM Build`는 해당 경로 미변경으로
  SKIPPED.
- CodeQL run `30148354316`: JavaScript/TypeScript, Python, Rust 분석 및 CodeQL 모두 SUCCESS.
- Render Diff run `30148354269`: preflight와 Canvas visual diff 모두 SUCCESS.

`5242c1fad`와 원 PR head `1178c68de`의 변경 대상 네 파일 최종 tree가 동일함을 확인했다.
따라서 PR의 squash merge는 검토한 보정 내용을 그대로 `devel`에 반영한다.

## merge 후 상태

- [#3289](https://github.com/edwardkim/rhwp/issues/3289)은 PR #3286 반영 시 이미 CLOSED였고,
  이번 PR은 그 후 발견된 마지막 공유 Rustup 쓰기 경로를 보정한다. 이슈 재-close는 하지 않는다.
- 신규 후속 이슈는 없다. self-hosted 공통 쓰기 경로의 전수 점검표는
  `mydocs/report/task_m100_3289_report.md`에 갱신됐다.

## 최종 권고

**수용·merge 완료.** #3293은 `5242c1fad`로 squash merge됐으며, 후속 기록 PR이 merge된 뒤
원 PR·관련 이슈에 CI 정상화와 종료 상태를 코멘트하고 작업 branch를 정리한다.
