# Task #549 Stage 2 보고서 — fix 시도 결과 + 본질 재분석

**제목**: resolve_cell_padding fix 적용 시도 + 본질 재분석
**브랜치**: `local/task549`
**이슈**: https://github.com/edwardkim/issues/549

---

## 1. Stage 1 fix 적용 결과 — **미해결**

### 1.1 적용 변경

`src/renderer/layout/table_layout.rs:791-798` `resolve_cell_padding` aim=false 분기에
옵션 B 적용:

```rust
(c as i32) > (t as i32) && (t as i32) > 0
```

[A] 셀 (cell[2]): pad(510, 141, 510, 141), table.padding=(0, 0, 0, 0).
fix 후 resolve_cell_padding 반환값: pad_left=0, pad_right=0, pad_top=0, pad_bottom=0.

### 1.2 RED 테스트 결과 — 여전히 FAIL

```
test test_549_cell_inline_brackets_centered_p14 ... FAILED
[A] cell line 0 "[" cell-rel x=2.33 px 가 너무 좌측 (3.0 px 이상 필요).
```

[A] 셀 렌더링 위치 변경 없음 — "[" at 607.79, "A" at 614.76, "]" at 613.79 (동일).

### 1.3 변화 없는 이유

`shrink_cell_padding_for_overflow` 가 보상하여 동일 결과 산출:

```
fix 전:
  pad=510 HU (6.80 px), inner_w=9.05
  estimate "[A" ~18 px > 9.05 → shrink trigger → pad 2.32 each
  inner_x = 605.47 + 2.32 = 607.79, inner_w = 18.01
  Line 0 "[A": text == inner → x_start = inner_x = 607.79

fix 후:
  pad=0 HU, inner_w=22.65
  estimate "[A" ~18 < 22.65 → no shrink
  inner_x = 605.47, inner_w = 22.65
  Line 0 "[A": x_start = 605.47 + (22.65 - 18)/2 = 607.79
```

두 경로 모두 "[" 위치 = 607.79 으로 동일 (수학적 등가).

→ Stage 0 의 "ROOT CAUSE = resolve_cell_padding hack" 진단이 **잘못됨**.

## 2. 본질 재분석 — wrap_text_x 와 table_right 위치 불일치

### 2.1 측정값 재검증

| 항목 | 우리 SVG | PDF 한컴 2010 |
|------|---------|----------------|
| cell[2] left | 605.47 px | 96.79 pt (cell-rel 기준 동일) |
| cell[2] right | 628.12 px | 113.76 pt |
| body text "반" left | **626.22 px** | **122.27 pt** |
| gap (cell_right ↔ body) | **-1.90 px (overlap!)** | **+8.51 pt (= 850 HU = outer_margin.right) ✓** |

→ **PDF body text 가 cell_right + outer_margin.right 위치에서 시작.
우리 SVG body text 가 cell_right 보다 1.90 px LEFT 에서 시작 (1.90 px overlap).**

### 2.2 IR LINE_SEG cs 분석

pi=299 ps_id=11: margin_left=1704 indent=1984. table common margin.right=850.
모든 9 줄 LINE_SEG: cs=3455, sw=27581 (동일).

```
col_area_width (HU) = sum(cs + sw + margin.right) = 3455 + 27581 + 850 = 31886 HU
(우리 SVG col_area_width = 425.17 px = 31888 HU ≈ 31886 ✓)
```

→ IR 의 sw 는 right margin 을 차감한 값. cs 는 wrap_text_x 직접.
→ 우리 wrap_text_x = col_x + cs = col_x + 3455 HU = 626.22 px ✓ (IR 값 그대로 사용)

### 2.3 table_right 위치 검증

```
우리 rendering:
  table_left = col_x + host_margin_left + h_offset = col_x + 1758 HU
  → 실측: col_x + 1898 HU (140 HU 차이) ← 알 수 없음

  table_right = table_left + table_w = col_x + 3595 HU
  cs (body wrap start) = col_x + 3455 HU
  → table_right (3595) > cs (3455) → body 가 table 안쪽에서 시작 (overlap)

PDF (역산):
  table_right = cs - margin.right = 3455 - 850 = 2605 HU
  table_left = 2605 - 1697 = 908 HU
  → PDF 기준 table_left = col_x + 908 HU (≈ col_x + 12.11 px)
```

우리 table_left (col_x + 1898) vs PDF 추정 table_left (col_x + 908) — **990 HU 차이**.

### 2.4 진짜 ROOT CAUSE 후보

1. **table_x 계산이 잘못됨** — `compute_table_x_position` 가 host_margin_left
   를 잘못 적용하여 table 이 990 HU 우측으로 이동
2. **IR cs 가 다른 의미** — cs 가 wrap_text_x 가 아닐 수도 (확인 필요)
3. **outer_margin.right 적용 누락** — wrap_text_x 가 cs 그대로 사용,
   일부 케이스에서 margin.right 추가 누락

## 3. 결정

Stage 0 진단이 잘못됨 → resolve_cell_padding fix 는 **revert** 완료. 1122 단위
테스트 무회귀.

본질 cause 가 table_x 계산 또는 wrap_text_x 산식에 있어 Task #549 scope 가 매우
달라짐. Stage 1 RED 테스트는 유효 (visible 결함 검출).

## 4. 옵션

### 옵션 A: Task #549 재시작

- Stage 0 본질 재분석 (table_x vs wrap_text_x 불일치 원인 확정)
- Stage 1 다시 (RED 테스트 보강)
- Stage 2~3 재실행

### 옵션 B: Task #549 close, Task #550 신규 등록

- Task #549 는 "cell padding hack 분석" 결과 (현 상태로 정리)
- Task #550 신규: "[A] body text overlap with table_right (wrap_text_x 산식 정정)"

### 옵션 C: Task #549 보류, 추후 우선순위 결정 후 재개

- 본질 cause 가 layout 깊은 곳에 있어 회귀 위험 큰 fix 가 필요
- 다른 priority task 먼저 진행 후 #549 재개

## 5. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/integration_tests.rs` | RED 테스트 1건 (#[ignore]) |
| `examples/scan_aim_cells.rs` | 광범위 사전 평가 도구 |
| `examples/inspect_cell.rs` | cell IR 검증 도구 |
| `examples/find_A_pi.rs` | [A] paragraph 검색 도구 |
| `mydocs/working/task_m100_549_stage1.md` | Stage 1 보고서 |
| `mydocs/working/task_m100_549_stage2.md` | 본 보고서 |

## 6. 승인 요청

본질 진단 정정 (Stage 0 가 잘못됨) — 솔직 보고합니다.

옵션 A/B/C 중 결정 필요합니다.

권고: **옵션 B** (Task #549 → 사전 분석 결과로 close, Task #550 신규 등록) — 본질
cause 가 다르므로 issue scope 재정의 필요.
