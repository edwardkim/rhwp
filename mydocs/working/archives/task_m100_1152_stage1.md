# Task #1152 Stage 1 — 진단 정밀화 + 인접 케이스 탐색

- 이슈: [#1152](https://github.com/edwardkim/rhwp/issues/1152)
- 브랜치: `local/task1152`
- 작성일: 2026-05-28

## 1. 진단 결과 — `is_tac` 식별 경로 확인

| 항목 | 값 |
|------|---|
| `parser/control.rs:151` | `table.attr = table.common.attr` (HWP5/HWP3 동기화) |
| `parser/control/shape.rs:338` | `common.treat_as_char = attr & 0x01 != 0` |
| → `typeset.rs:1854` | `let is_tac = table.attr & 0x01 != 0;` ← treat_as_char 로 동작 |
| pi=586 ci=1 raw common attr | `0x082A2311` (bit 0 = 1) → `is_tac = true` ✓ |
| pi=586 ci=0 raw common attr | bit 0 = 0 → `is_tac = false` (12×5 본문 표) |

→ 1×3 별첨 박스(ci=1)는 `format_table` 의 `ft.is_tac = true` 로 `typeset_tac_table()` 경로 진입.
→ 12×5 본문 표(ci=0)는 `typeset_block_table()` 경로.

## 2. `typeset_tac_table` 동작 추적

`src/renderer/typeset.rs:2234`:

```rust
let table_height = if tac_count > 1 { ... }
    else if fmt.total_height > 0.0 { fmt.height_for_fit }
    else { ft.total_height };

let available = st.available_height();
if st.current_height + table_height > available && !st.current_items.is_empty() {
    st.advance_column_or_new_page();
}
self.place_table_with_text(...);
```

`tac_count` (line 2012-2016) = `treat_as_char` 인 표 개수 = **1** (ci=1만)
→ `tac_count > 1` false → `fmt.height_for_fit` 사용.

실측 (page 32):
- body_h = 933.5 px
- PartialTable(8~12행) 배치 후 current_height ≈ 887.9 px
- 잔여 = 45.6 px
- 1×3 박스 `fmt.height_for_fit` ≤ 45.6 → fit 통과 → 같은 페이지 배치 (오류)

→ HWP 의 명시 신호 `ls[1].vpos == 0` 이 fit 검사 경로에 반영되지 않는 게 원인.

## 3. 인접 케이스 탐색 (회귀 영향 검증)

`samples/` 전수 스캔 (`/tmp/scan_intra_reset.sh`): `text_len=0 && controls=2+ && ls[i>0].vpos==0` 매칭 케이스.

| sample | pi | 비고 |
|--------|----|----|
| `2022년 국립국어원 업무계획.hwp` | pi=20 | 본 이슈와 동일 파일. `[쪽나누기]` 컨트롤 포함 — 이미 페이지 분할됨 |
| **`2022년 국립국어원 업무계획.hwp`** | **pi=586** | **본 이슈 대상** |
| `kps-ai.hwp` | pi=250 | ls[0].lh=773 px + ls[1].lh=276 px → 자연스럽게 fit 실패 → 기존 fit 검사로 advance ✓ |
| `kps-ai.hwp` | pi=254 | (유사 패턴) |
| `2025년 기부·답례품 실적...hwpx` | pi=57 | ls[0].vpos=37875, ls[1].lh=369 px → 자연 fit 실패 → 기존 advance ✓ |
| `2025년 기부·답례품...hwpx` | pi=111, 149, 158, 164 | (유사 패턴) |
| (비공개 sample A) | pi=322 | (유사 패턴) |

**핵심 관찰**: pi=586 만 "2번째 TAC 박스가 자연스럽게 fit 통과" 하는 유일 케이스. 나머지는 모두 `current_height + ls[1].lh > available` 로 기존 fit 검사가 이미 advance 를 트리거 — 우리 가드를 추가해도 결과 동일.

→ 가드 추가는 **strictly additive** (기존 자연 advance 케이스는 영향 없음, 미처리 케이스만 새로 advance).

## 4. 가드 조건 후보 (정공법)

```rust
// 호스트 문단이 empty-text + N controls + N line_segs 이고,
// 현재 컨트롤(ctrl_idx>0)의 매핑 line_seg vpos==0 (intra-paragraph reset 신호)
// → fit 검사 이전에 강제 advance.
let intra_reset = para.text.is_empty()
    && para.line_segs.len() == para.controls.len()
    && ctrl_idx > 0
    && para.line_segs.get(ctrl_idx).map(|s| s.vertical_pos).unwrap_or(-1) == 0;

if intra_reset && !st.current_items.is_empty() {
    st.advance_column_or_new_page();
}
```

위치 후보:
- (A) `typeset_tac_table()` 함수 진입부 (line 2234) — TAC 표에만 적용
- (B) `for (ctrl_idx, ctrl) in para.controls.iter().enumerate()` 루프 진입부 (line 2040) — 모든 컨트롤(TAC 포함)에 적용

본 이슈에서 advance 가 필요한 컨트롤은 ci=1 (TAC). 안전 우선으로 **(A) typeset_tac_table 진입부** 선택 제안. 비-TAC float (ci=0 같은 패턴) 도 동일 신호가 있을 수 있으나 본 이슈 범위 밖, 별도 이슈로 분리.

## 5. 회귀 방지 테스트 후보

- 신규 통합 테스트 `tests/issue_1152_intra_para_vpos_reset.rs`:
  - sample: `samples/2022년 국립국어원 업무계획.hwp`
  - assert: page 32 에 1×3 별첨 박스 (pi=586, ci=1) 없음 / page 33 에 있음.
- 또는 골든 SVG `tests/golden_svg/issue-1152/page-32.svg`, `page-33.svg`.

## 6. 결론 / 다음 단계

- Root cause 확정: TypesetEngine TAC 표 배치 시 intra-paragraph vpos-reset 미반영.
- 수정 위치 확정: `src/renderer/typeset.rs:2234 typeset_tac_table()` 진입부.
- 가드 조건 확정: 위 4 절.
- 회귀 영향: 5개 인접 케이스 모두 기존 fit 검사로 자연 advance → strictly additive 변경 예상.

→ Stage 2 (구현 계획서) 로 진행.
