# 최종 보고서 — #1658 통합 content-height(vpos) 회계 (라운드 3) / valign over-count

- 마일스톤: M100 / 브랜치: local/task1658-vpos (upstream/devel `30931679`) / 이슈: edwardkim/rhwp#1658
- 일자: 2026-06-30 / 성격: 다세션 연구·개선 **라운드 3 완결**.
- 선행: 라운드1 PR #1670(거대 셀 cut), 라운드2 PR #1688(continuation ≤3 + COM/클리핑 인프라).
- 협업: 외부 기여자 **@kkyu8925** 의 valign over-count 제보 + 합성 fixture 4종 제공.

## 1. 해결한 것 — valign over-count (소스: table_layout.rs +8/-5)
`has_nested` 셀의 세로정렬(Center/Bottom)이 상단으로 깨지던 결함을 해소.
- **근인**(Stage 1 계측): INLINE 셀 경로 `table_layout.rs:2461-2510` 가 중첩 표 높이를
  `text_height += nested_h` 로 **가산**한다. stored `last_seg_end`(중첩 포함) 및 final max 의
  `nested_bottom` 과 **double-count** → `total_content_height ≈ 2×` → `(inner_height - total)/2 ≈ 0`
  → 상단정렬.
- **수정**(Stage 2): 가산 제거(`Control::Table(_) => {}`). 중첩 표 기여는 final
  `max(composed, vpos_height, nested_bottom, wrap_object_bottom)` 가 담당 — composed 의 line_height
  가 중첩을 반영하는 케이스는 composed 가, 미반영(과소) 케이스는 nested_bottom 이 max 로 보정
  (**#44 under-count 가드 보존**).

### 검증 (합성 fixture, @kkyu8925 제공)
| fixture | align | before(BUG) | after | README FIX |
|---------|-------|-------------|-------|-----------|
| centered_cell_nested_table | Center | 115.7(상단) | **190.7(중앙)** | 185.7 |
| cell_vcenter_multi_nested_overcount | Center | 113.6 | **202.7(중앙)** | 197.7 |
| cell_vbottom_nested_overcount | Bottom | 117.8 | **267.8(하단)** | 265.8 |
| cell_vcenter_nested_undercount(#44 가드) | Center | 220.2 | 230.2 | 212.2(중앙 유지) |
over 3종 정상화(README FIX ±5pt), 가드 중앙 유지(상단 미회귀).

## 2. 통제 게이트 무회귀 (전부 통과)
| 게이트 | 결과 |
|--------|------|
| valign offset(신규) | over 3종 FIX + 가드 OK (exit 0) |
| 페이지수 소형(controlset 92) | 75 |
| 페이지수 대형(랜덤 452) | 442 |
| lib | 2006 passed (#1488 등) |
| hwpx_roundtrip | 4 passed |
| 클리핑(controlset 92)¹ | 회귀 0 (Stage 2 검증) |
¹ 클리핑 인프라는 round-2 PR #1688 산출물 — 검증 시 임시 반입.

## 3. 인프라 (재사용 자산)
- `samples/valign_fixtures/`(4 hwpx + README, @kkyu8925 제공, 실문서 비포함 합성).
- `tools/valign_offset_gate.py` — CENTERME 마커 offset BUG↔FIX 판정 회귀 게이트(상시 편입).

## 4. 분리한 것 — (B) block-continuation 측정 불일치 (round-4)
Stage 4 분석: (A) valign over-count 와 (B) byeolpyo4 클리핑은 **통합 vpos 회계 원리의 반대 방향**
위반(over vs under, INLINE vs 블록 경로) → Stage 2 수정 그대로 적용 불가. 독립성 확정(Stage 2
적용/미적용 모두 byeolpyo4 28쪽/23.5px 불변). (B)는 고위험(`test_block_cut_rowspan_giant_split`
계약 위반, typeset 정확 함수 미-pinpoint, PR #1688 인프라 의존) → **round-4 전용 작업으로 분리**
(선행조건: #1688 머지 → 함수 pinpoint → 계약 한글-검증 갱신 → 6게이트).

## 5. 결론
- @kkyu8925 제보 valign over-count 를 **데이터 정합으로 해소**(합성 fixture 회귀 게이트 동반).
- #44(under-count) 가드 보존, 전 통제 게이트 무회귀.
- (B) byeolpyo4 클리핑은 반대방향·고위험으로 **안전 분리**(round-4). **#1658 존속.**

## 6. 산출물
- 소스: `src/renderer/layout/table_layout.rs`(+8/-5)
- 인프라: `samples/valign_fixtures/`, `tools/valign_offset_gate.py`
- 문서: `plans/task_m100_1658_v2{,_impl}.md`, `working/task_m100_1658_v2_stage{1,2,4}.md`, 본 보고서
