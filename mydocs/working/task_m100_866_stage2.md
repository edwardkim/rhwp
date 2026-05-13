# Stage 2 진행 보고 — Task #866 (M100) — 추가 조사, 미수정

GitHub Issue: edwardkim/rhwp#866 · 브랜치: `local/task866` (← `pr-task853`) · 상태: **조사 — 미수정, 보류 권고.**

## 추가 측정·분석 결과

Stage 1 측정(2쪽 띠↔본문 +31.7px, 3쪽 +33.6px)에 더해 IR 구조 정밀 대조:

| | 1쪽 (pi=1) | 2쪽 (pi=36) | 3쪽 (pi=81) |
|---|---|---|---|
| text_len | 0 (빈 문단) | 2 ("파일") | 2 ("보기") |
| LINE_SEG | ls[0] vpos=0 lh=2332 (표 줄 1개) | ls[0] vpos=0 lh=1200 (텍스트) + ls[1] vpos=1200 lh=2332 (표) | ls[0] vpos=0 lh=1200 ls=480 (1개; 표가 이 줄에) |
| PS line | 100% | 100% | 140% |
| ColumnDef 간격 | **10.0mm** | 0.0mm | 0.0mm |
| rhwp(PR #868) 본문 zone offset | +138.0px (= 69.1 제목 + 5mm + 31.1 헤더 + 5mm, Stage 3-3 의 10mm ColumnDef 적용) | +47.1px | +59.9px |
| 한컴 PDF 본문 위치 | +137.9px ✓ | +74.8px | +83.4px |
| 차이 | 0 (정합) | **~28px 부족** | **~24px 부족** |

### 가설 (측정 기반)
2·3쪽의 부족분(~28~31px) ≈ TAC 표 band 의 line height(2332 HU ≈ 31.1px). 즉 한컴은 이 헤더 띠 표(`treat_as_char` + `wrap=위아래`) 아래에 표 band 높이만큼을 *추가로* 비워두는 것으로 보임 — pi=36: line0(16) + 표band 인라인(31) + 표band 추가(31) ≈ 78px ≈ PDF 74.8px. pi=81: line0(22.4) + 31 + 31 ≈ 84.6px ≈ PDF 83.4px.

### 미해결 — 게이팅 조건 불명확
- 1쪽 헤더 띠(pi=1)는 `text_len=0` + ColumnDef `간격=10mm` 이라 PR #868 Stage 3-3(ColumnDef 간격/2 분배)로 **이미 정합**. 여기에 "표band 추가 예약(~31px)"을 더하면 overshoot → 위 가설은 pi=1 에는 적용되면 안 됨.
- 즉 1쪽은 "ColumnDef 간격 10mm" 이 extra 의 출처이고, 2·3쪽은 "표band ~31px 추가"가 extra 의 출처 — **서로 다른 메커니즘**으로 보이는데, 이 두 가지가 사실은 하나의 규칙인지(예: ColumnDef 간격이 0 일 때만 표band 추가) 아니면 진짜 별개인지 측정만으로 확정 불가. ColumnDef 간격=0 이면 표band 추가, >0 이면 ColumnDef 간격 — 이라는 규칙은 가능하나 검증 표본이 shortcut.hwp 한정이라 일반 룰로 확정 못 함(`feedback_rule_not_heuristic`).
- 또 본문 zone offset 은 `process_multicolumn_break`(`vpos_zone_height`) + `layout_table_item`(표 band advance) 두 곳 협조 변경 필요 → composer/typeset/layout 광역 회귀 위험(`feedback_essential_fix_regression_risk`). 닫힌 PR #771/#770/#773/#776/RFC #774 가 이 ~28px 를 못 닫은 이유.

## 권고 — 보류

기여 요소(≈ 표 band 높이)와 강력한 가설은 확보했으나, 적용 게이팅(1쪽 vs 2·3쪽의 메커니즘 차이가 단일 룰인지)을 확정할 추가 표본/검증 수단(다른 문서의 동일 패턴, 한컴 편집기 직접 확인)이 없다. **#866 은 본 보고서까지로 마감하고, 검증 수단이 확보되는 시점에 가설(표 band 추가 예약, ColumnDef 간격=0 게이팅)을 검증·구현** 권고. 가설·측정은 본 보고서 + `mydocs/tech/hancom_zone_paragraph_spacing.md` §5 에 보존.

만약 강행한다면 Stage 3 = `process_multicolumn_break` 에서 "직전 zone 마지막 paragraph 가 `wrap=위아래` TAC 표 보유 & ColumnDef 간격=0 이면 `vpos_zone_height` 에 표 band 높이(`table.common.height + outer_margin_top + outer_margin_bottom` px) 추가" + `layout_table_item` 의 advance 정합 → `cargo test --release` + 전 fixture sweep + shortcut.hwp 7~8쪽 SVG↔`pdf/`(2022)·`pdf-2020/`. 게이팅이 표본 1개 기반이라 비권장.

## 산출물
- 본 보고서. 소스 변경 없음.
