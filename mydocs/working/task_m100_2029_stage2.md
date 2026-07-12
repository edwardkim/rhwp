# 단계 완료 보고 — Task M100 #2029 2단계: 추출 1 (셀 방출 루프)

- 작성일: 2026-07-08 / 브랜치: `local/task2029`

## 수행 내용

`layout_partial_table`의 셀 루프(1,257줄)를 `layout_partial_table_cells`로 통추출
(동작 불변, 본문 무변경 이동 — 라벨/return/`?` 0 사전 확인).

- 실측대로 셀-간 캐리 0 — 상태 struct 불요, 읽기 파라미터 + `table_node`/`tree` `&mut`.
- 컴파일러 확정 보정: 준비부 산출 읽기 14종 파라미터화(row/col 기하·edge 구조·헤더 행 등),
  **h/v_edges는 `&mut`** — 루프가 헬퍼에 `&mut`로 전달(변이)함을 컴파일러가 검출.
  → **스캐너 사각지대 5호 확정: `&mut var` 인자 전달형 변이** (체크리스트 등재).
- 이동 여파 1건 정정: 문자열 replace가 마무리부 `render_edge_borders` 호출의 borrow까지
  바꾼 것을 clippy가 검출(unnecessary_mut) → 원복.

## 게이트 결과 (전수 통과)

| 게이트 | 결과 |
|---|---|
| cargo fmt / clippy | 통과 / **0** (원복 후) |
| cargo test --tests | **2,929 통과 / 실패 0** |
| OVR baseline 5샘플 | **회귀 0건** — rowbreak는 현행화 baseline(a05e6f1b) 기준 첫 클린 판정 |
| 표적: rowbreak 계열 핀 | **20/20** |

## 계측

| 함수 | 이전 | 이후 |
|---|---|---|
| `layout_partial_table` | 1,773줄 · 분기 300 | **549줄 · 96** |
| `layout_partial_table_cells` (신규) | — | 1,292줄 · 204 |

## 다음 단계

3단계 — 준비부(356줄)/마무리(158줄) 재실측 후 결합 약한 쪽 추출 또는 축소(v2 §0 규칙 3).
