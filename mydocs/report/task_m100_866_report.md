# 최종 결과 보고서 — Task #866 (M100)

대상: shortcut.hwp 의 `쪽나누기`로 시작하는 페이지(2쪽 "파일" 등)에서 헤더 TAC 표 띠↔본문 사이 ~28px gap 미재현. (Task #853 의 잔존 #866; #847 의 잔여; 닫힌 PR #771/#770/#773/#776, RFC #774 가 못 닫은 본질.)
GitHub Issue: edwardkim/rhwp#866 · 브랜치: `local/task866` (← `pr-task853` = upstream/devel + PR #868)

## 결과 요약

| Stage | 내용 | 결과 |
|---|---|---|
| 1 | PDF 측정 (코드 미수정) | 헤더 띠 하단↔본문 gap = 2쪽 +31.7px, 3쪽 +33.6px (일정). ≈ TAC 표 band 의 line height(~31px). |
| 2 | 가설 정립 | 한컴은 `wrap=위아래` 글자처럼-취급 표(헤더 띠) 아래에 표 band 높이만큼 추가 여백. ColumnDef 간격>0 인 헤더 띠(1쪽)는 그 간격이 이미 여백이라 제외. |
| 3 | 구현 | ① `typeset.rs::process_multicolumn_break`: 직전 zone 마지막 paragraph 가 `wrap=위아래` 글자처럼-취급 표 & 1단 ColumnDef 간격=0 → `vpos_zone_height` 에 표 band 높이 추가. zone 가드 `3→4*one_line`. ② `layout.rs::build_columns`: zone-간 세로 여백(ColumnDef 간격/2 + 표 band)을 layout 의 `prev_zone_y_end` 누적에도 미러 — 종전엔 pagination 메타데이터(`current_zone_y_offset`)에만 반영되어 SVG 렌더에 미적용이었음. |

### 핵심 발견
PR #868(Task #853) Stage 3-3(1단 ColumnDef `간격` → zone 진입 세로 간격)은 pagination 의 `current_zone_y_offset`(메타데이터)만 갱신했고 layout 의 zone 스태킹(`build_columns` 의 `prev_zone_y_end` 누적)에는 미반영이라 SVG 렌더에 적용되지 않았었다(= PR #868 의 "1쪽 정합" 표기가 실제론 메타데이터만). Task #866 Stage 3 의 layout 미러로 비로소 1쪽도 시각 정합됨.

## 시각 정합 (shortcut.hwp ↔ `pdf/basic/shortcut-2022.pdf`, SVG baseline ↔ PDF baseline 추정)

| | 변경 전 | 변경 후 | 한컴 PDF | 평가 |
|---|---|---|---|---|
| 1쪽 본문 첫 줄 baseline (into body) | ~+111px | **+149.3px** | ~+148px | ✓ 정합 |
| 2쪽 본문 첫 줄 baseline | ~+58px | **+89.5px** | ~+85px | ~4.5px 초과 (종전 ~28px 부족) |
| 3쪽 본문 첫 줄 baseline | ~+71px | **+102.3px** | ~+93px | ~9px 초과 (종전 ~24px 부족) |

→ 사용자 주 증상("모든 구분 칸 위·아래 줄 간격 좁음", 특히 헤더 띠↔본문 ~28px 부족)이 **전 페이지에서 해소**(잔여 ~4~9px 초과).

## 검증

- `cargo test --release` 34 test suites 전건 통과. svg_snapshot 8/8 — golden 무변경.
- shortcut.hwp 8쪽 SVG ↔ `pdf/basic/shortcut-2022.pdf` 픽셀 측정으로 1·2·3쪽 본문 위치 확인.

## 잔존 (미수정)

1. `LAYOUT_OVERFLOW` 4건(4쪽 2단 본문 zone, pi=143~147) — 헤더 띠가 ~31px 커진 만큼 후속 zone 들이 밀려 본문 하단 초과. `process_multicolumn_break` 가드가 새 zone 의 *시작* 여유만 보므로 시작은 들어가나 zone 콘텐츠가 넘치는 케이스는 못 잡음. 가드를 콘텐츠 높이 추정 기반으로 보강 필요 → **#867** 영역(페이지 수 7≠8 포함). (원래 25 → PR #868 0 → Task #866 4.)
2. 2·3쪽 ~4~9px 초과 — tac_band(`table.height + om_top + om_bot`)가 한컴 실측 gap(~28~33px)보다 미세하게 큼. 정밀화는 한컴 편집기 cross-check 필요(macOS 환경 한계).

## 커밋 (브랜치 `local/task866`, PR #868 위에 stacked)
PR #868(`91e585e3`) → 수행 계획서(`79aa64f7`) → 구현 계획서(`a72daadb`) → Stage 1 측정(`e5d579c6`) → Stage 2 분석(`adbfcfa0`) → Stage 3 구현(`8c87cbf7`) → 본 보고서.
