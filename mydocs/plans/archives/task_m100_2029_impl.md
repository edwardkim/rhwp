# 구현계획서 — Task M100 #2029: layout_partial_table 해체 (라운드 6)

- 이슈: #2029 / 수행계획서: `task_m100_2029.md` / 작성일: 2026-07-07
- 1단계 실측 반영 (보정 스캐너 + **신규 보정 1건**: 중첩 필드 push `X.children.push`
  사각지대 — 본 라운드에서 발견·보정, 절차 문서화 대상).

## 1. 의존 실측 결과

### 셀 루프 (442~1698, 1,257줄)
- **muts 0 / carry-out 0** (bin_data_content=파라미터 오탐, seg=클로저 오탐 — 검증 완료).
- 외부 가변 sink 는 **`table_node.children.push` 단 하나** (+ `tree.next_id`) —
  셀-간 캐리가 로컬에 전혀 없는 순수 방출 루프. 통추출 최적 조건.
- 읽기 16종: fn 파라미터 통과군(paragraphs/styles/col_area/bin_data_content/start_cut/
  end_cut/start_row/end_row/…) + 준비부 산출 소수 — 직접 파라미터로 수용
  (fn 파라미터 통과가 대부분이라 struct 불요 판단, 컴파일러로 최종 확정).
- 내부 `!is_repeated_header_cell` 블록(569줄, reads 27/muts 2)은 셀-스코프 캐리
  (has_preceding_text/para_y)라 **통추출 시 자동 내부화** — 2차 분해는 후속.

### 준비부(356줄)/마무리(158줄)
- 추출 1 후 재실측로 결합도 확인 — 약한 쪽 1건만 추출(v2 §0 규칙 3 적용 가능).

## 2. 추출 설계

### 추출 1 (2단계) — `layout_partial_table_cells` (셀 루프 통추출)
```rust
fn layout_partial_table_cells(&self, tree: &mut PageRenderTree,
    table_node: &mut RenderNode, /* 읽기 ~16 */ ...)
```
- 본문 무변경 이동 (break/continue 자체 루프 대상 — 착수 시 중첩 스캔 재확인).
- 예상: 함수 1,773 → ~520줄, CC 169 → 대폭 감소(분기 밀도가 셀 루프에 집중).

### 추출 2 (3단계) — 준비부 or 마무리 (재실측 후 확정)

## 3. 게이트

수행계획서 §4 (OVR "회귀 0건" 기준 — 현행화 baseline). 표적: rowbreak 20/20 ·
issue_1853 · issue_1835 · golden SVG.

## 4. 절차 개선 기록

의존 스캐너 사각지대 목록 현행화: ①포맷 문자열 가짜 대입(r4) ②인덱스 대입(r4)
③클로저 파라미터/필드 리터럴 오탐(r5) ④**중첩 필드 push/대입(r6 신규)** — 재평가 문서에
체크리스트로 등재 예정.
