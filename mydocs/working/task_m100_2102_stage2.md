# Stage 2 완료보고서 — Task #2102

## 목표
이미지 page-fill 첫 쪽 게이트 단위 테스트 추가.

## 변경 내용
- `src/renderer/layout/tests.rs`
  - 헬퍼 `page_bg_color_and_image_present(is_section_first)`: 이미지+색 채우기를 가진
    쪽 테두리/배경으로 렌더 트리를 만들어 PageBackground 노드의 (색 유무, 이미지 유무) 반환.
  - 테스트 `page_bg_image_only_on_section_first_page`:
    - `is_section_first=true` → 이미지 Some, 색 Some.
    - `is_section_first=false` → 이미지 None, 색 Some (억제돼도 색은 유지).

## 검증
- `cargo test --lib page_bg_image_only_on_section_first_page` → ok (1 passed).
