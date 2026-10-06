# Stage 2 완료보고 — total_content_height 회계 재설계 (#1658 라운드3)

- 브랜치: local/task1658-vpos / 소스: `table_layout.rs` (+8/-5).

## 1. 수정 내용
INLINE 셀 경로(`table_layout.rs:2461-2510`)의 `total_content_height` 계산에서, 중첩 표
(`Control::Table`) 높이를 `text_height += nested_h` 로 **가산하던 것을 제거**한다.
- 근거(Stage 1): 가산하면 stored vpos(last_seg_end, 중첩 포함) 및 final max 의 nested_bottom 과
  double-count → total ≈ 2× → Center/Bottom offset≈0 → 상단정렬.
- 중첩 표 기여는 그대로 final `max(composed, vpos_height, nested_bottom, wrap_object_bottom)` 가 담당:
  composed 의 line_height 가 중첩을 반영하는 케이스는 composed 가, 미반영(과소) 케이스는
  nested_bottom 이 max 로 보정(#44 under-count 가드 보존).

```rust
// before: Control::Table(t) => { text_height += self.calc_nested_table_height(t, styles); }
// after:  Control::Table(_) => {}   // 가산 제거 (double-count 방지)
```

## 2. 검증 (valign 게이트 + 역산 일치)
| fixture | align | before(BUG) | after | README FIX | 판정 |
|---------|-------|-------------|-------|-----------|------|
| centered_cell_nested_table | Center | 115.7(상단) | **190.7** | 185.7 | FIX(중앙) |
| cell_vcenter_multi_nested_overcount | Center | 113.6 | **202.7** | 197.7 | FIX |
| cell_vbottom_nested_overcount | Bottom | 117.8 | **267.8** | 265.8 | FIX(하단) |
| cell_vcenter_nested_undercount | Center | 220.2 | 230.2 | 212.2 | 가드OK(중앙 유지) |
- over 3종 정상화(상단→중앙/하단), README FIX 값과 +5pt 이내 일치. valign-gate exit=0.
- under-count 가드: 245.3→218.7(nested_bottom, 기하학적으로 더 정확) 으로 +10pt 이동하나 **중앙 유지**
  (#44 회귀 아님 — 상단정렬로 떨어지지 않음, 게이트 tolerance 내).

## 3. 전 게이트 무회귀 (Stage 3 선행 확인)
| 게이트 | 결과 |
|--------|------|
| valign offset | over 3종 FIX + 가드 OK |
| 클리핑(controlset 92) | 회귀 0 |
| byeolpyo4 클리핑 | 23.5px 불변(valign 무영향) |
| 페이지수 소형 | 75/92 무회귀 |
| 페이지수 대형 | 442 무회귀 |
| lib | 2006 passed (#1488 등) |
| hwpx_roundtrip | 4 passed |

> 클리핑 무회귀 검증용 byeolpyo4 샘플은 본 PR에 포함(재현용). clipping_gate/baseline은 round-2 PR #1688 산출물이라
> 임시 반입 후 정리. round-3 변경은 `table_layout.rs` valign fix 단독.

## 4. 완료 기준 충족
- 빌드 통과 + over 3종 offset 정상(중앙/하단) + 가드 불변. ✓
- 전 통제 게이트 무회귀. ✓

## 5. 다음 (Stage 4) — 승인 요청
(A) valign over-count 해소 완결. Stage 4 = (B) block-continuation 측정 불일치(별표4 클리핑)에
동일 회계 원리 적용 가능성 분석. **승인 후 착수.**
