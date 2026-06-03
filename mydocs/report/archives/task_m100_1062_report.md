# 최종 결과 보고서 — Task #1062: 시험지 미주(Endnote) 본문 하단 overflow 정합

- 이슈: edwardkim/rhwp#1062 (#1049 후속)
- 브랜치: `local/task1062` (stream/devel 기준)
- 변경 범위: `src/renderer/typeset.rs` 미주 루프 한정

## 1. 문제

다단(2단) 시험지 문서에서 본문 컬럼 하단을 넘어 항목이 적층되는 `LAYOUT_OVERFLOW` 다수 발생,
한컴 2022 PDF 대비 쪽수 부족.

| 대상 (HWP·HWPX 동일) | LAYOUT_OVERFLOW(item) | 우리/PDF 쪽수 |
|------|------|------|
| 3-09월_교육_통합_2022 | 155 | 21/23 |
| 3-09월_교육_통합_2023 | 119 | 19/20 |
| 3-10월_교육_통합_2022 | 112 | 16/18 |
| 3-11월_실전_통합_2022 | 94 | 19/21 |

## 2. 근본 원인 (재진단으로 정정)

> 최초 가설 "본문 빈 문단 연속 trailing-ls drift" 는 코드 정독 결과 **위치 오귀속**으로
> 판명. 실제는 **미주(Endnote) 레이아웃**.

- IR 본문 문단 468개, 표 0개. 그러나 overflow 항목 `para_index` = 468~1181.
- `typeset.rs:1405` `en_para_idx = paragraphs.len() + st.endnote_paragraphs.len()` → overflow
  항목은 전부 **미주(해설) 문단**(미주 54개 / 미주 문단 ~714개).
- 다단 미주 누적(`typeset.rs`)이 `height_for_fit`(trailing_ls 제외)이라 렌더러의 vpos 전진
  (lh+ls)보다 미주당 ~6px(=`ls=452hu`) **과소 계상** → 페이지당 미주 과밀 → 렌더러가 vpos base
  리셋(다음 페이지) 전에 본문 하단 초과. 검산: 페이지당 ~30미주 × 6px ≈ 180px ≈ 관측 overshoot.

## 3. 구현

`typeset.rs` 미주 루프의 다단 누적/판정을 렌더러 vpos 전진과 통일 (단단은 종전 유지):

```rust
let (en_fit, en_advance) = if st.col_count > 1 {
    let advance = /* last.vpos + lh + ls − first.vpos (line_segs 기반, 없으면 total_height) */;
    let trailing_ls = /* last.line_spacing(px) */;
    // 안전 하한: 종전 height_for_fit 미만으로 내려가지 않게 floor (회귀 차단)
    ((advance - trailing_ls).max(fmt.height_for_fit), advance.max(fmt.height_for_fit))
} else {
    (fmt.height_for_fit, fmt.total_height)
};
```

## 4. 결과 (devel 대비)

| 지표 | 값 |
|------|------|
| 251 샘플 LAYOUT_OVERFLOW 합 | 1624 → **769 (−855, −53%)** |
| `cargo test --release` | **1550 passed / 0 failed** |
| 골든 SVG 8종 | 통과 (시각 회귀 0) |

| 대상 | overflow | 쪽수 우리/PDF |
|------|------|------|
| 3-09 2022 | 155→20 | 22/23 |
| 3-09 2023 | 119→20 | 20/20 ✓ |
| 3-10 2022 | 112→7 | 18/18 ✓ |
| 3-11 2022 | 94→4 | 21/21 ✓ |

비회귀: exam_eng 12→11, exam_kor 19→19, k-water-rfp 3→3, 복학원서 2→2, footnote-01/endnote-01 0→0.

## 5. 잔여 (별도 후속 권장)

- 3-09 2022 잔여 20건 + 악화 4파일(+8, 3~23px, 골든 무회귀: sungeo·hwpctl_API·hwp3-sample4/10).
- 조사 결과 누적 과소가 아님(누적 floor 로도 불변) — **typeset 미주 분할점 ↔ 렌더러 vpos base
  리셋 지점 미정렬**(렌더러 `height_cursor` 측). 본 과제 핵심(미주 trailing_ls)과 분리되는
  별도 결함 → 후속 이슈 권장.

## 6. 단계 기록

Stage 1~2(본문 trailing_ls 노선, 폐기) → revised Stage 1~2(미주 원인 정정) → Stage 3(구현) →
Stage 4(검증) → Stage 5(잔여 조사 + floor 보정). 상세 `mydocs/working/task_m100_1062_stage*.md`.
