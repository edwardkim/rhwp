# Task #1147 v2 구현계획서 — 렌더러 layout.rs 측 HWPX TopAndBottom 표 후행 line_spacing 보정

- **수행계획서**: [task_m100_1147_v2.md](task_m100_1147_v2.md)
- **상위 이슈**: https://github.com/edwardkim/rhwp/issues/1147
- **브랜치**: `feature/task_m100_1147` (계속 사용)

## 1. 변경 개요

`src/renderer/layout.rs::layout_table_item` 의 "표 아래 간격" 분기 (라인 4072-4094) 가 빈 앵커 float 케이스에서 앵커 line_seg.line_spacing 을 그대로 가산하여 표 직후 문단이 시각상 18 px 아래로 밀려나는 문제를, Task #1147 Stage 1 (typeset 측 `is_topbottom_empty_anchor_hwpx`) 과 동일한 HWPX 한정 트리거로 0 으로 억제한다.

## 2. 컨텍스트 전달 경로

`LayoutEngine` 구조체에 이미 `is_hwp3_variant: Cell<bool>` 가 있고 `rendering.rs:2845` 에서 `set_hwp3_variant()` 로 주입하는 패턴이 있음. 동일 패턴으로 `is_hwpx_source: Cell<bool>` 신설:

```rust
// layout.rs (LayoutEngine 구조체)
is_hwpx_source: std::cell::Cell<bool>,

// layout.rs (LayoutEngine::new)
is_hwpx_source: std::cell::Cell::new(false),

// layout.rs (impl LayoutEngine, 신설)
pub fn set_hwpx_source(&self, enabled: bool) {
    self.is_hwpx_source.set(enabled);
}

// rendering.rs:2848 직후 (set_hwp3_origin_flow_spacing_before 다음)
self.layout_engine
    .set_hwpx_source(matches!(self.source_format, crate::parser::FileFormat::Hwpx));

// typeset.rs:3615 (ad-hoc LayoutEngine 도 동기화)
layout_engine.set_hwp3_variant(st.is_hwp3_variant);
layout_engine.set_hwpx_source(st.is_hwpx_source);  // 신규
```

## 3. 산식 보정

`layout.rs:4082-4093` 의 "표 아래 간격" 빈 앵커 분기:

```rust
// 변경 전
if let Some(seg) = para.line_segs.last() {
    let gap = if is_current_empty_para_float {
        seg.line_spacing.max(0)
    } else if seg.line_spacing > 0 {
        seg.line_spacing
    } else {
        seg.line_height
    };
    if gap > 0 {
        y_offset += hwpunit_to_px(gap, self.dpi);
    }
}
```

```rust
// 변경 후 (HWPX 빈 앵커 TopAndBottom 비-TAC 표 한정으로 gap=0)
//
// [Task #1147 v2] HWPX 원본의 빈 앵커 TopAndBottom 비-TAC 표는 typeset 측
// is_topbottom_empty_anchor_hwpx 보정으로 host_line_spacing=0 처리되므로,
// 렌더러도 동일하게 앵커 line_spacing 을 표 아래 갭으로 가산하지 않는다.
// 가산 시 typeset 의 cur_h 와 layout 의 y_offset 가 18 px 어긋나 표 직후
// 문단이 시각상 아래로 밀려난다 (작업지시자 시각 검수, 권위 PDF 정합).
let is_topbottom_empty_anchor_hwpx =
    self.is_hwpx_source.get() && is_current_empty_para_float;
if let Some(seg) = para.line_segs.last() {
    let gap = if is_topbottom_empty_anchor_hwpx {
        0
    } else if is_current_empty_para_float {
        seg.line_spacing.max(0)
    } else if seg.line_spacing > 0 {
        seg.line_spacing
    } else {
        seg.line_height
    };
    if gap > 0 {
        y_offset += hwpunit_to_px(gap, self.dpi);
    }
}
```

**트리거 정합 근거**: typeset 의 `is_topbottom_empty_anchor_hwpx` 는 `is_hwpx_source && !is_tac && wrap=TopAndBottom && para.text.is_empty()`. layout 의 `is_current_empty_para_float` 는 `is_para_topbottom_float(&t.common) && !para_has_visible_text(para)` = `!treat_as_char && wrap=TopAndBottom && vert_rel_to=Para && !visible_text`. 두 조건은 vert_rel_to=Para 가드 차이만 있고 (typeset 은 미확인) 같은 케이스 집합을 가리킴. layout 측이 더 좁은 (vert_rel_to=Para) 조건이라 typeset 가 잡는 케이스를 모두 포함. HWPX 가드를 더하면 정합.

## 4. 단계

### Stage 1 — 컨텍스트 전달 + 산식 적용

- `LayoutEngine` 에 `is_hwpx_source: Cell<bool>` 필드 + `set_hwpx_source()` 추가
- `rendering.rs::find_page` 호출부에서 `source_format` 기준 set 호출
- `typeset.rs::typeset_block_table` 의 ad-hoc LayoutEngine 도 동기화
- `layout.rs:4082-4093` "표 아래 간격" 분기에 HWPX 트리거 추가
- 컴파일 확인 (`cargo build`)

### Stage 2 — 본 페이지 + 회귀 검증

- 본 페이지 SVG 재생성 (`export-svg --debug-overlay -p 7`)
- 표 하단 ↔ "※" 문단 박스 간격 측정 (목표 ≤ 8 px)
- `dump-pages -p 7` items 8 개 유지 확인 (Stage 1 v1 결과 회귀 없음)
- `cargo test --lib` 전수 통과
- `cargo test` (integration) 전수 통과
- golden SVG 회귀: 변경된 SVG 가 있으면 diff 분석 (drift 감소 방향만 허용)

### Stage 3 — 회귀 fix-up + 최종 보고서

- Stage 2 에서 발견된 회귀 (있는 경우) 분석 + 보정
- 최종 보고서 (`task_m100_1147_v2_report.md`) 작성
- Stage 1 / Stage 2 / 최종 보고서 + 산출물 커밋
- 작업지시자 승인 요청 → 머지 단계

## 5. 검증 명령

```bash
# 본 페이지
./target/debug/rhwp export-svg "samples/2. ...hwpx" -o output/svg/task1147_v2/ -p 7 --debug-overlay
./target/debug/rhwp dump-pages "samples/2. ...hwpx" -p 7   # items=8, used~931.5 유지

# 진단
RHWP_TYPESET_DRIFT=1 ./target/debug/rhwp dump-pages "samples/2. ...hwpx" -p 7 2>&1 | grep "pi=127"

# 회귀
cargo test --lib
cargo test
```

## 6. 회귀 모니터링 포인트

- `is_current_empty_para_float` 분기는 비-TAC TopAndBottom + vert_rel_to=Para + 빈 앵커 한정 → HWP5/HWP3 / TAC / Square wrap / 텍스트 있는 앵커는 영향 없음
- HWPX 만 변경되므로 HWP5/HWP3 회귀 (hwpspec.hwp 178 페이지) 는 자동 보존
- 영향 가능 페이지: HWPX 의 wrap=TopAndBottom 빈 앵커 표 직후 문단이 있는 페이지 → 본 페이지 + 유사 패턴 회귀 픽스처가 있을 경우 골든 SVG 변경

## 7. 롤백 계획

- Stage 1 한 커밋, Stage 2 검증 결과만 보고서 별도 커밋
- 회귀 광범위 발견 시 Stage 1 커밋 revert 로 즉시 롤백

## 8. 산출물

- `src/renderer/layout.rs` (필드 + 메서드 + 산식)
- `src/document_core/queries/rendering.rs` (set 호출 1 줄)
- `src/renderer/typeset.rs:3615` (ad-hoc engine 동기화 1 줄)
- `mydocs/working/task_m100_1147_v2_stage1.md`
- `mydocs/working/task_m100_1147_v2_stage2.md`
- `mydocs/report/task_m100_1147_v2_report.md`
- `output/svg/task1147_v2/` (검증용 SVG, .gitignore)
