# task m100 #1654 stage 1

## 변경

- `src/parser/hwpx/section.rs`
  - `<hp:visibility hideFirstEmptyLine="...">` 파싱 시 `SectionDef.hide_empty_line`과 HWP5 `SectionDef.flags` bit 19(`0x0008_0000`)를 함께 동기화했다.
  - 파서 단위 테스트로 `section.section_def`와 첫 문단 `Control::SectionDef`가 모두 동기화되는지 검증했다.

- `src/document_core/converters/hwpx_to_hwp.rs`
  - HWPX -> HWP 어댑터에서 `SectionDef.hide_empty_line` 기준으로 flags bit 19를 materialize한다.
  - 보정 횟수를 `AdapterReport.section_def_hide_empty_line_flag_materialized`에 기록하고 `changed_anything()`에 포함했다.

- `tests/hwpx_to_hwp_adapter.rs`
  - 어댑터 적용 후 실제 HWP 직렬화와 재로드까지 수행해 `hide_empty_line`과 flags bit 19가 보존되는지 검증했다.

## 검증

- `cargo fmt --check` 통과
- `git diff --check` 통과
- `env CARGO_INCREMENTAL=0 cargo test task1654 --lib` 통과
- `env CARGO_INCREMENTAL=0 cargo test --test hwpx_to_hwp_adapter task1654_hide_empty_line_flag_preserved_after_hwp_export_reload` 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` 통과
