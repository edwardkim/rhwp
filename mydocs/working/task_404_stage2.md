# Task #404 Stage 2 — heading-orphan trigger 구현

## 작업 내용

`src/renderer/typeset.rs::typeset_section` 메인 루프에 vpos 기반 heading-orphan 보정 로직을 추가했다. Stage 1 진단 로그(41건 중 1건만 진짜 orphan)를 기반으로 trigger 조건을 좁혀 false positive를 차단했다.

### Stage 1 코드 정리

- 진단 `eprintln!` 블록 제거
- `TypesetState.page_first_vpos` 필드 제거 (사용 안 함 — 아래 설계 변경 참고)
- `reset_for_new_page` 의 `page_first_vpos = None` reset 라인 제거

### page_top_vpos 설계 변경

처음 계획은 `TypesetState.page_first_vpos: Option<i32>` 필드에 페이지 첫 paragraph 의 vpos 를 추적하는 방식이었으나, 실제 검증 결과 다음 문제가 있었다:

- `typeset_paragraph` 내부에서 paragraph 가 fit 안 할 때 `reset_for_new_page` 호출 후 새 페이지에 배치 — 이 때 setter 가 발동되지 않음 (메인 루프의 setter 는 다음 iteration 진입 시점이라 늦음)
- 결과: page 9 의 첫 item pi=77 placement 직전이 아니라 직후 페이지가 시작되어 page_first_vpos 가 None 상태로 pi=78~83 처리

→ `current_items` 의 첫 item para_index 를 통해 매 iteration **즉시 계산**하는 방식으로 변경. 페이지 전환 직후 current_items.is_empty() 인 paragraph 자기참조 케이스는 trigger 가드(`!current_items.is_empty()`)로 회피.

### Trigger 조건 (모두 AND)

```rust
A) !st.current_items.is_empty()         // 페이지 첫 item 자기참조 방지
B) st.wrap_around_cs < 0                 // wrap-around zone 회피
   && st.col_count == 1                  // 단일 단에서만
C) current_height + para_h_px <= avail   // 현재 paragraph 가 누적 height 로 fit
D) vpos_end > page_bottom_vpos + 283     // vpos 기준 1mm 초과
E) next_h_px > 30.0                      // 다음 paragraph 가 substantial (>8mm)
   && current_height + para_h_px + next_h_px > avail   // 다음이 잔여 영역에 fit 안 함
```

발동 시 `st.advance_column_or_new_page()` → 현재 paragraph 가 다음 페이지로 push 됨.

## 검증 결과

### 타겟 샘플 — pi=83 push 확인 ✓

`samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx`:

**Stage 2 적용 전 (Stage 1)**:
- 페이지 9 (8 items, used=930.1px) — 마지막 item 이 pi=83 heading
- 페이지 10 시작 — pi=84 표 7×3, pi=85 표 3×3

**Stage 2 적용 후**:
- 페이지 9 (7 items, used=906.6px) — pi=82 표 2×1 마지막
- 페이지 10 시작 — **pi=83 heading + pi=84 표 + pi=85 표** 함께 배치

→ heading + 후속 표 분리 해소.

### 전체 페이지 수

30페이지 (변화 없음).

### 회귀 테스트 — 1073개 모두 통과 ✓

```
test result: ok. 1023 passed; 0 failed; 1 ignored  (lib)
test result: ok. 6 passed; 0 failed              (svg_snapshot)
test result: ok. 25 passed; 0 failed             (integration tables)
test result: ok. 14 passed; 0 failed             (integration paragraphs)
test result: ok. 2 passed; 0 failed              (integration gugeo)
test result: ok. 1 passed; 0 failed              (tab_cross_run)
... 그 외 모두 ok
```

### 10개 대표 샘플 — LAYOUT_OVERFLOW 카운트 비교

| 샘플 | Stage 1 | Stage 2 | 차이 |
|------|---------|---------|------|
| **2025년 기부·답례품 (타겟)** | **57** | **42** | **-15 ↓** |
| 2022 국립국어원 업무계획 | 0 | 0 | 0 |
| aift | 3 | 3 | 0 |
| exam_eng | 0 | 0 | 0 |
| biz_plan | 0 | 0 | 0 |
| 21_언어_기출 | 14 | 14 | 0 |
| **kps-ai** | **5** | **4** | **-1 ↓** |
| k-water-rfp | 0 | 0 | 0 |
| 20250130-hongbo | 0 | 0 | 0 |

**회귀 없음 + 2개 샘플 개선** (orphan paragraph 가 적절한 페이지로 이동하면서 overflow 감소).

## Trigger 발동 사례 (사이드 효과)

trigger 가 발동한 paragraph (Stage 1 로그 + Stage 2 동작 비교):

- **pi=22**: 3×3 wrap=TopAndBottom 표. 원래 line_segs sum=23.5px (placeholder) 으로 trigger 조건 매칭. 그러나 이 paragraph 는 실제 height 924.5px 라 어차피 page 1 에 fit 불가 → 페이지 2 push 가 정상 동작과 동일. 회귀 없음.
- **pi=83**: heading 14.7px. 본 task 의 의도된 push.
- 그 외 trigger 미발동 (조건 E 의 next_substantial 또는 next_doesnt_fit 필터로 false positive 차단).

## 검증 기준 충족

수행계획서(`task_404.md`) §검증 기준:

1. ✅ 페이지 9 SVG 에 pi=83 heading 미표시 (페이지 10 으로 이동)
2. ✅ 페이지 10 SVG 가 pi=83 heading + pi=84/85 표 함께 표시
3. ✅ 기존 회귀 테스트 모두 통과 (1073개)
4. ✅ 10개 대표 샘플 LAYOUT_OVERFLOW 회귀 없음 (+2 샘플 개선)

## 산출물

- 코드 변경: `src/renderer/typeset.rs` (heading-orphan trigger 블록)
  - `+33 / -15 line` (trigger 추가 + Stage 1 진단/필드 제거)
- 보고서: `mydocs/working/task_404_stage2.md` (본 문서)

## 다음 단계

Stage 3 — SVG 시각 검증 + 최종 결과 보고서 작성.
