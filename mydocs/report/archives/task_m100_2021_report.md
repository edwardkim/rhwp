# 최종 보고서 — Task M100 #2021: 대형 표 셀 입력 후 caret 좌표 계산 지연

- 이슈: #2021 (jangster77 계측·메인테이너 이관) / 브랜치: `local/task2021`
- 기간: 2026-07-07 / 결과: **후반 페이지 셀 입력 3,064ms → 33ms (93×), long task 소멸**

## 1. 원인 (계측 확정)

`getCursorRectByPath`가 후보 페이지(거대 표는 115쪽 전부)를 **page 0부터 선형 재빌드**
하며 캐럿 탐색 — 입력 직후 `page_tree_cache` 전체 무효화 상태라 비용이 **캐럿 페이지
인덱스에 선형 비례**. 이슈의 621ms는 중간 페이지 케이스였고 후반 페이지는 3초까지 악화
(#2010/#2012의 지연 페이지네이션 완화와 별개 축).

- 네이티브: 페이지 트리 1장 빌드 ~45ms / CDP: p0 셀 33ms ↔ p~92 셀 3,064ms.

## 2. 수정 — 캐럿 페이지 힌트 우선 탐색 (수정 A)

- Rust `order_pages_by_hint`(hint, hint±1 우선) + `get_cursor_rect_by_path_with_hint`
  — **좌표 값 불변**(순서만), 오힌트 시 종전 전체 탐색 fallback.
- wasm `getCursorRectByPathNear` 신설 (기존 시그니처 무변경 — 공개 계약 추가만).
- Studio `CursorState.updateRect()`: 직전 rect의 `pageIndex` 힌트 전달 (구버전 폴백 포함).
- **범위 축소(승인)**: 계획의 수정 B(낙관적 caret)는 수용 기준 충족으로 생략 —
  필요 시 후속 이슈.

## 3. 검증

| 게이트 | 결과 |
|---|---|
| 동등성 핀 (`issue_2021_hint_search_equivalence`) | 셀 3종 × 무힌트/정힌트/오힌트 좌표 완전 일치 |
| Rust `--tests` / lib | **2,925/0** / 2,144/0 |
| OVR baseline 5샘플 | 추가 변동 0 (렌더 무변경 확인) |
| Studio tsc / build / `npm test` | 0 에러 / ✓ / **171/0** |
| CDP 실계측 (전/후) | 3,064ms·long task 3,067 → **32.9ms·long task 0** |
| 수용 기준(이슈) | long task 해소 ✓ · caret 자연 이동 ✓ · pagination flush 정책 무변경 ✓ |

## 4. 산출물

- 커밋: `0e88743d`(수행계획) → `254f8ed7`(계측·프로브·구현계획) → `a505e315`(수정 A).
- 계측 자산(재사용 가능): 네이티브 프로브(`probe_2021_…`, #[ignore]) +
  CDP 프로브(`rhwp-studio/e2e/issue-2021-probe.mjs`).
- 관련 문서: `plans/task_m100_2021{,_impl}.md`, `working/task_m100_2021_stage2.md`.
