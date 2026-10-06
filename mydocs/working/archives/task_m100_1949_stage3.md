# #1949 Stage 3 — 구현 + 검증

- 브랜치: `local/task1949`

## 1. 수정 내용

거대 셀 콘텐츠 유닛(`cell_units`)을 **셀 포인터 키로 메모이즈**해 O(pages × cell) 재계산을
제거한다.

- `LayoutEngine`(문서 렌더 라이프사이클 지속, `document_core.layout_engine`)에
  `cell_units_cache: RefCell<HashMap<usize, Rc<Vec<CellUnit>>>>` 필드 추가.
- `cell_units` 를 메모이즈 래퍼로 변경(본체는 `cell_units_uncached`). 키 = `cell as *const
  Cell as usize`(렌더 중 IR 불변 → 셀 유일 식별). 반환 `Rc<Vec<CellUnit>>`(10개 호출부는
  읽기 전용이라 Deref 로 무변경 컴파일).
- 재조판 경계 `invalidate_page_tree_cache()` 에서 `clear_layout_caches()` 호출 —
  다른 IR 의 포인터 재사용 방지.

`cell_units` 는 `cell_units_fitting_height`·`cell_line_ranges_from_cut` 등 컷 판정 10경로가
호출하며, 거대 셀이 걸친 페이지마다 반복 → 캐시 적중률이 매우 높다. `composed_paras`(드로잉)
캐시는 **불필요**했다(cell_units 캐시만으로 목표 달성).

## 2. 검증 — 성능

| 파일 | 수정 전 | 수정 후 |
|---|---:|---:|
| 수면비행선박(0.3MB hwpx) | >400s | **3.2s** |
| 국가투명성(1.4MB hwp) | >400s | **0.9s** |
| 공정위공고(1.3MB hwp) | >400s | **1.5s** |

3파일 모두 export-svg 수 초 완료(>100× 개선). render-diff(수면비행선박)도 완주.

## 3. 검증 — 정확성 (출력 불변)

순수 함수 메모이제이션이므로 렌더 출력은 bit-identical 이어야 한다. 확인:

- **golden SVG 스냅샷(`svg_snapshot`)**: 5건 "실패" 는 전부 **CRLF 노이즈**(autocrlf=true
  로컬), `\r` 정규화 후 golden 대비 **내용 byte-동일**(표 케이스 `table-text/page-0` 포함).
  → 수정 전 golden 과 완전 동일 = 렌더 불변.
- **render-diff 자기정합**: byeolpyo1·수면비행선박 모두 구조 불일치 0페이지 PASS(캐시가
  원본/왕복 두 렌더 사이에 누출되지 않음).
- `cargo test --lib`: **2126 passed, 0 failed**.
- 중첩표 통합 테스트: `issue_1073_nested_table_split`·`issue_1133_nested_table_valign`
  ·`issue_1195_cell_table_empty_line`·`opengov_corpus_snapshot` 통과.

## 4. 회귀 가드 / 재현

- 공개 샘플: `samples/issue1949_giant_cell_nested_tables_perf.hwpx`.
- 회귀 테스트: `tests/issue_1949_giant_cell_render_perf.rs` — 전체 페이지 렌더 완주 +
  페이지 수 + 중간 페이지 콘텐츠 확인(캐시 부재 시 사실상 완료 불가 → CI 폭증 노출).

## 5. 리스크/잔여

- 포인터 키 재사용은 재조판 clear 로 차단(문서 간·편집 시). render-diff 두 문서는 각
  파싱→각 core→각 engine 이라 캐시 분리; 한 core 재사용 시에도 paginate clear 로 안전.
- 나머지 2파일(국가투명성/공정위)도 동일 캐시로 해소 — 별도 기전 아님.

## 6. 게이트 결론

성능 목표 달성(>100×) + 출력 bit-identical(golden CRLF-정규화 동일) + lib 2126 +
중첩표/render-diff 무회귀. 풀 스위트·스냅샷은 CI 확인.
