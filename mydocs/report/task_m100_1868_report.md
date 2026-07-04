# Task M100 #1868 최종 보고서 — CLI export-hwpx 명령 추가 (HWP→HWPX 직접 변환)

- 이슈: #1868 (외부 사용자 joahani3 기능 요청, milestone v1.0.0)
- 브랜치: `local/task1868` / 작성일: 2026-07-04

## 1. 결과

```
rhwp export-hwpx <입력.hwp|입력.hwpx> [출력.hwpx]
```

HWP5/HWP3/HWPX 입력을 자동 감지해 HWPX 로 직렬화 저장하는 CLI 명령을 추가했다. 코어는
기존 `export_hwpx_native()`(#1532/#1533, studio `exportHwpx` 와 동일 경로) 재사용 — 렌더링·
직렬화 로직 무변경, CLI 표면만 추가.

## 2. 조사 확정 사항

- 기존 `convert` 는 포맷 변환이 아니라 **배포용 해제 → 항상 .hwp 출력** — HWP→HWPX 방향
  CLI 부재가 사실이었고, 이슈 요청 타당.
- 설계 A안(`export-hwpx` 신설) 채택 — export-* 계열 명명 일관, `convert` 의미 보존,
  #1879 하니스 등 기존 사용처 무영향. (B안 convert 확장자 분기는 배제.)

## 3. 구현 (1단계, `src/main.rs`)

- dispatch + `fn export_hwpx()` + help 등재. 출력 생략 시 `<stem>.hwpx`, 비-.hwpx 확장자
  경고, **입력==출력 경로 거부**(HWPX 입력+출력 생략 조합의 원본 덮어쓰기 방지).

## 4. 검증 (2단계)

| 항목 | 결과 |
|---|---|
| 신규 `tests/issue_1868_export_hwpx_cli.rs` (3건) | HWP5(pr-1674)·HWP3(hwp3-sample)·HWPX 재직렬화 — ZIP 매직+재파싱+페이지 보존 전부 ✓ |
| 전체 `cargo test --tests` (release-test) | **FAILED 0** (2812 passed) |
| fmt / clippy | clean / 무경고 |
| 스모크 | pr-1674.hwp → 424KB HWPX, 재파싱 35쪽(정답지 35 일치) |
| **왕복 체인** | export-hwpx 산출물 → #1879 하니스(`convert` 재변환+dump-pages) → **SAME 35=35쪽** |

## 5. 문서 (3단계)

- `mydocs/manual/cli_commands.md` §3 변환·비교에 등재(+convert 의미 명확화 1줄).
- 시각 판정 게이트: 불필요(렌더링 무변경). 변환 산출물의 직렬화 충실도는 기존
  `hwpx_roundtrip_baseline` + #1879 하니스가 커버.

## 6. 산출물

- 코드: `src/main.rs`(dispatch+함수+help), `tests/issue_1868_export_hwpx_cli.rs`(신규 3)
- 문서: 수행/구현계획서, stage1/2, 본 보고서, cli_commands.md
