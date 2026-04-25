# Task #324 최종 결과 보고서

**제목**: 페이지네이션: 셀 분할 시 중첩 표가 limit 초과해도 항상 첫 페이지에 노출
**브랜치**: `local/task324`
**마일스톤**: M100 (v1.0.0)
**상태**: 완료
**기준일**: 2026-04-25

---

## 1. 문제 요약

`samples/hwpx/form-002.hwpx` SVG 출력에서 외부 표(26×27) row 19 의 셀 안에 있는 1×1 내부 표(`연구개발계획서 제출시…`)가 page 1 하단에 표출됨. PDF/HWP 기준 page 2 상단에 위치해야 함.

## 2. 근본 원인

`compute_cell_line_ranges` (`src/renderer/layout/table_layout.rs`) 가 셀 콘텐츠 분할 위치 결정 시 **잔량(remaining) 기반** 추적을 사용. 잔량이 0이 되는 순간 cumulative position 정보가 손실되어 중첩 표(atomic) 문단의 가시성을 정확히 판단할 수 없음.

또한 `layout_partial_table` 의 split-end 행 처리 분기에서, 다음 페이지로 미뤄진 atomic 문단의 렌더링 스킵 로직이 누락.

vpos 가 컬럼 단위로 0으로 리셋되어 vpos 기반 위치 계산은 불가능 → line 누적이 유일한 정확한 측정 수단.

## 3. 수정 내역

### 3-1. `src/renderer/layout/table_layout.rs::compute_cell_line_ranges`

Cumulative position(`cum`) 기반으로 전면 재작성:
- 셀 시작부터의 누적 px(`cum`)를 명시적으로 추적
- 일반 문단: 각 라인의 `line_end_pos = cum + line_h` 와 `content_offset` / `content_limit` 비교
- 중첩 표 atomic 문단: `para_end_pos` 로 \"이전 페이지 완료\"(`<= content_offset`) / \"다음 페이지 미룸\"(`> content_limit`) 결정
- 빈 문단(`line_count == 0`)과 has_table_in_para 분기를 단일 atomic 처리로 통합

### 3-2. `src/renderer/layout/table_partial.rs`

`start_line >= end_line` 인 중첩 표 문단 처리에 split-end 행 케이스 추가:

```rust
} else if has_nested_table && is_in_split_row && split_end_content_limit > 0.0 {
    let nested_h: f64 = ...;
    content_y_accum += nested_h;
    continue;
}
```

기존엔 split-start 만 처리되어, split-end 행에서는 (n,n) 마커가 무시되고 atomic 표가 두 번 렌더링됨.

## 4. 검증

### 출력 검증

| 항목 | Baseline | After |
|------|----------|-------|
| page 1 SVG | 269,233 B | **227,903 B** (-41 KB, 인너 표 제거) |
| page 2 SVG | 266,879 B | 266,879 B (변화 없음) |

page 2 의 인너 표는 베이스라인에서도 fall-through 경로로 렌더링되어 있었음 → **이번 수정은 page 1 의 잘못된 중복 렌더링만 제거**.

### 회귀 검증

- `cargo test --release`: **992 단위 테스트 + 71 통합 테스트 통과**
- `cargo clippy --release -- -D warnings`: **클린**
- 골든 갱신 1건: `tests/golden_svg/form-002/page-0.svg` (의도된 변경)

### dump-pages 검증

페이지 분할 결정 자체는 변경되지 않음 (page 1 rows=0..20, page 2 rows=19..26 동일):

```
페이지 2 used=925.6px hwp_used≈946.8px diff=-21.2px (변화 없음)
```

이는 본 수정이 **셀 콘텐츠 가시성 결정** 영역만 다루며, **페이지네이션 결정** 자체는 건드리지 않음을 확인.

## 5. 영향 범위

수정된 함수는 셀 분할이 발생하는 모든 표에 적용됨. 다른 샘플 골든 테스트가 모두 통과한 것으로 보아 회귀 없음.

## 6. 산출물

| 단계 | 파일 |
|------|------|
| 수행계획서 | `mydocs/plans/task_m100_324.md` |
| 구현계획서 | `mydocs/plans/task_m100_324_impl.md` |
| Stage 1 | `mydocs/working/task_m100_324_stage1.md` |
| Stage 2 (분석 v1) | `mydocs/working/task_m100_324_stage2.md` |
| Stage 2 (수정 v2) | `mydocs/working/task_m100_324_stage2_v2.md` |
| Stage 3 / 최종 | `mydocs/report/task_m100_324_report.md` (본 문서) |

## 7. 결론

`compute_cell_line_ranges` 의 잔량 기반 가시성 결정을 cumulative position 기반으로 재작성하고, `layout_partial_table` 의 split-end 행 atomic 스킵 분기를 추가하여 form-002 의 \"page 1 인너 표 중복\" 문제를 해결.
