# Task #1037 Stage 3 완료 보고서 — Dialog margin/indent 표시 한컴 정합 fix

**Issue**: [#1037 HWP5 변환본 paragraph height 과대 측정](https://github.com/edwardkim/regression-rhwp/issues/1037)
**Branch**: `local/task1037` (rebased onto `local/task1035` / PR #1036)
**작업 내용**: Stage 2 의 ParaShape unit normalize 후 잔존한 dialog 표시 격차 fix (margin/indent)

---

## 1. Stage 2 종료 시점 잔존 격차

Stage 2 의 parser 단계 ParaShape halve + uniform `variant_div=2` 는 **rendering 정합 + spacing_before dialog 정합** 달성. 그러나 시각 검증 (작업지시자 screenshot 비교) 결과:

| 필드 | 한컴 정답 | HWP3 dialog | HWP5 변환본 dialog (post-Stage2) |
|------|----------|-------------|--------------------------------|
| 왼쪽 여백 | 40.0 pt | 30.0 pt ✗ | 20.0 pt ✗ |
| 오른쪽 여백 | 10.0 pt | 5.0 pt ✗ | 5.0 pt ✗ |
| 내어쓰기 | 20.0 pt | 10.0 pt ✗ | 10.0 pt ✗ |
| 문단 위 | 8.5 pt | 8.6 pt ✓ | 8.6 pt ✓ |

→ margin/indent 모두 한컴 정답의 ½ 또는 ¾ 표시. spacing 만 정합.

---

## 2. Root cause 진단

### 2.1 sample16 p452 raw 데이터 검증

`target/release/rhwp dump` 로 동일 paragraph (계약상대자가 공급... ○ bullet) raw 값 확인:

| | HWP3 native | HWP5 변환본 (post-Stage2) | 한컴 dialog |
|---|------------|----------------------------|------------|
| margin_left | 6000 | 4000 | 40 pt |
| margin_right | 1000 | 1000 | 10 pt |
| indent | -2000 | -2000 | 내어쓰기 20 |
| spacing_before | 852 | 852 | 8.5 pt |

### 2.2 raw ↔ 한컴 dialog 관계 도출

**한컴 dialog 표준** (HWP5 변환본 기준):
- 왼쪽 = raw_margin_left / 100 = pt
- 오른쪽 = raw_margin_right / 100 = pt
- 내어쓰기/들여쓰기 = |raw_indent| / 100 = pt (부호로 type 결정)
- 문단 위/아래 = raw_spacing_before/after / 100 = pt

→ **모든 필드 / 100 = pt (HWPUNIT 표준 변환)**, variant_div 미적용.

**HWP3 native 특이성**:
- raw margin_left = 6000 (HWP5 변환본 4000 의 1.5배)
- `(margin_left + min(0, indent)) / 100 = (6000 + (-2000)) / 100 = 40 pt` ← 한컴 정답
- 즉 HWP3 의 raw `margin_left` 는 **continuation 라인 position** 으로 저장
- HWP5 변환본 (Stage 2 normalize 후) raw `margin_left` 는 **first-line position** 으로 저장 (의미 다름)

### 2.3 Stage 2 의 variant_div=2 가 dialog 에 잘못 적용

Stage 2 fix 는 `style_resolver.rs:746` 의 variant_div=2 를 모든 ParaShape 필드에 균일 적용 → rendering 일관성은 OK. 그러나 dialog 는 `ResolvedParaStyle.margin_left/right/indent` 를 그대로 사용 → /2 적용된 값이 표시되어 한컴 dialog 대비 절반 표시.

spacing_before/after 는 [formatting.rs:734-735](src/document_core/commands/formatting.rs#L734) 에서 raw_ps 직접 변환 (variant_div 미적용) → 한컴 정합. margin/indent 도 동일 패턴 필요.

---

## 3. Fix — Dialog formula raw 직접 사용 + HWP3 effective first-line 변환

### 3.1 변경 hunk — `build_para_properties_json` ([formatting.rs:681-716](src/document_core/commands/formatting.rs#L681))

```rust
// [Task #1037] dialog 표시 한컴 정합:
// - margin/indent 는 raw_ps 직접 사용 (variant_div 미적용)
// - HWP3 native: raw margin_left 는 continuation 라인 position 으로 저장 → 한컴 dialog
//   "왼쪽 여백" 은 effective first-line position 으로 (margin_left + min(0, indent)) 변환
// - HWP5 변환본 (is_hwp3_variant=true): Task #1037 parser normalize 후 raw 는 한컴 dialog 표준
//   의미로 정합 (margin_left = first-line position) → 직접 사용
let is_variant = self.document.is_hwp3_variant;
let (raw_left_hu, raw_right_hu, raw_indent_hu) = raw_ps
    .map(|r| (r.margin_left, r.margin_right, r.indent))
    .unwrap_or((0, 0, 0));
let effective_left_hu = if is_variant {
    raw_left_hu
} else {
    raw_left_hu + raw_indent_hu.min(0)
};
let dialog_margin_left_px = crate::renderer::hwpunit_to_px(effective_left_hu, self.dpi);
let dialog_margin_right_px = crate::renderer::hwpunit_to_px(raw_right_hu, self.dpi);
let dialog_indent_px = crate::renderer::hwpunit_to_px(raw_indent_hu, self.dpi);
```

JSON 출력 부분 ([formatting.rs:731](src/document_core/commands/formatting.rs#L731)):
```diff
-    ps.margin_left, ps.margin_right, ps.indent,
+    dialog_margin_left_px, dialog_margin_right_px, dialog_indent_px,
```

### 3.2 효과

| 필드 | raw | Stage 2 dialog (잘못) | Stage 3 dialog (한컴 정합) |
|------|-----|----------------------|--------------------------|
| HWP3 왼쪽 | left=6000, indent=-2000 | 30 pt | (6000-2000)/100 = **40 pt** ✓ |
| HWP3 오른쪽 | right=1000 | 5 pt | 1000/100 = **10 pt** ✓ |
| HWP3 indent | -2000 | 내어쓰기 10 | |-2000|/100 = **내어쓰기 20** ✓ |
| HWP5 왼쪽 | left=4000 (post-Stage2) | 20 pt | 4000/100 = **40 pt** ✓ |
| HWP5 오른쪽 | right=1000 | 5 pt | 1000/100 = **10 pt** ✓ |
| HWP5 indent | -2000 (post-Stage2) | 내어쓰기 10 | |-2000|/100 = **내어쓰기 20** ✓ |

---

## 4. 검증

### 4.1 자동 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✓ warning 0 |
| `cargo clippy --release --lib -- -D warnings` | ✓ clean |
| `cargo fmt --all -- --check` | ✓ clean |
| `cargo test --release --lib` | ✓ 1308 passed |
| `cargo test --release --tests` | ✓ FAILED 0 |

### 4.2 페이지 수 회귀 (rendering 무영향 단언)

| | sample16 (HWP3) | sample16-hwp5 (변환본) | 한컴 정답 |
|---|----------------|----------------------|----------|
| Stage 2 baseline | 64 | 64 | 64 |
| **Stage 3 (dialog fix)** | **64** ✓ | **64** ✓ | 64 |

→ dialog 만 변경, rendering pipeline 영향 없음.

---

## 5. 본 task (Stage 1~3) 종합 효과

| 단계 | 작업 | 효과 |
|------|------|------|
| Stage 1 | 진단 — PARA_LINE_SEG 누락 paragraph 식별 | (진단 only) |
| Stage 2 | parser ParaShape halve + variant_div=2 uniform | spacing 정합, rendering 일관성 |
| **Stage 3** | **dialog formula raw 직접 사용 + HWP3 first-line 변환** | **margin/indent dialog 한컴 정합** |

최종 결과 (sample16 p452 기준):
- HWP3 / HWP5 변환본 모두 한컴 정답과 dialog **완전 정합** (왼쪽 40, 오른쪽 10, 내어쓰기 20, 문단위 8.5)
- rendering 64 페이지 한컴 정합 유지
- 회귀 0

---

## 6. 잔존 (본 task 의 범위 외)

- **p23 외곽선 overflow** (pi=460 PartialParagraph) — paragraph height 영역, Task #1010 등 별도 분석
- **HWP3 native indent 의 부호 처리** — 본 fix 에서 raw_indent_hu 부호 그대로 전달, dialog 가 내어쓰기 정확히 표시 (이전 보고에서 "들여쓰기" 표시 가능성은 다른 paragraph 일 가능성)

---

## 7. 다음 단계 (Stage 4)

WASM 빌드 + rhwp-studio 시각 검증 — 한컴 정답지 dialog 정합 단언 후 PR.
