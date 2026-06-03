# Task #604 Stage 2b — `wrap_precomputed` 필드 제거 + HWP3 후처리 청산

## 본 단계 목표

Stage 2 의 wrap_anchors 메타데이터 채널 도입 후, **`Paragraph.wrap_precomputed` 필드
제거** + HWP3 파서 후처리 30 LOC 청산. typeset.rs 매칭 분기를 anchor 종류 (Picture vs
Table) 기반으로 본질화.

## 변경 영역

### A. `src/renderer/typeset.rs:495~` 매칭 분기 본질화

기존 `if para.wrap_precomputed` 분기를 anchor paragraph 의 controls 검사로 교체:

```rust
let anchor_is_picture = paragraphs.get(st.wrap_around_table_para)
    .map(|p| p.controls.iter().any(|c| match c {
        Control::Picture(pic) => !pic.common.treat_as_char,
        Control::Shape(s) => {
            if let crate::model::shape::ShapeObject::Picture(pic) = s.as_ref() {
                !pic.common.treat_as_char
            } else { false }
        }
        _ => false,
    }))
    .unwrap_or(false);
if anchor_is_picture {
    // Picture anchor: wrap_anchors 등록 + FullParagraph 통과
    st.current_column_wrap_anchors.insert(...);
} else {
    // Table anchor: 어울림 문단 흡수 (기존 흐름)
    st.current_column_wrap_around_paras.push(...);
    continue;
}
```

본질: HWP3 wrap_precomputed 플래그가 표시했던 "LineSeg cs/sw 사전 인코딩" 의 본질은
**anchor 가 그림 (Picture) Square wrap 인 경우**. anchor 의 종류로 직접 판정 가능.

### B. `src/model/paragraph.rs` `wrap_precomputed` 필드 제거

- struct 정의 (`pub wrap_precomputed: bool`) 제거
- `Paragraph::clone_text_subset` 의 `wrap_precomputed: false` 초기화 제거

### C. `src/parser/hwp3/mod.rs` 후처리 30 LOC 제거

PR #589 보완6/8 의 wrap_precomputed 후처리 (LineSeg vertical_pos==0 패턴 검출 + multi/single
LineSeg 분기) 전체 제거. typeset.rs 매칭 + ColumnContent.wrap_anchors 메타데이터 채널로
정합 대체됨.

### D. 잔존 주석 정리

- `src/renderer/layout.rs:2984, 3374`: "Task #460 보완6의 wrap_precomputed IR 플래그로"
  → "Task #604 Stage 2 의 wrap_anchors 메타데이터 채널로"
- `src/renderer/pagination.rs:133`: "현 wrap_precomputed 메커니즘 대체" → "PR #589 ..."
- `src/renderer/layout/paragraph_layout.rs:910`: 주석 갱신

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo build` + `cargo build --tests` | ✅ 통과 |
| `cargo test --lib` | ✅ **1130 passed** / 0 failed / 2 ignored |
| `cargo clippy --lib -- -D warnings` | ✅ 0건 |
| `cargo test --test issue_546` (Task #546) | ✅ 1 passed (exam_science 4페이지) |
| `cargo test --test issue_554` (HWP3 변환본) | ✅ 12 passed |
| `cargo test` (통합 31) | ✅ 모두 통과 |

## LOC 합계

| 파일 | 변경 |
|------|-----|
| `src/renderer/typeset.rs` | -10 / +18 (매칭 분기 본질화 — Picture vs Table) |
| `src/model/paragraph.rs` | -10 / 0 (wrap_precomputed 필드 + init 제거) |
| `src/parser/hwp3/mod.rs` | -27 / +5 (후처리 청산, 주석 5 LOC) |
| `src/renderer/layout.rs` | -4 / +4 (주석 갱신 2곳) |
| `src/renderer/pagination.rs` | -1 / +1 (주석) |
| `src/renderer/layout/paragraph_layout.rs` | -1 / +2 (주석) |
| **소스 합계** | **-53 / +30 (-23 LOC)** |

## IR 부채 청산 본질

**Before (PR #589 + Stage 2):**
- IR 에 `Paragraph.wrap_precomputed: bool` 필드 (HWP3 휴리스틱 누설)
- HWP3 파서 후처리 30 LOC (LineSeg 패턴 검출)
- typeset 의 매칭 분기 조건 = `para.wrap_precomputed`

**After (Stage 2b):**
- IR 의 `wrap_precomputed` 필드 제거 — 포맷 독립성 회복
- HWP3 파서 후처리 청산 — IR 가 HWP5 origin 본질에 정합
- typeset 의 매칭 분기 = anchor controls 검사 (Picture vs Table) — 본질적 판정

## CLAUDE.md HWP3 파서 규칙 정합

- HWP3 휴리스틱 (vertical_pos==0 패턴) 의 IR 누설 청산 → IR 의 포맷 독립성 회복
- typeset/layout 의 wrap zone 처리는 모든 포맷 일관 동작 (anchor 종류 + LineSeg cs/sw)

## 잔존 영역 (Stage 3 본격)

- HWP3 파서 cs/sw 인코딩 결함 (Issue #604 본질): pi=75 첫 3 줄 cs=0/sw=0
- Stage 3 에서 `mod.rs:1399~` wrap zone pgy 범위 검사 정정

## 작업지시자 승인 요청

본 Stage 2b (IR 부채 청산) 완료 보고. 다음 단계 (Stage 3: HWP3 파서 cs/sw 인코딩 정정,
Issue #604 결함 본질 정정) 진입 승인 요청.

## 참조

- 수행계획서: `mydocs/plans/task_m100_604.md`
- 구현계획서: `mydocs/plans/task_m100_604_impl.md`
- LineSeg 표준: `mydocs/tech/document_ir_lineseg_standard.md`
- Stage 1 보고서: `mydocs/working/task_m100_604_stage1.md`
- Stage 2 보고서: `mydocs/working/task_m100_604_stage2.md`
- Issue: #604
