# Task #386 단계 4: 통합 검증 및 회귀 측정 — 완료보고서

> **이슈**: [#386](https://github.com/edwardkim/rhwp/issues/386)
> **브랜치**: `local/task386`
> **작성일**: 2026-04-27

---

## 전체 테스트

```
$ cargo test --release
- lib: 1017 passed, 0 failed, 1 ignored
- svg_snapshot: 6 passed, 0 failed
- 통합 테스트 14 + 25 + 2 + 1 + 1 + ... 모두 0 failed
```

기존 1014 + 신규 3 (Task #386) = 1017 lib 테스트 PASS.

## 다중 샘플 회귀 (Before / After 비교)

### LAYOUT_OVERFLOW + 페이지 수

| 샘플 | Before pages | After pages | Before overflow | After overflow |
|------|---|---|---|---|
| exam_eng.hwp | 11 | 11 | 0 | 0 |
| exam_math.hwp | 20 | 20 | 0 | 0 |
| aift.hwp | 77 | 77 | 3 | 3 |
| k-water-rfp.hwp | 27 | 27 | 0 | 0 |
| kps-ai.hwp | 79 | 79 | 5 | 5 |
| 2010-01-06.hwp | 6 | 6 | 0 | 0 |
| hwpspec.hwp | 177 | 177 | 16 | 16 |
| 2022년 국립국어원 업무계획.hwp | 37 | 37 | 0 | 0 |

회귀 0건 ✅

### 바이트 단위 SVG diff

| 샘플 | 변경 페이지 수 |
|------|---------------|
| **exam_eng.hwp** | **3 / 11** (page 1, 2, 3) ← 의도된 변경 |
| exam_math.hwp | 0 / 20 |
| aift.hwp | 0 / 77 |
| k-water-rfp.hwp | 0 / 27 |
| kps-ai.hwp | 0 / 79 |
| 2022년 국립국어원 업무계획.hwp | 0 / 37 |

→ 본 수정은 **`VertRelTo::Paper` + TopAndBottom + body-wide(>80% body width) + non-TAC** 도형/표가 col 0 첫 문단에 있는 경우만 발동.

## exam_eng.hwp 1페이지 단 1 검증

`dump-pages -p 0`:

| 항목 | Before | After |
|------|--------|-------|
| 단 0 items | 38 | 38 |
| 단 0 used | 1114.4 px | 1114.4 px |
| 단 1 items | 25 (1~11번 ③) | **34 (1~12번 모두)** ✅ |
| 단 1 used | 1186.2 px | 1207.6 px |
| 단 1 hwp_used | 969.1 | 1204.0 |
| 단 1 diff | +217.1 px | **+3.6 px** ✅ |

시각 확인: 한컴 PDF 1페이지와 동일한 항목 분포 (1번~12번).

## Task #62 가드 영향

수정한 함수는 `compute_body_wide_top_reserve_for_para` (typeset.rs)이며, Task #62의 가드는 `pagination/engine.rs:251-254` (글앞으로/글뒤로 Shape vpos bypass) — 별개 코드 경로. 영향 없음.

## 잔여 사항 (별도 추적)

- rhwp 페이지 수 11 vs 한컴 PDF 8 — 본 이슈 범위 외. 필요 시 별도 이슈로 분리.

## 수정 요약

`src/renderer/typeset.rs:1947-1956` — VertRelTo::Paper 케이스에 body_top 차감 (약 6줄):

```rust
let bottom_raw = shape_y_offset + shape_h + outer_bottom;
let bottom = if matches!(common.vert_rel_to, VertRelTo::Paper) {
    (bottom_raw - body_top).max(0.0)
} else {
    bottom_raw
};
```

추가 단위 테스트 3건 (typeset.rs `mod tests`):
- `t386_body_wide_reserve_paper_relative_returns_body_relative` (핵심 회귀)
- `t386_body_wide_reserve_paper_relative_inside_header_skipped`
- `t386_body_wide_reserve_para_relative_unchanged`

## 산출물

- 본 보고서 (`mydocs/working/task_m100_386_stage4.md`)
- 다음: 최종 보고서 (`mydocs/report/task_m100_386_report.md`) + orders 갱신 + 커밋
