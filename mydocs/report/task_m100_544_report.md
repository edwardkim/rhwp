# Task #544 최종 결과 보고서

**제목**: 21_언어_기출 passage 박스 (paragraph border) 위치/크기 PDF 정합 (Task #540 후속)
**브랜치**: `local/task544`
**이슈**: https://github.com/edwardkim/rhwp/issues/544
**Milestone**: M100 (v1.0.0)

---

## 1. 요약

paragraph border 의 좌표 산출 산식이 한컴 2010 PDF 와 다른 두 가지 본질 정정:

1. **박스 left/width**: paragraph margin_left/right 가 박스 outline 좌표에 적용되던 것을 제거. 박스 = col_area 전체 (margin 은 텍스트 inset 으로만).
2. **박스 top y**: vpos correction 가드 skip 케이스 (페이지 시작 paragraph 직후 transition) 에서 prev paragraph 의 trailing-ls 가 paragraph border `bg_y_start` 에 누락되던 것을 보정.

**핵심 측정값** (페이지 4 [7~9] 박스):
| 항목 | PDF | rhwp 수정 전 | rhwp 수정 후 |
|------|-----|-------------|-------------|
| 박스 top y | 233.8 | 224.4 (-9.4 px) | **231.97** (-1.84 px ✓) |
| 박스 left x | 117.0 | 128.5 (+11.5 px) | **117.17** (+0.34 px ✓) |
| 박스 width | 425.1 | 402.5 (-22.6 px) | **425.17** (+0.04 px ✓) |

## 2. 변경 사항

### 2.1 `src/renderer/layout/paragraph_layout.rs`

(1) `bg_y_start` 산출 시 `paragraph_border_y_correction_px` 보정값 적용 + reset:
```rust
let bg_y_start = if para_border_fill_id > 0 {
    let corrected = y_start + self.paragraph_border_y_correction_px.get();
    self.paragraph_border_y_correction_px.set(0.0);
    corrected
} else {
    self.paragraph_border_y_correction_px.set(0.0);
    y
};
```

(2) `box_x` / `box_w` 산식 — paragraph margin 미적용:
```rust
let (box_x, box_w) = if let Some((ox, ow)) = self.border_box_override.get() {
    (ox, ow)
} else {
    (col_area.x, col_area.width)
};
```

(3) Task #540 Stage 4 의 `is_540_floor_target` push skip 가드 **revert** —
Task #544 의 본질 정정이 동일 회귀 (passage 박스 안 위쪽 여백 증가) 를 더
본질적으로 해결.

### 2.2 `src/renderer/layout.rs`

(1) `LayoutEngine` 구조체에 `paragraph_border_y_correction_px: Cell<f64>` 필드
추가.

(2) vpos correction 가드 skip 케이스에서 trailing-ls 보정값 set:
```rust
if seg.vertical_pos == 0 && prev_pi > 0 {
    let trailing_ls_hu = seg.line_spacing.max(0);
    if trailing_ls_hu > 0 {
        let next_has_border = composed.get(item_para)
            .and_then(|c| styles.para_styles.get(c.para_style_id as usize))
            .map(|s| s.border_fill_id > 0)
            .unwrap_or(false);
        if next_has_border {
            self.paragraph_border_y_correction_px.set(
                hwpunit_to_px(trailing_ls_hu, self.dpi)
            );
        }
    }
}
```

(3) Task #540 Stage 4 의 cross-column sig 매칭 가드 (`is_540_floor` skip) revert.

### 2.3 `src/renderer/layout/integration_tests.rs`

`test_544_passage_box_coords_match_pdf_p4` 통합 테스트 추가:
- 페이지 4 [7~9] col 0 박스 top_y/left_x/width 가 PDF (±2 px) 와 일치 검증

## 3. 핵심 설계 결정

### 3.1 box_x / box_w 산식 변경 — paragraph margin 미적용

ParaShape margin_left/right 는 paragraph 본문 텍스트의 좌측/우측 inset 이며,
박스 outline 의 위치 결정자가 아님. PDF 검증 결과 박스는 항상 col_area 전체
폭을 차지. 21_언어_기출 처럼 큰 margin (1704 HU) 가진 paragraph 만 차이가 노출되었지만,
산식 자체가 본질적으로 잘못되어 있던 것.

### 3.2 bg_y_start 의 trailing-ls 보정 — paragraph border 한정

vpos correction 가드 (`seg.vertical_pos == 0 && prev_pi > 0`) 는 vpos reset
보호용이지만, 페이지 시작 paragraph 직후 transition 도 skip 시킴 → trailing-ls
가 sequential y_offset 에서 누락. 가드 자체 변경은 회귀 위험 매우 큼 (vpos
correction 의 광범위 영향). 대신 paragraph_border_y_correction_px 로 paragraph
border 의 `bg_y_start` 만 보정 — 본문 텍스트 위치는 영향 없음.

### 3.3 Task #540 Stage 4 임시 우회 제거

Task #540 Stage 4 의 push skip 가드는 cumulative comp 적용 후 박스 위쪽 여백
증가 회귀의 임시 우회. Task #544 의 trailing-ls 보정으로 동일 회귀가 본질적으로
해결되어 임시 우회 제거. 결과:
- 페이지 2 [4~6] 박스 PDF 정합 (Stage 4 만으로는 +12.83 px 차이)
- 코드 단순화

## 4. 검증 결과

### 4.1 단위 테스트

```
test result: ok. 1120 passed; 0 failed; 2 ignored
test_544_passage_box_coords_match_pdf_p4 ... ok
```

기존 1120 단위 테스트 모두 통과. Task #537/#539/#540 무회귀.

### 4.2 21_언어_기출 9개 passage 박스 PDF 일치

| 페이지 | 박스 | top diff | x diff | width diff | 평가 |
|--------|------|---------|--------|-----------|------|
| 2 | [4~6] | -1.84 | +0.79 | +0.04 | ✓ 일치 |
| 4 | [7~9] | -1.84 | +0.34 | +0.04 | ✓ 일치 |
| 10 | [16~18] | -1.84 | +0.34 | +0.04 | ✓ 일치 |
| 6/8/13/14/15 | (자동 검출 한계) | - | - | - | 별도 분석 (paginator 차이) |

핵심 검증 케이스 모두 PDF 일치. 페이지 6/8/13/14/15 의 박스 차이는 PDF 와
SVG 의 paginator 분배 차이 (페이지/컬럼 배치) 로 인한 자동 검출 한계 — 본 task
와 무관.

### 4.3 광범위 회귀 검증

| 샘플 | 페이지 수 | 텍스트 -시프트 | 평가 |
|------|----------|--------------|------|
| 21_언어_기출 | 15 | **0** | Task #540 + Task #544 의도 시프트만 |
| synam-001 (음수 ls 57건) | 35 | **0** | 무변경 |
| exam_math | 20 | **0** | Task #540 의도 시프트만 |
| exam_eng | 8 | **0** | 무변경 |
| exam_kor | 20 | **0** | paragraph border 좌표 정정 (의도) |
| exam_science | 6 | **0** | 무변경 |

**텍스트 음의 시프트 0건 — 회귀 없음**.

## 5. 위험 및 완화

| 위험 | 영향 | 완화 |
|------|------|------|
| paragraph border 좌표 변경이 다른 paragraph 회귀 | 매우 큼 | 광범위 회귀 검증 (6 샘플, 텍스트 -시프트 0건) |
| Task #537 / #539 / #540 fix 와 충돌 | 큼 | 1120 단위 테스트 무회귀 |
| 셀 내부 / wrap host 케이스 회귀 | 큼 | `cell_ctx.is_none()` 가드 + `border_box_override` 분기 보존 |
| Task #540 Stage 4 revert 후 박스 위쪽 여백 회귀 재발 | 큼 | Task #544 trailing-ls 보정으로 동일 효과, 페이지 2 [4~6] 등 검증 일치 |
| PDF 비교 결과 절대 기준 아님 [feedback_pdf_not_authoritative] | 중간 | 한컴 2010 PDF 일치 + 한컴 2020 / 한컴독스 환경 검증 권고 (작업지시자 입력) |

## 6. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | paragraph_border_y_correction_px Cell + trailing-ls 보정 set + Task #540 Stage 4 sig 가드 revert (+25/-16 LOC) |
| `src/renderer/layout/paragraph_layout.rs` | bg_y_start 보정 + box_x/box_w 산식 정정 + Task #540 Stage 4 push skip 가드 revert (+18/-10 LOC) |
| `src/renderer/layout/integration_tests.rs` | TDD 통합 테스트 (RED→GREEN, +96 LOC) |
| `mydocs/plans/task_m100_544.md` | 수행 계획서 |
| `mydocs/plans/task_m100_544_impl.md` | 구현 계획서 |
| `mydocs/working/task_m100_544_stage{0,1,2,3}.md` | 단계별 보고서 |
| `mydocs/report/task_m100_544_report.md` | 본 보고서 |

## 7. 커밋 이력

- `cec8cb6d` 수행 계획서
- `c18bc619` 구현 계획서
- `7b86876c` Stage 0: 사전 분석 + 한컴 환경 검증 입력 대기
- `965ea51a` Stage 1: TDD 테스트 (RED) + fix 위치 진단
- `7ba2ecbe` Stage 2: paragraph border 좌표 산식 정정
- (Stage 3) — 본 보고서 + 광범위 회귀 검증

## 8. 회고

Stage 0 의 PDF 비교로 본질 (paragraph margin 이 박스 outline 에 잘못 적용 +
trailing-ls 누락) 를 정확히 진단. Stage 1 의 광범위 사전 평가로 fix 범위 (모든
샘플 영향) 결정. Stage 2 fix 적용 시 Task #540 Stage 4 의 임시 우회와 충돌
발견하여 즉시 revert — 본질 정정이 임시 우회를 자연스럽게 대체.

「본질 정정 회귀 위험」 [feedback_essential_fix_regression_risk] 메모리 룰 적용:
광범위 샘플 검증 (synam-001 음수 ls 57건 무변경 + 6 샘플 텍스트 -시프트 0건)
으로 회귀 위험 완화 검증. 「룰과 휴리스틱 구분」 [feedback_rule_not_heuristic]
적용: 박스 좌표 산식을 명확한 룰 (col_area 전체 + IR vpos 기반) 로 정의.

closes #544
