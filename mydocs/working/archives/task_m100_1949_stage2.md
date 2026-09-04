# #1949 Stage 2 — 메모이제이션 설계

- 브랜치: `local/task1949`

## 1. 핫스팟 정밀화 (Stage 1 보강)

거대 셀이 112쪽 전부에 렌더될 때, **한 셀당 페이지마다 반복되는 순수 계산** 3종:

| # | 위치 | 내용 | 페이지당 비용 |
|---|---|---|---|
| A | `table_partial.rs:581-615` | `compose_paragraph`×전체문단 + `recompose_for_cell_width`×전체 (드로잉) | O(cell 문단) |
| B | `cell_units_fitting_height`/`cell_line_ranges_from_cut` → `cell_units`(table_layout.rs:4526) | 컷 유닛 산출(전체 문단 조판·높이) | O(cell 문단) |
| C | `row_cut_content_height`(667/680) | 온전 행 컷 높이 | O(cell 문단) |

셀 폭(`inner_width`)·콘텐츠는 페이지 간 **불변** → A/B/C 결과는 매 페이지 동일한
**순수 중복 계산**. 112쪽 × 2507문단 = ~28만 조판.

## 2. 설계 — 셀 단위 순수 함수 메모이제이션

`LayoutEngine`(interior-mutable, 렌더 라이프사이클 지속)에 셀 단위 캐시를 추가한다.

### 2-1. 캐시 키 — 셀 포인터
`(cell as *const Cell as usize)`. 렌더 1회 동안 IR 셀 객체는 이동/변경되지 않으므로
포인터가 셀(중첩 셀 포함)을 유일 식별한다. `(para,ctrl,cell_idx)` 다중 인덱스보다
중첩 표까지 모호성 없이 안전. `inner_width`는 셀에서 파생(불변)이라 키에 불필요.

### 2-2. 캐시 대상 (우선순위)
1. **`cell_units`**(B) — 가장 자주 호출(컷 판정 2경로). 순수 `Vec<CellUnit>` →
   `Rc<Vec<CellUnit>>` 로 캐시, 재사용 시 O(1) Rc clone. `cell_units_fitting_height`
   ·`cell_line_ranges_from_cut` 가 공유.
2. **`composed_paras`**(A) — 드로잉 경로. compose+recompose 결과를 셀 키로 캐시.
   드로잉이 소비 중 변형하면 clone(memcpy, 재조판보다 저렴), 아니면 `Rc` 참조.
3. **`row_cut_content_height`**(C) — 필요 시 (cell,row) 키 캐시(경미).

### 2-3. 라이프사이클/무효화
- 캐시는 `RefCell<HashMap<usize, …>>` 필드. **섹션/문서 렌더 시작 시 clear**(포인터
  재사용 방지 — 다음 문서에서 다른 IR이 같은 주소 가질 수 있음).
- IR 은 렌더 중 불변이므로 무효화는 렌더 경계에서만.

## 3. Stage 3 구현 순서 (점진, 각 단계 측정)

1. `cell_units` 캐시(1) 적용 → 3파일 시간 측정. (컷 경로가 가장 반복적이라 최대 효과 기대)
2. 부족 시 `composed_paras` 캐시(2) 추가.
3. 목표: 3파일 render 수 초 이내(한컴 수 초 정합). O(pages×cell) → O(cell + pages).

## 4. 불변성 보장 (핵심)

- **render-tree 좌표 before/after 완전 동일**을 게이트로 사용(`export-render-tree` diff).
  순수 함수 캐시이므로 결과 bit-identical 이어야 한다.
- `cargo test --lib` + 표/중첩표/분할표 통합 테스트(issue_1073 등) 무회귀.
- 대표 3파일 외 표 다수 문서 표본 회귀(시각/페이지 불변) 확인.

## 5. 리스크

- **포인터 키 재사용**: 문서 간 캐시 미clear 시 오재사용 → 렌더 경계 clear 필수.
- **메모리**: 거대 셀 units/composed 캐시 상주(수 MB). 렌더 종료 시 해제되므로 허용.
- 캐시 대상 함수가 self 상태에 의존(순수 아님)하면 오재사용 → `cell_units`/compose 는
  순수 확인됨(dpi·padding·styles 만 의존). 신규 캐시 대상 추가 시 순수성 재확인.

→ Stage 3(구현 + 게이트) 진행 승인 요청. `cell_units` 캐시부터 점진 적용·측정하겠습니다.
