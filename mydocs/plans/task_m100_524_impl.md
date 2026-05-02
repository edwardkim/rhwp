# Task #524 구현 계획서

## 1. 정정 대상 (Stage 1 진단 결과)

페이지 1 단 1의 페이지네이션 `current_height` 가 vpos 기반 실제 종료점보다 **+59 px** 누적. 가장 큰 단일 inflation은 `pi=21` (5번 문제 stem, 그림 tac=false 포함)에서 발생.

## 2. 코드 분석

### 의심 코드: `src/renderer/pagination/engine.rs:1069-1085`

```rust
Control::Picture(pic) => {
    st.current_items.push(PageItem::Shape { ... });
    if !pic.common.treat_as_char
        && matches!(pic.common.text_wrap,
            crate::model::shape::TextWrap::Square
            | crate::model::shape::TextWrap::TopAndBottom)
    {
        let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
        let margin_top = hwpunit_to_px(pic.common.margin.top as i32, self.dpi);
        let margin_bottom = hwpunit_to_px(pic.common.margin.bottom as i32, self.dpi);
        st.current_height += pic_h + margin_top + margin_bottom;  // ← 이중 가산 의심
    }
}
```

### 호출 구조

```
paginate_paragraph 루프
├─ current_height += para_height           (line 671/682/787)
│   └─ para_height = MeasuredParagraph 의 line_heights 합 + sb + sa
└─ process_controls (line 337)
    └─ ctrl_idx 순회
        └─ Control::Picture (line 1069)
            └─ wrap=Square/TopAndBottom 이고 비-TAC 이면
                current_height += pic_h + margin_top + margin_bottom
```

### 이중 가산 가설

`pi=21` 의 LINE_SEG `lh` 합:
- ls[0..5]: 1350+1350+1350+1350+1350+1150 = **7900 HU = 105.3 px** (텍스트 줄)
- para_height ≈ 105.3 + sa(13.3) = **118.6 px**

그림 (tac=false, wrap=어울림): 10230 HU = **136.4 px** (paragraph height에 포함되지 않은 별도 영역).

HWP의 vpos diff (pi=21 → pi=22): 10840 HU = **144.5 px**. 즉 HWP는 max(텍스트 줄, 그림) + sa ≈ 144.5 px 로 인코딩. 줄 105.3 + 그림 추가분 → 단일 가산이면 정합.

**현재 알고리즘**: para_height(118.6) + pic_h(136.4) + margins ≈ **255 px** 누적.
**HWP 실제**: 144.5 px.
**과대 가산**: ~110 px (pi=21 단독).

전체 단 1 의 +59 px 드리프트와 일치하지 않는 것은:
- 후속 항목에서 보정되는 메커니즘 (vpos 기반 layout 보정 — line 240-258 의 `if !prev_has_tac_eq` 분기 등) 이 일부 오프셋을 흡수.
- 그러나 결과적으로 마지막 항목 기준 +59 px 잔존.

## 3. 정정 방향

### 3.1 그림 tac=false, wrap=Square/TopAndBottom 의 height 가산 정합 (1차)

현재 `para_height + pic_h + margin` 합산.
**올바른 누적**: `max(para_height_without_pic, pic_h + margins) + sa` 또는 `vpos_diff_to_next_para` 기반.

대안 a (간단): `current_height += pic_h + margins - lines_height` 만큼만 추가 (그림이 텍스트 줄보다 큰 경우의 차이분만 가산).
대안 b (정합): para_height 계산 시 그림의 추가 차이분(`max(0, pic_extent - lines_extent)`)을 포함하도록 `MeasuredParagraph` 또는 `process_controls` 측에서 일관 처리.

대안 a 로 시작 — 변경 범위 최소.

### 3.2 vpos 기반 보정의 일관성 점검 (2차)

`engine.rs:240-258` 의 `prev_has_tac_eq` 분기에서 `Picture(InFrontOfText|BehindText)` 만 bypass. `Square/TopAndBottom` 는 보정 적용. 이 분기와 1083 의 픽처 높이 가산이 충돌하지 않는지 확인.

### 3.3 trailing line_spacing 정합 (3차, 필요 시)

페이지 1 단 1 내 작은 갭 6~13 px 의 출처. pi=18, pi=22, pi=26 등에서 발생. `paginate_text_lines` 의 trailing_ls 처리 (`engine.rs:611-661`) 와 일관성 확인.

## 4. 단계 구분 (4단계)

### Stage 2-A — 그림 tac=false 높이 가산 정정

**대상 파일**: `src/renderer/pagination/engine.rs`

**변경 내용**:
1. line 1069-1085 의 Picture 분기에서 pic_h 추가 시 lines_height 차감.
2. 또는 이중 가산이 발생하지 않도록 process_controls 호출 전후 current_height 변화량을 검증하는 디버그 로깅 추가 (RHWP_LAYOUT_DEBUG=1).

**검증**:
- `cargo run --release --bin rhwp -- dump-pages samples/exam_science.hwp -p 0` → 단 1 used 값 확인.
- 목표: used 1112.6 → ~1053 px (vpos 기반 종료점 정합).
- 페이지 2: 단 0 (좌측) 에 문제 7 (`pi=32`+) 또는 보기/답지가 페이지 1 우측 단으로 이동 후 [쪽나누기] 작동.

### Stage 2-B — 회귀 검증 (단위 + svg_regression_diff)

- `cargo test --lib` (1103+ 통과)
- `scripts/svg_regression_diff.sh` (7 샘플 170 페이지)
- diff 발생 페이지는 의도된 정정 vs 회귀 분류
- exam_science 6 페이지 모두 export → 페이지 수 변화 확인

**완료 기준**: 단위 테스트 0 회귀, svg_regression_diff 의도된 정정만 (회귀 0).

### Stage 2-C — 시각 정합 + 한컴 정합 확인

- exam_science 6 페이지 SVG 출력 (`output/svg/`)
- 작업지시자 시각 판정 — PDF 정합 확인
- 페이지 2 우측 단에 문제 8 stem 표시 확인 (PDF 일치)
- 페이지 8 번 문제 위치 확인 (페이지 3 → 페이지 2 이동)

**완료 기준**: 작업지시자 승인.

### Stage 2-D — 최종 보고서 + close

- `mydocs/working/task_m100_524_stage2.md` (구현 + 검증)
- `mydocs/report/task_m100_524_report.md` (최종)
- `mydocs/orders/20260502.md` 갱신
- local/task524 → local/devel merge → devel push
- `gh issue close 524`

## 5. 위험·회귀

| 위험 | 영향 | 대응 |
|------|------|------|
| 다른 샘플의 그림 tac=false 페이지 배치 변화 | 페이지 수 변경 가능 | svg_regression_diff 로 식별 + 시각 판정 |
| 그림 tac=true 미영향 확인 | 별도 분기, 영향 없어야 함 | `treat_as_char` 가드 검증 |
| TextWrap::Square + TopAndBottom 만 영향 | 다른 wrap 모드 영향 없음 | 코드 분기 가드 검증 |
| 페이지 break 시 picture 가 다른 페이지로 이동하는 경우 | 상호작용 가능 | 회귀 테스트 |

## 6. 검증 게이트 (요약)

| 게이트 | 도구 | 기준 | 단계 |
|--------|------|------|------|
| 코드 컴파일 | `cargo build --release` | warning 0 | Stage 2-A |
| 단위 테스트 | `cargo test --lib` | 1103+ pass | Stage 2-B |
| 회귀 검증 | `scripts/svg_regression_diff.sh` | 7 샘플 회귀 0 (의도된 정정 OK) | Stage 2-B |
| Clippy | `cargo clippy` | warning 0 | Stage 2-B |
| 측정 정합 | `dump-pages` used 값 | 1112.6 → ~1053 px | Stage 2-A |
| 시각 정합 | 작업지시자 판정 | exam_science p2 PDF 정합 | Stage 2-C |

## 7. 진행 순서

1. Stage 2-A 코드 수정 + dump-pages 측정 검증 → 보고
2. Stage 2-B 회귀 검증 → diff 결과 보고
3. Stage 2-C 시각 정합 → 작업지시자 판정 대기
4. Stage 2-D 최종 보고서 + merge

각 stage 종료 시 보고서 작성 + 승인 요청. 승인 후 다음 단계 진행.

---

승인 요청: 본 구현 계획대로 Stage 2-A 진행 가능 여부.
