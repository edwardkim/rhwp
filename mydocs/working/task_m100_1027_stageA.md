# Stage A 완료보고서 — #1027: VPOS_CORR 클램프 순수 함수 추출

- 타스크: #1027 / 브랜치 `local/task1027`
- 작성일: 2026-05-20
- 단계: Stage A — 공유 측정 엔진 리팩터 1단계 (무동작 추출)

## 1. 변경

`layout.rs` VPOS_CORR 의 **보정 목표 y(end_y) 계산 + 클램프**를 순수 함수로 추출:

```rust
pub(crate) fn vpos_corrected_end_y(
    is_page_path, col_anchor_y, col_area_y, col_area_height,
    vpos_end, base, curr_sb, y_offset,
    curr_has_topbottom_para_table, dpi,
) -> (f64, bool)  // (end_y, applied)
```

- 클램프 규칙(본문 내 + 단계당 ≤8px 백워드 `MAX_BACKWARD_PX` + stale-table-host forward>100px 가드)을 함수로 캡슐화.
- 렌더러는 `curr_sb`/`curr_has_topbottom_para_table` 만 계산 후 함수 호출, `if applied { y_offset = end_y }`. **로직 동일**.
- `pub(crate)` 로 페이지네이터(typeset)도 추후 호출 가능.

## 2. 무동작 검증 (병합본 baseline 대비)

| 지표 | baseline(병합본) | Stage A | 판정 |
|------|------------------|---------|------|
| 노트 "추진일정은" | 9쪽 | 9쪽 | ✅ 동일 |
| 총 페이지 수 | 185 | 185 | ✅ 동일 |
| LAYOUT_OVERFLOW | 13 | 13 | ✅ 동일 |
| svg_snapshot | 5 pass / 3 debt(267/617/677) | 5 pass / 3 debt | ✅ 동일 |

- clippy 무경고. (참고: "12" 는 병합 전 기본경로 값, 병합본 baseline 은 13.)
- 골든 3건(267/617/677)은 병합 시 골든=theirs 로 둔 사전 부채(Stage A 무관).

## 3. 다음 (Stage B)
per-paragraph advance(조건부 spacing_before + Σ(lh+ls) + outer_margin) 계산을 동일하게 무동작 추출 → 이후 Stage C(HeightCursor)에서 vpos_corrected_end_y 와 결합.
