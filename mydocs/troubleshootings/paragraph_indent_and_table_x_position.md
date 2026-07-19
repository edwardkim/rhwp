---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 문단 들여쓰기/내어쓰기 모델 및 표 x 위치 수정

## 날짜
2026-02-17

## 증상

### 1. 문단 텍스트 x 위치 오류
- 들여쓰기/내어쓰기가 적용된 문단에서 첫줄과 다음줄의 x 시작 위치가 반대로 적용됨
- 예: 내어쓰기 문단의 "1)" 번호가 본문 여백 위치가 아닌 들여쓰기된 위치에 렌더링

### 2. 표 x 위치 오류
- 표가 항상 body left (x=80)에 위치하여 호스트 문단의 여백이 반영되지 않음
- PartialTable(페이지 분할된 표)에 host_margin_left가 전달되지 않음

## 원인 분석

### HWP 문단 들여쓰기/내어쓰기 모델

문단이 새로 시작하면 문단의 여백(`margin_left`)은 무조건 적용된다.

| 모드 | 첫줄 (시작줄) | 다음줄 (2줄~) |
|------|-------------|-------------|
| **보통** (indent=0) | margin_left | margin_left |
| **들여쓰기** (indent>0) | margin_left + indent | margin_left |
| **내어쓰기** (indent<0) | margin_left | margin_left + \|indent\| |

- **들여쓰기**: 첫줄만 지정한 pt만큼 오른쪽으로 여백 추가, 다음줄부터는 문단 여백만 적용
- **내어쓰기**: 첫줄은 문단 여백만 적용, 두 번째 줄부터 지정한 pt만큼 오른쪽 여백 추가
- 내어쓰기의 indent 값은 HWP 내부적으로 음수로 저장됨

### 실제 데이터 예시 (k-water-rfp.hwp 5페이지)

| 줄 | pi | psid | margin_left | indent | 모드 | 첫줄 x | 내용 |
|----|-----|------|-------------|--------|------|--------|------|
| 1 | 45 | 81 | 1.33 | 0.00 | 보통 | 81.33 | "1.2. 제안참가안내" |
| 2 | 47 | 2 | 0.00 | 0.00 | 보통 | 80.00 | "  가. 제안참가신청" |
| 3 | 48 | 82 | 46.67 | -22.61 | 내어쓰기 | 126.67 | "1) 제출기한..." |
| 4 | 49 | 82 | 46.67 | -22.61 | 내어쓰기 | 126.67 | "2) 제안서..." |
| 5 | 50 | 83 | 60.00 | -23.71 | 내어쓰기 | 140.00 | "가) 제출할..." |
| 6 | 51 | 84 | 46.67 | -23.71 | 내어쓰기 | 126.67 | "- 입찰에..." |
| **표** | **52** | **75** | **46.67** | **0.00** | **보통** | **126.67** | **[표]** |

- 표를 삽입한 문단(pi=52)은 윗줄(pi=51)과 다른 스타일(psid=75)이지만 동일한 margin_left(46.67)를 상속
- 표의 indent는 0이므로 margin_left만 적용 → 표와 "1)", "2)" 텍스트가 같은 x 위치에 정렬

### 표 x 위치 계산

표의 수평 위치는 CommonObjAttr 비트 필드로 결정된다:

```
attr 비트 필드:
  bit 0:     treat_as_char (글자처럼 취급)
  bit 8-9:   horz_rel_to (가로 기준: Paper/Page/Column/Para)
  bit 10-12: horz_align (가로 정렬: Left/Center/Right/Inside/Outside)

raw_ctrl_data 레이아웃:
  [0..4]:  attr (u32)
  [4..8]:  h_offset (i32, HWPUNIT)
```

가로 기준 영역:
- `HorzRelTo::Para` → col_area.x + host_margin_left
- `HorzRelTo::Column` 등 → col_area.x

## 수정 내용

### 1. 문단 텍스트 indent (`layout.rs` layout_composed_paragraph)

**수정 전 (잘못됨):**
```rust
// 첫줄에 indent 적용 (내어쓰기일 때 첫줄이 왼쪽으로 이동)
let line_indent = if line_idx == 0 { indent } else { 0.0 };
```

**수정 후:**
```rust
let line_indent = if indent > 0.0 {
    // 들여쓰기: 첫줄만 추가
    if line_idx == 0 { indent } else { 0.0 }
} else if indent < 0.0 {
    // 내어쓰기: 다음줄부터 |indent| 오른쪽 추가
    if line_idx == 0 { 0.0 } else { indent.abs() }
} else {
    0.0
};
let effective_margin_left = margin_left + line_indent;
```

### 2. 표 x 위치 — CommonObjAttr 기반 (`layout.rs` layout_table)

**수정 전:**
```rust
// 문단 정렬(Alignment) 기반 — 부정확
let table_x = match host_alignment {
    Alignment::Center => col_area.x + (col_area.width - table_width) / 2.0,
    _ => col_area.x,
};
```

**수정 후:**
```rust
// CommonObjAttr 비트 필드에서 horz_rel_to, horz_align, h_offset 파싱
let (ref_x, ref_w) = match horz_rel_to {
    HorzRelTo::Para => (col_area.x + host_margin_left, col_area.width - host_margin_left),
    _ => (col_area.x, col_area.width),
};
let table_x = match horz_align {
    HorzAlign::Left => ref_x + h_offset,
    HorzAlign::Center => ref_x + (ref_w - table_width).max(0.0) / 2.0 + h_offset,
    HorzAlign::Right => ref_x + (ref_w - table_width).max(0.0) + h_offset,
};
```

### 3. PartialTable에도 동일 로직 적용

- `layout_partial_table` 함수에 `host_margin_left` 매개변수 추가
- `PageItem::PartialTable` 핸들러에서 호스트 문단의 effective_margin 계산하여 전달
- 매개변수 순서 주의: `split_end_content_limit` 뒤에 `host_margin_left` 배치

## 관련 파일

| 파일 | 변경 내용 |
|------|---------|
| `src/renderer/layout.rs` | indent 모델 수정, 표 x 위치 계산, PartialTable host_margin_left 전달 |

## 교훈

1. HWP의 들여쓰기/내어쓰기는 **시작줄이 아닌 다음줄**의 x축 시작 위치를 결정한다
2. 표 위치는 문단 정렬(Alignment)이 아닌 **CommonObjAttr 비트 필드**(horz_rel_to, horz_align, h_offset)로 결정된다
3. 표의 호스트 문단의 **margin_left**가 표 위치에 반영되어야 한다 (HorzRelTo::Para일 때)
4. 함수 매개변수 추가 시 기존 매개변수 사이에 삽입하면 값이 뒤섞일 수 있으므로 **마지막에 추가**하는 것이 안전하다
