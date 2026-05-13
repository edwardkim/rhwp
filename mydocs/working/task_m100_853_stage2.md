# Stage 2 진행 보고서 — Task #853 (M100) — 부분 결과 + 판단 요청

GitHub Issue: edwardkim/rhwp#853 · 브랜치: `local/task853` · 상태: **소스 변경 1건 적용(미커밋), 판단 요청**

## 적용한 변경 (미커밋)

`src/renderer/layout/paragraph_layout.rs::layout_composed_paragraph` (현행 745-748 부근):
- 현행: column-top 문단(`is_column_top`)은 `spacing_before` 를 통째 드롭(`!is_column_top` 일 때만 적용).
- 변경: column-top 이면서 **섹션의 첫 문단(`para_index == 0`)** 인 경우, `spacing_before` 를 그 문단 첫 LINE_SEG 의 `vertical_pos` 로 상한 클램프해 적용 (`y += spacing_before.min(hwpunit_to_px(line_segs[0].vertical_pos).max(0.0))`). 페이지 break 후 이어진 column-top(`para_index > 0`)은 종전대로 0.

## 결과

### ✅ 효과 — shortcut.hwp 제목 위치 정합
- 제목 "글 2010 단축키 일람표" baseline y=79.4 → **105.8** (+26.4px). top ≈ 83.8px ≈ 한컴 PDF top 83.6px (`pdf/basic/shortcut-2022.pdf`, stage5 측정). 한컴이 적용하는 `LINE_SEG.vertical_pos = 1984 HU (26.45px)` 와 일치.
- `height_measurer` 는 이미 제목 높이에 26.5px(=vpos)를 포함하고 있었으나(`dump-pages` 단0 `sb=26.5`), `paragraph_layout` 만 0 으로 드롭해 비대칭이었음 → 이번 변경으로 페이지네이션↔배치 정합.

### ⚠ 문제 1 — shortcut.hwp 페이지 수 7 → 8 (한컴 PDF = 7)
- 제목 +26.4px 만큼 1쪽 콘텐츠가 아래로 밀리며 1쪽 끝의 `다단나누기` band 하나가 2쪽으로 → 연쇄 → 8쪽.
- 한컴은 제목 정합 상태로도 7쪽에 담으므로, rhwp 2~8쪽이 한컴보다 콘텐츠가 짧지 않다는 뜻 — 즉 **band-transition spacing deficit(제목↔헤더 ~15px, 헤더↔본문 ~20px; stage5)이 미해결**이라 한쪽으로만 늘어남. 더불어 기존 3쪽 overflow(콘텐츠 y=766 > body_bottom 758.4; pre-change 에도 존재한 버그)가 page break 로 풀리며 +1쪽에 기여.

### ⚠ 문제 2 — svg_snapshot 2건 시프트 (회귀/개선 미확정)
- `cargo test --test svg_snapshot`: `issue_267_ktx_toc_page` (목차 제목 y 129.0 → 132.8, +3.8px), `issue_617_exam_kor_page5` (셀 "6" y 179.1 → 186.7, +7.6px; "홀수형" y 169.9 → 174.8, +4.9px) 두 건 FAILED. 나머지 6건 통과.
- 두 문서 모두 섹션-시작 문단(`para_index==0`)에 `spacing_before > 0` 이 있어 변경 영향. 해당 문서 한컴 PDF 대조 없이는 개선(정합)인지 회귀(한컴은 페이지-top 섹션-시작에서 드롭)인지 판정 불가.

### ⚠ 문제 3 — 사용자 주 증상 미해소
- 사용자 보고 "모든 구분 칸 위·아래 줄 간격 좁음" 의 핵심은 `다단나누기` band(섹션 헤더 띠 = 1×1 TAC 표) 주변 간격인데, 이들 band-top 문단은 `spacing_before == 0` 이라 본 변경으로 바뀌지 않음. band-transition spacing 은 별도 근원(TAC 표 띠 line-height / `다단나누기` zone 전환 암묵 간격) — RFC #774 영역.

## 판단 요청 (작업지시자)

| 옵션 | 내용 | 평가 |
|------|------|------|
| A | 제목 정정 유지 + 2 snapshot 한컴 PDF 검증 → 개선이면 `UPDATE_GOLDEN`, 회귀면 조건 추가 narrow + 페이지수 7→8 수용 | 제목만 정합. 페이지수 한컴 불일치 잔존. 주 증상 미해소. |
| B (권고) | 본 변경 revert → #853 을 진단·RFC #774 선행 타스크로 전환. band-transition spacing(TAC 표 띠 + `다단나누기` zone 전환) 본질 분석 후 제목·band·페이지수·overflow 일괄 정정 | `feedback_essential_fix_regression_risk` 정합. 부분 정정의 페이지수 회귀·snapshot 불확정·주 증상 미해소 회피. |
| C | 계속 진행 — band-transition spacing 정정 (RFC #774 흡수) | 광역 회귀 위험 큼. 한컴 2010/2020 정답지 + 전 fixture sweep 필수. 장기 작업. |

권고: **B**. 본 변경은 1쪽 제목 1건만 정합되고 페이지 수가 한컴(7쪽)과 어긋나며(8쪽) snapshot 2건이 불확정 상태가 되고 정작 사용자 주 증상(band 간격)은 그대로다. RFC #774(한컴 paragraph/zone spacing 알고리즘) 선행 분석 후 일괄 정정이 안전.

## 첨부
- 변경 후 SVG: `output/svg/sc853b/` (8개)
- `cargo test --test svg_snapshot`: 6 passed / 2 failed (issue_267, issue_617). 나머지 `cargo test --release`: 본 변경 전 전건 통과 확인됨(svg_snapshot 외).
