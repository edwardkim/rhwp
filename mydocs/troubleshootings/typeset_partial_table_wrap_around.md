# TypesetEngine PartialTable + Square wrap 처리 누락 — 트러블슈팅 (Task #362)

## 증상

`samples/kps-ai.hwp` 의 다양한 결함 — v0.7.3 (Paginator) 에서는 정상이었으나 v0.7.6 (TypesetEngine default) 에서 회귀:

1. **p56 외부 표 안 콘텐츠 클립** — Task #347 의 vpos 적용 로직이 nested table 셀에 부적절
2. **p67 PartialTable nested 표 누락** — 한 페이지보다 큰 nested table 이 atomic 미루기로 표시 안됨
3. **p68 외곽 표 height 결함** — PartialTable 잔여 행 높이 계산 결함
4. **p68-70 빈 페이지 2개 발생** — 외부 표 옆 76 빈 paragraph 가 표시되어 빈 페이지 발생
5. **p72 표 누락** — 빈 텍스트 + 표 컨트롤 paragraph 가 잘못 skip

## 원인 — 핵심 요약

### 1. Task #347 의 vpos 보정이 nested table 케이스에 과적용

`src/renderer/layout/table_layout.rs:1287-` 의 `text_y_start = cell_y + pad_top + vpos` 가 nested table 셀에서 콘텐츠를 외부 셀 끝 너머로 밀어내어 클립 발생.

### 2. PartialTable nested 처리 결함 (Task #324 stage2 v2 도입)

`compute_cell_line_ranges` 의 `exceeds_limit` 시 atomic 미루기가 한 페이지보다 큰 nested table 케이스에 부적절. `bigger_than_page` 예외 필요.

### 3. wrap-around (Square wrap 어울림) 메커니즘 미이식 — 가장 큰 결함

Paginator (`engine.rs:288-372`) 의 wrap-around 매커니즘이 TypesetEngine 에 **전혀 이식 안됨**.

HWP 의 어울림(Square wrap) 표는 표 옆에 paragraph 들을 배치 — 같은 cs/sw 를 가진 LineSeg 들은 표 옆에 표시되어 height 소비 안 함. Paginator 는 이 paragraph 들을 `WrapAroundPara` 로 흡수해 height 0 처리. TypesetEngine 은 이 매커니즘 누락으로 모든 paragraph 를 정상 height 로 누적 → 빈 페이지 발생.

### 4. Task #359 의 가드 조건 불충분

빈 텍스트 (`text.is_empty()`) 만 체크하던 skip 가드가 표/도형 컨트롤 보유 paragraph 도 skip 시킴.

## 해결

### TypesetState 확장
```rust
wrap_around_cs: i32,           // -1 = 비활성
wrap_around_sw: i32,
wrap_around_table_para: usize,
current_column_wrap_around_paras: Vec<WrapAroundPara>,
```

### typeset_section 의 paragraph 처리 루프
1. **wrap zone 매칭 검사** (typeset_paragraph 호출 전):
   ```rust
   if st.wrap_around_cs >= 0 && !has_table {
       // cs/sw 매칭 또는 sw=0 어울림 매칭
       if matched {
           st.current_column_wrap_around_paras.push(WrapAroundPara { ... });
           continue;
       } else {
           st.wrap_around_cs = -1;
       }
   }
   ```
2. **Square wrap 표 직후 활성화**:
   ```rust
   if has_non_tac_table && is_square_wrap {
       st.wrap_around_cs = first_seg.column_start;
       st.wrap_around_sw = first_seg.segment_width;
   }
   ```

### vpos-reset 가드 wrap zone 안에서 무시
```rust
if para_idx > 0 && !st.current_items.is_empty() && st.wrap_around_cs < 0 {
    // ... vpos-reset 가드
}
```

### 빈 paragraph skip 가드 강화
```rust
let is_empty_no_ctrl = para.text.is_empty() && para.controls.is_empty();
if is_empty_no_ctrl { continue; } else { ... }
```

## 재발 방지 체크리스트

### 새 페이지네이션 엔진 도입 시
- [ ] Paginator 의 모든 시멘틱 (vpos-reset, hide_empty_line, **wrap-around**, page_num, NewNumber 등) 점검
- [ ] 특히 `WrapAroundPara` 메커니즘은 **TypesetEngine 에 누락되기 쉬움** — Square wrap 표가 있는 모든 샘플 (kps-ai 등) 회귀 검증 필수
- [ ] `PaginationResult.wrap_around_paras` 가 비어있지 않은 케이스 확인
- [ ] `RHWP_USE_PAGINATOR=1` fallback 과 페이지 분할 비교

### PartialTable nested 처리 변경 시
- [ ] 한 페이지보다 큰 nested table 케이스 (`para_h > content_limit`) 분할 허용 확인
- [ ] `calc_visible_content_height_from_ranges_with_offset` 의 nested 잔여 높이 계산 정확성

### 셀 vpos 적용 변경 시
- [ ] nested table 보유 셀에서 vpos 적용 제외 (`has_nested_table` 분기)

## 진단 도구

- `RHWP_USE_PAGINATOR=1` — 옛 Paginator 로 fallback (회귀 비교 기준)
- `dump-pages -p N` — 특정 페이지의 items / used / hwp_used / pi 비교
- SVG diff — clipPath, line y 좌표 비교

## 관련 task

- Task #362 (본 task)
- Task #313 (TypesetEngine default 전환 — 회귀 도입)
- Task #324 (compute_cell_line_ranges 재작성 — atomic 처리 도입)
- Task #347 (셀 vpos 적용)
- Task #359 (typeset fit drift + 빈 paragraph skip 가드)
- Task #361 (page_num + PartialTable fit)
