# Task m100 #1654 최종 보고서

## 개요

- 이슈: https://github.com/edwardkim/rhwp/issues/1654
- 주제: HWPX -> HWP 변환에서 `hideFirstEmptyLine` 값을 HWP5 `SectionDef.flags` bit 19로 동기화
- 브랜치: `local/task_m100_1654`
- 기준: `upstream/devel`

## 원인

HWPX 파서의 `<hp:visibility>` 처리에서 `hideFirstEmptyLine="1"`은 `SectionDef.hide_empty_line` bool에는 저장됐지만, HWP5 저장 경로가 직렬화하는 `SectionDef.flags` bit 19(`0x0008_0000`)에는 반영되지 않았다.

HWPX -> HWP 어댑터는 `section.section_def`를 `Control::SectionDef`에 복사하므로, flags가 stale이면 HWP 저장 결과도 stale 값으로 남을 수 있었다.

## 변경

- `src/parser/hwpx/section.rs`
  - `hideFirstEmptyLine` 파싱 시 `SectionDef.hide_empty_line`과 `SectionDef.flags & 0x0008_0000`을 함께 set/clear한다.
  - 파서 단위 테스트로 `Section.section_def`와 첫 문단 `Control::SectionDef` 양쪽 동기화를 검증한다.

- `src/document_core/converters/hwpx_to_hwp.rs`
  - HWPX -> HWP 어댑터에서 `SectionDef.hide_empty_line` 기준으로 HWP5 flags bit 19를 저장 직전 materialize한다.
  - 보정 횟수를 `AdapterReport.section_def_hide_empty_line_flag_materialized`에 기록하고 `changed_anything()` 판정에 포함한다.

- `tests/hwpx_to_hwp_adapter.rs`
  - 어댑터 적용 후 실제 HWP 직렬화와 재로드를 수행해 `hide_empty_line`과 flags bit 19가 함께 보존되는지 검증한다.

## 검증

- `cargo fmt --check` 통과
- `git diff --check` 통과
- `env CARGO_INCREMENTAL=0 cargo test task1654 --lib` 통과
- `env CARGO_INCREMENTAL=0 cargo test --test hwpx_to_hwp_adapter task1654_hide_empty_line_flag_preserved_after_hwp_export_reload` 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` 통과

## PR 준비

- PR base: `devel`
- PR head 예정: `task_m100_1654`
- PR 본문: `mydocs/report/task_m100_1654_pr_body.md`
- 자동 종료 키워드: `Closes #1654`

