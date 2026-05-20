# Task #1008 Stage 1 완료 보고서 — 종합 진단

**Issue**: [#1008 HWP3 sample16 Shape/Text 정합 격차 종합](https://github.com/edwardkim/regression-rhwp/issues/1008)
**Branch**: `local/task1008`
**작업 내용**: 4 격차 (A/B/C/D) root cause 위치 단언 + 영향 범위 + 분리 여부 결정

---

## 1. 격차 A — HWP3 Shape 박스 배경 gradient 누락 (HWP3 한정)

### 1.1 단언 — gradient 데이터는 **이미 파싱됨**, IR 매핑만 누락

`src/parser/hwp3/drawing.rs` 검토 결과:

```rust
// drawing.rs:149-170 — gradient 구조 + 파싱 ✓ 존재
pub struct Hwp3DrawingObjectGradientAttr {
    pub start_color: u32,
    pub end_color: u32,
    pub kind: u32,
    pub angle: u32,
    pub center_x: u32,
    pub center_y: u32,
    pub step: u32,
}

// drawing.rs:252-253 — basic_attr.has_gradient() 시 파싱 ✓
let gradient_attr = if basic_attr.has_gradient() {
    Some(Hwp3DrawingObjectGradientAttr::read(&mut reader)?)
};
```

그러나 `drawing.rs:792-806` 의 최종 Fill IR 구축:

```rust
let fill = Fill {
    fill_type: crate::model::style::FillType::Solid,  // ← 하드코딩
    solid: Some(...),
    gradient: None,                                    // ← 파싱된 데이터 무시
    image: None,
    alpha: 0,
};
```

→ **파싱된 `gradient_attr` 가 IR 에 전혀 매핑되지 않음**. Stage 2 에서 단순 매핑 추가로 해결.

### 1.2 HWP5 매핑 비교 (`src/parser/doc_info.rs:385-413`)

| HWP3 field | HWP5 field | IR (`GradientFill`) |
|-----------|------------|---------------------|
| `kind: u32` | `gtype: u32` | `gradient_type: i16` |
| `angle: u32` | `angle: u32` | `angle: i16` |
| `center_x: u32` | `cx: u32` | `center_x: i16` |
| `center_y: u32` | `cy: u32` | `center_y: i16` |
| `step: u32` | `blur: u32` | `blur: i16` ← 매핑 단언 |
| (없음) | (없음) | `step_center: 0` |
| `start_color, end_color` | count + colors[] | `colors: vec![start, end]` (2-stop) |
| (없음, 2-stop) | (count==2 시 positions 부재) | `positions: vec![]` |

### 1.3 renderer 측 호환성

`src/renderer/layout/utils.rs:167`:
```rust
let positions: Vec<f64> = if g.positions.is_empty() {
    let n = g.colors.len();
    (0..n).map(|i| i as f64 / (n.max(2) - 1).max(1) as f64).collect()
};
```

→ empty positions → 균등 분포 (2-stop 시 [0.0, 1.0]). HWP3 가 `positions: vec![]` 로 주입해도 renderer 가 정상 처리.

### 1.4 Fix 위치 확정

`src/parser/hwp3/drawing.rs:792-806` 단일 파일 변경.

---

## 2. 격차 B — Shape border style 실선/점선 (HWP3 + HWP5 공통)

### 2.1 단언 — 둘 다 LineType 2~3 으로 해석되어 점선 렌더

dump 비교 (sample16 사업개요 박스):

```
HWP3 native pi=71:  선 style=0x0002 → attr & 0x3F = 2 → 점선
HWP5 변환본 pi=71:  선 style=0xc0010043 → attr & 0x3F = 3 → 점선
한컴 viewer:                                              실선
```

### 2.2 추가 조사 필요 사항

- HWP3 의 line_style binary 값 2 의 실제 의미 (HWP3 spec / hwplib reference)
- HWP3 와 HWP5 의 LineType 비트 enumeration 일치성
- 한컴이 LineType=2/3 을 어떤 width/style 조합에서 실선 처리하는지

### 2.3 Fix 위치 후보

- `src/parser/hwp3/drawing.rs:758~775` (border attr 비트 조정)
- 또는 `src/renderer/layout/utils.rs:197` (`shape_line_type = border.attr & 0x3F` 매핑 정합)

Stage 4 시작 시 추가 조사 + 결정.

---

## 3. 격차 C — HWP3 HEAD numbering 라벨 형식 (HWP3 한정)

### 3.1 단언 — ■ 출처

`src/parser/hwp3/johab.rs:81`:
```rust
0x3441 => 0x25A0,  // ■ BLACK SQUARE
```

HWP3 raw stream 에 johab 코드 0x3441 가 있어 ■ 으로 디코딩됨. 이는 HWP3 의 HEAD/numbering 영역에서 marker 로 사용되는 special character.

### 3.2 격차 위치

- pi=5 cover RFP 박스 textbox 내 자식 paragraph 텍스트
- "■1.추진목적■" → 한컴 정답 "1. 추진목적"
- HEAD numbering 형식 spec 에서 head marker prefix/suffix 가 johab 0x3441 (■) 로 저장됨
- HWP5 변환본은 marker strip 또는 별도 numbering 처리

### 3.3 Fix 위치 후보

- `src/parser/hwp3/` 내 HEAD/numbering detection 추가
- 또는 special char 처리 시 marker strip

Stage 3 시작 시 정확한 위치 단언 후 결정.

---

## 4. 격차 D — HWP3 한글 단어 공백 누락 — **renderer 측 문제**

### 4.1 단언 — 데이터는 정상

```
$ ./target/release/rhwp dump samples/hwp3-sample16.hwp -s 0 -p 4

--- 문단 0.4 --- cc=31, text_len=31, controls=0
  텍스트: "        \"세계 3대 물 서비스 기업\" 실현을 위한"
  [CS] pos=0 id=57 ...
  [CS] pos=0 id=58 bold=true ...
  [CS] pos=8 id=59 ... char="\""
  [CS] pos=31 id=60 ...
```

→ parser 추출 텍스트에 **공백 모두 포함**. 격차는 visual rendering 단계에서 발생.

### 4.2 회귀 위치 가설

- text run 분할 시 일부 char 누락
- SVG 출력 시 char_positions 계산 오류
- char_shape boundary 처리 시 공백 char 의 width 계산 결함

### 4.3 작업지시자 결정 — 본 task 에 포함 유지

renderer 측 문제임에도 본 task 의 4 격차 중 하나로 유지. Stage 5 에서 root cause 진단 + fix.

---

## 5. Stage 진행 영향

### 5.1 Stage 2 (격차 A) — 진행 준비 완료

매핑 표 단언 완료. 변경 위치 단일 파일 (`drawing.rs:792-806`).

### 5.2 Stage 3 (격차 C) — 추가 조사 필요

`src/parser/hwp3/mod.rs` 의 HEAD/numbering 처리 코드 위치 또는 부재 단언.

### 5.3 Stage 4 (격차 B) — 추가 조사 필요

HWP3 line_style binary 값 의미 + HWP5 와 차이 조사.

### 5.4 Stage 5 (격차 D) — renderer 영역

`src/renderer/` 내 text run / char_position 계산 영역 진단.

---

## 6. baseline SVG 보존

`output/poc/pr1008/before/`:
- `hwp3-sample16_001.svg` (cover)
- `hwp3-sample16_003.svg` (사업개요 p2)
- `hwp3-sample16-hwp5_001.svg` (HWP5 변환본 cover)
- `hwp3-sample16-hwp5_003.svg` (HWP5 변환본 p2)

Stage 5 sweep 시 BEFORE/AFTER 비교용.

---

## 7. 다음 단계 (Stage 2)

격차 A — `src/parser/hwp3/drawing.rs:792-806` 의 Fill IR 구축 시 `header.gradient_attr` 매핑 추가.
