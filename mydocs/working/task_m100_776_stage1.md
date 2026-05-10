# Task #776 Stage 1 — RED test 가드 (정정 전)

**Issue**: [#776](https://github.com/edwardkim/rhwp/issues/776) Task #773 후속 정정 (H1' + H3b)
**Stage**: 1 — RED test 가드 작성 + 정정 전 FAIL 확인
**작성일**: 2026-05-10

---

## 작성된 가드

`tests/issue_776.rs` (신규):

| Test | 측정 | 정정 전 (현재) | 기대 (PDF) | 결과 |
|------|------|------------|----------|------|
| `issue_776_h1prime_shortcut_heading_offset` | shortcut.hwp pi=0 offset | 0.00 px | 26.83 px | **RED** |
| `issue_776_h3b_shortcut_body_offset` | shortcut.hwp pi=2 offset | 73.76 px | 137.87 px | **RED** |
| `issue_776_h1prime_sungeo_heading_offset` | sungeo.hwp pi=0 offset | 0.00 px | 12.63 px | **RED** |
| `issue_776_h1prime_treatise_heading_offset` | treatise sample.hwp pi=0 offset | 0.00 px | 23.69 px | **RED** |

## 측정 분석

### shortcut.hwp 페이지 1

```
pi=0 (heading) y_in = 56.69 (= body_top, sb 누락)
pi=2 (본문) y_in = 130.45 (= body_top + 73.76)
```

PDF 정합:
- pi=0 should be at body_top + sb (3968 HU = 26.45 px) = 83.14 px
- pi=2 should be at body_top + 137.87 = 194.56 px

정정 후 예측:
- pi=0: 56.69 + 26.45 (H1') = 83.14 → offset 26.45 ≈ 26.83 ✓
- pi=2: 130.45 + 26.45 (H1') + 37.80 (H3b 누적) = 194.70 → offset 138.01 ≈ 137.87 ✓

### sungeo.hwp / treatise sample.hwp

단일 zone 샘플. H1' 만 적용:
- sungeo: 132.28 + 13.33 (H1') = 145.61 → offset 13.33 ≈ 12.63 ✓
- treatise: 132.28 + 24.00 (H1') = 156.28 → offset 24.00 ≈ 23.69 ✓

## 회귀 baseline

**정정 전 cargo test**: 1217 passed (issue_776 4 RED 제외).

## Body filter

`find_body_first_textline_y` 함수: 바탕쪽/머리말/꼬리말 (Header/Footer 노드) 제외, Body 하위에서만 paragraph_index 일치 TextLine 검색.

이는 shortcut.hwp 의 바탕쪽 페이지 번호 ("1") textline 이 pi=0 으로 잡히는 것을 방지.

## 다음 단계

Stage 2 — H1' 정정 (paragraph_layout.rs:744-748). 정정 후 sungeo / treatise heading test 통과 + shortcut pi=0 통과 + shortcut pi=2 부분 통과 (H3b 미적용).
