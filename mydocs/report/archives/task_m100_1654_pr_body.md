## 변경 내용

- HWPX `<hp:visibility hideFirstEmptyLine="...">` 파싱 시 `SectionDef.hide_empty_line`과 HWP5 `SectionDef.flags` bit 19(`0x0008_0000`)를 함께 동기화했습니다.
- HWPX -> HWP 어댑터에서 저장 직전 `hide_empty_line` 기준으로 bit 19를 materialize하도록 보강했습니다.
- 파서 단위 테스트와 HWP 직렬화 후 재로드 통합 테스트를 추가했습니다.

## 원인

기존 HWPX 파서는 `hideFirstEmptyLine="1"`을 bool 필드에는 저장했지만, HWP5 저장 경로가 실제로 쓰는 `SectionDef.flags`에는 반영하지 않았습니다. 그 결과 HWPX -> HWP 변환 시 `Control::SectionDef`로 복사되는 flags가 stale 상태일 수 있었습니다.

## 검증

- `cargo fmt --check`
- `git diff --check`
- `env CARGO_INCREMENTAL=0 cargo test task1654 --lib`
- `env CARGO_INCREMENTAL=0 cargo test --test hwpx_to_hwp_adapter task1654_hide_empty_line_flag_preserved_after_hwp_export_reload`
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`

Closes #1654
