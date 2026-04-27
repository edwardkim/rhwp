# Task #386 단계 1: 재현 및 정량 진단 — 완료보고서

> **이슈**: [#386](https://github.com/edwardkim/rhwp/issues/386)
> **브랜치**: `local/task386`
> **작성일**: 2026-04-27

---

## 핵심 결론

**근본 원인**: `src/renderer/typeset.rs:1908`의 `compute_body_wide_top_reserve_for_para` 함수가 `VertRelTo::Paper` 케이스에서 좌표계를 잘못 사용. Paper-absolute 좌표(page-top 기준)를 body-relative 좌표(body-top 기준)로 그대로 반환 → col 1 `current_height` 시작값이 228.6 px 과다 → col 1 누적 끝부분에서 11번 ④⑤+12번이 다음 페이지로 밀림.

## 가설 변경 이력

### 1차 가설 (이슈 본문) — ❌ 무효

`pagination/engine.rs:245-285` vpos 보정 로직. 그러나 **기본 렌더링 경로는 `TypesetEngine` (`typeset.rs`)**이며, `paginate_with_measured_opts`는 `RHWP_USE_PAGINATOR=1`일 때만 호출 (`document_core/queries/rendering.rs:825`).

### 2차 가설 (#359 drift 누적) — ❌ 무효

`fmt.total_height` 누적이 `vpos_h`보다 항목당 +7~14 px 크게 누적되는 현상을 drift로 의심. 그러나:
- Task #359 (`e5d383f`, 2026-04-27 00:02): `cur_h += height_for_fit` → `cur_h += total_height`로 변경
- 이는 인접 문단 간 line_spacing/sa를 누적에 포함하여 HWP vpos와 정확히 일치시키는 의도 (k-water-rfp p3 311 px overflow 해결)
- 항목당 차이는 drift가 아니라 **inter-paragraph spacing**으로 정상

### 3차 가설 (좌표계 버그) — ✅ 확정

## 실측 데이터

### col 1 시작 cur_h (page 1)

```
[T386] advance_to_col=1 page=1 ch_set_to=307.67 (pending_body_wide_top_reserve)
```

### `compute_body_wide_top_reserve_for_para` 수치 분해

대상: pi=0의 ctrl[4] (TopAndBottom 표, body-wide)

| 필드 | HU | px |
|------|-----|-----|
| `vert_rel_to` | Paper | — |
| `vertical_offset` | 10885 | **145.2** (page-abs) |
| `height` | 11058 | **147.4** |
| `margin.bottom` | 1132 | **15.1** |

함수 반환:
```
bottom = vertical_offset + height + margin.bottom
       = 145.2 + 147.4 + 15.1 = 307.7 px
```

### 좌표계 비교

| 좌표 | 값 |
|------|-----|
| body_top (px) | 213.5 |
| 표 bottom (Paper-abs) | 145.2 + 147.4 = 292.6 |
| 표 bottom + margin (Paper-abs) | 307.7 |
| **body 상대 표 bottom + margin** | 307.7 − 213.5 = **94.2** ← 옳은 값 |
| HWP col 1 첫 항목 vpos | 7060 HU = **94.1 px** ← 일치 |

→ 옳은 reserve는 약 **94.2 px**이지만 현재 함수는 **307.7 px** 반환. 차이 **+213.5 px** ≈ body_top의 page-abs 좌표.

### col 1 누적 결과 (TYPESET_DRIFT 트레이스 요약)

| 항목 | 누적 cur_h |
|------|------------|
| col 1 시작 (잘못된 reserve) | 307.7 |
| pi=33~57 누적 | 1186.2 |
| 가용 공간 (avail) | 1199.4 |
| 잔여 | 13.2 px |
| pi=58 (11번 ④) `height_for_fit` | 22.5 px |
| **fit 실패** → 다음 페이지로 carry | |

만약 reserve = 94.2 (옳은 값)이었다면:
- col 1 시작 cur_h = 94.2
- pi=33~57 누적 = 94.2 + 878.5 = 972.7
- 잔여 = 226.7 px
- pi=58~66(12번 ⑤까지) 필요 ≈ 130 px → **fits** ✅

## Paper-relative 좌표계 버그 분석

`compute_body_wide_top_reserve_for_para` (typeset.rs:1908-1953):

```rust
if matches!(common.vert_rel_to, VertRelTo::Paper) {
    let shape_top_abs = hwpunit_to_px(common.vertical_offset as i32, dpi);
    let shape_bottom_abs = shape_top_abs + hwpunit_to_px(common.height as i32, dpi);
    if shape_bottom_abs <= body_top {
        continue;  // 머리말 영역 전체에 위치 → 제외
    }
}
let shape_y_offset = hwpunit_to_px(common.vertical_offset as i32, dpi);  // ← Paper일 때 page-abs
// ...
let bottom = shape_y_offset + shape_h + outer_bottom;  // ← page-abs 그대로 반환
```

- 가드 `shape_bottom_abs <= body_top`은 **머리말에만 위치한 도형 제외** 의도 — 정상 동작
- 그러나 가드를 통과한 경우 `shape_y_offset`가 그대로 반환식에 들어감 → page-abs 좌표가 col 1 cur_h(=col-rel) 시작값으로 사용됨 → 좌표계 혼선
- VertRelTo::Para/Page 케이스는 paragraph/body-top 기준 오프셋이라 body-rel 의미가 자연스러움

## 수정 방안 (단계 3 예정)

```rust
// VertRelTo::Paper 일 때 body_top 차감하여 body-rel로 변환
let bottom = if matches!(common.vert_rel_to, VertRelTo::Paper) {
    let bottom_abs = shape_y_offset + shape_h + outer_bottom;
    (bottom_abs - body_top).max(0.0)
} else {
    shape_y_offset + shape_h + outer_bottom
};
```

영향 범위:
- VertRelTo::Paper + TopAndBottom + body-wide(width ≥ body_w * 0.8) + non-TAC 도형/표
- exam_eng.hwp 1페이지가 직접 케이스. 다른 샘플은 단계 4 회귀 측정으로 확인.

## 산출물

- 본 보고서 (`mydocs/working/task_m100_386_stage1.md`)
- 소스 변경: 없음 (디버그 출력 모두 제거 완료)
- 다음 단계: 단계 2 — `compute_body_wide_top_reserve_for_para` 단위 테스트 추가 (Red 상태)
