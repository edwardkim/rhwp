# 구현계획서 — Task M100 #2145: 개요번호 `^n`/`^N` 레벨 경로 자동코드

- 수행계획서: `task_m100_2145.md` / 브랜치: `fix/2145-outline-levelpath`

## 1단계 — `expand_numbering_format` 확장 (utils.rs)

- 시그니처에 `current_level: usize`(0-based, 호출부의 `level_idx`) 추가.
- 수준 1개 번호 포맷 로직(카운터+시작번호 → `format_number`)을 지역 클로저
  `format_level(idx)`로 추출 (기존 `^digit` 분기와 신규 분기가 공유).
- `^` 다음 문자 분기 확장:
  - ASCII 숫자(`1`~`7`): 기존 동작 유지 (`format_level(digit-1)`)
  - `n`: `format_level(0)`~`format_level(current_level)`을 `.`로 join
  - `N`: `^n` 결과 + 후행 `.`
  - 그 외: 기존대로 리터럴 통과
- 호출부 1곳(`paragraph_layout.rs` `apply_paragraph_numbering`)에 `level_idx` 전달.

## 2단계 — 회귀 테스트 (renderer/layout/tests.rs)

- 기존 `expand_numbering_format` 호출 2건에 파라미터 추가 (동작 불변 확인).
- 신규 테스트 `test_expand_numbering_format_level_path`:
  - `^N`, level 0, counters=[1,..] → `"1."`
  - `^N`, level 1, counters=[1,1,..] → `"1.1."`
  - `^n`, level 1 → `"1.1"` (후행 마침표 없음)
  - `^N`, level 1, 수준별 number_format 혼합(L2=HangulGaNaDa) → `"1.가."`
  - 상위 수준 카운터 0(start 폴백) 케이스

## 3단계 — 게이트 + 시각 검증 + 보고

- `cargo build --release --features native-skia` / `cargo test` / `cargo clippy`
- 로컬 `1.hwp` `export-png -p 1..3` ↔ oracle PDF 대조:
  `1. 개 요`, `1.1.`~`1.4.`, `2. 점검 결과`, `3. 상세 점검 결과`
- 최종 보고서 `mydocs/report/task_m100_2145_report.md` + 이슈 클로즈(승인 후)
