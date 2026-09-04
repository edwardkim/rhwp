# 수정 보고서 — Task #4128 셀 커서 조회의 PartialTable 행/컷 메타데이터 좁히기

## 이슈

- Issue: https://github.com/edwardkim/rhwp/issues/4128
- 선행: #4126 / PR #4127 (빈 호스트 문단의 페이지 순회 스킵 — 본 트랙의 우회 케이스)
- 스택: `fix/issue-4126-cursor-rect-empty-para-skip` 위 레이어

## 결론 (요약)

셀 캐럿 질의(`get_cursor_rect_in_cell_native` / path API /
`last_rendered_para_in_container`)가 **후보 페이지를 render tree 없이 pagination
메타데이터만으로 정확히 좁히도록** `find_pages_for_cell_position` 을 신설했다.
115쪽 거대 셀 문서의 깊은 행 콜드 질의가 page tree 빌드 **평균 ~57·최악 115회 → 1~2회**,
문단축 120질의 벤치 **723s → 수 초**.

## 병목 함수 (정확한 이름·경로)

| 역할 | 함수 | 위치 |
|---|---|---|
| 과잉 후보 생성 (**원인**) | `DocumentCore::find_pages_for_paragraph` | `src/document_core/commands/text_editing.rs` |
| 좁힌 후보 생성 (**신설**) | `DocumentCore::find_pages_for_cell_position` | `src/document_core/commands/text_editing.rs` |
| 영향 받는 질의 진입점 | `DocumentCore::get_cursor_rect_in_cell_native` (js: `getCursorRectInCell`) / path API / `last_rendered_para_in_container` | `src/document_core/queries/cursor_rect.rs` |
| 후보당 낭비 비용의 실체 | `DocumentCore::build_page_tree_cached` → `build_page_tree` | `src/document_core/queries/rendering.rs` |
| 좁히기가 대조하는 메타데이터 | `PageItem::PartialTable` (start_row/end_row/start_cut/end_cut) | `src/renderer/pagination.rs` |
| 컷 창 단일 권위 (**추출**) | `cell_cut_window` / `single_row_cut_index` | `src/renderer/layout/table_layout.rs` · `table_partial.rs` |

## 원인

`find_pages_for_paragraph`(src/document_core/commands/text_editing.rs)는 `para_index`
만 매칭한다. `PageItem::PartialTable` 은 `start_row`/`end_row`/`start_cut`/`end_cut` 로
"이 페이지에 이 표의 어느 행·어느 컷 구간이 보인다" 를 이미 알고 있지만, 후보 산출이
이를 버려서 분할 표가 걸친 **전 페이지**가 후보로 나왔다. 셀 캐럿 질의는 후보를
오름차순으로 render tree 를 지어보며 탐색하므로, 대상이 뒤쪽 행이면 사실상 전체를
빌드했다 (#4126 이 우회한 빈 문단 케이스와 같은 계열, 텍스트 있는 셀은 미해결로 남았던
부분).

## 수정

- `find_pages_for_cell_position`(신규): 대상 (control, cell, cell_para, offset) 을
  `cell_units` 서수로 환산해 PartialTable 의 행/컷 창과 대조, 실제 렌더되는 페이지
  (보통 1, 컷 경계 2)만 반환. 셀 해석 실패·빈 결과는 legacy 전 페이지 후보로 폴백해
  정확성 불변.
- `cell_cut_window`/`single_row_cut_index` 추출: 컷 창 도출 인라인 2개소를 단일 권위로
  명명 — 렌더러와 질의가 같은 산식을 공유 (동작 불변).
- line_segs 없는 문단(중첩 표 host)은 첫 유닛으로, spacer 만 있는 빈 문단은 spacer
  서수로 매핑 — 전 페이지 강등 방지.
- 컷 경계 `ord==eu` 는 offset 이 줄 시작일 때만 이전 조각을 후보에 남겨 legacy
  오름차순 inclusive 매치와 결과 페이지 일치.

## 검증

- **A/C 차분**: 문단축 120점 캐럿 좌표·pageIndex 완전 동일 (diff 0).
- **SVG 6쪽(0/5/53/67/111/114) A/C 바이트 동일** — 렌더 무변경.
- **red→green 회귀 테스트**: `tests/issue_4128_cell_cursor_page_narrowing.rs` —
  거대 셀 앞(0)/중간(1250)/끝(2400) 문단 3위치 콜드 질의의 누적 page tree 빌드를
  작업량 카운터로 판별. 수정 원복 실측 **165회 → FAIL**, 수정 적용 **≤12 → PASS**
  (release-test 1.22s). 행 서수↑ ⇒ 페이지↑, 끝 위치 ≥100쪽 sanity 포함.
- release-test 전체·fmt·clippy·Native Skia 3종·wasm-pack build 통과 (수정 커밋 본문).

## 남긴 것

- 좁히기는 질의 경로 전용이다. paginate 자체의 병목(같은 샘플 16.9s)은 별개 원인 —
  #4129 트랙(스택 상위 레이어, task_4129 보고서)이 해소.
