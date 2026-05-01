# Task #460 Stage 6 완료 보고서: 페이지에 걸친 HWP3 대형 그림 anchor 처리 (파서)

## 개요

HWP3 문서에서 대형 비-TAC 그림(bin_id=2, 533.6×474.4px)이 anchor 문단(pi=41)과 함께
페이지 2 하단에서 페이지 3으로 분할 표시되던 현상을 수정하였다.

## 접근 방식

**HWP5/HWPX 방식 분석**: HWP5/HWPX에서는 word processor가 레이아웃을 사전 계산하여
LINE_SEG `vertical_pos`를 페이지별 상대 좌표로 저장한다. 새 페이지에 배치된 문단의
vpos가 0에 가까워지면 TypesetEngine의 vpos-reset 감지가 쪽 나눔을 처리한다.

**HWP3 binary 분석**: HWP3 binary의 LINE_SEG `break_flag`에 이미 word processor의
레이아웃 결정이 인코딩되어 있다:
- `break_flag & 0x8000 != 0 && break_flag & 0x0001 != 0` → 새 페이지 시작

pi=41의 첫 LINE_SEG: `tag=0x00060003` (bit 0 = 첫 페이지 경계, bit 1 = 첫 단 경계).
HWP3 word processor가 이 문단을 페이지 3에 배치했음을 이미 기록해두었다.

## 수정 내용

**파일**: `src/parser/hwp3/mod.rs`

비-TAC TopAndBottom 그림을 포함한 문단에서, 첫 LINE_SEG의 break_flag bit 0이 설정된
경우 `column_type = ColumnBreakType::Page`로 설정:

```rust
// HWP3 쪽 경계: 비-TAC TopAndBottom 그림을 가진 문단에서
// 첫 LINE_SEG break_flag bit 0 = HWP3 word processor가 새 페이지에 배치했음.
// → column_type = Page로 변환하여 TypesetEngine이 자연스럽게 처리하게 함.
// 단순 쪽 넘김(그림 없는 heading 등)은 TypesetEngine 높이 측정으로 처리되므로 제외.
let has_non_tac_float_pic = para.controls.iter().any(|c| {
    matches!(c, crate::model::control::Control::Picture(pic)
        if !pic.common.treat_as_char
            && matches!(pic.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom))
});
if has_non_tac_float_pic {
    if let Some(first_seg) = para.line_segs.first() {
        if first_seg.tag & 0x01 != 0 {
            para.column_type = crate::model::paragraph::ColumnBreakType::Page;
        }
    }
}
```

**필터 조건**:
- 비-TAC TopAndBottom 그림이 있는 문단 (단순 쪽 넘김 heading 제외)
- 첫 LINE_SEG break_flag bit 0 (`tag & 0x01 != 0`)

**제거된 렌더러 수정** (잘못된 접근):
- `src/renderer/typeset.rs`: 비-TAC 그림 선제적 쪽 나눔 (이전 Stage 6 수정 롤백)
- `src/renderer/pagination/engine.rs`: 동일 로직 dead-code 제거

## 핵심 발견

### HWP3 break_flag = 레이아웃 사전 계산 정보

HWP3 binary는 word processor가 이미 레이아웃을 계산한 결과를 LINE_SEG `break_flag`에
저장한다. 이것은 HWP5/HWPX가 `vertical_pos`를 페이지별 좌표로 저장하는 방식과 동일한
철학이다. 파서에서 이를 `column_type`으로 변환하면 렌더러 수정 없이 올바른 배치가 가능하다.

### 필터링의 필요성

다른 heading 문단(pi=28 "1. 소개", pi=47 "2. 연관된 작업들" 등)도 `tag=0x00060003`을
가지지만, 이들은 TypesetEngine의 높이 측정으로 충분히 처리된다. 그림이 있는 문단에만
break_flag를 적용하면 페이지 수가 16으로 유지되면서 그림 anchor가 올바르게 처리된다.

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 1068 passed, 0 failed |
| `cargo test` (전체) | 1068 lib + 6 svg_snapshot + 1 tab_cross_run = 전부 통과 |
| `cargo build --release` | 성공 |
| hwp3-sample.hwp SVG 내보내기 | 16페이지 정상 완료 |
| 페이지 2 pi=41 제거 | 확인 — pi=40까지만 |
| 페이지 3 pi=41 + Shape | 확인 — 그림 533.6×474.4px 페이지 상단 |
| 기존 LAYOUT_OVERFLOW_DRAW 경고 | 꼬리말 36px 초과 — 기존 잔존 경고만 |

## 수정 파일

| 파일 | 변경 내용 |
|------|---------|
| `src/parser/hwp3/mod.rs` | break_flag → column_type = Page 변환 (비-TAC 그림 문단) |
| `src/renderer/typeset.rs` | 이전 renderer 수정 롤백 (-27줄) |
| `src/renderer/pagination/engine.rs` | dead-code 제거 (-45줄) |
