# Task #1008 Stage 4 완료 보고서 — 격차 B fix (HWP3 Shape border LineType normalize)

**Issue**: [#1008 HWP3 sample16 Shape/Text 정합 격차 종합](https://github.com/edwardkim/regression-rhwp/issues/1008)
**Branch**: `local/task1008`
**작업 내용**: HWP3 raw line_style 2~7 (Dash/Dot/...) 을 1 (Solid) 로 normalize — 한컴 viewer 정합

---

## 1. 진단 — HWP3 line_style 분포 단언

| Sample | line_style 분포 |
|--------|----------------|
| hwp3-sample10/11/13/14/4/5 | 없음 또는 0x0000 |
| hwp3-sample16 | **0x0001 + 0x0002** |
| hwp3-sample19 | 0x0001 |

`0x0002` (LineType 2 = Dash per HWP5 spec) 은 **sample16 한정**. 다른 HWP3 fixture 에는 부재 — narrow fix 회귀 risk 0.

## 2. SVG dasharray 단언 (BEFORE Stage 4)

```xml
<rect ... fill="url(#grad1)" stroke="#000000" stroke-width="1" stroke-dasharray="6 3"/>
```

→ 점선 (6px dash + 3px gap) — 한컴 정답 (실선) 과 다름.

---

## 3. Fix — HWP3 parser LineType normalize

`src/parser/hwp3/drawing.rs:758~775` 의 `border_line.attr` 산출 변경:

```rust
attr: {
    let raw_attr = header.basic_attr.line_style as u32;
    let line_type = raw_attr & 0x3F;
    if line_type == 0 && header.basic_attr.line_width > 0 {
        raw_attr | 0x01  // Task #877: None + width>0 → Solid
    } else if (2..=7).contains(&line_type) {
        // [Task #1008 격차 B] HWP3 LineType 2~7 (점선/일점쇄선 등) → Solid
        // 한컴 viewer 가 HWP3 raw 의 dashed/dotted 를 solid 로 렌더하는 동작
        // 정합. HWP3 sample 분포 sweep: line_style=2 sample16 한정.
        (raw_attr & !0x3F) | 0x01
    } else {
        raw_attr
    }
},
```

영향 범위: HWP3 한정 (HWP5/HWPX parser 무관). 회귀 risk: 다른 HWP3 fixture 에 LineType 2+ 가 없어 안전.

---

## 4. dump 단언

### 4.1 sample16 pi=71 (사업개요 박스)

```
BEFORE: 선: color=0x00000000, width=56, style=0x0002
AFTER:  선: color=0x00000000, width=56, style=0x0001  (Solid normalize)
```

### 4.2 sample16 pi=5 (cover RFP 박스, line_style=1)

```
BEFORE/AFTER 동일: style=0x0001 (영향 없음)
```

---

## 5. SVG 단언 (AFTER Stage 4)

```xml
<rect ... fill="url(#grad1)" stroke="#000000" stroke-width="0.747"/>
```

→ `stroke-dasharray` 제거 — 실선 렌더 정합.

---

## 6. 회귀 sweep

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✓ warning 0 |
| `cargo clippy --release --lib -- -D warnings` | ✓ clean |
| `cargo fmt --check` | ✓ clean |
| `cargo test --release --lib` | ✓ 1307 passed |
| `cargo test --release --test issue_1008_gradient` | ✓ 3 passed (격차 A + B + C) |

### 6.1 페이지 수 sweep (HWP3 9종 + HWP5/HWPX 변환본 + 일반 fixture)

| Sample | 페이지 |
|--------|--------|
| hwp3-sample10 | 763 ✓ |
| hwp3-sample11 | 151 ✓ |
| hwp3-sample13 | 3 ✓ |
| hwp3-sample14 | 11 ✓ |
| hwp3-sample16 | 64 ✓ |
| hwp3-sample19 | 2 ✓ |
| hwp3-sample4 | 36 ✓ |
| hwp3-sample5 | 64 ✓ |
| hwp3-sample16-hwp5 | 64 ✓ |
| exam_kor / eng / math | 20 / 8 / 20 ✓ |
| aift / biz_plan | 74 / 6 ✓ |

→ 모든 fixture 페이지 수 회귀 0.

---

## 7. 단위 테스트 추가

`tests/issue_1008_gradient.rs::hwp3_sample16_business_box_border_solid`:
- HWP3 sample16 pi=71 Rectangle 의 `border_line.attr & 0x3F == 1 (Solid)` 단언

---

## 8. 한계 (HWP5 변환본)

HWP5 변환본 (`hwp3-sample16-hwp5.hwp`) 의 동일 박스도 `attr=0xc0010043` → LineType 3 (Dot) → 점선 렌더 중. 한컴 정답 = 실선. 본 stage 에서는 **HWP3 만 fix** (작업지시자 "hwp5 포맷은 정상임" 발언).

HWP5 변환본도 fix 필요 시:
- HWP5 parser 또는 renderer 에 variant-gated override 추가
- 회귀 risk: 일반 HWP5 의 의도된 점선 영향 — 정밀 단언 필요

본 task 후속 또는 사용자 결정.

---

## 9. 성공 기준 충족

| 조건 | 결과 |
|------|------|
| C2: HWP3 Shape border 실선 정합 | ✓ (HWP3 fix 적용) |
| C2 (HWP5 변환본): | (사용자 결정 — 본 stage 외) |
| C5: 페이지 수 유지 | ✓ |
| C6: 회귀 0 | ✓ |
| C7: cargo test | ✓ |
| C8: 시각 검증 | (Stage 6 시점) |

---

## 10. 다음 단계 (Stage 5)

격차 D — HWP3 한글 단어 공백 시각 누락 (renderer 영역). parser 데이터는 정상 — visual rendering 단계 (text run / char_position) 진단.
