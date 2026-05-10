# Task #776 Stage 4 — 회귀 검증

**Issue**: [#776](https://github.com/edwardkim/rhwp/issues/776)
**Stage**: 4 — H1' + H3b 정정의 cargo test + 다단 layout + 시각 검증
**작성일**: 2026-05-10

---

## cargo test 전체

```
$ cargo test --release
test result: ok. 1217 passed; 0 failed; 2 ignored; 0 measured
```

**1217 passed, 0 failed** — 회귀 0.

## 영역별 테스트

| 영역 | 결과 |
|------|------|
| re_sample (재현 검증) | 13 passed |
| exam_math (다단 레이아웃) | 4 passed |
| exam_eng | 1 passed |
| column (다단 layout) | 27 passed |
| issue_716 (페이지 1 본문 LAYOUT_OVERFLOW) | passed |
| issue_703, 712, 713 등 | passed |

## clippy

```
$ cargo clippy --release --all-targets
... 52 warnings (모두 기존 경고, 신규 코드 영역 경고 0건) ...
```

신규 코드 (paragraph_layout.rs:744-749, layout.rs:1237-1257) 의 clippy 경고: **없음**.

## 페이지 카운트 검증

| 샘플 | 페이지 수 (정정 전/후) | 회귀 |
|------|--------------------|------|
| shortcut.hwp | 8 / 8 | 없음 ✓ |
| sungeo.hwp | 93 / 93 | 없음 ✓ |
| treatise sample.hwp | 7 / 7 | 없음 ✓ |
| exam_kor.hwp | 20 / 20 | 없음 ✓ |
| exam_eng.hwp | 8 / 8 | 없음 ✓ |
| exam_math.hwp | 20 / 20 | 없음 ✓ |

## 시각 정합 (PDF 정합 검증)

### shortcut.hwp 페이지 1 (issue_776 본 케이스)

| 항목 | rhwp 정정 후 | PDF | 차이 |
|------|------------|-----|------|
| pi=0 (heading) offset | 26.45 | 26.83 | -0.38 ✓ |
| pi=2 (body) offset | 138.01 | 137.87 | +0.14 ✓ |

### sungeo.hwp 페이지 1

| 항목 | rhwp 정정 후 | PDF | 차이 |
|------|------------|-----|------|
| pi=0 (heading) offset | 13.33 | 12.63 | +0.70 ✓ |

### treatise sample.hwp 페이지 1

| 항목 | rhwp 정정 후 | PDF | 차이 |
|------|------------|-----|------|
| pi=0 (heading) offset | 24.00 | 23.69 | +0.31 ✓ |

**모든 검증 ±1 px 이내 정합** — RFC #774 예측 정확.

## 회귀 분석

### H1' 정정 영향 분석

`is_column_top` 가드 → `cell_ctx.is_none()` 가드 변경:

영향 범위:
- 단/페이지 첫 paragraph 의 sb (정합 회복)
- 셀 안 paragraph: 변경 없음 (cell_ctx.is_some() → skip)

회귀 가능 영역 (cargo test 통과로 검증):
- 셀 padding + sb 중복 가능성 → 회귀 0
- PartialParagraph 후속 줄: start_line > 0 가드로 보호 → 회귀 0
- sb=0 paragraph: 변경 없음 → 회귀 0

### H3b 정정 영향 분석

zone 전환 시 ColumnDef.spacing / 2 가산:

영향 범위:
- ColumnDef control 을 가진 paragraph 가 새 zone 첫 항목인 경우만 적용
- ColumnDef.spacing = 0: 변경 없음
- 첫 zone (zone_y_offset = 0): 변경 없음
- ColumnDef 없음: unwrap_or(0.0) → 변경 없음

회귀 가능 영역 (cargo test 통과로 검증):
- 다단 zone 전환: 시각 정합 ✓
- 페이지 over-flow: 페이지 카운트 회귀 0
- typeset base_available_height 동기화: 자동 (페이지 카운트 보존으로 검증)

**typeset 측 동기화 추가 작업 불필요** — pagination 이 이미 zone_y_offset 기반으로 가용 공간 계산하므로 H3b 의 vertical 가산이 layout 단계에서 처리되어도 typeset 영향 없음.

## 다음 단계

Stage 5 — 광범위 검증 (다양한 샘플 + edge case).
