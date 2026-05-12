# 최종 결과 보고서 — Task #842 (M100)

대상: shortcut.hwp(`samples/basic/shortcut.hwp`) ↔ 한컴 PDF(`pdf/basic/shortcut-2022.pdf`) 시각 정합성 잔여 결함 4건.
GitHub Issue: edwardkim/rhwp#842 · 브랜치: `local/task842` (← upstream/devel)

## 결과 요약

| # | 결함 | 결과 |
|---|------|------|
| 4 | 단축키 우측탭 정렬 일부가 단 우측 끝 초과 | ✅ 수정 (Stage 2 + 2b) |
| 3 | 두 단 가운데 구분선이 실선 (점선이어야 함) | ✅ 수정 (Stage 3) |
| 2 | 페이지 2~8 섹션 헤더 바 +28px 우측 편위 | ✅ 수정 (Stage 4) |
| 1 | 헤더 1×1 TAC 표 앞뒤 수직 spacing 압축 | ⏸ 미수정 — RFC #774 영역, 후속 이슈로 분리 (Stage 5 조사 완료) |

`cargo test` 전건 통과 (svg_snapshot 8/8 포함). 회귀 없음. (`cargo clippy` 는 본 타스크 무관한 pre-existing `error: unwrap() will always panic` — `table_ops.rs:1007`, `object_ops.rs:304` — 으로 컴파일 실패하나 본 변경과 무관.)

## 변경 내역

### 결함 #4 — `src/renderer/layout/paragraph_layout.rs`, `src/renderer/layout/text_measurement.rs`
- `right_tab_block_width()` 헬퍼: cross-run 우측·가운데 탭 정렬 시, 탭 직후 run 부터 `\t` 없는 연속 composed run 들의 폭을 합산해 정렬 시작 x 산출. composer 가 스크립트·char-shape 경계로 run 을 쪼개는 케이스(`"Ctrl+(회색)5"` → `["Ctrl+(", "회색)", "5"]`)에서 나머지 run 이 탭스톱 우측으로 흘러넘치던 ~32px 오버플로 해소.
- `compute_char_positions` 의 in-run RIGHT 인라인 탭 분기 `(2, _) if fill_low != 0` → `(2, _)` 로 확장: RIGHT 인라인 탭은 leader 유무 무관하게 `body_right - our_seg_w` 로 정렬(한컴 `ext[0]` 무시). char-shape 경계가 `\t` 앞에 놓여 run 이 `\t` 로 시작하는 케이스(`"끝"`(id7) + `"\tAlt+X"`(id8))의 ~28px 오버플로 해소.
- 결과: shortcut.hwp 8페이지 우측정렬 단축키 항목 전부 정렬 폭 ±6px 수렴.

### 결함 #3 — `src/renderer/layout.rs::build_column_separators`
- `separator_type → StrokeDash` 매핑에 `6 => Dash`(LongDash 근사), `7 => Dot`(Circle/원형 점선) 추가. `doc_info.rs:294` line_type 의미와 정합. shortcut.hwp 2단 ColumnDef 의 `구분선 type=7` 이 실선 → 점선(`stroke-dasharray="2 2"`)으로 렌더.

### 결함 #2 — `src/renderer/layout.rs::layout_table_item` (`is_tac` 분기)
- 다줄 문단(`composed.lines.len() > 1`)이고 line 0 에 `char::is_alphanumeric()` 글자(한글 음절/라틴/숫자/한자)가 있으면 → 표는 line 0 텍스트 *다음* 이 아니라 자체 줄 좌측에서 시작하므로 `leading = 0` (line 0 폭 미합산).
- line 0 이 HWP TAC 필러(`U+F081C`·`U+F012B` 등 PUA)/공백뿐인 경우(예 복학원서.hwp pi=16 — 한컴이 표 폭만큼 필러를 채워 줄바꿈시킨 케이스)는 종전대로 `compute_tac_leading_width`. `is_alphanumeric()` 판정으로 PUA 필러 자동 제외.
- 결과: shortcut.hwp 헤더 바 페이지 1~8 전부 rect x = body 좌측(94.5), `issue_677_bokhakwonseo_page1` snapshot 유지.

## 미수정 — 결함 #1 (후속 이슈로 분리)
헤더 1×1 TAC 표 앞뒤 수직 여백(과 제목 위 여백)이 PDF 대비 ~15~25px 부족. 본문 행 pitch 자체는 정상. 명시 spacing(`spacing before/after`)에 해당 여백이 없어, 한컴이 zone 전환(1단↔2단)/TAC 표 문단 line-height 기반으로 넣는 암묵 간격으로 추정 — 닫힌 이슈 #770/#773/#776 + RFC #774 의 주제. 본질 정정 위험군이라 RFC 분석 + 광역 회귀 검증과 함께 별도 처리. 정밀 PDF 비교 데이터는 `mydocs/working/task_m100_842_stage5.md`.

부수 발견(별개): (1) 제목 첫 글자 "흔" 누락("흔글 2010" → "글 2010"), (2) 페이지 3→4 column-break 행 밀림(`<편집 화면 분할에서>` "화면 이동" — 닫힌 #768 과 동일). 둘 다 후속 이슈 대상.

## 커밋
`f1665bff`(Stage 1) → `aac23bc7`(Stage 2) → `63a41829`(Stage 2b) → `6f0a0784`(Stage 3) → `2663eb32`/`5f2d85ab`(Stage 4 조사) → `bc2e8e54`(Stage 4 수정) → `3ed8da48`(Stage 5 조사) → 본 보고서.
