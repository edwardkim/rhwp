# Task #460 Stage 7 완료 보고서: HWP3 비-TAC 그림 LAYOUT_OVERFLOW 해소 (파서 전용)

## 개요

HWP3 문서에서 비-TAC TopAndBottom 그림 문단(pi=76, pi=78, pi=97)이
`LAYOUT_OVERFLOW` 경고를 발생시키던 현상을 `src/parser/hwp3/` 디렉토리만 수정하여 해소하였다.

렌더러(`layout.rs`, `typeset.rs`, `paragraph_layout.rs`) 변경 없음.

## 배경

- **HEAD (Stage 6)**:
  - `LAYOUT_OVERFLOW: para=76, overflow=336.5px`
  - `LAYOUT_OVERFLOW: para=97, overflow=201.4px`
  - Stage 2 (`fixup_hwp3_mixed_para_line_segs`) 및 Stage 6 (break_flag 기반
    `column_type=Page`)이 적용되어 있었으나 두 OVERFLOW 미해소 상태였음.

## 원인 분석

### Stage 2 함수의 부작용
Stage 2 `fixup_hwp3_mixed_para_line_segs()`는 혼합 단락에서 LINE_SEG 하나의
`line_height`를 그림 하단까지 확장하고 `text_height=0`으로 설정하였다.
이로 인해 렌더러의 `advance = line_height + line_spacing` 계산에서
그림 높이만큼의 LINE_SEG가 추가로 누적되어 OVERFLOW 유발.

### border_fill_id 버그 (Stage 4 도입)
표 셀의 `border_fill_id`가 `doc_border_fills.push()` 후 `len()`(1-기반)으로 설정되어
0-기반 `Vec::get()` 조회 시 항상 `None`이 반환되는 버그.

## 수정 내용

**파일**: `src/parser/hwp3/mod.rs`

### 1. border_fill_id 버그 수정
```rust
// 수정 전 (버그):
cell.border_fill_id = doc_border_fills.len() as u16;
// 수정 후:
cell.border_fill_id = (doc_border_fills.len() - 1) as u16;
```
push 후 `len()-1`이 새 요소의 0-기반 인덱스.

### 2. Stage 2 제거 (fixup_hwp3_mixed_para_line_segs)
LINE_SEG 높이를 그림 하단까지 확장하는 함수를 제거하였다.
이 함수가 OVERFLOW의 실제 원인이었으며, 제거 후 pi=76, pi=97 OVERFLOW 해소.

### 3. Stage 6 제거 (break_flag 기반 column_type)
break_flag bit 0만 확인하는 기존 Stage 6 코드를 제거하고 Fix 2로 대체.

### 4. Fix 1: 자리차지 LINE_SEG 방어 코드
HWP3 원본 바이너리에 혹시 `th=0, lh≈그림높이`인 자리차지 LINE_SEG가 있을 경우를
대비하여 이를 제거하는 방어 코드 추가:
```rust
para.line_segs.retain(|seg| {
    !(seg.text_height == 0
        && non_tac_pic_heights.iter().any(|&h| (seg.line_height as i32 - h).abs() < 1000))
});
```

### 5. Fix 2: 단일 LINE_SEG 비-TAC TopAndBottom 그림 문단 → 새 페이지
typeset.rs `pushdown_h` (Task #409 v2)는 모든 문서 포맷에서 비-TAC TopAndBottom
그림 높이를 `current_height`에 추가한다. HWP3 단일 LINE_SEG 문단은 텍스트 높이
(21px)만으로 페이지 배치를 결정하므로 pushdown_h 후 OVERFLOW 발생.
파서에서 `column_type=Page`를 설정하여 TypesetEngine이 새 페이지에서 시작하게 함:
```rust
if has_non_tac_float_pic && para.line_segs.len() == 1 {
    para.column_type = crate::model::paragraph::ColumnBreakType::Page;
}
```

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 1068 passed, 0 failed |
| `cargo test` (전체) | 1068 lib + 6 svg_snapshot + 1 tab_cross_run = 전부 통과 |
| `cargo build --release` | 성공 |
| hwp3-sample.hwp SVG 내보내기 | **17페이지** 정상 완료 |
| `LAYOUT_OVERFLOW` (TypesetEngine 오버플로) | **0건** (pi=76, 78, 97 모두 해소) |
| 잔존 `LAYOUT_OVERFLOW_DRAW` 경고 | 꼬리말 36px 초과 — 기존 잔존 경고만 |
| 렌더러 수정 | **없음** (hwp3/ 디렉토리만) |

### 페이지 수 변화

HEAD (Stage 6) 16페이지 → 17페이지.
Stage 2 제거로 pi=76 단락이 짧아지고 Fix 2로 pi=78, pi=97, pi=41이 각각 새 페이지
시작 → 그림 OVERFLOW 없이 올바르게 배치됨.

## 수정 파일

| 파일 | 변경 내용 |
|------|---------|
| `src/parser/hwp3/mod.rs` | border_fill_id 버그 수정 + Stage 2 제거 + Stage 6 교체 + Fix 1/2 추가 |
