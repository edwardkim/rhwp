# 최종 보고서 — Task #386: exam_eng.hwp 1페이지 단 1 끝부분 밀림

- **이슈**: [#386](https://github.com/edwardkim/rhwp/issues/386)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task386` (base: `devel`)
- **샘플**: `samples/exam_eng.hwp`
- **단계**: 4/4 (최종 보고)

---

## 1. 배경 및 증상

`samples/exam_eng.hwp` 1페이지 SVG 출력에서 한컴 PDF 출력과 1페이지 구성이 어긋남.

| 출력 | 1페이지 단 1 마지막 항목 |
|------|--------------------------|
| 한컴 PDF | 12번 ⑤ (1~12번 모두 1페이지 포함) |
| rhwp SVG (수정 전) | 11번 ③ (11번 ④⑤ + 12번이 2페이지로 밀림) |

LAYOUT_OVERFLOW 경고는 발생하지 않아 Task #356의 검증망(detect_inter_paragraph_vpos_reset)으로 잡히지 않은 별개 결함.

## 2. 원인 분석

### 가설 변경 이력 (단계 1 진단)

| 차수 | 가설 | 결과 |
|------|------|------|
| 1차 | `pagination/engine.rs:245-285` vpos 보정 (이슈 본문) | ❌ 미사용 경로 — 기본 렌더링은 `TypesetEngine` |
| 2차 | Task #359 `total_height` 누적 drift | ❌ #359 정책은 정확 (HWP vpos와 일치) |
| **3차** | `compute_body_wide_top_reserve_for_para` Paper-rel 좌표계 버그 | ✅ 확정 |

### 근본 원인

`src/renderer/typeset.rs:1908`의 `compute_body_wide_top_reserve_for_para` 함수는 col 0 첫 문단에 body-wide TopAndBottom 도형/표가 있을 때 col 1 진입 시 `current_height` 시작값(reserve)을 계산. `VertRelTo::Paper` 케이스에서 `vertical_offset`은 page-top 절대 좌표지만 함수가 그대로 반환 → col 1 cur_h 시작값(=col-top 상대)에 page-abs 좌표가 들어가 `body_top` 만큼 시프트.

### 수치 (exam_eng.hwp pi=0 ctrl[4])

```
입력: VertRelTo::Paper, vertical_offset=10885 HU(145.13px), height=11058 HU(147.44px),
      margin.bottom=1132 HU(15.09px)
A3 layout: body_top = 16013 HU = 213.51 px

수정 전: bottom = 145.13 + 147.44 + 15.09 = 307.67 px (page-abs)
수정 후: bottom = max(0, 307.67 - 213.51) = 94.13 px (body-rel)
HWP col 1 첫 항목 vpos = 7060 HU = 94.13 px ← 일치
```

213.5 px 시프트가 누적되어 col 1 끝부분(11번 ④⑤ + 12번)이 다음 페이지로 밀림.

## 3. 해결

`src/renderer/typeset.rs:1947-1956`:

```rust
let outer_bottom = crate::renderer::hwpunit_to_px(common.margin.bottom as i32, dpi);
// [Task #386] VertRelTo::Paper 인 경우 vertical_offset 은 page-top 절대 좌표.
// reserve 는 col 1 cur_h 시작값(=body-top 상대)으로 사용되므로 body_top 차감.
let bottom_raw = shape_y_offset + shape_h + outer_bottom;
let bottom = if matches!(common.vert_rel_to, VertRelTo::Paper) {
    (bottom_raw - body_top).max(0.0)
} else {
    bottom_raw
};
```

기존 가드(머리말 영역 전체 도형 제외)는 보존. Para/Page 케이스는 변동 없음.

### 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `src/renderer/typeset.rs` | 좌표계 가드 추가 (6줄) + 단위 테스트 3건 + 헬퍼 함수 |

### 추가된 테스트

- `t386_body_wide_reserve_paper_relative_returns_body_relative` (핵심 회귀)
- `t386_body_wide_reserve_paper_relative_inside_header_skipped`
- `t386_body_wide_reserve_para_relative_unchanged`

## 4. 검증

### exam_eng.hwp 1페이지

| 항목 | Before | After |
|------|--------|-------|
| 단 1 items | 25 (1~11번 ③) | **34 (1~12번 모두)** ✅ |
| 단 1 used | 1186.2 px | 1207.6 px |
| 단 1 diff | +217.1 px | **+3.6 px** ✅ |
| 시각 1페이지 | 11번 ③까지 | 12번 ⑤까지 (PDF와 일치) |

### 전체 테스트

```
cargo test --release: 1017 lib + 14 + 25 + 6 + 1 + 2 + 1 PASS, 0 FAIL
```

### 다중 샘플 회귀

| 샘플 | 페이지 변경 | overflow Before/After | 바이트 diff |
|------|----|----------------|--------|
| exam_eng.hwp | 11 → 11 | 0 / 0 | 3 페이지 (의도된 변경) |
| exam_math.hwp | 20 → 20 | 0 / 0 | 0 |
| aift.hwp | 77 → 77 | 3 / 3 | 0 |
| k-water-rfp.hwp | 27 → 27 | 0 / 0 | 0 |
| kps-ai.hwp | 79 → 79 | 5 / 5 | 0 |
| 2010-01-06.hwp | 6 → 6 | 0 / 0 | (미측정) |
| hwpspec.hwp | 177 → 177 | 16 / 16 | (미측정) |
| 2022년 국립국어원 업무계획.hwp | 37 → 37 | 0 / 0 | 0 |

→ exam_eng.hwp 외 모든 샘플 0건 변경. Task #62 가드 영향 없음.

## 5. 잔여 사항

- rhwp 페이지 수 11 vs 한컴 PDF 8 — 본 이슈 범위 외. 필요 시 별도 이슈로 추적.

## 6. 커밋 히스토리

- 단계 1: 진단 보고서 (소스 변경 없음)
- 단계 2: 단위 테스트 3건 추가 (Red)
- 단계 3: 좌표계 가드 구현 (Green) — 6줄
- 단계 4: 통합 검증 + 최종 보고서

각 단계는 `Task #386: <단계 제목>` 형식의 단일 커밋.

## 7. 결론

`compute_body_wide_top_reserve_for_para`의 Paper-rel 좌표계 버그를 6줄 수정으로 해결. exam_eng.hwp 1페이지 단 1 분포가 한컴 PDF와 동일하게 복원되었으며, 다른 샘플 회귀 0건 + 1017 lib 테스트 PASS.
