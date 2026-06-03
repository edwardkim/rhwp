# Task #489 Stage 1 완료 보고서 — Picture+Square wrap 지원

## 1. 작업 범위

구현계획서(`mydocs/plans/task_m100_489_impl.md`)의 Stage 1: `layout_composed_paragraph` 에 Picture/Shape Square wrap 감지 + LINE_SEG.cs/sw 적용.

Stage 2 (시각·회귀 검증) 도 본 보고서에서 함께 수행 — 본질 변경 없이 광범위 점검만 필요했고 결과가 명확했기 때문.

## 2. 변경 사항

### 2.1 `src/renderer/layout/paragraph_layout.rs`

#### 변경 1: import 추가

```diff
-use super::super::{TextStyle, ShapeStyle, TabStop, hwpunit_to_px, format_number, NumberFormat as NumFmt, AutoNumberCounter};
+use super::super::{TextStyle, ShapeStyle, TabStop, hwpunit_to_px, px_to_hwpunit, format_number, NumberFormat as NumFmt, AutoNumberCounter};
```

#### 변경 2: 함수 시작부 — Picture/Shape Square wrap 감지 (1회)

```rust
let auto_tab_right = para_style.map(|s| s.auto_tab_right).unwrap_or(false);

// [Task #489] 비-TAC Picture/Shape with wrap=Square 보유 여부.
// 한컴은 어울림 그림이 있는 paragraph 의 LINE_SEG.cs/sw 를 그림 너비만큼 좁혀
// 인코딩한다. 표 Square wrap (#362/#439/#463) 은 caller 가 col_area 를 좁혀
// wrap_area 로 우회하지만, Picture/Shape 는 호스트 paragraph 와 같은 paragraph
// 에 anchor 되므로 별도 우회 경로가 없다. 이 플래그가 true 면 줄별 루프에서
// LINE_SEG.cs/sw 를 effective col_x/col_width 로 사용한다.
let has_picture_shape_square_wrap = para
    .map(|p| p.controls.iter().any(|c| {
        use crate::model::shape::TextWrap;
        let common_opt = match c {
            Control::Picture(pic) if !pic.common.treat_as_char => Some(&pic.common),
            Control::Shape(s) if !s.common().treat_as_char => Some(s.common()),
            _ => None,
        };
        common_opt.map(|cm| matches!(cm.text_wrap, TextWrap::Square)).unwrap_or(false)
    }))
    .unwrap_or(false);
let col_area_w_hu = px_to_hwpunit(col_area.width, self.dpi);
```

#### 변경 3: 줄별 루프 — effective_col_x / effective_col_w 계산

```rust
let effective_margin_left = margin_left + line_indent;

// [Task #489] Picture/Shape Square wrap (어울림) 시 LINE_SEG.cs/sw 적용.
// 한컴이 인코딩한 정답값을 그대로 사용 (휴리스틱 없음).
// 표 Square wrap 케이스는 caller 가 col_area 를 이미 wrap_area 로 좁혀
// 호출하므로 segment_width ≈ col_area_w_hu → 조건 미발동 (회귀 차단).
// 200 HU 임계값은 paragraph_layout 의 multi-col filter 와 동일.
let (effective_col_x, effective_col_w) = if has_picture_shape_square_wrap
    && comp_line.segment_width > 0
    && comp_line.segment_width < col_area_w_hu - 200
{
    let cs_px = hwpunit_to_px(comp_line.column_start, self.dpi);
    let sw_px = hwpunit_to_px(comp_line.segment_width, self.dpi);
    (col_area.x + cs_px, sw_px)
} else {
    (col_area.x, col_area.width)
};
```

#### 변경 4: line_node BoundingBox 와 available_width 적용

`col_area.x` → `effective_col_x`, `col_area.width` → `effective_col_w` 로 교체. 영향 위치:

- `BoundingBox::new(effective_col_x + effective_margin_left, text_y, effective_col_w - effective_margin_left - margin_right, line_height)`
- `let available_width = effective_col_w - effective_margin_left - margin_right - inline_offset - num_offset;`
- `Alignment::{Center, Distribute, Right, _}` 의 `x_start` 계산 (4 곳)
- 인라인 TAC 이미지/도형 배치: `let mut img_x = effective_col_x + effective_margin_left + align_offset;`
- 빈 텍스트 줄의 인라인 TAC 컨트롤: `let mut inline_x = effective_col_x + effective_margin_left;`

`text_style.line_x_offset = x - col_area.x` 등 col_area.x 를 직접 참조하는 좌표 보정값은 유지 (탭 계산 reference 가 col_area.x 기준이며 cs=0 인 본 케이스에서 effective_col_x == col_area.x 이므로 동일).

### 2.2 `src/renderer/layout/integration_tests.rs`

`test_489_picture_square_wrap_text_does_not_overlap_image` 추가:
- `samples/exam_science.hwp` 페이지 1 SVG 에서 그림(width=150, height≈136) 과 텍스트 위치를 파싱
- 그림의 가로/세로 영역 안에 있는 텍스트가 0 건임을 검증

### 2.3 `src/main.rs` (보조 변경)

`dump` 명령의 도형 위치 출력에 `horz_align`, `vert_align` 추가 (조사용 — 향후 디버깅에도 유용).

```diff
-위치: 가로={} 오프셋={:.1}mm({}), 세로={} 오프셋={:.1}mm({})
+위치: 가로={} 오프셋={:.1}mm({}) 정렬={:?}, 세로={} 오프셋={:.1}mm({}) 정렬={:?}
```

## 3. 검증 결과

### 3.1 단위 테스트

`cargo test --lib --release`: **1093 passed; 0 failed**

신규 테스트 `test_489_picture_square_wrap_text_does_not_overlap_image` 통과.

### 3.2 svg_snapshot

`cargo test --release --test svg_snapshot`: **6/6 passed** (회귀 0)

### 3.3 issue_418

`cargo test --release --test issue_418`: **1/1 passed** (회귀 0)

### 3.4 광범위 byte 비교 (9 종 샘플 / 263 페이지)

| 샘플 | 페이지 수 | 동일 | 차이 |
|------|---------|------|------|
| exam_kor.hwp | 20 | 20 | 0 |
| **exam_science.hwp** | 4 | 2 | **2** ✓ (정정) |
| exam_social.hwp | 4 | 4 | 0 |
| exam_math.hwp | 20 | 20 | 0 |
| exam_eng.hwp | 8 | 8 | 0 |
| 21_언어_기출_편집가능본.hwp | 15 | 15 | 0 |
| aift.hwp | 77 | 77 | 0 |
| kps-ai.hwp | 80 | 80 | 0 |
| synam-001.hwp | 35 | 35 | 0 |
| **합계** | **263** | **261** | **2** |

**차이 페이지 분석:**

#### exam_science 페이지 1 (의도된 정정 — 본 이슈 핵심)

- pi=21 (5번 문제) "5.-그림은 밀폐된 진공 용기에 H₂O(l)을…"
- Picture: 11250×10230 HU, wrap=Square, horz_align=Right
- LINE_SEG (6 줄): cs=0, sw=19592
- 수정 전: 텍스트 첫 줄 x=534..944 (풀컬럼 410px), "용기에"(815) "을"(944) 가 그림 영역(807..957) 안 → 가려짐
- 수정 후: 텍스트 첫 줄 x=535..798 (sw=261px 좁아짐), 그림(807..957) 과 분리. 단어 간격 정상화

#### exam_science 페이지 2 (예상 외 추가 정정)

- pi=37 (8번 문제) "8.-그림은 수소와 원소…"
- Picture: 13296×9240 HU, wrap=Square, horz_align=Right
- LINE_SEG: 첫 6 줄 cs=0 sw=17546, 마지막 2 줄 cs=0 sw=31692 (그림 끝난 후)
- 수정 전: 첫 6 줄 텍스트 풀컬럼으로 spread, 그림(315..493)과 겹침
- 수정 후: 첫 6 줄 텍스트 sw=234px 영역(70..292)으로 좁아짐, 그림과 분리

부수 효과: body-clip clipPath 너비가 946 → 897 로 좁아짐 (텍스트 bbox extent 가 좁아진 자연스러운 결과).

### 3.5 회귀 영역 점검 결과

| 영역 | 결과 |
|------|------|
| 표 Square wrap (#362/#439/#463) | 회귀 0 — caller 가 col_area 를 wrap_area 로 좁혀 호출 → segment_width ≈ col_area_w_hu → 조건 미발동 (예상대로) |
| Picture TopAndBottom wrap (#409 v2) | 회귀 0 — text_wrap≠Square 이므로 조건 미발동 |
| TAC Picture/Shape | 회귀 0 — !treat_as_char 조건으로 제외 |
| 일반 paragraph (그림 없음) | 회귀 0 — has_picture_shape_square_wrap=false |
| Multi-col paragraph filter | 회귀 0 — 별개 로직, 영향 없음 |
| 인라인 TAC 컨트롤 | exam_math/aift/kps-ai/synam 모두 회귀 0 |

### 3.6 Clippy

`cargo clippy --release --lib`: 본 변경 영역에서 신규 경고 0.

(기존 `src/document_core/commands/object_ops.rs:1007, 298` 의 `panicking_unwrap` 에러 2 건은 devel 베이스라인에 이미 존재 — 본 이슈와 무관.)

## 4. 결정 사항 / 발견

### 4.1 cs>0 케이스 (그림이 컬럼 좌측) 처리

본 변경은 cs/sw 둘 다 effective 값으로 적용하므로 이론상 cs>0 케이스도 올바르게 동작. 단 `text_style.line_x_offset = x - col_area.x` 같은 탭 계산 reference 는 col_area.x 기준 유지 — exam_science p1/p2 모두 cs=0 이므로 동등. cs>0 인 실 샘플이 발견되면 별도 회귀 검증 필요 (현재 9 종 샘플에 cs>0 케이스 없음 → 차이 0).

### 4.2 추가 정정 (exam_science p2 pi=37)

본 이슈 보고에 명시되지 않은 동일 패턴 1 건 자동 정정. 의도된 정정 범위 안.

### 4.3 200 HU 임계값

`segment_width < col_area_w_hu - 200` 의 200 HU 가드는 multi-col filter (`paragraph_layout.rs:762`) 와 동일 임계값. 페이지네이션 / 단 경계 노이즈 (HU↔px 반올림) 흡수. 휴리스틱이 아닌 노이즈 제거.

## 5. 다음 단계

Stage 2 (시각·회귀 검증) 본 보고서에서 통합 완료. **Stage 3 으로 직접 진행** 가능:
- 최종 결과보고서 작성
- orders 갱신
- task489 → local/devel merge → devel push

## 6. 작업지시자 승인 요청

- [ ] Stage 1 + Stage 2 통합 결과 (단위 테스트 1093 통과, svg_snapshot 6/6, byte diff 263 중 2 — 모두 의도된 정정) 승인
- [ ] exam_science p2 pi=37 추가 정정 (동일 패턴) 수용
- [ ] Stage 3 (최종 보고서 + merge) 진행 승인
