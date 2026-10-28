# Stage 1 완료보고 — 측정 정합 진단 (#1658 라운드3)

- 브랜치: local/task1658-vpos / 성격: 진단(소스 무변경, env-gated 디버그 계측 후 제거).

## 1. 렌더 경로 확정
valign offset 코드는 3곳(embedded/partial/inline). RHWP_VALIGN_DBG 계측 결과 합성 fixture(자리차지
표)는 **INLINE 경로 `table_layout.rs:2461-2588`** 를 탄다(embedded/partial 미발동).

## 2. 3개 항 + total 계측 (over 3종 + 가드 1종)
| fixture | align | stored | composed | nested_bottom | **total** | inner_h | offset | 결과 |
|---------|-------|--------|----------|---------------|-----------|---------|--------|------|
| centered_cell_nested_table | Center | 226.7 | 234.7 | 218.7 | **434.7** | 440.2 | 5.6 | BUG(상단) |
| cell_vcenter_multi_nested_overcount | Center | 274.7 | 282.7 | 266.7 | **522.7** | 520.2 | 0.0 | BUG |
| cell_vbottom_nested_overcount | Bottom | 226.7 | 234.7 | 218.7 | **434.7** | 440.2 | 5.6 | BUG |
| cell_vcenter_nested_undercount | Center | 37.3 | 45.3 | **218.7** | 245.3 | 529.6 | 284.2 | OK(중앙) |

## 3. 근인 확정
- **over-count = 중첩 표 높이 double-count**: INLINE 경로(`table_layout.rs:2461-2510`)는
  `text_height = calc_composed_paras_content_height(...) + Σ nested_table_height`(line 2504
  `text_height += nested_h`)로 **가산**한다. 그러나 stored `last_seg_end`(226.7)는 HWP 저장 레이아웃상
  **이미 중첩 표를 포함한 콘텐츠 바닥**이다. 가산 결과 total=434.7 ≈ 2× → `inner_h` 근접 → Center/Bottom
  offset≈0 → 상단정렬.
- **under-count 가드(#44 방향)**: stored=37.3 가 중첩 표를 미포함(과소)이라, 가산이 nested 를 반영해
  245.3 ≈ 정상 중앙을 만든다. 즉 **stored 단독 권위화는 이 케이스를 깨뜨린다**(#44 회귀).

## 4. Stage 2 회계식 (도출)
가산을 제거하고 **stored 권위 + nested_bottom 보정의 max** 로:
```
total_content_height = max(stored last_seg_end_px, nested_bottom)   // (+ 기존 max_inline 등 비-중첩 항 유지)
```
검증(역산):
- over-count: max(226.7, 218.7)=226.7 → offset (440.2−226.7)/2=106.8 → **중앙** (BUG 434.7 대비 정상화).
- under-count 가드: max(37.3, 218.7)=218.7 → offset (529.6−218.7)/2=155.5 → **중앙 유지**(#44 보존).
- vbottom: max(226.7,218.7)=226.7 → offset 213.5(하단) → **하단** 정상화.

단, INLINE 경로의 `total_content_height` 블록은 비-중첩 컨트롤(그림/도형 treat_as_char, max_inline)과
vpos_height 보정도 포함하므로, 가산 분기만 정밀 교체하고 나머지 항은 보존한다(Stage 2 구현 시 정밀).

## 5. 완료 기준 충족
- over 원인 항(중첩 표 가산 double-count) + #44 보정 항(nested_bottom) 정량 확정. ✓
- Stage 2 회계식 도출 + 4 fixture 역산 검증. ✓
- 소스 무변경(디버그 제거, git clean). ✓

## 6. 다음 (Stage 2) — 승인 요청
`table_layout.rs:2461-2510` 의 `text_height += nested_h` 가산을 stored/nested_bottom max 회계로 교체.
6게이트(valign/클리핑/소형/대형/lib/roundtrip) 동시 통제. **승인 후 착수.**
