# #1949 최종 결과보고서 — 거대 셀 렌더 O(pages×cell) 재계산 제거

- 이슈: edwardkim/rhwp#1949 (M100)
- 브랜치: `local/task1949`
- 유형: 성능(렌더 재계산 핫스팟)
- 결과: **수정 완료, 출력 불변 + >100× 성능 개선**

## 1. 문제

20k 서베이의 `render-diff` 420초 타임아웃 3건(0.3~1.4MB 소형 문서 포함).

## 2. 근본 원인 (프로파일링)

- 단계 분해: 파싱 즉시, 페이지네이션 19.5s(정상), **렌더 >380s 지배**.
- 문서 구조: 바깥 **3×1 RowBreak 표**의 셀[2] = **2507문단 + 중첩표 수십 개**,
  **112쪽 전부에 렌더**.
- 핫스팟: 렌더가 각 페이지에서 셀 콘텐츠 유닛(`cell_units`)을 재계산
  (`cell_units_fitting_height`·`cell_line_ranges_from_cut` 등 컷 판정 10경로가 호출).
  셀 폭·콘텐츠는 페이지 간 불변인데도 매 페이지 재계산 → **O(pages × cell) = 112 × 2507**.

## 3. 수정

`cell_units`(순수 함수)를 **셀 포인터 키로 메모이즈**:
- `LayoutEngine`(문서 렌더 라이프사이클 지속)에
  `cell_units_cache: RefCell<HashMap<usize, Rc<Vec<CellUnit>>>>` 추가.
- `cell_units` → 메모이즈 래퍼(본체 `cell_units_uncached`). 키 = `cell as *const Cell as usize`.
  반환 `Rc<Vec<CellUnit>>`(10 호출부 Deref 로 무변경 컴파일).
- 재조판 경계 `invalidate_page_tree_cache()` 에서 `clear_layout_caches()` — 다른 IR 포인터
  재사용 방지.

`composed_paras`(드로잉) 캐시는 불필요(cell_units 캐시만으로 목표 달성).

## 4. 검증

### 성능
| 파일 | 전 | 후 |
|---|---:|---:|
| 수면비행선박(0.3MB hwpx) | >400s | **3.2s** |
| 국가투명성(1.4MB hwp) | >400s | **0.9s** |
| 공정위공고(1.3MB hwp) | >400s | **1.5s** |

### 정확성 (출력 불변 — 순수 함수 캐시)
- **golden SVG(`svg_snapshot`)**: 5건 실패는 CRLF 노이즈(autocrlf=true), `\r` 정규화 후
  golden 대비 **byte-동일**(표 케이스 `table-text/page-0` 포함) → 렌더 불변.
- **render-diff 자기정합**: byeolpyo1·수면비행선박 구조 불일치 0페이지 PASS(캐시 무누출).
- `cargo test --lib`: **2126 passed**. 중첩표 통합 테스트(`issue_1073`/`1133`/`1195`
  ·`opengov_corpus_snapshot`) 통과.

## 5. 산출물

- 계획: `mydocs/plans/task_m100_1949.md`
- 단계 보고: `mydocs/working/task_m100_1949_stage1~3.md`
- 소스: `src/renderer/layout.rs`, `src/renderer/layout/table_layout.rs`,
  `src/document_core/queries/rendering.rs`
- 회귀 가드: `tests/issue_1949_giant_cell_render_perf.rs`
- 재현 샘플(공개): `samples/issue1949_giant_cell_nested_tables_perf.hwpx`

## 6. 잔여

- 3파일 모두 동일 캐시로 해소(별도 기전 아님). 그 외 거대 셀 문서 전반에 동일 효과 기대.
