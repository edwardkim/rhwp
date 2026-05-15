# Task #901 Stage 12 보고서 — HWPX parser secPr/colPr utf16 정합

**Stage**: 12
**상태**: HWPX paragraph 0 vertical 정합 완성 ✅ (1402 test 회귀 없음)

## 1. 진단

HWPX (pic2.hwpx) 의 paragraph 0 rendering 이 broken — 우/리/나/라 모두 y=164 (horizontal) 으로 렌더. 한편 HWP (pic2.hwp) 는 우 224 / 리 223.7 / 나 283 / 라 343 (vertical, 60 px gap).

### 1.1 ROOT CAUSE

HWPX parser 의 `secPr` 와 `colPr` 처리가 `text_parts` 에 `\u{0002}` placeholder 를 push 하지 않아 utf16_pos 가 8씩 증가 못함.

| 영역 | HWP utf16 contrib | HWPX utf16 contrib (수정 전) |
|------|------------------|----------------------------|
| SectionDef | +8 | 0 (누락) |
| ColumnDef (secPr 내부) | +8 | 0 (누락) |
| ColumnDef (ctrl/colPr Empty) | +8 | 0 (누락) |
| Picture × 2 | +16 | +16 |
| 텍스트 " 우리나라" | +5 | +5 |
| **총 (+1)** | **38** | **22** |

line_seg.text_start 값은 HWP file format 의 utf16 기반 인코딩 (예: ls[1].text_start=33). HWPX 에서 char_offsets 가 [16, 17, ...] 로 시작하면 text_start=33 매핑 실패 → `utf16_range_to_text_range` 가 text_len 반환 → compose_lines 가 모든 chars 를 line 0 에 packing.

## 2. Fix

`src/parser/hwpx/section.rs`:

1. `b"secPr"` 처리 시 `text_parts.push("\u{0002}")` 추가 (SectionDef)
2. `parse_sec_pr_children` 의 inner `colPr` 결과 처리 시 `text_parts.push("\u{0002}")` (ColumnDef)
3. `parse_ctrl` 의 `Event::Empty` `b"colPr"` 처리 시 `text_parts.push("\u{0002}")` 추가

## 3. 결과

HWPX paragraph 0 dump:
- 이전: cc=22, controls=3
- 이후: cc=38, controls=3 (HWP 정합)

HWPX paragraph 0 rendering:
| char | 이전 y | 이후 y |
|------|--------|--------|
| 우 | 164 (horizontal) | **164** (vertical) ✓ |
| 리 | 164 | **223.7** ✓ |
| 나 | 164 | **283** ✓ |
| 라 | 164 | **343** ✓ |

60 px line gap 정합 (한컴 정합, HWP 결과와 동일).

## 4. 회귀 검증

- ✅ `cargo test --release --all-targets`: **1402 passed, 0 failed**
- ✅ HWPX paragraph 0 한컴 정합 (HWP 결과와 동일)
- ✅ HWP 결과 회귀 없음

## 5. 잔존 차이 재평가

이전 "~10mm 공통 vertical offset" 추정 재측정:
- rhwp 우 baseline y=164 (96 DPI)
- PDF 우 box top y=150 (108 DPI) → font ascender 변환 → baseline ≈ 178 @ 108 DPI = 158 @ 96 DPI
- 실제 offset ≈ 6 px = ~1.6 mm

PDF (pdftohtml) 의 y 가 box top, SVG y 가 baseline 인 좌표 의미 차이로 이전 과대 측정. 실제 차이는 시각적 임팩트 적음.
