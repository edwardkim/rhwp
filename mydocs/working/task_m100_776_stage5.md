# Task #776 Stage 5 — 광범위 검증 + edge case

**Issue**: [#776](https://github.com/edwardkim/rhwp/issues/776)
**Stage**: 5 — 다양한 샘플 + edge case 검증
**작성일**: 2026-05-10

---

## 다양한 샘플 검증

### `samples/basic/*.hwp` 핵심 샘플 분석

| 샘플 | pi=0 sb | body_top | H1' 영향 | H3b 영향 |
|------|--------|---------|---------|---------|
| shortcut.hwp | 3968 | 56.7 | ✓ +26.45 px | ✓ 다단 zone 전환 |
| sungeo.hwp | 2000 | 132.3 | ✓ +13.33 px | (단일 zone) |
| treatise sample.hwp | 3600 | 132.3 | ✓ +24.00 px | (다단 있으나 미적용 케이스) |
| Textmail.hwp | 0 | 113.4 | (sb=0, 영향 없음) | (단일 zone) |
| BookReview.hwp | 0 | 37.8 | (sb=0, 영향 없음) | (단일 zone) |
| english.hwp | 0 | 132.3 | (sb=0, 영향 없음) | (단일 zone) |
| KTX.hwp | 0 | 37.8 | (sb=0, 영향 없음) | (단일 zone) |
| calendar_year.hwp | 0 | 37.8 | (sb=0, 영향 없음) | (단일 zone) |
| Hyper(hwp2010).hwp | 0 | 146.7 | (sb=0, 영향 없음) | (단일 zone) |
| interview.hwp | 0 | 132.3 | (sb=0, 영향 없음) | (단일 zone) |

H1' / H3b 영향 받는 샘플 비율: 3/10 (30%). 대부분 샘플은 변경 없음 (회귀 안전).

### 다단 layout 샘플 페이지 카운트

| 샘플 | 페이지 수 | 회귀 |
|------|---------|------|
| exam_kor.hwp | 20 | 없음 ✓ |
| exam_eng.hwp | 8 | 없음 ✓ |
| exam_math.hwp | 20 | 없음 ✓ |
| shortcut.hwp | 8 | 없음 ✓ |

## Edge case 검증

### Edge 1: ColumnDef.spacing = 0

```rust
.unwrap_or(0.0)  // ColumnDef 없을 때
.spacing == 0   // ColumnDef.spacing 이 0 일 때 → / 2.0 = 0.0
```

→ 변경 없음 (회귀 안전).

### Edge 2: pi=0 sb = 0

`spacing_before > 0.0` 조건으로 가드 → 변경 없음.

### Edge 3: 셀 안 paragraph

`cell_ctx.is_some()` 가드 → sb skip (변경 없음). cell padding + sb 중복 방지.

### Edge 4: 첫 zone (zone_y_offset = 0)

`if col_content.zone_y_offset > 0.0` 조건으로 가드 → 변경 없음.

### Edge 5: PartialParagraph (start_line > 0)

`start_line == 0` 조건으로 가드 → 변경 없음. PartialParagraph 후속 줄에 sb 중복 적용 방지.

## 시각 정합 검증 (SVG → PDF)

### shortcut.hwp 페이지 1

SVG 측정 (100 dpi → 96 dpi 환산):
- 글 (heading) baseline ≈ 101.58 px → PDF 110.24 (diff -8.66, baseline vs yMax)
- 서 (TAC table) baseline ≈ 159.23 px → PDF 160.32 (diff -1.09)
- 빈 (body) baseline ≈ 197.80 px → PDF 207.85 (diff -10.05)

paragraph y_in 비교 (issue_776 test 측정):
- pi=0 offset 26.45 vs PDF 26.83 (±0.38) ✓
- pi=2 offset 138.01 vs PDF 137.87 (±0.14) ✓

baseline vs y_in 차이는 글꼴의 ascent/descent 미세 차이로, 본 task 의 paragraph spacing 결함과 무관.

## RFC #774 예측 정확도 재검증

| 측정 | 정정 전 | 정정 후 | 변화 | RFC 예측 | 정확도 |
|------|--------|--------|-----|---------|------|
| shortcut.hwp pi=0 offset | 0.00 | 26.45 | +26.45 | sb=26.45 | ±0.00 |
| shortcut.hwp pi=2 offset | 73.76 | 138.01 | +64.25 | H1'+H3b=64.25 | ±0.00 |
| sungeo.hwp pi=0 offset | 0.00 | 13.33 | +13.33 | sb=13.33 | ±0.00 |
| treatise pi=0 offset | 0.00 | 24.00 | +24.00 | sb=24.00 | ±0.00 |

**RFC #774 예측 100% 정확** — 모든 변화량이 RFC 예측과 일치.

## 미해결 영역 (RFC 권고)

본 task 범위 외:
- `/2` factor 의 의미 (이론적 근거)
- 셀 안 paragraph 의 한컴 PDF 정합 패턴
- ParaShape.spacing_after (sa) 정합 패턴
- 다중 zone (3+ zone) 의 H3b 누적 동작

→ 별도 RFC 또는 후속 task 영역.

## 다음 단계

Stage 6 — 최종 보고서 + 이슈 close.
