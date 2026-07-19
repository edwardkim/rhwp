---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 표 페이지 분할 렌더링 및 셀 내 렌더링 수정

## 날짜
2026-02-17

## 증상

### 1. 표 인트라-로우 분할 시 빈 셀 / 중복 텍스트
- `samples/k-water-rfp.hwp` 페이지 015-016에서 표의 행이 두 페이지에 걸쳐 분할될 때:
  - Page 015: 마지막 행이 빈 셀로 표시 (테두리만 있고 텍스트 없음)
  - Page 016: 이전 페이지에서 렌더링된 내용이 중복 표시

### 2. 셀 내 줄간격 미적용
- 폰트 크기 11pt, 줄간격 150% 설정인 셀에서 줄간격이 적용되지 않음
- 일부 표에서는 적용됨 (다중 문단 셀에서만 문제 발생)

### 3. 셀 내 인라인 이미지 위치 오류
- `samples/table-miss.hwp`: [텍스트][엔터][이미지] 구조의 셀에서 이미지가 텍스트 바로 아래가 아닌 셀 하단에 표시
- 텍스트(y=148.8)와 이미지(y=314.6) 사이 약 166px 간격 발생

### 4. 1x1 래퍼 표 과도한 높이
- `samples/k-water-rfp.hwp` 페이지 020: 1x1 외곽 표 > 3x2 내부 표 구조
- HWP 프로그램에서는 3x2 표 하나로 표시되지만, 외곽 표의 저장된 cell.height(844.7px)가 내부 표 실제 높이(410.6px)보다 훨씬 커서 불필요한 공간 차지

## 원인 분석

### 1. 인트라-로우 분할

`min_first_line_height_for_row()` 체크 부재로 한 줄도 들어가지 않는 상황에서 행 분할 시도.

- Page 015: split_end_content_limit=15.7px < 최소 줄 높이(~20px) → 0줄 포함 → 빈 셀
- Page 016: `line_ranges = None`으로 모든 줄 렌더링 → 이전 페이지 내용 중복

### 2. 줄간격

`layout_composed_paragraph`에서 줄간격 제외 조건:
```rust
// 기존 (잘못됨): 모든 문단의 마지막 줄에서 줄간격 제외
if line_idx + 1 < end || cell_ctx.is_none() {
    y += line_height + line_spacing;
} else {
    y += line_height;  // 줄간격 제외
}
```
- 셀에 문단이 여러 개일 때, 각 문단의 마지막 줄마다 줄간격이 제외됨
- 실제로는 셀의 마지막 문단의 마지막 줄에서만 제외해야 함

### 3. 인라인 이미지

셀 내 인라인 이미지(treat_as_char) 문단 구조:
- 문단의 LineSeg.line_height가 이미지 높이를 이미 포함
- `layout_composed_paragraph` 호출 시 para_y가 이미지 높이만큼 전진
- 이후 컨트롤 루프에서 이미지를 전진된 para_y에 배치 → 이미지 높이 이중 계산

```
para_y = 148.8 (compose 전)
↓ layout_composed_paragraph → para_y = 314.6 (이미지 높이 166px 포함)
↓ 컨트롤 루프에서 이미지 배치 → y = 314.6 (잘못됨, 148.8이어야 함)
```

### 4. 1x1 래퍼 표

HWP 파일에서 간혹 표를 1x1 래퍼 표 안에 넣는 구조가 존재.
HWP 스펙의 표 속성(attr)에는 래퍼 식별 플래그가 없음 (bit 0-1: 쪽 나눔, bit 2: 제목 줄 반복만 정의).
외곽 셀의 height가 원본 테이블 높이로 저장되어 있어 과도한 높이 발생.

## 수정 내용

### 1. 인트라-로우 분할 (커밋 eae33ad)

- `height_measurer.rs`: `min_first_line_height_for_row()` 메서드 추가 — 행의 최소 첫 줄 높이 계산
- `pagination.rs`: 분할 전 `avail_content >= min_first_line` 체크 추가
- `layout.rs`: `compute_cell_line_ranges()` 재활성화하여 분할 행의 줄 범위 정확히 계산
- `render_tree.rs`: `TableCellNode.clip: bool` 필드 추가
- `svg.rs`, `web_canvas.rs`: 분할 행 셀에 클리핑 적용

### 2. 줄간격 (커밋 477b882)

- `layout_composed_paragraph`에 `is_last_cell_para: bool` 파라미터 추가 (11개 호출 사이트 갱신)
- 줄간격 제외 조건 변경:
```rust
let is_cell_last_line = is_last_cell_para && line_idx + 1 >= end;
if !is_cell_last_line || cell_ctx.is_none() {
    y += line_height + line_spacing;
}
```
- 셀 높이 계산 3곳에도 동일 로직 적용

### 3. 인라인 이미지 (커밋 477b882)

- `layout_table`, `layout_partial_table` 셀 루프에서 compose 전 para_y를 보존:
```rust
let para_y_before_compose = para_y;
// ... layout_composed_paragraph(para_y) → para_y 전진 ...
// 컨트롤 루프:
let pic_y = if pic.common.treat_as_char {
    para_y_before_compose  // compose 전 위치 사용
} else {
    para_y
};
```
- 인라인 이미지는 LineSeg가 이미 높이를 포함하므로 para_y 추가 전진 불필요

### 4. 1x1 래퍼 표 (현재 작업)

- `layout_table()` 시작부에 래퍼 감지 로직 추가:
```rust
if table.row_count == 1 && table.col_count == 1 && table.cells.len() == 1 {
    // 셀에 보이는 텍스트 없고, Control::Table만 있으면 내부 표로 위임
    return self.layout_table(tree, col_node, nested, ...);
}
```
- `measure_table_impl()`에도 동일 감지 로직 적용 (페이지네이션 높이 정확성)
- 외곽 표의 TableNode, 셀 배경, 테두리 모두 스킵

## 수정 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/renderer/layout.rs` | 줄간격, 인라인 이미지, 1x1 래퍼 감지 |
| `src/renderer/height_measurer.rs` | min_first_line, 1x1 래퍼 감지 |
| `src/renderer/pagination.rs` | 분할 행 최소 높이 체크 |
| `src/renderer/render_tree.rs` | TableCellNode.clip 필드 |
| `src/renderer/svg.rs` | 셀 클리핑 SVG |
| `src/renderer/web_canvas.rs` | 셀 클리핑 Canvas |

## 검증

- 565개 테스트 통과
- k-water-rfp.hwp: 30 → 29페이지 (외곽 표 과도한 높이 해소)
- 페이지 015-017: 분할 표 빈 셀 및 중복 텍스트 해소
- 페이지 020: 1x1 래퍼 제거, 3x2 내부 표만 렌더링
- table-miss.hwp: 이미지가 텍스트 바로 아래에 배치
