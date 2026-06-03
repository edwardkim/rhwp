# Task #741 Stage 8 — HWP3 차례 inline page 번호 (ch=1) 정합

## 영역

HWP3 차례 (TOC) entry paragraph 의 inline page 번호 control (`ch=1`) 해석 정합.

## 본질 진단

### 결함

HWP3 차례 entries (paragraph 28~339 영역) 가 페이지 번호 reference 를 inline `ch=1` control 로 저장. 본 환경 catch-all `_` 분기에서 ch=1 처리 시:
- header_val1 (4 bytes) + ch2 (2 bytes) 읽지만 unused
- text_string 에 `'\u{FFFC}'` placeholder push
- Unknown control 등록

→ 시각: 차례 entries 가 "EXPORT/IMPORT Q & A\t" 까지만 표시, page 번호 누락. 한컴 viewer + HWP5 변환본 정합 부재.

### Byte 영역 진단 (HWP3_DIAG_CTRL1 임시)

`hwp3-sample10.hwp` 의 ch=1 occurrences = **290** (paragraph 29~339 차례 영역).

byte 패턴 분석:
- header_val1 first u16 (LE): 항상 **0x0009** (page reference marker)
- header_val1 second u16: digit1 **ASCII 0x0030~0x0039** ('0'~'9')
- ch2: digit2 ASCII 또는 **0x000D (CR)** for 1-digit terminator

예시:
- paragraph 29 (page 1): header_val1=0x00310009, ch2=0x000D → "1"
- paragraph 50 (page 45): header_val1=0x00340009, ch2=0x0035 → "45"
- paragraph 339 (page 757): header_val1=0x00370009, ch2=0x0035 → "75" + 원본 literal "7" = "757" (3-digit 자연 정합)

3-digit 페이지 번호는 ch=1 가 first 2 digit 만 저장, 세 번째 digit 은 literal char 로 paragraph text 안에 별도 저장.

## 정정 영역 ([src/parser/hwp3/mod.rs](src/parser/hwp3/mod.rs))

```rust
1 => {
    let header_val1 = body_cursor.read_u32::<LittleEndian>()?;
    let ch2 = body_cursor.read_u16::<LittleEndian>()?;
    i += 3;

    let digit1_u16 = ((header_val1 >> 16) & 0xFFFF) as u16;
    let mut page_str = String::new();
    if (0x0030..=0x0039).contains(&digit1_u16) {
        page_str.push(char::from_u32(digit1_u16 as u32).unwrap_or('?'));
    }
    if (0x0030..=0x0039).contains(&ch2) {
        page_str.push(char::from_u32(ch2 as u32).unwrap_or('?'));
    }

    if !page_str.is_empty() {
        for c in page_str.chars() {
            char_offsets.push(utf16_len);
            utf16_len += c.len_utf16() as u32;
            text_string.push(c);
        }
    } else {
        // unrecognized → fallback placeholder
        text_string.push('\u{FFFC}');
        controls.push(...);
    }
}
```

## 검증

| paragraph | 정정 전 | 정정 후 | HWP5 변환본 | PDF 한컴 viewer |
|-----------|---------|---------|-------------|-----------------|
| 29 (page 1) | "...\t￼" | "...\t1" ✓ | "...\t1" ✓ | "...\t1" ✓ |
| 50 (page 45) | "...\t￼" | "...\t45" ✓ | "...\t45" ✓ | "...\t45" ✓ |
| 339 (page 757) | "...\t￼7" | "...\t757" ✓ | "...\t757" ✓ | "...\t757" ✓ |

### 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | 1166 passed |
| `cargo clippy --release --lib` | 신규 경고 0 |
| `scripts/svg_regression_diff.sh build b793ad5 HEAD` | TOTAL pages=170 same=170 diff=0 (회귀 0) |

### 시각 정합 (PDF cross-check)

`pdf/hwp3-sample10-hwp5-2022.pdf` 페이지 2 차례 영역 — 모든 TOC entries 의 페이지 번호 (1, 4, 5, 16, 17, 18, ..., 102, 108, ..., 134) 가 정상 표시. 한컴 viewer 정합.

## 후속

- 4-digit 페이지 번호 (1000+) 등장 시 추가 verification
- ch=1 외 다른 inline reference control 등장 시 동일 패턴 처리
