# Task #332: typeset/layout drift 통합 — 구현 계획서

- **수행계획서**: `task_m100_332.md`
- **브랜치**: `task332`
- **작성일**: 2026-04-25

---

## 코드 위치 매핑 (조사 결과)

| 단계 | 파일·라인 | 현재 코드 |
|------|-----------|-----------|
| Stage 1 | `src/renderer/typeset.rs:612, 622` | `st.current_height += fmt.total_height` (fit 검사 후 누적) |
| Stage 1 보조 | `src/renderer/typeset.rs:1059, 1090` | TAC/표 advance 누적 — 동일 모델 통합 검토 |
| Stage 2 | `src/renderer/layout/paragraph_layout.rs` (advance 누적부) | per-paragraph `y` 진행. `paragraph_layout.rs` 의 line_spacing 누적 위치 식별 필요 |
| Stage 3 | `src/renderer/layout.rs:1367, 1389, 1392-1395` | `vpos_end = vp + lh + ls`, `end_y >= y_offset - 1.0` 단방향 가드 |
| Stage 4 | `src/renderer/layout/paragraph_layout.rs:807-816, 2529-2542` | `text_y = (col_bottom - line_height).max(col_area.y)` clamp pile |
| Stage 5 | `src/renderer/height_measurer.rs`, `src/renderer/typeset.rs:800-1140` | 표 매니저(HeightMeasurer) 와 typeset 의 `total_height` / TAC seg 측정 정합 |

## 설계 원칙

- 각 단계는 **단일 PR-급 commit** 으로 분리 (revert 단위)
- 각 단계 commit 직전 `cargo test --lib` + golden SVG 6 개 실행
- golden SVG baseline 갱신은 단계별 보고서에 diff 첨부 후 승인 필요
- 회귀 가능성 높은 단계(3,4,5) 는 단계 내 sub-step 으로 더 잘게 분할

## 상세 구현 단계

### Stage 1 — typeset advance 를 `height_for_fit` 로 변경

**변경**:

```rust
// typeset.rs:612, 622
- st.current_height += fmt.total_height;
+ st.current_height += fmt.height_for_fit;
```

- TAC/표 경로(`typeset.rs:1059, 1089-1090`) 도 동일하게 `height_for_fit` 모델로 정렬 검토. 단, 표는 `effective_height` 가 trail_ls 와 무관하므로 host paragraph 의 `total_height - height_for_fit` 보정 로직(`1089-1090`)이 의미를 잃을 수 있음 — **별도 sub-step 1b** 로 분리해 검증.
- `format_paragraph` 의 `height_for_fit` 정의는 그대로 유지(`total_height - trailing_ls`).

**검증**:

- 단독 적용 후 21_언어 page 1 col 1 에서 pi=10 partial / pi=26 글자 겹침 발생 여부 측정
- `cargo test --lib` (Task #331 시도에서 5 개 calibration 필요했음 — 동일 가능)
- golden 6 개 baseline 갱신

**커밋**: `Task #332: stage1 typeset advance 를 height_for_fit 기반으로 변경`

---

### Stage 2 — layout per-paragraph advance 정합

**조사 필요**: `paragraph_layout.rs` 에서 paragraph 단위 y 누적이 일어나는 위치(아마 마지막 line 후 trailing line_spacing 을 더하는 부분). Stage 2 시작 시 grep 으로 pin-point.

**변경**:

- 마지막 visible 줄에서 trail_ls 를 advance 누적에서 제외
- 단, 현재 줄 자체의 y 좌표는 유지 (그림 위치는 그대로, 다음 문단 시작점만 shift)

**검증**:

- 단독으로는 vpos correction(`layout.rs:1392`) 이 다시 ls 를 더해 효과 상쇄 가능 → Stage 3 와 함께 검증
- 본 단계 commit 시점에서는 collapse 회귀 측정만

**커밋**: `Task #332: stage2 layout per-paragraph advance 를 height_for_fit 와 정합`

---

### Stage 3 — vpos correction 양방향 + collapse 가드

**변경 (`layout.rs:1389-1396`)**:

```rust
let end_y = col_area.y + hwpunit_to_px(vpos_end - base, self.dpi);
// 양방향 보정 + collapse 가드
let min_advance = MIN_LINE_ADVANCE_PX;  // 인접 문단 간 최소 advance
if end_y >= col_area.y && end_y <= col_area.y + col_area.height
    && (end_y - y_offset).abs() <= MAX_VPOS_DRIFT_PX  // 비정상적 점프 방지
    && end_y >= prev_paragraph_y + min_advance  // collapse 방지
{
    y_offset = end_y;
}
```

또한 `vpos_end` 에서 trail_ls 제외 (`vpos_end = vp + lh`):

```rust
- let vpos_end = seg.vertical_pos + seg.line_height + seg.line_spacing;
+ let vpos_end = seg.vertical_pos + seg.line_height;
```

**Sub-step 분할**:
- 3a: `vpos_end` 의 trail_ls 제외 (단방향 유지)
- 3b: 양방향 보정 + collapse 가드

**검증 (각 sub-step 별)**:
- 21_언어 col 1 collapse 케이스 (이전 시도 회귀)
- form-002, multi-table, tac-case 회귀
- golden 6 개

**커밋**: `Task #332: stage3a vpos_end 에서 trail_ls 제외`, `Task #332: stage3b vpos correction 양방향 + collapse 가드`

---

### Stage 4 — clamp pile 버그 수정

**변경 (`paragraph_layout.rs:807-816`)**:

기존 clamp → **skip rendering**:

```rust
let col_bottom = col_area.y + col_area.height;
if cell_ctx.is_none() && text_y + line_height > col_bottom + 0.5 {
    // 단 하단을 넘어가는 라인은 그리지 않음.
    // typeset 측 fit 산정이 정합된 상태에서는 이 분기에 들어오는 빈도가 0 이어야 함.
    log::warn!(
        "LAYOUT_OVERFLOW_SKIP: section={} pi={} line={} y={:.1} col_bottom={:.1}",
        section_index, para_index, line_idx, text_y + line_height, col_bottom
    );
    continue;  // 또는 break, 후속 라인 처리 검토
}
```

- `2529-2542` 의 두 번째 clamp 도 동일 정책 적용
- typeset overflow signal 경로는 본 task 범위 외(Plan B). 본 단계는 stop drawing + warn 로그까지.

**검증**:
- Stages 1~3 적용된 상태에서 모든 샘플 export-svg 시 `LAYOUT_OVERFLOW_SKIP` 발생 빈도 측정 → 0 이어야 함
- 글자 겹침 0 (pi=10 케이스 해결)
- Task #321 v5/v6 의 multi-table/tac-case 회귀

**커밋**: `Task #332: stage4 clamp pile 제거, overflow 시 stop drawing`

---

### Stage 5 — header (표/Shape) 측정 통합

**조사 우선**: `height_measurer.rs` 의 측정 결과와 `typeset.rs` 의 host paragraph + table 누적 결과 간 차이 정량화.

**변경 후보**:
- HeightMeasurer 가 typeset 과 동일한 `height_for_fit` / `total_height` 분리 제공
- 표·Shape 의 outer_margin 계산 일치
- typeset 의 `cur_h` jump 로직(`typeset.rs:1122-1140`) 이 layout 의 vpos correction base 와 동기화

**Sub-step 분할 (예상)**:
- 5a: HeightMeasurer 와 typeset 의 단순 표 advance 정합 (margin/wrap)
- 5b: TAC seg 케이스 정합
- 5c: Shape (TopAndBottom) 측정 정합

**검증**:
- 21_언어 page 1 col 1 의 pi=26 + 보기 ①②③ fit (PDF 일치) ← Task #331 원 의도
- 표가 있는 모든 샘플 회귀
- golden 6 개

**범위 결정 가능성**: Stage 5 가 표 매니저 API 변경 동반하면 sub-task 분기 → 사용자 승인 후 별도 이슈로.

**커밋**: `Task #332: stage5 header(표/Shape) 측정 통합`

---

## 단계 간 의존성

```
Stage 1 (typeset)
   │
   ├─ 단독 적용 시 layout drift 확대 가능 → 보고서에 측정 결과 기재
   ▼
Stage 2 (layout per-para)
   │
   ├─ vpos correction 와 충돌 → Stage 3 와 함께 검증
   ▼
Stage 3 (vpos correction)
   │   3a → 3b 순서
   ▼
Stage 4 (clamp pile)  ← Stages 1~3 정합되면 trigger 빈도 0
   │
   ▼
Stage 5 (header)  ← Task #331 원 의도 자연 해결
```

## 회귀 테스트 셋

각 단계 commit 전 실행:

```bash
cargo test --lib                                              # 992 passed 유지/calibration
cargo test --test golden_svg                                  # golden 6 개
cargo run --release -- export-svg samples/21_언어_기출_편집가능본.hwp -p 0 --debug-overlay  # 핵심 케이스
cargo run --release -- export-svg samples/form-002.hwp        # multi-table
# 추가 회귀 샘플은 단계별로 식별
```

## 산출물

- 단계별 commit 7개 (Stage 1, 2, 3a, 3b, 4, 5a/5b/5c — 5 는 sub-step 수에 따라 변동)
- `mydocs/working/task_m100_332_stage{1..5}.md` 5개 보고서
- `mydocs/report/task_m100_332_report.md` 최종 보고서

## 미해결 질문

1. Stage 5 가 단일 단계로 처리 가능한가, 별도 task 로 분기해야 하는가? — Stage 5 시작 시 조사 후 결정
2. typeset overflow signal (Plan B) 을 Stage 4 에 부분 도입할 가치가 있는가? — Stages 1~3 정합 후 overflow 빈도 0 이면 불필요
3. golden SVG baseline 갱신 정책 — 의도된 변경인지 회귀인지 단계별 사용자 승인
