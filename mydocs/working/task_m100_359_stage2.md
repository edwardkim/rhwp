# Task #359 Stage 2 — 원인 분석 + 수정 방향 확정

## Stage 1 의 정량 결과 (요약)

`samples/k-water-rfp.hwp` page 3 (section=1, page_num=1):
- col_height (가용): 915.5px
- pagination used: 915.5px
- hwp_used (LINE_SEG.vpos 기준): 1226.7px
- **drift: -311.2px** (pagination 이 311.2 px 적게 산정)

LAYOUT_OVERFLOW 4건 (pi=32~35, overflow 25.5~287.9px).

## 가설 검증

### 가설 A — fit 누적의 trailing_ls 누락 (★ 확정)

`typeset.rs::typeset_paragraph` 의 fit 분기:
```rust
if st.current_height + fmt.height_for_fit <= available {
    st.current_items.push(PageItem::FullParagraph { para_index: para_idx });
    st.current_height += fmt.height_for_fit;  // ← 누적도 height_for_fit 사용
    return;
}
```

`height_for_fit = total_height - trailing_ls`. fit 판정에는 trailing_ls 제외 (페이지 끝에서는 다음 줄 없음 → ls 무의미) 가 맞지만, **누적에는 trailing_ls 포함이 맞다** (다음 항목 시작 위치는 trailing_ls 만큼 아래).

검증: 36 items × 평균 trailing_ls ~9px = ~311px ≈ 본 case 의 drift 311.2px 와 정확히 일치.

### 가설 B — layout 의 vpos 보정이 다른 기준 사용 (기각)

layout 단계는 LINE_SEG.vpos 기반 절대 좌표. typeset 의 누적과 layout 의 y 진행이 어긋나는 것은 typeset 측 origin 이지 layout 측 origin 이 아님.

### 가설 C — Shape (TAC) 의 fit 반영 부정확 (기각)

본 case 의 36 items 중 Shape 1개 (pi=31). Shape 의 fit 반영은 line 940-948 에서 단순 push (높이 없음). 311px drift 에 기여 안함.

### 가설 D — 빈 문단의 누적 누락 (부분 영향)

빈 문단도 fit 누적 = `height_for_fit` (= 0 또는 작은 값). 본 case 는 36 items 중 빈 문단 약 7~8개. 가설 A 와 같은 origin (trailing_ls 누락) 이므로 가설 A 로 통합.

### 가설 E — 표 (TAC/non-TAC) 의 누적 (부분 영향)

본 case 에는 표 없음. k-water-rfp p3 와 무관.

## 수정 방향 — Option A 확정

**변경**: fit 판정과 누적을 분리.
- **fit 판정**: `height_for_fit` (trailing_ls 제외) 유지
- **누적**: `total_height` (full) 로 변경

코드:
```rust
if st.current_height + fmt.height_for_fit <= available {
    st.current_items.push(PageItem::FullParagraph { para_index: para_idx });
    st.current_height += fmt.total_height;  // ← total_height 로 변경
    return;
}
```

`line_count == 0` 분기 (line 627) 도 동일 변경.

## 대안 검토

### Option B — fit 판정도 total_height 로

- 문제: 마지막 항목의 trailing_ls 가 fit 안 되어 페이지에 너무 적게 들어감 (잔여 공간 낭비)

### Option C — vpos 절대 좌표 기반으로 layout.rs 재작성

- 문제: 대규모 변경 (layout.rs 3,178 줄 + paragraph_layout.rs 2,796 줄). Stage 2 시점에 작업지시자가 Option C 선택했다가 범위 너무 큼을 인식하고 Option A 로 변경 결정.

## layout.rs 정합성 점검 (Stage 1 추가 점검 항목)

5개 layout 파일 합계 18,889 줄. 6 commit 중 5건이 layout.rs 직접 변경. 코드 중복은 일부 발견 (table_layout / shape_layout / paragraph_layout 각각 line_height 산정 로직 중복) 하지만 본 task 범위 외. 별도 task 후보로 기록.

## 다음 단계 (Stage 3)

1. Option A 적용 → typeset.rs 수정
2. 자동 회귀 (cargo test --lib, svg_snapshot, issue_301, clippy, wasm32)
3. 7 핵심 샘플 + 추가 샘플 (kps-ai, hwp-multi-001) 회귀
4. 시각 판정 (작업지시자) 케이스 발견 시 추가 가드
