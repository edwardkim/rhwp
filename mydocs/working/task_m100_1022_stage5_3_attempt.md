# Stage 5-3 시도 보고서 — #1022: 방향 3 (paginator VPOS gap) 적용 실패

- 타스크: #1022 / 브랜치 `local/task1022`
- 작성일: 2026-05-20
- 단계: Stage 5-3 — 방향 3 (단순 vpos gap 가산) 실험

## 1. 적용 내용

`typeset.rs` 의 paragraph 처리 for-loop 직전에 vpos gap 가산:

```
gap_hu = curr.line_segs.first().vpos - prev.line_segs.last().(vpos+lh+ls)
if gap_hu > 0 && gap_hu < ~100px: current_height += gap_hu_to_px
```

게이트: 직전 항목이 PartialTable / Shape 아닐 때만. 페이지/단 advance 시 리셋.

## 2. 측정 결과

| 지표 | 값 |
|------|-----|
| LAYOUT_OVERFLOW | 38 → **37** (1건 감소) |
| 페이지 22 18.3px | **변경 없음** |
| 페이지 수 | 184 → **190** (6 페이지 증가) |
| 테스트 회귀 | `test_task76_multi_001_group_images` FAIL |

## 3. 원인 분석 — 단순 vpos gap 가산은 VPOS_CORR 와 다르다

페이지 22 pi=223 의 VPOS_CORR 가 `+13.87px` 추가하지만 단순 gap 계산은
`+0px`:

- 단순 gap: `curr.first_vpos(1223725) - prev.vpos_end(1223725) = 0`.
- VPOS_CORR end_y: `col_y + (vpos_end - lazy_base) / scale = 105.81 + (1223725 - 1182205)/75 = 659.41`. 자연 y_offset 645.55 와 차 13.87.

차이의 원인: VPOS_CORR 의 `lazy_base` 산식에 **prev 의 trailing_ls_hu 가 추가**된다 (`layout.rs:2374`):
```
y_delta_hu = (y_offset - col_area.y) / dpi * 7200 + trailing_ls_hu
lazy_base = prev_vpos_end - y_delta_hu
```

즉 `lazy_base = prev.first_vpos - y_delta_natural(prev) - trailing_ls`. 그 결과 후속 vpos 변환 시 trailing_ls 만큼 더 큰 y 가 산출. paragraph_layout 이 trailing_ls 를 advance 에 포함하는데도(Task #537 주석과 모순) lazy_base 에 trailing_ls 보정이 들어가 있어 결과적으로 +trailing_ls 효과.

이 비대칭은 paragraph_layout 의 다양한 분기(셀 컨텍스트·wrap zone·줄 skip)와 VPOS_CORR 의 trailing_ls 보정 사이 long-standing 미정합으로 보인다.

## 4. 결론

방향 3 (단순 gap 가산)은:
- 페이지 22 미해결 (gap=0 인 케이스에서 VPOS_CORR 가 trailing_ls 보정).
- 다른 페이지에서 작은 효과 (gap≠0 케이스 정합) 1건 감소.
- 페이지 수 6 증가 + 테스트 회귀 → 회귀 위험 큼.

방향 1 (lazy_base 포함 전체 미러링)이 본질적이나, 다수 task 누적 보정(#332/#412/#537 등)을 paginator 에 정확히 미러링해야 함 — 작업량 multi-day 이상 + 회귀 위험.

방향 2 (VPOS_CORR 비활성화)는 다수 문서 회귀 위험 큼.

## 5. 정정 권고

방향 3 실험 결과로 **paragraph 위치 정합은 본 #1022 의 명시 범위에서
처리 곤란** 함이 확인됨. 본 타스크는 명시 범위(`HeightMeasurer ↔
cell_units`) 완료(Stage 3, 42→38 events) 로 마무리하고, VPOS_CORR ↔
paginator 정합을 **별도 후속 타스크**로 분리하는 것이 안전하다.

후속 타스크에서 다룰 사항:
- VPOS_CORR 의 `lazy_base` 산식 의도 재확인 (trailing_ls 보정이 의도된 것인지, paragraph_layout 변경으로 인한 stale 인지).
- `paragraph_layout` 의 모든 advance 경로 감사 — trailing_ls 가산 실제 정책.
- 두 방향 비대칭 해소 후 paginator 미러링.
- 광범위 골든 회귀 점검.

multi-day 작업이며 본 #1022 보다 깊은 paragraph layout 정합 작업이다.
