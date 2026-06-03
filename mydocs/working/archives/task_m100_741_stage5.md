# Task #741 Stage 5 — HWP3 사적 graphic char 매핑

## 영역

HWP3 hchar 0x0080~0x7FFF 영역 (표준 KSSM 조합형 영역 0x8000+ 외) 한컴 사적 인코딩 char 정정.

## 본질 진단

### 미매핑 ch 통계 (hwp3-sample10.hwp 전체 파싱)

| ch 값 | 횟수 | 비율 | 한컴 변환본 (HWP5) | 의미 |
|-------|------|------|---------------------|------|
| **0x301C** | **24,695** | **94.5%** | U+F080F (PUA) | 한컴 사적 굵은 가로선 |
| 0x35E1 | 892 | 3.4% | U+2500 ─ | BOX DRAWINGS LIGHT HORIZONTAL |
| 0x303D | 129 | 0.5% | U+F0827 (PUA) | 한컴 사적 |
| 0x3479 | 11 | 0.04% | U+25B7 ▷ | WHITE RIGHT-POINTING TRIANGLE |
| 0x347A | 4 | 0.02% | U+25B6 ▶ | BLACK RIGHT-POINTING TRIANGLE |
| 0x3441 | 7 | 0.03% | U+25A0 ■ | BLACK SQUARE |
| 기타 ~298 | ~389 | 1.5% | (low confidence) | tab leader / 사적 |

**상위 6값 = 25,738 / 26,127 = 98.5% coverage.**

### 매핑 도출 영역

`hwp3-sample10.hwp` ↔ `hwp3-sample10-hwp5.hwp` (한컴 변환본) paragraph 별 cross-reference:

- HWP3 paragraph N 의 미매핑 ch 위치 → HWP5 paragraph N 동일 위치 Unicode char 비교
- multi-paragraph 검증: 각 ch 값을 5+ paragraphs 에서 동일 Unicode 매핑 확인

검증 예시:
- HWP3 p.1145 (0x301C × 94) ↔ HWP5 p.1145 ("󰠏󰠏...󰠏" U+F080F × 94) — 5+ paragraphs 검증
- HWP3 p.343 (0x35E1 × 48) ↔ HWP5 p.343 ("───...─" U+2500 × 47) — 5+ paragraphs 검증
- HWP3 p.28 (0x3479 char_i=0) ↔ HWP5 p.28 ("▷ EXPORT") — 3 paragraphs 검증

### Reference 자료 한계

사용자 제공 `Source-완성조합-코드변환` (강승식 교수, 1993-2005):
- KSSM 한글 영역 0x8000+ → KSC → Unicode 표준 변환 영역 cover
- 본 환경 미매핑 0x0080~0x7FFF 영역 **cover 불가** (한컴 사적 인코딩)

→ **한컴 변환본 cross-ref** 가 유일한 신뢰성 있는 매핑 source.

## 정정 영역

### `src/parser/hwp3/johab.rs`

```rust
} else if ch >= 0x0080 {
    // [Task #741 Stage 5] HWP3 사적 graphic char 영역 (0x0080~0x7FFF).
    if let Some(c) = decode_hwp3_extra(ch) {
        return c;
    }
}

fn decode_hwp3_extra(ch: u16) -> Option<char> {
    let codepoint: u32 = match ch {
        0x301C => 0xF080F,  // 한컴 PUA — 굵은 가로선 (94.5% 발생)
        0x35E1 => 0x2500,   // ─ BOX DRAWINGS LIGHT HORIZONTAL
        0x303D => 0xF0827,  // 한컴 PUA
        0x3479 => 0x25B7,   // ▷ WHITE RIGHT-POINTING TRIANGLE
        0x347A => 0x25B6,   // ▶ BLACK RIGHT-POINTING TRIANGLE
        0x3441 => 0x25A0,   // ■ BLACK SQUARE
        _ => return None,
    };
    char::from_u32(codepoint)
}
```

매핑 target: **PUA 보존** (HWP5 변환본 IR 정합) — 사용자 결정.

## 검증

### 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | 1166 passed |
| `cargo clippy --release --lib` | 신규 경고 0 |
| `scripts/svg_regression_diff.sh build a63114e HEAD` | TOTAL: pages=170 same=170 diff=0 |

### 시각 정합 (PDF 권위 자료 cross-check)

`pdf/hwp3-sample10-hwp5-2022.pdf` (한글 2022 편집기 PDF) 페이지 2 (제목차례) 와 본 환경 `samples/hwp3-sample10.hwp` 페이지 2 (HWP3 native) 비교:

**개선 효과:**
- ✓ ▷ EXPORT/IMPORT/LOADER markers 표시 (이전 누락)
- ✓ ■ 제목차례 ■ markers 표시
- ✓ TOC ▷ ORACLE GRAPHICS / ▷ RDBMS 등 모두 표시
- ✓ 가로선 ─ 표시 (paragraph 343 등)

**잔여 결함 (Stage 5 영역 외):**
- 가로선 ═════ 제목차례 ═════ borders — paragraph 26 char encoding 영역이 아닌 별도 paragraph 또는 ParaShape tab fill (Stage 6 영역)
- TOC tab leader 가로선 (────) — ParaShape tab_def 영역 (Stage 6)
- 페이지 번호 우측 정렬 — ParaShape tab 영역 (Stage 6)

## 영역 외 후속

Stage 6: ParaShape tab_def 처리 (페이지 번호 우측 정렬 + 가로선 채움) — 별도 task 영역.

## 자료 추가

| 경로 | 용도 |
|------|------|
| `samples/hwp3-sample10.hwp` | HWP3 native sample (Oracle 기술 문서) |
| `samples/hwp3-sample10-hwp5.hwp` | 한컴 HWP5 변환본 (cross-ref 권위) |
| `samples/hwp3-sample10-hwpx.hwpx` | 한컴 HWPX 변환본 |
| `pdf/hwp3-sample10-hwp5-2022.pdf` | 한글 2022 편집기 PDF (PDF 권위) |
