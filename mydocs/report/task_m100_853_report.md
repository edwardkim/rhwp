# 최종 결과 보고서 v2 — Task #853 (M100)

대상: `samples/basic/shortcut.hwp` ↔ 한컴 PDF `pdf/basic/shortcut-2022.pdf` — (증상1) 모든 구분 칸 위·아래 줄 간격 압축, (증상2) 일부 페이지 본문영역 초과 렌더링.
GitHub Issue: edwardkim/rhwp#853 · 브랜치: `local/task853` (← upstream/devel `2bd50a3a`)

> 본 보고서는 v2. v1(`13cd40bb` 시점)은 옵션 B(소스 변경 0, RFC 선행 전환)였으나, 작업지시자 지시로 범위를 "제목 + band 간격 + overflow 전부"로 확대해 Stage 2/3 를 정정 구현했다.

## 결과 요약

| # | 결함 | 결과 |
|---|------|------|
| Stage 2 | 섹션-top 제목 spacing_before 압축 | ✅ 수정 — `paragraph_layout.rs` 에서 `is_column_top && para_index==0` 시 `spacing_before` 를 LINE_SEG.vertical_pos 로 클램프. 제목 baseline 79.4→105.8 (top≈83.8px ≈ 한컴 PDF 83.6px). |
| Stage 3-2 | 헤더 띠(TAC 표) line0 텍스트 흡수 | ✅ 수정 — `typeset.rs::place_table_with_text` 에서 전폭 TAC 표가 자기 줄에 놓인 경우 표 줄 높이와 일치하는 LINE_SEG 인덱스로 표 앞 텍스트 줄(line0)을 표보다 먼저 배치. PUA 필러 케이스(복학원서.hwp pi=16)는 `is_alphanumeric()` 로 제외. 2쪽 "파일" 헤더 띠 상단 +3.8→+19.8px (한컴 PDF +19.1px ✓), 하단 +27.3→+43.3px (PDF +43.1px ✓). |
| Stage 3-3 | 1단 ColumnDef `간격` 의 zone 진입 세로 간격 미반영 | ✅ 수정 — `typeset.rs` zone 전환(`process_multicolumn_break` / `force_new_page`+diff_col_def)에서 (이전 zone 디자인 spacing /2)+(새 zone 디자인 spacing /2) 를 `zone_y_offset` 에 더함. '디자인 spacing'=1단 ColumnDef 의 `간격`(가로 무의미), 다단은 0. 1쪽 헤더 띠 zone_y_offset 69.1→88.0 (한컴 PDF +87.6px ✓), 본문 zone 100.2→138.0 (PDF +137.9px ✓). |
| 잔존 a | 2쪽(`쪽나누기`로 시작) 헤더 띠↔본문 ~28px gap | ⏸ 미수정 — pi=36 ColumnDef `간격=0mm` 이라 Stage 3-3 가설로 0px, 출처 미규명. 후보(분석 문서 §5): TAC `wrap=위아래` 추가 예약 / `쪽나눔=RowBreak(0x04000006)` / 1단→2단 전환 고정 간격. 한컴 편집기(Windows) cross-check + 추가 샘플 측정 필요. |
| Stage 3-3b/c | 다단 zone 누적 시 잔여 콘텐츠가 본문영역 초과 | ✅ 보강 — ① `process_multicolumn_break`: 새 zone 시작 여유 < 헤더 띠 1개 높이(~56px)면 `push_new_page`. ② vpos-reset 검출(Distribute 다단): 직전 문단 `vpos+line_height` 기준으로 비교 → 1줄짜리 컬럼(prev vpos=0, curr vpos=0)도 컬럼 전환 인식(shortcut.hwp 스타일/속성 섹션). LAYOUT_OVERFLOW 25→13→2. |
| 잔존 b | shortcut.hwp 페이지 수 7≠8 + 잔존 LAYOUT_OVERFLOW 2건 | ⏸ 미수정 — 잔존 2건(pi=149/150, 4쪽 col 0/1, 6.7px)은 zone 이 본문 하단 직전에서 시작하는 경계 케이스. 페이지 수 7≠8 은 §a(2쪽 ~28px gap) 미해결로 zone 누적이 한컴과 어긋난 결과. |

`cargo test --release` 전건 통과(34 test suites). svg_snapshot 8/8 — golden 2건(`issue-267/ktx-toc-page`, `issue-617/exam-kor-page5`) 갱신(Stage 2 — 섹션-시작 문단이 LINE_SEG.vertical_pos 기준으로 재배치, 한컴 기록값 정합). 다른 문서 회귀 없음.

## 변경 내역

### Stage 2 — `src/renderer/layout/paragraph_layout.rs::layout_composed_paragraph`
- `is_column_top` 시 `spacing_before` 를 통째 드롭하던 것을, `para_index == 0`(섹션 첫 문단)인 경우 `y += spacing_before.min(hwpunit_to_px(line_segs[0].vertical_pos))` 로 변경. 페이지 break 후 column-top(`para_index>0`)은 종전대로 0.

### Stage 3-2 — `src/renderer/typeset.rs::place_table_with_text`
- `pre_table_end_line` 계산에 분기: `table.common.treat_as_char && total_lines > 1 && para.text.chars().any(is_alphanumeric)` 이면, 표 줄 높이(표 본체 + outer_margin top/bottom)와 일치하는 LINE_SEG 인덱스를 사용 → 그 앞 줄을 `PageItem::PartialParagraph{0..pre}` 로 표보다 먼저 emit. `tac_wrap_split` 플래그로 높이 이중계산 방지(`table_total_height` 만 누적) + post-text 시작을 `pre+1` 로.

### Stage 3-3 — `src/renderer/typeset.rs` (`TypesetState`, `process_multicolumn_break`, `paginate_section`, `force_new_page` 경로)
- `current_zone_design_spacing_px` 필드 추가, `column_def_design_spacing_px(cd, dpi)` 헬퍼(1단이면 `간격`, 다단이면 0).
- zone 전환 시 `zone_y_offset += (이전 zone 디자인 spacing /2) + (새 zone 디자인 spacing /2)`. 새 페이지 첫 zone 은 새 zone /2 만(이전 zone 은 이전 페이지).

## 미수정 — 잔존 항목 (신규 후속 이슈로 등록)

분석 문서 `mydocs/tech/hancom_zone_paragraph_spacing.md` (RFC #774 후속) 에 정리:
1. **#866** — 쪽나누기로 시작하는 페이지의 헤더 TAC 띠↔본문 ~28px gap 미재현. pi=36 ColumnDef `간격=0mm` 이라 Stage 3-3 가설로 미설명. 닫힌 PR #771/Issue #770/RFC #774 도 못 닫은 본질(line0 흡수 ~16px 만 다룸 — Stage 3-2 가 동일 효과). 한컴 정답지 cross-check / 다른 헤더 띠 PDF 측정 필요.
2. **#867** — 페이지 수 7≠8 + 잔존 LAYOUT_OVERFLOW 2건(pi=149/150, 4쪽, 6.7px). #866 에 종속(zone 누적이 한컴과 어긋난 결과). 별개로 vpos=0 연속 문단(#768 패턴)이 비-배분 다단에 잔존 가능.
3. 부수: 제목 PUA 첫 글자 `\u{f53a}` — SVG 에 출력되며 폰트 폴백 영역(본 타스크 무관).

본 타스크는 위 잔존을 #866/#867 로 분리하고 종료. Issue #853 close.

## 검증

- `cargo build --release` 성공. `cargo test --release` 34 suites ok / 0 failed. svg_snapshot 8/8.
- shortcut.hwp 7~8쪽 SVG ↔ `pdf/basic/shortcut-2022.pdf` 시각 비교(`mutool draw -r 100` 픽셀 측정): 1쪽 제목·헤더 띠·본문 첫 줄, 2쪽 헤더 띠 상/하단 모두 한컴 PDF 와 ±1px 수렴. 2쪽 본문은 ~28px 잔존.

## 커밋
`38042d05`(계획서) → `6faf8250`(Stage 1) → `b65c1624`(impl v2) → `13cd40bb`(v1 보고/일시 revert) → `f0d34713`(Stage 2 재적용) → `bd9d5148`/`0fca34ed`(Stage 3-1 조사·분석) → `c765e36b`(3-2 1차 실패) → `1f7328b2`(3-2 성공) → `4ae34c55`(3-3 조사) → `d011d43d`(3-3 구현) → 본 보고서. → `b74967bf`(Stage 3-3 page break 보강).
