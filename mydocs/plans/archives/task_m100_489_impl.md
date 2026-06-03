# Task #489 구현계획서 — Picture+Square wrap 지원

## 1. 구현 영역

**핵심 결함**: `src/renderer/layout/paragraph_layout.rs:860` 의

```rust
let available_width = col_area.width - effective_margin_left - margin_right - inline_offset - num_offset;
```

가 LINE_SEG.segment_width(`comp_line.segment_width`) 를 사용하지 않아, 어울림 그림이 있는 paragraph 의 텍스트가 풀컬럼 너비로 justify 됨.

**비교 (표 Square wrap)**: `layout.rs:2399`/`:2634` 가 `layout_wrap_around_paras` 호출 시 `col_area` 자체를 `wrap_area`(좁아진 영역) 로 교체 → `layout_composed_paragraph` 가 좁아진 `col_area.width` 를 그대로 사용. 즉 표 케이스는 caller 에서 col_area 를 좁혀서 우회.

**Picture+Square wrap 의 특성**: 표와 달리 그림은 **호스트 paragraph 와 같은 paragraph** 에 anchor (vert=Para+0). 별도 wrap_around_paras 흐름 없음. 따라서 `layout_composed_paragraph` 가 LINE_SEG.cs/sw 를 직접 사용해야 함.

## 2. 구현 전략 (선택안)

### Option A: `layout_composed_paragraph` 내부에서 LINE_SEG.cs/sw 자동 인식 (권장)

**조건** (모두 AND):
1. `para` 가 non-TAC `Control::Picture` 또는 `Control::Shape` 를 보유
2. 해당 control 의 `text_wrap == TextWrap::Square`
3. `comp_line.segment_width > 0` 그리고 `comp_line.segment_width < col_area.width_HU`

**적용**:
- `available_width` 를 `hwpunit_to_px(comp_line.segment_width, dpi) - effective_margin_left - margin_right - inline_offset - num_offset` 로 교체
- line `BoundingBox.width` 를 동일 좁힘
- `line_x` 는 `col_area.x + hwpunit_to_px(comp_line.column_start, dpi) + effective_margin_left` (cs 반영)

**장점**:
- caller 변경 없이 자체 완결
- 표 Square wrap 회귀 0 (별도 wrap_area 우회 경로 유지, 좁아진 col_area 진입 시 segment_width≈col_area.width_HU → 조건 미발동)
- LINE_SEG 가 한컴 정답값이므로 휴리스틱 없음

**단점**:
- 모든 paragraph 호출에서 control 순회 추가 (오버헤드 미미)

### Option B: caller 에서 wrap_area 생성 (표 Square wrap 패턴 모사)

`layout.rs` paragraph 처리 시 Picture+Square 감지 → `wrap_area` 생성 → `layout_composed_paragraph` 호출.

**단점**:
- caller 코드 복잡화 (typeset.rs, layout.rs 양쪽 수정)
- 표 Square wrap 코드 패스와 충돌 위험 (같은 paragraph 에 표+그림 혼재 시 우선순위 문제)

**결정**: **Option A** 채택. caller 변경 없이 LINE_SEG 정답값 직접 사용.

## 3. 단계 분할 (3 단계)

### Stage 1 — Picture/Shape Square wrap 감지 + segment_width override

**대상**: `src/renderer/layout/paragraph_layout.rs:layout_composed_paragraph`

**변경 사항**:
1. 함수 시작부에서 `para` 의 controls 순회 → 비-TAC Picture/Shape with wrap=Square 존재 여부 1회 계산 (`has_picture_square_wrap: bool`).
2. 줄별 루프 내에서:
   - `has_picture_square_wrap && comp_line.segment_width > 0 && comp_line.segment_width < col_w_hu` 조건 평가
   - 조건 충족 시 `seg_w_px = hwpunit_to_px(comp_line.segment_width, dpi)`, `cs_x_px = hwpunit_to_px(comp_line.column_start, dpi)` 사용
   - `effective_col_x = col_area.x + cs_x_px`
   - `effective_col_width = seg_w_px`
   - `available_width = effective_col_width - effective_margin_left - margin_right - inline_offset - num_offset`
   - `line_node` BoundingBox 의 x/width 를 위 값 기반으로 보정

**단위 테스트** (`tests` mod 또는 `layout/tests.rs`):
- 모의 Paragraph (Picture wrap=Square + 텍스트) 의 layout 결과에서 line_node bbox.x, bbox.width 가 LINE_SEG cs/sw 반영
- 정상 paragraph (그림 없음) 는 col_area 그대로 사용 (회귀 차단)

**완료 기준**:
- 단위 테스트 통과
- `cargo build --release` 성공
- exam_science.hwp p1 col2 5번 문제: 텍스트 첫 줄에 "용기에 H₂O(l)을" 정상 표시, 단어 간격 정상

### Stage 2 — 시각 검증 + 회귀 점검

**검증 절차**:
1. **exam_science.hwp p1 정상화 확인**:
   - `rhwp export-svg samples/exam_science.hwp -p 0 -o /tmp/exam_sci_after/`
   - 페이지 1 컬럼 1 5번 문제 영역 시각 비교 (그림과 텍스트 좌우 분리, 단어 간격 정상)
   - `samples/pdf/hwp2022/exam_science.pdf` 와 시각 비교
2. **회귀 검증** (광범위 byte 비교):
   - 7~8 종 샘플: `exam_kor`, `exam_science`, `exam_social`, `21_언어_기출`, `synam-001`, `kps-ai`, `aift`, `2025년_기부`
   - 차이 발생 페이지 모두 debug-overlay 로 시각 점검 → 정정 또는 회귀 판정
   - 표 Square wrap (#362, #439, #463) 회귀 0 확인

**완료 기준**:
- exam_science p1 결함 시각 해소
- 회귀 0 확인 또는 모든 차이가 정정으로 판정

### Stage 3 — 최종 정리 + 보고서

**작업**:
1. `cargo test --lib` + svg_snapshot 통과 재확인
2. `cargo clippy --release` 0 경고
3. 최종 결과보고서 (`mydocs/report/task_m100_489_report.md`) 작성
4. orders 갱신 (`mydocs/orders/20260430.md` #489 → 완료)
5. local/devel merge → devel push

## 4. 단위 테스트 시나리오

`layout/tests.rs` 또는 `layout/paragraph_layout.rs` 내부 `#[cfg(test)] mod tests`:

**테스트 1: Picture+Square wrap 적용**
```rust
// Setup: Paragraph with single Picture(wrap=Square, !TAC) + 6 line_segs (sw=19592, cs=0)
// col_area.width 가 sw 보다 큼 (예: 30945 HU)
// Action: layout_composed_paragraph 호출
// Verify: line_node[0].bbox.width ≈ hwpunit_to_px(19592)
//        line_node[0].bbox.x == col_area.x (cs=0)
```

**테스트 2: 일반 paragraph (Square wrap 없음)**
```rust
// Setup: text-only Paragraph, line_seg sw == col_w
// Verify: line_node bbox.width == col_area.width - margins
```

**테스트 3: TAC Picture 미발동**
```rust
// Setup: Picture(wrap=Square, TAC=true) + 텍스트
// Verify: TAC 이므로 narrowing 미발동, 일반 경로 사용
```

## 5. 위험 요소 / 대응

| 위험 | 대응 |
|------|------|
| 표 Square wrap (#362/#439/#463) 회귀 | 표 케이스는 caller 가 col_area 자체를 wrap_area 로 교체 → 진입 시 col_area.width 가 이미 narrowed → `segment_width < col_w_hu` 조건이 위양성 미발동. 단위 테스트로 보장 |
| Shape (글상자) Square wrap 영향 | Shape 도 동일 적용. 회귀 검증에서 Shape Square wrap 케이스 수동 점검 |
| col_area.width 와 segment_width 의 미세 차이 (200 HU 이내) | `segment_width < col_w_hu - 200` 가드로 노이즈 제거 (표/페이지네이션의 multi-col filter 와 동일 임계값) |
| TopAndBottom Picture (Task #409 v2) 영향 | `text_wrap == Square` 조건으로 TopAndBottom 제외 |
| 음수 indent (예: ps_id=74 indent=-2260) 와 cs 충돌 | cs 는 LINE_SEG 단위 (한컴 인코딩), indent 는 ParaShape 단위. 두 값은 직교 (cs 는 column 기준, indent 는 첫 줄 보정). LINE_SEG 가 indent 결과까지 흡수했다면 추가 보정 불요. 단위 테스트로 검증 |

## 6. 작업지시자 승인 요청

- [ ] Option A 전략 (LINE_SEG cs/sw 자동 인식) 승인
- [ ] 3 단계 분할 (Stage 1 구현 + 단위 테스트 / Stage 2 시각 검증 / Stage 3 최종 정리) 승인
- [ ] 단위 테스트 3 시나리오 동의
- [ ] 회귀 검증 범위 (7~8 종 샘플 byte 비교) 동의

승인 후 Stage 1 구현 시작.
