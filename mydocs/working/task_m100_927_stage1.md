# Task #927 단계별 완료 보고서 — Stage 1 (진단 + 수정 통합)

## 이슈

[#927](https://github.com/edwardkim/zhwp/issues/927) — `samples/hwp3-sample16-hwp5.hwp` (HWP5) 가 rhwp-studio 에서 **98p** 표시. CLI/메타 62p 대비 +58% 오버.

## Root cause 확정

`reflow_linesegs_on_demand` (HWPX/HWP5 자동 보정) 가 reflow 직후 **`recalculate_section_vpos` 호출 누락**.

원인 메커니즘:
- 빈 lineseg 였던 문단 (sample16-hwp5 의 59 문단) 의 reflow 시 `vpos_start = 0` (orig 없음)
- 후속 문단의 `vertical_pos` 가 누적되지 않아 구역 내 vpos 연속성 깨짐
- paginator (`engine.rs:258-285`) 의 `vpos_h = (seg.vertical_pos + seg.line_height + seg.line_spacing) - base` 계산이 잘못된 값으로 `current_height` 조정
- 잘못된 current_height 가 페이지 break 결정에 영향 → 페이지 과다 분할

## 진단 과정 (확인된 후보 / 폐기된 후보)

| 후보 | 결과 |
|------|------|
| `compute_line_spacing_hwp` Percent 산출 (line_height vs baseline) | 페이지 수 무관 (검증 후 폐기) |
| `corrected_line_height` 인플레이션 | 페이지 수 무관 (검증 후 폐기) |
| `vertical_pos == 0` forced break | 정상값 있음, 무관 (검증 후 폐기) |
| **`recalculate_section_vpos` 누락** | ✅ Root cause |

## 수정

### `src/document_core/commands/document.rs::reflow_linesegs_on_demand`

reflow 루프 안에서 가장 이른 reflowed paragraph 인덱스 추적 후, 루프 종료 시 `recalculate_section_vpos(start)` 호출:

```rust
let mut min_reflowed_idx: Option<usize> = None;
for (pi, para) in section.paragraphs.iter_mut().enumerate() {
    if Self::needs_reflow_broadly(para) {
        reflow_line_segs(para, ...);
        reflowed += 1;
        if min_reflowed_idx.is_none() {
            min_reflowed_idx = Some(pi);
        }
    }
    // ... 셀 내부 처리
}

// reflow 후 vpos 일관성 재계산
if let Some(start) = min_reflowed_idx {
    crate::renderer::composer::recalculate_section_vpos(
        &mut section.paragraphs,
        start,
    );
}
```

## 검증 결과

| 항목 | 결과 |
|------|------|
| sample16-hwp5 페이지 (reflow 후) | **98 → 69** (29p 감소, -30%) |
| 한컴 viewer 정합도 | +58% 오버 → +11% 오버 (수용 범위) |
| cargo test --lib | 1275 passed / 0 failed |
| cargo check wasm32-unknown-unknown | OK |
| cargo clippy -- -D warnings | clean |
| CLI 다른 sample 페이지 수 회귀 | 없음 (HWP3 sample16: 64p / exam_kor: 20p 변화 없음) |

## scope 외 / 관찰

남은 +7p (62→69) 는 reflow 가 빈 lineseg 문단의 본문 5386 chars (49개 의미있는 본문) 을 정상 분할한 결과. 한컴 viewer 의 HWP5 정합 (62p) 과 미세 차이 — reflow 알고리즘이 한컴보다 약간 덜 빡빡한 line break. 추가 정합은 별도 후속 작업 (font width measurement 정밀화, 한컴 line break 휴리스틱 추가 분석) 필요.

## 진단 도구

`examples/diag_927_reflow.rs` — reflow 회귀 검증용 (재발 시 페이지 수 폭증 즉시 감지).

작업지시자 승인 후 커밋 + PR 진행.
