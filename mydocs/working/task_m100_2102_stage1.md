# Stage 1 완료보고서 — Task #2102

## 목표
쪽 배경 **이미지 채우기**를 구역 첫 쪽에만 적용하도록 렌더 경로 수정.

## 변경 내용
- `src/renderer/layout.rs`
  - `LayoutEngine`에 `current_page_is_section_first: Cell<bool>` 필드 추가 (기본값 `true` = 기존 동작 보존).
  - `set_current_page_is_section_first(bool)` 세터 추가 (기존 `set_total_pages` 등과 동일한 페이지별 컨텍스트 패턴).
  - `build_page_background`: 이미지 채우기(`page_bg_image`)를 `current_page_is_section_first` 참일 때만 생성. 색/그라데이션 채우기, 배경 노드 생성 자체는 무변경.
- `src/document_core/queries/rendering.rs`
  - `build_render_tree` 호출 직전, 현재 페이지가 소속 구역의 첫 글로벌 페이지인지
    (`pagination[sec].pages.first().page_index == page_content.page_index`) 판정하여 세터로 전달.

## 검증
- `cargo build --release` 성공.
- 대상 문서 `export-svg` 재확인:
  - 1쪽: 전면 배경 이미지 유지 (YES)
  - 2~5쪽: 전면 배경 이미지 없음 (no)
- 시각 확인: 1쪽 표지 그대로, 2쪽 격자 배경 + 본문 관통 가로줄 제거.

## 회귀 가드
- 색/그라데이션 채우기, 쪽 테두리선(`build_page_borders`), 쪽번호 배치 무변경.
- 세터 기본값 true → 렌더 경로 밖(테스트 등) 기존 동작 보존.
