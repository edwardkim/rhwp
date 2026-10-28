# 구현계획서 — Task M100 #1868: CLI export-hwpx 명령 (A안, 3단계)

- 이슈: #1868 / 수행계획서: `task_m100_1868.md` (A안 승인, 2026-07-04)
- 브랜치: `local/task1868`

## 1단계 — `export-hwpx` 서브커맨드 구현

`src/main.rs`:

1. dispatch 에 `Some("export-hwpx") => export_hwpx(&args[2..])` 추가
   (기존 export-* 계열 나란히, line ~17 부근).
2. `fn export_hwpx(args: &[String])` 신설 — 기존 export 함수 관례(위치 인자 + 친절한
   한국어 오류) 준수:
   - `args[0]` = 입력 경로(필수). 없으면 사용법 출력.
   - `args[1]` = 출력 경로(선택). 생략 시 `<입력 stem>.hwpx`(입력과 같은 폴더).
     출력 확장자가 `.hwpx` 아니면 경고 후 진행.
   - `fs::read` → `HwpDocument::from_bytes`(포맷 자동 감지: HWP5/HWP3/HWPX 모두 수용)
     → `export_hwpx_native()` → `fs::write`.
   - 완료 메시지: `저장 완료: <출력> (NKB)` (convert 관례 정합).
3. `print_usage` 에 `export-hwpx <파일.hwp|파일.hwpx> [출력.hwpx]` 등재
   (export-pdf 다음 위치).

완료 기준: `cargo build` + 수동 스모크(`samples/pr-1674.hwp` → .hwpx, PK 매직 확인).

## 2단계 — 테스트 + 회귀

1. 신규 통합 테스트 `tests/issue_1868_export_hwpx_cli.rs`:
   - 코어 경로 검증: HWP5 fixture(`samples/pr-1674.hwp`) 로드 → `export_hwpx_native()` →
     산출 bytes 가 ZIP 매직(`PK\x03\x04`) + 재파싱 성공 + **페이지 수 보존**(35).
   - HWP3 fixture 1건(`samples/…V3…`) 동일 검증(HWP3→HWPX 경로 커버) — 적합 샘플 부재 시
     HWP5 2건으로 대체하고 보고서에 명시.
2. 회귀: `cargo test --profile release-test --tests` 전체 + `hwpx_roundtrip_baseline` +
   fmt/clippy.
3. 실전 스모크: 변환 산출 HWPX 를 #1879 하니스에 입력해 dump-pages 정합(SAME) 확인.

완료 기준: 신규 테스트 green + 전체 FAILED 0.

## 3단계 — 문서 + 마무리

1. `mydocs/manual/cli_commands.md` 에 export-hwpx 절 추가(사용법·기본 출력 규칙).
2. 최종 보고서 `mydocs/report/task_m100_1868_report.md` + 오늘할일(`orders/20260704.md`) 갱신.
3. 이슈 #1868 회신(사용법 안내) — close 는 작업지시자 승인 후.

## 위험/비고

- 렌더링·직렬화 로직 무변경(CLI 표면만) — 시각 판정 게이트 불필요, blast-radius 최소.
- HWPX 입력 → HWPX 출력도 허용(사실상 re-serialize) — 거부하지 않음(roundtrip 디버깅에 유용).
- CLAUDE.md 문서(cli 사용법 표)는 별도 커밋으로 분리하지 않고 manual 만 갱신(루트 CLAUDE.md
  는 export-svg 중심이라 필수 아님 — 필요 시 작업지시자 판단).
