# Task #1147 v2 최종 결과 보고서 — 렌더러 layout.rs 측 HWPX TopAndBottom 표 후행 line_spacing 보정

- **상위 이슈**: https://github.com/edwardkim/rhwp/issues/1147
- **브랜치**: `feature/task_m100_1147` (Task #1147 Stage 1 위에 v2 누적)
- **선행 보고서**: [task_m100_1147_report.md](task_m100_1147_report.md) (v1, typeset 측)
- **수행계획서**: [task_m100_1147_v2.md](../plans/task_m100_1147_v2.md)
- **구현계획서**: [task_m100_1147_v2_impl.md](../plans/task_m100_1147_v2_impl.md)
- **Stage 보고서**: [v2_stage1](../working/task_m100_1147_v2_stage1.md), [v2_stage2](../working/task_m100_1147_v2_stage2.md)

## 1. 문제 요약

Task #1147 v1 (commit `695ed980`) 의 typeset 측 보정 후, **페이지네이션은 권위 PDF 와 정합** (한 줄 문단이 본 페이지 하단에 정상 배치) 되었으나, 작업지시자 시각 검수 결과 본 페이지의 표와 직후 한 줄 문단 사이 **시각 간격이 18 px 과도** (권위 PDF 약 0~5 px).

## 2. 근본 원인

`src/renderer/layout.rs::layout_table_item` 의 "표 아래 간격" 분기 (라인 4082-4093) 가 빈 앵커 float (`is_current_empty_para_float == true`) 케이스에서 앵커의 last `line_seg.line_spacing` (1352 HU = **18.03 px**) 을 그대로 가산.

Task #1147 v1 의 typeset 측 보정 (`host_line_spacing = 0 for is_topbottom_empty_anchor_hwpx`) 과 시멘틱 불일치:
- typeset (페이지네이션 결정 cur_h) → +0 px
- layout (실제 렌더 좌표 y_offset) → +18 px

→ 표 직후 문단이 typeset 의 예상 위치보다 18 px 아래로 밀려나 렌더됨.

## 3. 변경 내용

### `src/renderer/layout.rs`

- `LayoutEngine.is_hwpx_source: Cell<bool>` 필드 신설 (기본 false)
- `LayoutEngine::new()` 초기화 추가
- `LayoutEngine::set_hwpx_source(enabled: bool)` 메서드 신설
- `layout_table_item` 의 "표 아래 간격" 분기에 HWPX 한정 트리거 추가:
  ```rust
  let is_topbottom_empty_anchor_hwpx =
      self.is_hwpx_source.get() && is_current_empty_para_float;
  let gap = if is_topbottom_empty_anchor_hwpx {
      0
  } else if is_current_empty_para_float {
      seg.line_spacing.max(0)
  } else if seg.line_spacing > 0 {
      seg.line_spacing
  } else {
      seg.line_height
  };
  ```

### `src/document_core/queries/rendering.rs`

- `find_page` 호출부 (set_hwp3_origin_flow_spacing_before 직후) 에 1 줄 추가:
  ```rust
  self.layout_engine
      .set_hwpx_source(matches!(self.source_format, crate::parser::FileFormat::Hwpx));
  ```

### `src/renderer/typeset.rs`

- `typeset_block_table` 의 ad-hoc LayoutEngine (advance_row_cut 측정용) 도 동기화:
  ```rust
  layout_engine.set_hwp3_variant(st.is_hwp3_variant);
  layout_engine.set_hwpx_source(st.is_hwpx_source);  // 신규
  ```

## 4. 검증 결과

### 본 페이지 (HWPX, section 0 page_num=5)

| 항목 | v1 후 (변경 전) | v2 후 (변경 후) |
|------|----------------|----------------|
| 표 9×18 하단 abs_y | 1000.03 | 1000.03 |
| "※ 추진일정은…" 박스 상단 abs_y | 1018.05 | **1006.45** |
| 표↔문단 간격 | 18.0 px | **6.4 px** |
| 페이지 items | 8 | **8** (유지) |
| 페이지 used | 931.5 px | **931.5 px** (유지) |

권위 PDF (한컴 2022 출력) 와 시각 정합. 작업지시자 지적 해소.

### 회귀

| 영역 | 결과 |
|------|------|
| `cargo test --lib` | **1411 passed, 0 failed** |
| `cargo test --tests` (integration) | 전수 통과 |
| golden SVG (svg_snapshot 8 케이스) | 전부 통과, **회귀 없음** |
| HWPX (aift.hwpx) drift 분포 | 정상 범위 (대부분 0~-10 px), 본 변경 영향 페이지 없음 |
| HWP5/HWP3 | 트리거 격리로 영향 없음 |

## 5. 회귀 격리 근거

본 변경은 다음 조건이 모두 충족될 때만 발동:
1. `self.is_hwpx_source.get() == true` (HWPX 원본만)
2. `is_current_empty_para_float == true`:
   - `!treat_as_char`
   - `wrap == TopAndBottom`
   - `vert_rel_to == Para`
   - 앵커 문단에 가시 텍스트 없음

→ HWP5/HWP3, TAC 표, Square wrap, vert_rel_to=Page/Paper, 텍스트 있는 앵커 케이스는 모두 기존 동작 보존.

## 6. typeset/layout 시멘틱 정합

| 분기 조건 | typeset (host_line_spacing) | layout (gap) |
|-----------|----------------------------|--------------|
| HWPX 빈 앵커 TopAndBottom 비-TAC | **0** ✓ | **0** ✓ (이번 변경) |
| HWPX 텍스트 있는 앵커 TopAndBottom 비-TAC | seg.line_spacing | seg.line_spacing |
| HWP5/HWP3 빈 앵커 TopAndBottom 비-TAC | seg.line_spacing | seg.line_spacing.max(0) |

이로써 typeset 의 `cur_h` 와 layout 의 `y_offset` 가 표 직후 문단 진입 시 시멘틱 정합.

## 7. 남은 / 후속 이슈

본 타스크에서 다루지 않은 latent 이슈 (Task #1147 v1 보고서 5장 동일):
- TAC 1×1 표 단독 페이지 +42.9px 드리프트
- 일부 페이지의 음수 본문 텍스트 라인 드리프트 (-5 ~ -13)
- aift.hwpx p18 +21.8 px (다른 케이스)

## 8. 커밋 / 머지 단계

- v2 단일 커밋: `Task #1147 v2: layout.rs HWPX TopAndBottom 빈 앵커 표 후행 line_spacing 보정`
- `feature/task_m100_1147` → `local/devel` 머지 → `devel` 머지 (작업지시자 승인 후)
