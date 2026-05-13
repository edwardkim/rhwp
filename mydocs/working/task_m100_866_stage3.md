# Stage 3 완료 보고서 — Task #866 (M100) — 헤더 띠↔본문 gap 정정 + zone-간격 layout 미러

GitHub Issue: edwardkim/rhwp#866 · 브랜치: `local/task866` (← `pr-task853`)

## 변경

### `src/renderer/typeset.rs::process_multicolumn_break`
- `tac_band_extra`: 직전 zone 의 마지막 paragraph 가 `wrap=위아래`(TopAndBottom)인 글자처럼-취급 표(헤더 띠)를 보유하고 그 zone 의 1단 ColumnDef 간격이 0 이면, `vpos_zone_height` 에 표 band 높이(`table.common.height + outer_margin_top + outer_margin_bottom` px)를 추가 → 다음 zone 진입 offset 을 그만큼 내림. (한컴 PDF 측정: 헤더 띠 하단↔본문 ~28~33px = 표 band 높이 ≈ 31px. ColumnDef 간격>0 인 헤더 띠(1쪽 등)는 그 간격이 이미 zone 여백이 되므로 제외.)
- 다단 zone 시작 여유 가드 임계값: `available - 3*one_line` → `available - 4*one_line`(표 band 추가로 zone 이 커진 만큼).

### `src/renderer/layout.rs::build_columns`
- **핵심**: 이전엔 zone-간 세로 여백(PR #868 Stage 3-3 의 ColumnDef 간격, 본 Stage 3 의 tac_band)이 pagination 의 `current_zone_y_offset`(메타데이터)에만 반영되고 layout 의 zone 스태킹(`prev_zone_y_end` 누적)에는 안 반영되어 SVG 렌더에 미적용이었다. → layout 에도 미러:
  - zone 전환 시 `current_zone_start_y += (이전 zone 디자인 spacing /2) + (새 zone 디자인 spacing /2)`. 디자인 spacing = 1단 ColumnDef 의 `간격`(다단은 0). pagination 의 process_multicolumn_break 와 동일 시멘틱.
  - 헤더 띠 zone(마지막 item 이 `wrap=위아래` 글자처럼-취급 표 & 1단 ColumnDef 간격=0) 처리 후 `prev_zone_y_end += 표 band 높이`.

## 결과 (shortcut.hwp ↔ `pdf/basic/shortcut-2022.pdf`, SVG baseline ↔ PDF baseline 추정)

| | 변경 전(PR #868) | 변경 후 | 한컴 PDF | 평가 |
|---|---|---|---|---|
| 1쪽 본문 첫 줄 baseline (into body) | ~+111px (메타데이터 +138 이지만 렌더 미반영) | **+149.3px** | ~+148px | ✓ 정합 |
| 2쪽 본문 첫 줄 baseline | ~+58px | **+89.5px** | ~+85px | ~4.5px 초과 (대폭 개선, 종전 ~28px 부족) |
| 3쪽 본문 첫 줄 baseline | ~+71px | **+102.3px** | ~+93px | ~9px 초과 (개선, 종전 ~24px 부족) |
| `LAYOUT_OVERFLOW` | 0 (PR #868 = Stage 3-3c) / 원래 25 | **4** ⚠ | — | 4쪽 2단 본문 zone — #867 영역 |
| `cargo test --release` | 34/34 ✓ | **34/34 ✓** (svg_snapshot 8/8 golden 무변경) | — | 회귀 0 |

→ 사용자 주 증상("모든 구분 칸 위·아래 줄 간격 좁음", 특히 헤더 띠↔본문 ~28px 부족)이 **전 페이지에서 해소**(잔여 ~4~9px 초과). PR #868 의 "1쪽 정합" 표기는 실제론 메타데이터만이었고 본 Stage 3 의 layout 미러로 비로소 시각 정합됨.

## 잔존

- `LAYOUT_OVERFLOW` 4건(4쪽 2단 본문 zone, pi=143~147) — 헤더 띠가 ~31px 커진 만큼 후속 zone 들이 밀려 하단 초과. `process_multicolumn_break` 가드는 새 zone 의 *시작* 여유만 보므로, 시작은 들어가나 zone 콘텐츠가 넘치는 케이스는 못 잡음. 가드를 콘텐츠 높이 추정 기반으로 보강 필요 → **#867** 영역(페이지 수 7≠8 포함).
- 2·3쪽 ~4~9px 초과 — tac_band(`table.height + om_top + om_bot`)가 한컴 실측 gap(~28~33px)보다 미세하게 큼. 정밀화하려면 한컴 편집기 cross-check 필요(macOS 환경 한계).

## 커밋
PR #868(Task #853) → `local/task866`: 수행/구현 계획서 → Stage 1 측정 → Stage 2 분석 → 본 Stage 3 구현.
