# Task #386 단계 3: 좌표계 가드 구현 (Green) — 완료보고서

> **이슈**: [#386](https://github.com/edwardkim/rhwp/issues/386)
> **브랜치**: `local/task386`
> **작성일**: 2026-04-27

---

## 변경 사항

`src/renderer/typeset.rs:1947-1956` `compute_body_wide_top_reserve_for_para`:

```rust
// [Task #386] VertRelTo::Paper 인 경우 vertical_offset 은 page-top 절대 좌표.
// reserve 는 col 1 cur_h 시작값(=body-top 상대)으로 사용되므로 body_top 차감.
let bottom_raw = shape_y_offset + shape_h + outer_bottom;
let bottom = if matches!(common.vert_rel_to, VertRelTo::Paper) {
    (bottom_raw - body_top).max(0.0)
} else {
    bottom_raw
};
```

기존 가드(머리말 영역 전체 도형 제외, `if shape_bottom_abs <= body_top { continue }`)는 보존.

## 단위 테스트 결과

```
$ cargo test --release --lib t386_body_wide_reserve
test t386_body_wide_reserve_paper_relative_returns_body_relative ... ok
test t386_body_wide_reserve_paper_relative_inside_header_skipped ... ok
test t386_body_wide_reserve_para_relative_unchanged ... ok
test result: ok. 3 passed; 0 failed
```

단계 2의 Red 테스트가 Green으로 전환.

## exam_eng.hwp 1페이지 검증

### `dump-pages -p 0` Before/After

| 항목 | Before | After |
|------|--------|-------|
| 단 0 items | 38 | 38 |
| 단 0 used | 1114.4 px | 1114.4 px |
| 단 1 items | **25** (1~11번 ③까지) | **34** (1~12번 모두) |
| 단 1 used | 1186.2 px | **1207.6 px** |
| 단 1 hwp_used | 969.1 | 1204.0 |
| 단 1 diff | **+217.1 px** | **+3.6 px** ✅ |

### 시각 확인

1페이지 SVG에 1번~12번 모두 표시 확인 (11번 ④⑤+12번이 더 이상 2페이지로 밀리지 않음).

## 전체 lib 테스트 회귀

```
$ cargo test --release --lib
test result: ok. 1017 passed; 0 failed; 1 ignored
```

기존 1015 + 신규 3 = 1018 항목 중 1 ignored, 0 failed. 회귀 0건 ✅

## 참고: 전체 SVG 페이지 수

- rhwp 출력: 11 페이지 (수정 전후 동일)
- 한컴 PDF: 8 페이지
- 차이는 본 이슈와 별개 (별도 추적 필요 시 신규 이슈로 분리). 단계 4에서 골든 SVG 비교로 정량 확인.

## 산출물

- 본 보고서 (`mydocs/working/task_m100_386_stage3.md`)
- 소스 변경: `src/renderer/typeset.rs:1947-1956` (좌표계 가드 추가, 약 6줄)
- 다음 단계: 단계 4 — 통합 검증 + 다중 샘플 회귀 측정 + 최종 보고
