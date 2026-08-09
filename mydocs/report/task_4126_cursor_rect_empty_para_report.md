# 수정 보고서 — Task #4126 빈 호스트 문단 getCursorRect O(pages) 재렌더 제거

## 이슈

- Issue: https://github.com/edwardkim/rhwp/issues/4126
- PR: https://github.com/edwardkim/rhwp/pull/4127 (`fix/issue-4126-cursor-rect-empty-para-skip`)
- 후속: #4128 (텍스트 있는 셀 내부 질의의 같은 계열 병목 — 별도 트랙), #4129 (paginate 병목)

## 결론 (요약)

표/도형 컨트롤만 호스팅하는 빈 문단(텍스트 0자)에서 `getCursorRect` 가 후보 페이지
전부의 render tree 를 짓던 것을, **텍스트 매칭이 원리적으로 성공할 수 없는 경우 페이지
순회를 건너뛰고 앵커 폴백으로 직행**하도록 고쳤다. 115쪽 분할 표 문서의 콜드 캐럿 배치가
wasm 실측 **5,093ms → 35ms**, page tree 빌드 **116회 → ≤2회**.

## 병목 함수 (정확한 이름·경로)

| 역할 | 함수 | 위치 |
|---|---|---|
| 진입점 (JS→wasm) | `HwpDocument::get_cursor_rect` (js: `getCursorRect`) | `src/wasm_api.rs` |
| 질의 본체 · **수정 지점** | `DocumentCore::get_cursor_rect_native` | `src/document_core/queries/cursor_rect.rs` |
| 과잉 후보 생성 | `DocumentCore::find_pages_for_paragraph` | `src/document_core/commands/text_editing.rs` |
| 낭비 비용의 실체 (후보당 1회) | `DocumentCore::build_page_tree_cached` → `build_page_tree` | `src/document_core/queries/rendering.rs` |

수정은 `get_cursor_rect_native` 의 페이지 순회 진입 조건 한 곳이다 — 나머지 함수는
호출량이 줄어들 뿐 무변경.

## 증상과 진단 경로

`samples/issue1949_giant_cell_nested_tables_perf.hwp`(3×1 RowBreak 표가 PartialTable
continuation 으로 115쪽에 걸침) 를 studio 에서 열면 로드 자체는 끝났는데 스크롤이
수 초간 잠겼다. 진단은 브라우저에서 수행했다:

1. `PerformanceObserver`(longtask) — 문서 open 후 **5,427ms 단일 롱태스크** 확인.
2. 앱의 `[initDoc]` 단계 로그 타임스탬프 수집 — 블록이 8단계
   (`inputHandler.activateWithCaretPosition`)와 9단계 사이임을 특정.
3. wasm 모듈(`HwpDocument.prototype`)의 전 메서드에 타이밍 래퍼 패치 후 재현 —
   **`getCursorRect` 1회 5,093ms** 로 단일 호출 특정.

## 원인

`get_cursor_rect_native`(src/document_core/queries/cursor_rect.rs)는

1. `find_pages_for_paragraph` 로 후보 페이지를 얻고 — 분할 표 host 문단은
   para_index 만 매칭되어 **115쪽 전부**가 후보 (#4128 트랙),
2. 후보마다 `build_page_tree_cached` 로 render tree 를 지어 char_offset 텍스트 매칭을
   시도한다.

캐럿이 (0,0,0)인 문서 open 직후, 문단 0 은 표만 호스팅하는 **빈 문단**이라 텍스트 매칭이
어느 페이지에서도 성공할 수 없고, 루프는 항상 115쪽을 전부 소진한 뒤 기존 폴백(앵커
위치 산출)으로 떨어졌다. 즉 전량이 낭비 작업이었다.

## 수정

대상 문단이 **페이지 스캔으로 원리적으로 아무것도 찾을 수 없는 경우에만** 순회를
건너뛰고 앵커 폴백으로 간다: 텍스트가 비어 있고, **인라인(treat_as_char) 컨트롤과
각주/미주 마커도 없는** 문단. 폴백 산출은 종전과 동일 경로이므로 결과 좌표 불변.

첫 판(텍스트 유무만 판정)은 TAC 그림·미주 수식만 호스팅하는 빈 문단의 캐럿(그림 끝
zero-width 앵커)을 폴백으로 강등시켜 `issue_1452_saved_caret` 4건·`issue_1139` 1건을
깨뜨렸다(CI 실측, devel 기준선 green). 인라인 앵커 보유 문단을 스캔 대상으로 되돌려
해소 — 거대 표 host 문단(비인라인)은 계속 스킵되어 성능 효과 불변.

## 검증

- **red→green 회귀 테스트**: `tests/issue_4126_cursor_rect_empty_para_pages.rs` —
  시계가 아니라 작업량 카운터(`diagnostics::perf_counters::PAGE_TREE_BUILDS`,
  `build_page_tree` 비캐시 빌드 누적)로 판별. 수정 원복 실측 **116회 빌드 → FAIL**,
  수정 적용 **≤8 상한 → PASS** (release-test 1.15s).
- studio(wasm) 실측: open 후 최장 롱태스크 5,427ms → 207ms (#4129 와 합산 결과는
  task_4129 보고서 참조), `getCursorRect` 5,093ms → 35ms.
- 페이지 수 핀: 115쪽 불변 (렌더·페이지네이션 무변경 — 질의 경로만 수정).

## 남긴 것

- 후보 목록 자체가 넓은 문제(para_index-만 매칭)는 이 수정의 범위 밖 — #4128 이
  `find_pages_for_cell_position` 으로 해소 (스택 상위 레이어).
