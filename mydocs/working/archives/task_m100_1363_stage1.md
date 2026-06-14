# Stage 1 — Task #1363 모델 매핑 + 골든 베이스라인

## 1. 골든 베이스라인 (stream/devel `4574299f`, 회귀 판정 기준)

### issue_1082 미주 오버플로 총합(px)
| exam | px |
|------|-----|
| 3-09월 2023 (.hwp / .hwpx) | **0.0** / 0.0 |
| 3-11월 2022 practice | **0.0** |
| 3-09월 2022 | **0.0** |
| **3-09월 2024 below20above20 (대상)** | **50.1** |

→ 목표: 대상 50.1 → 감소(특히 p17 C×C·p22 해소), 그 외 exam **0.0 유지**.

### 시각 회귀 sweep (`task1274_visual_sweep.py`, flagged/총)
2022-09:1, 2023-09:0, 2024-below20:1, 2024-between20:1, 2022-10:0, 2022-11:1
(below20above20 타겟은 sweep 에 추가 필요 — PR #1358 에 있음). 회귀 시 flagged 증가.

### cargo test: 전체 0 failed (특히 issue_1082/1139/1274/1284).

## 2. 모델 매핑 — typeset 누적 ↔ layout 렌더

### 경로
- **typeset**: `compute_en_metrics(prev_en_bottom_vpos) → (fit, acc)` (typeset.rs ~2555).
  `acc = metric_advance_px.max(min_h)`, `metric_advance_px = (this.tb − base)` saved-vpos
  delta. `current_height += acc` (또는 split 시 line_advances_sum).
- **layout**: `HeightCursor.vpos_adjust`(height_cursor.rs:146) 가 para 첫 줄 y 산출 +
  렌더러가 format 줄높이로 순차 배치. 실제 렌더 높이 = format `line_advances_sum`.

### Divergence A — 내부 saved-vpos rewind (최대, pi=894 −61.2)
- **layout**: para 첫 줄만 vpos_adjust 로 위치, 이후 줄은 **순차 format 배치**(내부 rewind
  무시) → 렌더 높이 = line_advances_sum(99.6).
- **typeset**: `internal_vpos_rewind` 감지 → `min_h = min_vpos_rewind_height`(첫 줄),
  `metric_advance_px = advance_px`(작은 saved delta) → acc 38.4. **과소 61.2**.
- SSOT: 순차 렌더되는 rewind para 는 acc 도 line_advances_sum 써야 정합.

### Divergence B — trailing line-spacing (pi=872/874 −6)
- **layout** (height_cursor.rs:214‑222): lazy_base trailing_ls bridge. `vpos_continuous &&
  prev_has_text` 면 trailing_ls=0(이미 순차 y 포함), 아니면 prev trailing ls 가산.
- **typeset**: `fit` 은 trailing_ls drop, `acc` 는 saved delta(this.tb−prev.tb). 이 delta 가
  format 대비 trailing ls 만큼 작게 나오는 케이스(pi=872/874) → acc 과소 6.
- SSOT: trailing-ls 회계를 양쪽 동일 규칙(layout 의 vpos_continuous&&prev_has_text 게이트)으로.

### 기타 분기 (Stage 2 에서 영향 exam 식별)
typeset: `compact_local_rewind`, `capped_new_endnote_advance`, `stale_forward_vpos`,
`inline_object_formatter_overestimate`. layout: page_path vs lazy_path(vpos_page_base/
vpos_lazy_base), backward clamp, compact endnote backtrack. 각각이 어느 exam 가드에
의존하는지 Stage 2 에서 1:1 매핑.

## 3. SSOT 방향 (Stage 2 입력)
**layout 의 순차 format 렌더 높이(line_advances_sum + trailing-ls bridge 규칙)** 를 SSOT 로
삼아, typeset `acc` 가 이를 공유. Divergence A(rewind) → B(trailing-ls) 순으로 점진 이전,
매 단계 골든 베이스라인 대비 무회귀 게이트.

## 다음 (Stage 2)
공유 높이 함수 시그니처 설계 + 분기별 exam 의존도 매핑 + A/B 골든 비교 하니스.
