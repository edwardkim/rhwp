# Task #1129 Stage 3 - HWP5/HWPX 격자 관련 스펙 보존

- 이슈: [#1129](https://github.com/edwardkim/rhwp/issues/1129)
- 브랜치: `local/task_m100_1129`
- 일자: 2026-05-26

## HWP5

- HWP5 `SECTION_DEFINE`의 줄 격자/글자 격자 값을 `SectionDef`에 보존하도록 했다.
- 저장 시 기존처럼 0으로 버리지 않고 파싱된 `line_grid`, `char_grid` 값을 다시 쓴다.

## HWPX

- HWPX `hp:secPr/hp:grid`의 `lineGrid`, `charGrid` 값을 `SectionDef`에 반영했다.
- HWPX 문단 모양의 `snapToGrid` 속성을 HWP5 문단 속성 bit로 보존했다.
- OWPML 기본값이 `snapToGrid=true`인 점을 고려해 속성 누락 시 기본 bit를 세운다.

## 변경 파일

- `src/model/document.rs`
- `src/parser/body_text.rs`
- `src/parser/body_text/tests.rs`
- `src/parser/hwpx/header.rs`
- `src/parser/hwpx/section.rs`
- `src/serializer/control.rs`

## 검증

- `cargo test test_parse_section_with_section_def --lib -- --nocapture` 통과
- `cargo test test_parse_section_grid_preserves_line_and_char_grid --lib -- --nocapture` 통과
- `cargo test test_parse_hwpx_para_shape_snap_to_grid_bit --lib -- --nocapture` 통과
