# PR #4094 메인터너 보정 기록

## 목적

PR #4094의 Square 어울림 표 흐름 보정이 최소 fixture에서 14쪽으로 과소 분할되던 상태를,
HWP 2020 MCP 기준 17쪽과 저장 line 경계가 일치하도록 보완했다. 기여자 원 commit은 rewrite하지
않고 같은 source branch 위에 메인터너 commit만 추가했다.

## 적용 순서

| 단계 | commit | 내용 |
| --- | --- | --- |
| contributor fixture | `e689ef041` | 최소화 HWPX fixture와 17쪽 기준 재현 |
| Stage 1 보정 | `2d6b819e9` | 저장 HWPX tail explicit page break 경계 복원 |
| Stage 2 회귀 | `8ac7d4714` | CLI `dump-pages --json` 17쪽·세 tail 분할 계약 |
| Stage 3 가드 | `fd4d773ab` | 마지막 stored line에만 적용되는 unit test 두 건 |
| 기준 증적 | `a9c297006` | HWP 2020 MCP PDF를 `pdf/issue4090`에 보존 |

## 보정 범위

- `src/renderer/typeset.rs`에 저장 HWPX tail 전용 page break predicate를 추가했다.
- layout·pagination 경로에 같은 물리 line 경계를 전달해 document end에서만 쪽을 닫도록 했다.
- 외부 fixture test는 17쪽과 `pi=59`, `pi=74`, `pi=183`의 세 경계를 검증한다.
- 내부 unit test는 마지막 stored line 조건과 다음 logical line이 있으면 적용하지 않는 조건을 검증한다.

## 최신 기준선과 검증 결과

focused Rust test 3건, fmt, clippy, WASM package build를 로컬에서 통과했다. 이후 code candidate
`a9c2970…` 뒤 source branch에 최신 `devel`을 병합한 `afc83ff…`에서 GitHub
[CI](https://github.com/edwardkim/rhwp/actions/runs/31148171463)·
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/31148171307)·
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/31148171363)·Native Skia·전체 shard가
성공했다. 작성 시점 PR은 `MERGEABLE/CLEAN`이다. PDF 기준은 HWP 2020 MCP `PrintToPDFEx`
`PrintMethod=0`의 A4 17쪽 산출물이다.

## 남은 범위와 rollback

이번 보정은 #4094가 재현한 page count와 tail line ownership만 다룬다. 전체 페이지의 raster/pixel
fidelity는 #3820에서 기준 PDF와 직접 비교해 별도로 판단한다. source branch에는 최신 `devel` 병합 뒤의
review-only 기록만 추가하므로, `afc83ff…`의 녹색 CI를 재사용하는 fast-pass aggregate가 실패하면 그
원인을 확인한 뒤 후속 조치를 별도로 결정한다.
