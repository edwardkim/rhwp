# 최종 보고서 — Task M100 #2029: 1차 리팩토링 라운드 6 (layout_partial_table 해체)

- 이슈: #2029 (계획 #1883 v2, umbrella #1582, 선행 5라운드) / 브랜치: `local/task2029`
- 기간: 2026-07-07~08 / 거버넌스: v2 전 조항 준수

## 1. 결과 요약 (공식, 스냅샷 `2026-07-08-r6/`)

| 함수 | 이전 | 이후 |
|---|---|---|
| `layout_partial_table` | **CC 169 (전체 1위) · 1,773줄** | **CC 55 · 549줄** |
| `layout_partial_table_cells` (신규) | — | CC 115 · 1,292줄 |
| 전체 최대 CC | 169 | **153** (`typeset_endnote_paragraphs`) |

**전체 1위 함수 6라운드 연속 해소** (282→104, 288→226, 234→76, 226→146, 179→6, **169→55**).
영점(288) 대비 최대 CC **−47%**. 행동 회귀 통산 **0건**
(테스트 2,938/0 · OVR 5/5 회귀 0 — rowbreak 현행화 baseline · rowbreak 핀 20/20).

## 2. 산출물

- **`layout_partial_table_cells`** — 셀 방출 루프 통추출. 실측 근거: **muts 0 / carry-out 0 /
  외부 sink = `table_node.children.push` 단일** — 6라운드 중 가장 깨끗한 추출 조건
  (상태 struct 불요, 읽기 파라미터 + `&mut` 2개).
- **축소 판단 1건** (v2 §0 규칙 3): 준비부(357)/마무리(159)는 "표 프레임 기하"의 생산-소비
  쌍(결합 19종)이라 절단 대신 유지 — `PartialTableFrame` 설계는 표 계열 2차 라운드
  (`layout_table_cells` 124·`typeset_block_table` 129 병행)로 이연.

커밋: `47ac7492`(수행계획) → `e2bbe9ca`(실측·구현계획) → `24d63d5a`(추출 1) →
`279484b4`(축소 판단) → 본 보고서. 중간에 원격 전진분(#2039/#2040 등) 동기화·재검증 수행.

## 3. 재평가

- CC>25: 86 → 87 (+1 과도기 — cells 115 진입 ↔ 원함수 169 해소).
- **의존 스캐너 사각지대 체크리스트 확정** (r4~r6 누적 5종): ①포맷 문자열 가짜 대입
  ②인덱스 대입 `v[i]=` ③클로저 파라미터/필드 리터럴 오탐 ④중첩 필드 push `X.f.push`
  ⑤**`&mut var` 인자 전달형 변이** (r6 신규 — h/v_edges 사례). 차기 라운드 분석 절차의
  표준 점검 항목.

## 4. 다음 라운드 후보

1. `typeset_endnote_paragraphs`(153, 현 1위 — 라운드 5 산물 내부 분해, EndnoteFlowState 기반)
2. `layout_composed_paragraph`(146) 잔여 (D 블록 등)
3. `typeset_block_table`(129) + `layout_table_cells`(124) — 표 계열 2차 (+PartialTableFrame)
4. `layout_partial_table_cells`(115) 내부 2차 분해

## 5. 산출물 위치

계획 `plans/task_m100_2029{,_impl}.md` / 단계 보고 `working/task_m100_2029_stage{2,3}.md` /
스냅샷 `mydocs/metrics/2026-07-08-r6/`
