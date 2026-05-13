# Stage 1 완료 보고서 — Task #866 (M100) — PDF 측정 + IR 대조

GitHub Issue: edwardkim/rhwp#866 · 브랜치: `local/task866` (← `pr-task853`) · 코드 미수정 (측정 전용)

## 1. PDF 측정 (`pdf/basic/shortcut-2022.pdf`, `mutool draw -r 100`, @96dpi 환산, body_top=15mm 기준)

| 페이지 | 헤더 | 헤더 띠 상단 | 헤더 띠 하단 | 본문 첫 줄 상단 | 띠 하단↔본문 |
|--------|------|-------------|-------------|----------------|-------------|
| 2 | "파일" (pi=36) | +19.1px | +43.1px | +74.8px | **+31.7px** |
| 3 | "보기" (pi=81) | +25.8px | +49.8px | +83.4px | **+33.6px** |
| 4 | (헤더 없음 — 내용 흐름) | — | — | +69.0px(이전 zone 연속) | — |
| 5 | (헤더 없음) | — | — | — | — |

## 2. rhwp 상태 (PR #868 Task #853 적용 후)

- 2쪽: 헤더 띠 +19.8px ✓ / 하단 +43.3px ✓ / 본문 zone_y_offset = +47.1px(= pi=36 `vpos_zone_height` = ls[1].vpos 1200 + lh 2332 + ls 0 = 3532HU) → 띠 하단↔본문 ≈ **+4px**. **PDF(+31.7px) 대비 ~28px 부족.**
- 3쪽: pi=81 은 IR 상 LINE_SEG 1개(`ls[0]: vpos=0, lh=1200, ls=480`) — `vpos_zone_height` = 1680HU = 22.4px (표 23.5px 보다도 작음). dump-pages 의 본문 zone_y_offset = 59.9px (layout 의 표 band 처리로 보정됨). PDF 본문(+83.4px) 대비 ~24px 부족.

## 3. 분석 — 후보 검토

### 띠 하단↔본문 gap ≈ ~32px ≈ TAC 표 band 의 line height (~31px)
- pi=36 의 line1(표 줄) lh = 2332 HU = 31.1px (= 표 본체 1766 + outer_margin 283×2 = 566). PDF gap ≈ 31.7px ≈ 이 값. pi=81 도 ≈ 33.6px ≈ 비슷.
- 가설: 한컴은 `wrap=위아래(TopAndBottom)` 인 TAC 표를 (a) 인라인으로 line(text line 다음 줄)에 배치 + (b) `위아래` 어울림으로 표 band 높이만큼 그 아래에 *추가* 예약 → 본문이 `(line0)+(표 band 인라인)+(표 band 추가 예약)` 만큼 아래에서 시작. pi=36: 16 + 31 + 31 ≈ 78px ≈ PDF 74.8px(±3px). pi=81 도 ≈ 84px ≈ PDF 83.4px.

### 미확정 — 게이팅 조건이 불명확 + 회귀 위험
- 1쪽 헤더 띠(pi=1, `text_len=0`, ColumnDef `간격=10mm`)는 PR #868 Stage 3-3(ColumnDef 간격 → zone 진입 간격)로 **이미 PDF 정합**(헤더 zone +88px, 본문 +138px ≈ PDF +87.6/+137.9px). 여기에 "표 band 추가 예약(~31px)"을 또 더하면 overshoot. → 위 가설은 1쪽 pi=1 에는 적용되면 안 됨.
- 차이점: pi=1 은 `text_len=0`(line0 텍스트 없음, ColumnDef 간격 10mm), pi=36 은 `text_len=2`(line0 텍스트, ColumnDef 간격 0mm), pi=81 은 `text_len=2`·LINE_SEG 1개·ColumnDef 간격 0mm·PS `line=140%`(pi=36 은 100%) — IR 구조가 페이지마다 제각각이라 "표 band 추가 예약" 적용 조건을 룰로 확정하기 어려움(`feedback_rule_not_heuristic` 위배 소지).
- 또 본문 zone_y_offset 계산은 `process_multicolumn_break`(`vpos_zone_height`) + `layout_table_item`(표 band 렌더) 두 곳이 협조해야 하므로 양쪽 동시 수정 필요 → composer/typeset/layout 광역 회귀 위험(`feedback_essential_fix_regression_risk`). 닫힌 PR #771/Issue #770(line0 흡수 ~16px 만 다룸), 닫힌 #773/#776, RFC #774 가 모두 이 ~28px 를 못 닫은 이유.

## 4. 권고 — 보류

PDF 측정만으로는 ~32px 의 *기여 요소*(≈ 표 band 높이)는 좁혀졌으나, **적용 게이팅 조건이 IR 구조 차이로 확정 불가**하다(1쪽 pi=1 vs 2쪽 pi=36 vs 3쪽 pi=81 이 모두 다름). 한컴 편집기(Windows)에서 실제 layout 구조(표 band 의 위아래 예약 여부, 본문 paragraph 의 zone 진입 offset)를 직접 확인하지 않으면 추측 구현이 되고, composer/typeset/layout 협조 변경이라 회귀 위험이 큼.

→ **본 타스크는 Stage 1(측정) 로 마감하고 #866 은 한컴 편집기 접근 가능 환경/시점까지 보류** 권고. 측정 데이터·가설은 본 보고서 + `mydocs/tech/hancom_zone_paragraph_spacing.md` §5 에 보존. (#770 코멘트 "필요 시 신규 이슈로 재등록" 의 #866 이 그 신규 이슈이며, 본 Stage 1 으로 가설을 한 단계 좁힘.)

만약 강행한다면: Stage 2 = `process_multicolumn_break` 의 `vpos_zone_height` 에 "직전 zone 의 마지막 paragraph 가 `wrap=위아래` TAC 표 보유 & multi-line(line0 텍스트 존재) & ColumnDef 간격=0 이면 표 band 높이(`table.common.height + outer_margin_top + outer_margin_bottom` px)를 추가" + `layout_table_item` 에 동일 advance 보장 → `cargo test` + 전 fixture sweep + `pdf-2020/` 대조. 단 게이팅 조건이 휴리스틱이므로 비권장.

## 산출물
- 본 보고서 + 측정 PNG(`/tmp/sc_p{2..5}.png` — 임시). 소스 변경 없음.
