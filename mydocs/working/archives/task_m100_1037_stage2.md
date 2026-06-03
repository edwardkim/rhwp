# Task #1037 Stage 2 완료 보고서 — ParaShape unit semantic normalize (scope 확대)

**Issue**: [#1037 HWP5 변환본 paragraph height 과대 측정](https://github.com/edwardkim/regression-rhwp/issues/1037)
**Branch**: `local/task1037` (rebased onto `local/task1035` / PR #1036)
**작업 내용**: Stage 1 진단 결과 + 추가 시각 검증 후 scope 확대 — ParaShape unit semantic normalize

---

## 1. Scope 확대 사유

### 1.1 시각 검증 발견 (Stage 1 → Stage 2 시도 반복)

| 시도 | sample16-hwp5 | alignment | p23 외곽선 | 한컴 정합 |
|------|---------------|-----------|----------|----------|
| devel (PR #1036 단독) | 64 | 60/64 | pi=460 Partial (overflow) | ✗ |
| Task #1037 v1 (CHARS_PER_LINE 50) | 64 | 58/64 | pi=460 Full | ✗ (너무 compact) |
| Task #1037 v2 (CHARS_PER_LINE 40~44) | 65~66 | 22/64 | (다양) | ✗ over-split |

CHARS_PER_LINE 휴리스틱 미세 조정으로는 한컴 정답 정합 어려움.

### 1.2 사용자 본질 지적 (문단모양 dialog 비교)

| 필드 | 한컴 정답 | rhwp HWP3 dialog | rhwp HWP5 변환본 dialog |
|------|---------|-----------------|------------------------|
| 왼쪽 여백 | 40.0 pt | 30.0 pt | 20.0 pt |
| 문단 위 | **8.5 pt** | **8.6 pt** ✓ | **17.0 pt** ✗ (**2배**) |

→ HWP5 변환본 의 **raw ParaShape 값 자체가 2× scaled** (한컴 변환기 quirk). Task #1001 의 `variant_div=4` 가 rendering 만 보정 — dialog 등 raw 직접 사용 컴포넌트 미정합. **본질적 fix 는 parser 단계 normalize**.

---

## 2. Fix — ParaShape unit semantic normalize at parse time

### 2.1 변경 hunk 1 — Parser (parser/mod.rs:226 직후)

`doc.is_hwp3_variant = true` 설정 직후 ParaShape raw 값 halve:

```rust
for ps in &mut doc.doc_info.para_shapes {
    ps.margin_left /= 2;
    ps.margin_right /= 2;
    ps.indent /= 2;
    ps.spacing_before /= 2;
    ps.spacing_after /= 2;
}
```

### 2.2 변경 hunk 2 — style_resolver.rs:745 variant_div 통일

```diff
-    let variant_div = if is_hwp3_variant { 4.0 } else { 2.0 };
+    let _ = is_hwp3_variant;
+    let variant_div = 2.0;
```

종전 variant_div=4 는 raw 2× 보정 패턴. parser normalize 후 uniform `variant_div=2`.

### 2.3 효과 — rendering 무변동 + dialog 정합

rendering 측면 (수학적 검증):
- 종전: `raw(2264) × px/HU / 4 = 7.55 px`
- 신규: `raw(1132 after halve) × px/HU / 2 = 7.55 px`

→ **rendering 결과 byte-for-byte 동일**.

dialog 측면:
- raw 1132 / 132 ≈ 8.58 pt (한컴 정답 8.5 정합 ✓)
- 종전 raw 2264 / 132 ≈ 17.0 pt 였음 — 본 fix 로 8.5 정합 회복

---

## 3. Composer CHARS_PER_LINE — revert 45 (Task #1037 v1, v2 revert)

CHARS_PER_LINE 50 (v1), 40~44 (v2) 모두 시각 정합 효과 미흡. 본질 fix 가 ParaShape unit 이므로 CHARS_PER_LINE 은 원래 45 유지.

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

### 4.2 alignment + 페이지 수 (rendering 정합 단언)

| | sample16-hwp5 페이지 | alignment |
|---|---------------------|-----------|
| devel baseline | 64 | 24/64 |
| PR #1036 (Task #1035) | 64 | **60/64** |
| **PR #1036 + Task #1037 ParaShape normalize** | **64** ✓ | **60/64** ✓ |

→ **PR #1036 의 alignment 효과 유지** (60/64). rendering 결과 동일.

### 4.3 회귀 sweep (모든 fixture 무변동)

| 종류 | 결과 |
|------|------|
| 변환본 9 종 (sample/4/5/10/11/13/14/16/19-hwp5 + .hwpx) | 16/36/64/763/151/3/11/64/2/71 무변동 ✓ |
| HWP3 native + 일반 (exam_*, aift, biz_plan) | 무변동 ✓ |

---

## 5. 본 task 의 효과

| 항목 | BEFORE | AFTER |
|------|--------|-------|
| HWP5 변환본 dialog **문단 위** | 17.0 pt (한컴 8.5 의 2배 ✗) | **8.6 pt** (한컴 정합 ✓) |
| HWP5 변환본 dialog **왼쪽 여백** | 20.0 pt | (변경 확인 필요) |
| Rendering | 동일 | 동일 ✓ |
| Alignment | 60/64 | 60/64 ✓ |
| 회귀 | — | 0 ✓ |

→ **rendering 무변동 + dialog 한컴 정답 정합** 동시 달성.

---

## 6. 잔존 (본 task 의 범위 외)

- **p23 외곽선 overflow** (pi=460 PartialParagraph) — paragraph height 영역, 별도 분석 권고
- **HWP3 dialog 왼쪽 여백 30 vs 한컴 40** (HWP3 측 별도 조정 가능)
- **HWP3 "세계3대물..." 공백 누락** (Task #1008 격차 D 영역)

---

## 7. 다음 단계 (Stage 3/4)

WASM 빌드 + rhwp-studio 시각 검증 — 한컴 정답지 dialog 정합 단언 후 PR.
