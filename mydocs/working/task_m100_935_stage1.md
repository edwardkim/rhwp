# Task #935 단계별 완료 보고서 — Stage 1 (진단 + 수정 통합)

## 이슈

[#935](https://github.com/edwardkim/rhwp/issues/935) — HWPX 의 WMF Picture 위치 산출 오류. 페이지 외곽선 밖으로 다이어그램이 overflow.

## scope 정정

- 초기 가설 ❌ "WMF binary 누락" — 진단 결과 binary 는 정상 로드 (6개 모두), `bin_data_id` 매핑 정상, image 데이터 전달 정상.
- 실제 회귀 ✅ **HWPX 변환본의 wrap=TopAndBottom Picture 가 선행 TAC width 만큼 우측 shift 되어 페이지 외곽으로 overflow**.

이전 PR #918 follow-up 시 "page 18 SVG 156KB" 로 진단했던 것은 **페이지 번호 매핑 불일치** 로 인한 잘못된 비교 (HWP3 page 18 ≠ HWPX page 18). 실제 다이어그램은 HWPX page 19 에 있고 image 자체는 존재.

## Root cause

`paragraph_layout` 의 inline emit 경로 (paragraph_layout.rs:1973~):

```rust
// 이전: wrap 종류 무관하게 inline emit
if let Control::Picture(pic) = ctrl {
    let pic_h = ...;
    let img_y = ...;
    // BoundingBox(x, ...) - x 는 선행 TAC 후 cursor 위치
    let img_node = RenderNode::new(...,
        BoundingBox::new(x, img_y, tac_w, pic_h));
    line_node.children.push(img_node);
}
```

HWPX `pi=394` 구성:
- 3 controls: Table[0] (89mm) → Picture (161mm, wrap=TopAndBottom) → Table[2] (47mm)
- 모두 tac=true → inline 으로 직렬 배치
- Picture 의 inline x = Table[0] width = 336 px = 89mm shift
- Picture 의 right edge = 472 + 608 = **1081 px**, page right edge = 794 px → **287px overflow**

대신에 wrap=TopAndBottom Picture 는 본래 별도 line 에 배치되어야 함. `layout_shape_item` 이 적절한 pic_x (col_area.x + paragraph margin) 로 emit 하도록 위임해야 함.

추가로 `layout_shape_item` 의 emit 조건 `!has_real_text` 가 whitespace 도 "real text" 로 판정 → HWPX whitespace-only 텍스트 paragraph 의 Picture emit 회피하던 문제 동반 수정.

## 수정

### 1. `src/renderer/layout/paragraph_layout.rs:1975-1985`

inline Picture emit 전 조건 검사 추가:

```rust
// [Task #935] HWPX 변환본의 top-level paragraph 에 wrap=TopAndBottom Picture 가
// 선행 TAC (table 등) 뒤에 inline 으로 emit 되면 선행 width 만큼 우측 shift →
// 페이지 외곽 overflow. 표 셀 안의 Picture 는 영향 없음 (cell_ctx.is_some()).
// top-level + TopAndBottom 인 경우만 layout_shape_item 으로 위임.
if cell_ctx.is_none()
    && matches!(pic.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
{
    x += tac_w;
    continue;
}
```

### 2. `src/renderer/layout.rs:3329-3331`

`layout_shape_item::has_real_text` 가 whitespace 제외:

```rust
// 이전: c > '\u{001F}' && c != '\u{FFFC}'  (whitespace 도 true)
// 변경: !c.is_whitespace() 추가
let has_real_text = para.text.chars()
    .any(|c| !c.is_whitespace() && c > '\u{001F}' && c != '\u{FFFC}');
```

## 검증

### 시각

| 항목 | 수정 전 | 수정 후 |
|------|--------|--------|
| HWPX page 19 image x | 472.91 px (overflow) | **96.69 px** (정상) |
| 페이지 외곽선 overflow | 287 px | 없음 |
| 다이어그램 표시 | 페이지 밖 | 페이지 안 |

### 자동화 검증

| 항목 | 결과 |
|------|------|
| cargo test --lib | 1275 passed / 0 failed |
| cargo check --target wasm32-unknown-unknown --lib | OK |
| cargo clippy -- -D warnings | clean |
| 다른 sample (sample14, sample16-hwp3/hwp5, exam_kor) 페이지 수 회귀 | 없음 |

## scope 외 / 후속

- **다이어그램 y 좌표 정합**: 수정 후 image y=75.6 (페이지 상단). 한컴 viewer 는 "가. 주전산센터..." 라벨 뒤에 위치. 별도 후속 작업 필요.
- **다른 HWPX 변환본 검증**: sample14-hwp5.hwpx 등도 회귀 점검 필요 (CLI 페이지 수만 동일 확인).
- **HWP5 의 WMF 처리** (이슈 #936) — 다른 세션에서 진행 예정.

## 진단 도구

`examples/diag_935.rs` — WMF binary / TAC wrap=TopAndBottom Picture 개수 검증.

작업지시자 승인 후 커밋 + PR 진행.
