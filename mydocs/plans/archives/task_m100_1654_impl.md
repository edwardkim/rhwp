# task m100 #1654 구현 계획

## Stage 1

- `src/parser/hwpx/section.rs`
  - `hideFirstEmptyLine` 파싱 시 `SectionDef.flags`의 `0x0008_0000` bit를 set/clear한다.
  - minimal HWPX section fixture로 `section.section_def`와 `Control::SectionDef`가 모두 동기화되는지 확인한다.

- `src/document_core/converters/hwpx_to_hwp.rs`
  - HWPX -> HWP 어댑터에서 `SectionDef.hide_empty_line` 기준으로 flags bit 19를 동기화한다.
  - 기존 컨트롤이 없거나 오래된 경우에도 `insert_section_def_control` 이후 저장 가능한 값이 유지되는지 확인한다.

## 비범위

- HWPX serializer의 visibility 보존은 #1637/#1642에서 처리된 영역이므로 변경하지 않는다.
- 렌더러 pagination 의미는 변경하지 않는다.
