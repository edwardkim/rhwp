# 구현계획서 — Task #2102

수정 원칙: 최소·표적 변경. 이미지 page-fill 배경만 "구역 첫 쪽" 조건으로 게이트.
단색/그라데이션 채우기·쪽 테두리선·쪽번호 배치는 무변경.

## Stage 1 — "구역 첫 쪽" 판정값을 배경 생성 경로에 전달 + 이미지 게이트

- `src/document_core/queries/rendering.rs` (build_render_tree 호출부, ~L3745):
  - 현재 페이지가 소속 구역의 첫 쪽인지 계산:
    `is_section_first_page = pagination[section_index].pages.first().page_index == page_content.page_index`
  - 이 값을 `build_render_tree` 인자로 전달.
- `src/renderer/layout.rs`:
  - `build_render_tree` 시그니처에 `is_section_first_page: bool` 추가 → `build_page_background`로 전달.
  - `build_page_background`: `page_bg_image` 를 `if is_section_first_page { image } else { None }` 로 게이트.
    - 단색(`page_bg_color`)·그라데이션(`page_bg_gradient`)은 무변경(모든 쪽 유지).
    - 배경 노드 자체는 계속 생성(흰 배경/테두리 계약 유지), 이미지만 첫 쪽 외 제외.
- 다른 build_render_tree 호출부(테스트 등) 인자 보정.

산출물: 소스 diff + 빌드 통과.

## Stage 2 — 단위 테스트

- `src/renderer/layout` 테스트에 추가:
  - 이미지 채우기 page_border_fill + `is_section_first_page=true` → PageBackground.image = Some.
  - 동일 입력 + `is_section_first_page=false` → PageBackground.image = None, 단색/그라데이션은 유지.
- 기존 `build_page_background`/`PageBackground` 관련 테스트 회귀 없음.

산출물: 테스트 추가 + `cargo test` 통과.

## Stage 3 — 시각/골든 회귀 검증 + 최종 보고서

- 대상 문서: `export-svg` 로 1쪽 배경 유지 / 2~5쪽 배경·가로줄 제거 확인 (before/after SVG diff).
- 쪽 테두리선(border line)이 있는 기존 샘플 1~2건: before/after SVG diff 로 테두리 무변경 확인.
- `cargo test` 전체 재확인.
- `mydocs/report/task_m100_2102_report.md` 작성 + orders 상태 갱신.

## 회귀 가드
- 이미지 없는 문서: 코드 경로 영향 없음(색/그라데이션 무변경).
- 모든 쪽 테두리선 문서: `build_page_borders` 무변경 → 영향 없음.
- 다중 구역: 각 구역의 첫 쪽 기준으로 판정(구역별 독립).
