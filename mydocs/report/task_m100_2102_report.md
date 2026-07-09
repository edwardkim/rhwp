# 최종 결과보고서 — Task #2102

## 제목
쪽 배경 이미지 채우기(page_border_fill)가 모든 쪽에 그려지는 문제 — 구역 첫 쪽에만 적용

이슈: edwardkim/rhwp#2102

## 증상
구역 쪽 테두리/배경에 **그림(이미지) 채우기** 배경이 설정된 HWP5 문서에서, 표지(구역 첫 쪽)용
배경 이미지가 2쪽 이후 모든 쪽에 반복 렌더링됨. 이미지에 상단 가로줄 등 표지 장식이 포함된
경우, 2쪽 이후 본문 텍스트를 관통하는 가로줄이 그려지는 시각 결함 발생.

한컴 편집기 실측(작업지시자 확인): 배경 이미지는 **구역 첫 쪽에만** 표시, 2~N쪽 없음.

## 근본 원인
- 렌더러(`src/document_core/queries/rendering.rs`)가 구역 내 **모든 쪽**에 대해
  `section_def.page_border_fill`(record[0])을 그대로 사용.
- `src/renderer/layout.rs` `build_page_background`가 페이지 적용 범위를 고려하지 않고
  배경(색/그라데이션/이미지) 노드를 생성.
- 대상 문서 구조: 바탕쪽(master page) 없음, 배경 이미지는 오직 구역 page_border_fill
  record[0](BOTH, `border_fill_id=23`, 이미지 `FitToSize`, `fillArea=종이 전면`).

## 조사로 확정한 사실
- HWP5 `PAGE_BORDER_FILL`은 3-레코드 `[BOTH,EVEN,ODD]` 구조이며, 적용 범위는
  **양쪽/홀수/짝수만** 존재 (참조 UI 리소스 `border.ui`로 교차 확인). "첫 쪽만"을 인코딩하는
  파일 필드는 없음.
- 즉 한컴이 이미지 page-fill을 첫 쪽에만 그리는 것은 파일 플래그가 아니라 한컴 렌더 동작으로
  판단됨. 본 수정은 그 동작에 맞추는 규칙.

## 수정 내용
- `src/renderer/layout.rs`
  - `LayoutEngine.current_page_is_section_first: Cell<bool>` (기본 true) + 세터 추가.
  - `build_page_background`: **이미지 채우기만** 구역 첫 쪽일 때 생성. 색/그라데이션 채우기,
    배경 노드 생성, 쪽 테두리선(`build_page_borders`)은 무변경.
- `src/document_core/queries/rendering.rs`
  - `build_render_tree` 직전 현재 페이지가 구역 첫 글로벌 페이지인지 판정하여 세터로 전달.
- `src/renderer/layout/tests.rs`
  - 단위 테스트 `page_bg_image_only_on_section_first_page` 추가.

## 검증
- `cargo build --release` 성공.
- `cargo test` 전체 통과 (exit 0, 실패 0; lib + 통합/골든 스냅샷 포함).
- 신규 단위 테스트 통과.
- 대상 문서 시각 회귀:
  - 1쪽: 표지 배경(격자+가로줄+컬러바+로고) 유지.
  - 2~5쪽: 전면 배경 이미지 및 본문 관통 가로줄 제거 (before/after SVG 대조 확인).

## 한계 / 후속
- 본 수정은 "이미지 page-fill = 구역 첫 쪽" 규칙(한컴 동작 정합)이며, 파일에 인코딩된 규칙이
  아님. 만약 향후 "모든 쪽 이미지 워터마크"가 정당한 문서가 발견되면 규칙 재검토 필요
  (현 저장소 골든/샘플에는 해당 사례 없음 → 회귀 표면 제한적).
- 색/그라데이션 page-fill과 쪽 테두리선은 종전대로 전 쪽 적용.
