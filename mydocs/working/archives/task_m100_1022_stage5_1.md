# Stage 5-1 완료보고서 — #1022: VPOS_CORR 감사

- 타스크: #1022 / 브랜치 `local/task1022`
- 작성일: 2026-05-20
- 단계: Stage 5-1 — `layout.rs:2152~2475` 의 VPOS_CORR 아키텍처 정리

## 1. VPOS_CORR 개요

`layout.rs` 의 페이지/단 layout 루프(2152~2469)에서 각 PageItem 배치 전에
**HWP LINE_SEG.vertical_pos** 기반으로 y_offset 을 정정한다. 자연 누적
y_offset 이 HWP 인코딩 vpos 와 어긋날 때 vpos 우선.

### 핵심 변수

| 변수 | 의미 |
|------|------|
| `vpos_page_base` | 첫 PageItem(FullPara/PartialPara/Table) 의 first LINE_SEG.vpos. base 차감 없이 절대 위치 사용. |
| `vpos_lazy_base` | sequential y_offset 으로부터 역산. 첫 PageItem 이 신뢰 불가(Shape/PartialTable)일 때 사용. col_area.y = vpos=lazy_base 로 매핑. |
| `prev_layout_para` | 직전 배치된 paragraph 인덱스(`prev_pi`). vpos_end 계산용. |
| `prev_vpos_end` | `prev.last_seg.vpos + line_height + line_spacing`. 다음 paragraph 가 들어갈 expected vpos. |
| `curr_first_vpos` | 현재 paragraph 의 first LINE_SEG.vpos. |
| `end_y` | `col_area.y + hwpunit_to_px(vpos_end - base)` 또는 `col_anchor_y + ...` (page_path). |

### 정정 발동 조건 (모두 만족 시)

```
applied = end_y >= col_area.y
       && end_y <= col_area.y + col_area.height
       && end_y >= y_offset - 8.0       // MAX_BACKWARD_PX
       && !stale_table_host_vpos        // TopAndBottom+vert=Para Table 호스트의 stale vpos 차단
```

발동 시: `y_offset = end_y`.

### 발동 차단 게이트

- `shape_jumped` — Shape 배치로 jumped 한 직후.
- `prev_tac_seg_applied` — TAC 표 LINE_SEG 보정 직후.
- `prev_pi == curr_pi` — 동일 paragraph 내 (PartialParagraph 등).
- `prev_has_overlay_shape` — Shape/Picture(InFrontOfText/BehindText 또는 TopAndBottom+vert=Para)가 prev 에 있어 vpos 가 객체 높이 포함해 과대.
- `prev_item_was_partial_table` — prev 가 PartialTable. (vpos 가 표 셀 내부 vpos 일 수 있어 부정확.)
- `seg.vertical_pos == 0 && prev_pi > 0` — vpos reset.
- `lazy_base < 0` — 역산 실패.

### 다수 task 누적 보정

`#332/#359/#391/#412/#474/#479/#485/#537/#539/#643/#716/#874` 등 다수 회귀 케이스에서 게이트·계산식이 보강됐다. 변경 시 모두 검증 필요.

## 2. 페이지 22 사례 — VPOS_CORR 발동 실제값

`RHWP_VPOS_DEBUG=1` 실행 결과:

```
VPOS_CORR: path=lazy pi=223 prev_pi=222 ... base=1182205 y_in=645.55 end_y=659.41 applied=true   → +13.87
VPOS_CORR: path=lazy pi=225 prev_pi=224 ... base=1181165 y_in=996.36 end_y=1010.23 applied=true → +13.87
VPOS_CORR: path=lazy pi=226 prev_pi=225 ... base=1181165 y_in=1041.43 end_y=1041.43 applied=true → 0
```

페이지 22 VPOS_CORR 총 보정량 ≈ 27.74px. paginator 가 이 보정을 인지하지 못해 27.74px 만큼 적게 누적 → 자연 진행이 본문 안에 들어가 있다고 판정 → pi=226 까지 페이지 22 에 배치 → 렌더러는 보정 후 본문 초과.

산식 검증: `end_y = col_y + (vpos_end - base) * (dpi / 7200_HWPUNIT/inch) - curr_sb`.
- pi=223: col_y=105.81, vpos_end=1223725, base=1182205, scale=96/7200=1/75 → end_y = 105.81 + 553.6 = 659.41. ✓
- pi=225: end_y = 105.81 + 904.4 = 1010.22. ✓

## 3. paginator 측 누적

`MeasuredParagraph.total_height = sb + sum(lh + ls) + sa`. 항목 사이 vpos
gap 미반영.

`typeset.rs` 의 paragraph 배치(`typeset_paragraph` line 1415~):
- `current_height += fmt.total_height` (col_count==1).
- inter-item vpos gap 누락 → 누적 drift.

## 4. 정합 방향 비교

### 방향 1: paginator 가 vpos gap 가산 (선호)

각 PageItem 배치 전에:
```
vpos_gap_px = max(0, (curr.first_vpos - prev_vpos_end) * scale)
current_height += vpos_gap_px  // VPOS_CORR 의 effect mirror
```

VPOS_CORR 의 게이트 조건도 동일하게 paginator 측에서 평가. paginator 와 렌더러가 동일 정정.

장점: 의미적으로 HWP 권위(vpos) 우선. paginator 가 정확히 예측.
단점: VPOS_CORR 의 모든 게이트 · 두 base 경로(page/lazy)를 paginator 에 미러링. 작업량 큼.

### 방향 2: VPOS_CORR 비활성화

렌더러가 LINE_SEG.vpos 무시. paragraph 들이 sequential advance.

단점: 다수 task 누적 정합 회귀 위험 큼. 사실상 불가.

### 방향 3: paginator 의 partial_height 에 vpos delta 후처리 가산

가장 작은 변경: typeset.rs 의 `current_height += fmt.total_height` 줄 직전에
prev.vpos_end vs curr.first_vpos 비교해 gap 가산. VPOS_CORR 의 백워드 보정·
overlay 게이트 등 일부만 미러링. 일부 케이스만 해결.

## 5. Stage 5-2 권장

방향 1 (전체 미러링) — 의미적 정합·중장기 안정성.
방향 3 (부분 미러링) — 단기 회귀 해소 / 작업량 절약 / 일부 케이스 잔존.

먼저 방향 3 을 실험으로 적용해 페이지 22 해소 여부 + 다른 회귀 측정. 효과 좋으면 채택, 부족하면 방향 1 로 확장.

## 6. 다음 단계

Stage 5-3 (구현) — 방향 3 부터 실험.
