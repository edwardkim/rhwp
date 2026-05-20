# Task #1008 Stage 2 완료 보고서 — 격차 A fix (HWP3 Shape gradient IR 매핑)

**Issue**: [#1008 HWP3 sample16 Shape/Text 정합 격차 종합](https://github.com/edwardkim/regression-rhwp/issues/1008)
**Branch**: `local/task1008`
**작업 내용**: `src/parser/hwp3/drawing.rs` Fill IR 구축에 `gradient_attr` 매핑 추가 + 단위 테스트

---

## 1. 변경 hunk

`src/parser/hwp3/drawing.rs:792-806` 의 Fill 구축 변경:

```diff
+let (fill_type, gradient) = if let Some(g) = header.gradient_attr.as_ref() {
+    let grad = crate::model::style::GradientFill {
+        gradient_type: g.kind as i16,
+        angle: g.angle as i16,
+        center_x: g.center_x as i16,
+        center_y: g.center_y as i16,
+        blur: g.step as i16,
+        step_center: 0,
+        colors: vec![g.start_color, g.end_color],
+        positions: vec![],
+    };
+    (crate::model::style::FillType::Gradient, Some(grad))
+} else {
+    (crate::model::style::FillType::Solid, None)
+};
 let fill = Fill {
-    fill_type: crate::model::style::FillType::Solid,
+    fill_type,
     solid: Some(...),
-    gradient: None,
+    gradient,
     image: None,
     alpha: 0,
 };
```

매핑 contract:
- HWP5 `doc_info.rs:404` 와 동일 (step→blur, 2-stop colors, empty positions → renderer 균등 분포)
- renderer 측 (`utils.rs:167`) 의 empty positions 분기 의존 — 무수정

---

## 2. dump 단언

### 2.1 HWP3 sample16 사업개요 박스 (pi=71)

```
BEFORE: 채우기: Solid
AFTER:  채우기: Gradient
```

### 2.2 HWP3 sample16 cover RFP 박스 (pi=5)

```
BEFORE: 채우기: Solid
AFTER:  채우기: Gradient
```

---

## 3. SVG gradient element count

| 파일 | BEFORE | AFTER | 비고 |
|------|--------|-------|------|
| hwp3-sample16_001.svg (cover) | 0 | **2** | gradient 복원 ✓ |
| hwp3-sample16_003.svg (사업개요 p2) | 0 | **2** | gradient 복원 ✓ |
| hwp3-sample16-hwp5_001.svg (HWP5 cover) | 2 | 2 | 무변동 ✓ |
| hwp3-sample16-hwp5_003.svg (HWP5 p2) | 2 | 2 | 무변동 ✓ |

→ **격차 A 해소** + HWP5 변환본 무영향 단언.

---

## 4. 단위 테스트 추가

`tests/issue_1008_gradient.rs`:
- HWP3 sample16 → pi=71 Shape Rectangle 추출
- `fill.fill_type == FillType::Gradient` 단언
- `fill.gradient.is_some()` + `colors.len() == 2` 단언

```
$ cargo test --release --test issue_1008_gradient
test hwp3_sample16_business_box_has_gradient ... ok
test result: ok. 1 passed; 0 failed
```

---

## 5. 회귀 sweep

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✓ warning 0 |
| `cargo clippy --release --lib -- -D warnings` | ✓ clean |
| `cargo fmt --check` | ✓ clean (fmt 수정 1회 반영) |
| `cargo test --release --lib` | ✓ 1307 passed |
| `cargo test --release --tests` | ✓ all passed, FAILED 0 |
| `issue_1008_gradient`: 신규 회귀 가드 | ✓ 1 passed |

### 5.1 페이지 수 sweep (HWP3 sample 전체 + 변환본 + 일반 fixture)

| Sample | 페이지 | 비고 |
|--------|--------|------|
| hwp3-sample10.hwp / -hwp5.hwp | 763 / 763 | ✓ |
| hwp3-sample11.hwp / -hwp5.hwp | 151 / 151 | ✓ |
| hwp3-sample13.hwp / -hwp5.hwp | 3 / 3 | ✓ |
| hwp3-sample14.hwp / -hwp5.hwp | 11 / 11 | ✓ |
| hwp3-sample16.hwp / -hwp5.hwp / -hwp5.hwpx | 64 / 64 / 71 | ✓ |
| hwp3-sample19.hwp / -hwp5.hwp | 2 / 2 | ✓ |
| hwp3-sample4.hwp / -hwp5.hwp | 36 / 36 | ✓ |
| hwp3-sample5.hwp / -hwp5* | 64 (4 variants) | ✓ |
| exam_kor/eng/math | 20 / 8 / 20 | ✓ |
| aift / biz_plan | 74 / 6 | ✓ |

→ **모든 fixture 회귀 0**.

---

## 6. 성공 기준 충족

| 조건 | 결과 |
|------|------|
| C1: HWP3 sample16 p2 Shape gradient 표시 | ✓ (SVG 2 elements) |
| C5: 페이지 수 유지 | ✓ (64) |
| C6: 변환본 + 일반 fixture 회귀 0 | ✓ |
| C7: cargo test 1307+ passed | ✓ |
| C8: 작업지시자 시각 검증 | (PR/Stage 6 시점) |

---

## 7. 다음 단계 (Stage 3)

격차 C — HWP3 HEAD numbering "■1.추진목적■" → "1. 추진목적".

`src/parser/hwp3/johab.rs:81` 의 `0x3441 => 0x25A0 (■)` 매핑이 numbering marker 인지, 일반 character 인지 단언 후 fix 위치 결정.
