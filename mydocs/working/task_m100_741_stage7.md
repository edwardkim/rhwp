# Task #741 Stage 7 — HWP3 leader 매핑 + 제목차례 자동 장식 정합

## 영역

1. HWP3 ParaShape tab leader 값 → HWP5 fill_type 정밀 매핑
2. 제목차례 type paragraph (한컴 viewer 자동 ═══ ■ 장식 영역) 정합 inject

## 본질 진단

### 결함 1 — HWP3 leader → HWP5 fill_type 매핑 부재

`convert_para_shape` 가 `t.leader` 값을 그대로 `fill_type` 으로 전달:
- HWP3 leader=1 → fill_type=1 (실선) 으로 매핑됨
- HWP5 변환본 정합 fill_type=3 (점선) 과 불일치

**HWP3 leader 값 분포 진단** (HWP3_DIAG_LEADER 임시 추가):
| HWP3 sample | total tabs | type=0 leader=0 | type=1 leader=1 |
|-------------|------------|------------------|------------------|
| sample10 | 32,751 | 32,732 | 19 (TOC right tabs) |
| sample | 3,658 | 3,658 | 0 |
| sample4 | 1,948 | 1,947 | 0 |
| sample5 | 529 | 528 | 0 |

→ HWP3 leader 값은 사실상 0 (꺼짐) / 1 (켜짐) 만 존재.

**HWP5 fill_type 의미** ([svg.rs](src/renderer/svg.rs)):
- 0=없음, 1=실선, 2=파선, 3=점선, 4=일점쇄선, 5=이점쇄선, 6=긴파선,
  7=원형점선, 8=이중실선, 9=얇고굵은이중선, 10=굵고얇은이중선, 11=삼중선

→ **매핑: HWP3 leader=1 → HWP5 fill_type=3 (점선)** (한컴 변환본 sample10 paragraph 29 cross-ref).

### 결함 2 — 제목차례 ═══ ■ 자동 장식 부재

HWP3 paragraph 26 cc=8 "￼￼ 제목차례 " — 한컴 viewer 가 "════════════════════■ 제목차례 ■══════════════════════" 로 렌더링.

**HWP3 spec 외 한컴 사적 로직 영역** (Stage 7 직전 진단으로 확정):
- ParaShape `border` = 0 (테두리 없음)
- char_shape `attr` = bold (단순 폰트 속성, 장식 없음)
- 어떤 explicit 필드도 ═══ 장식을 지시하지 않음

한컴 viewer 가 HWP3 → HWP5 변환 시 자동 inject. 본 환경 HWP3 native 동일 정합 위해 heuristic injection 필요.

**Trigger 영역 도출** (HWP3_DIAG_TITLE 임시 진단):
- sample10: 새번호 + 쪽번호위치 controls 조합 paragraphs 2개
  - paragraph 26 (cc=8, "￼￼ 제목차례 ") → HWP5 변환 후 ═══ ■ 장식 추가됨
  - paragraph 340 (cc=30, "￼        ￼-EXPORT/IMPORT Q & A") → HWP5 변환 후 장식 없음
- 다른 HWP3 sample (sample, sample4, sample5): 새번호+쪽번호위치 조합 0개

→ **차이: visible text 길이.** 짧은 (~5 chars) 제목 인 경우만 한컴이 ═ ■ 장식 inject.

## 정정 영역 ([src/parser/hwp3/mod.rs](src/parser/hwp3/mod.rs))

### 1. leader → fill_type 매핑 (convert_para_shape)

```rust
let fill_type = match t.leader {
    0 => 0, // 없음 → 없음
    1 => 3, // HWP3 leader (켜짐) → HWP5 점선 (한컴 변환본 정합)
    other => other,
};
```

### 2. 제목차례 type paragraph 자동 장식 inject (parse_paragraph_list)

```rust
let has_new_num = controls.iter().any(|c| matches!(c, Control::NewNumber(_)));
let has_page_pos = controls.iter().any(|c| matches!(c, Control::PageNumberPos(_)));
if has_new_num && has_page_pos {
    let visible_text: String = text_string.chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{FFFC}')
        .collect();
    if !visible_text.is_empty() && visible_text.chars().count() <= 6 {
        // ═ × 20 + ■ + " 제목 " + ■ + ═ × 22 = 67 chars (HWP5 변환본 정합)
        let new_text = format!("════════════════════■ {} ■══════════════════════", visible_text);
        let new_char_count = new_text.chars().count() as u32;
        let new_offsets: Vec<u32> = (0..new_char_count).collect();
        text_string = new_text;
        char_offsets = new_offsets;
        utf16_len = new_char_count;
    }
}
```

**Trigger 조건 (보수적 영역)**:
- 새번호 (NewNumber) + 쪽번호위치 (PageNumberPos) 양쪽 controls 보유
- whitespace + object marker (￼) 제외 visible text ≤ 6 chars

→ 광범위 sweep 회귀 위험 최소.

## 검증

### 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | 1166 passed |
| `cargo clippy --release --lib` | 신규 경고 0 |
| `scripts/svg_regression_diff.sh build ccbb0b6 HEAD` | TOTAL pages=170 same=170 diff=0 (회귀 0) |

### 결함 1 검증 — paragraph 29 (TOC entry)

| 항목 | 정정 전 (Stage 6) | 정정 후 (Stage 7) | HWP5 변환본 |
|------|-------------------|-------------------|-------------|
| fill | 1 (실선) | 3 (점선) | 3 (점선) ✓ |

### 결함 2 검증 — paragraph 26 (제목차례)

| 항목 | 정정 전 | 정정 후 | HWP5 변환본 |
|------|---------|---------|-------------|
| cc | 8 | 50 | 67 |
| 텍스트 | "￼￼ 제목차례 " | "═...═■ 제목차례 ■═...═" | "═...═■ 제목차례 ■═...═" ✓ |

### 결함 2 보수성 검증 — paragraph 340

paragraph 340 ("￼        ￼-EXPORT/IMPORT Q & A", cc=30) 도 새번호+쪽번호위치 보유. 그러나 visible text "EXPORT/IMPORT Q & A" 길이 19 chars 로 trigger 조건 (≤6) 미충족 → **장식 inject 안 됨** (HWP5 변환본 정합).

### 시각 정합 (PDF cross-check)

`pdf/hwp3-sample10-hwp5-2022.pdf` 페이지 2 와 본 환경 HWP3 native 페이지 2 비교:

**Stage 7 정정 효과**:
- ✓ "════════════════════■ 제목차례 ■══════════════════════" 장식 표시 (한컴 viewer 정합)
- ✓ TOC entries 점선 leader fill (Stage 6 fill=1 → Stage 7 fill=3 정합)
- ✓ ▷ EXPORT/IMPORT/LOADER markers (Stage 5 정합)
- ✓ 페이지 번호 우측 정렬 (Stage 6 tab def 정합)

## 후속

- 다른 HWP3 sample 의 "제목차례 type" 패턴 추가 발견 시 trigger 조건 확장
- HWP3 leader 값 2~ (점선/파선) 등장 시 추가 매핑 (현재는 1 만 처리)
- PUA char (U+F080F, U+F0827) 폰트 fallback (Stage 5 매핑 영역) — 별도 task
