# 구현계획서 — Task #846 (M100)

대상: shortcut.hwp 페이지 3 끝 `<편집 화면 분할에서>`(좌) | `화면 이동 ⟶ Ctrl+W,N`(우) 가 페이지 4 로 밀리는 결함 (≈ 닫힌 #768, #844 결함 B 분리). Issue edwardkim/rhwp#846. 브랜치 `local/task846` (← upstream/devel).

핵심: SVG 렌더 경로는 `src/renderer/typeset.rs::TypesetEngine` 을 탄다 (`pagination/engine.rs` 아님). 마지막 단에서 명시적 `ColumnBreakType::Column` 을 만나면 — 현재는 무조건 `push_new_page()` — 한컴처럼 **같은 페이지에 새 단-밴드(column band)** 를 시작하도록 확장한다.

---

## Stage 1 — 진단 및 column-band 확장 설계 (소스 수정 없음)

목표: "마지막 단 + 명시적 단 나누기 → 같은 페이지 새 밴드" 를 `typeset.rs` 모델로 어떻게 표현할지 + `layout.rs` 좌표 연동점을 확정.

작업:
- `rhwp dump-pages -p 2`(페이지 3) / `-p 3`(페이지 4) — pi=82~96 의 zone/단 배치, vpos, used_height, column-break 컨트롤 위치 추적.
- `rhwp dump -s {sec} -p 82~96` — 해당 문단의 `ColumnBreakType`, LINE_SEG vpos, ParaShape, 단 정의(`column_count`) 확인.
- `typeset.rs` 정독: `advance_column_or_new_page`, `flush_column`, `ColumnContent`(`column_index`/`zone_y_offset`/`used_height`), `current_zone_y_offset` 진행 시점, `paginate` 본문의 `ColumnBreakType::Column` 처리 분기.
- `renderer/layout.rs` 의 `ColumnContent` → 단 영역 x/y 좌표 산출 로직: 새 밴드의 단들이 `zone_y_offset` + (이전 밴드 최대 단 높이) 를 기준으로 배치되는지, body-wide reserve / 머리말꼬리말 / 각주 영역과 충돌 없는지 확인.
- 새 밴드가 본문 영역 하단을 넘으면? → 그 시점에 새 페이지로. 이 판정을 어디서 할지 확정.
- 닫힌 #768 흔적이 `mydocs/` 에 남아있는지 재확인.
- **단일 룰 판정**: "마지막 단 + `ColumnBreakType::Column` → 새 밴드" 가 분기·허용오차 없이 표현 가능한지. 안 되면 자문 (메모리 `feedback_rule_not_heuristic`).
- 회귀 대상 샘플 목록 확정 (다단: exam_* 류, k-water-rfp, 다단+표분할, 다단+목차 등).

산출물: `mydocs/working/task_m100_846_stage1.md`. → 승인 요청.

---

## Stage 2 — 구현: 마지막 단 column-break → 같은 페이지 새 밴드

목표: pi=94 의 단 나누기에서 페이지 3 에 새 2단 밴드(좌=`<편집...>`, 우=`화면 이동...`) 가 생기고 이후 흐름이 PDF 와 정합.

작업:
- `paginate` 의 명시적 단나누기 경로: `ColumnBreakType::Column` + `!has_diff_col_def` + 마지막 단(`current_column+1 >= col_count`) + `col_count > 1` → `advance_column_or_new_page` 대신 신규 `start_new_column_band`.
- `start_new_column_band`: (a) `flush_column`, (b) 다음 밴드 콘텐츠에 떠다니는(글자처럼 취급 아닌) 표·그림·그리기 개체가 있으면 새 페이지로, (c) 현 밴드 높이 = 그 밴드 각 단의 마지막 문단 `vpos_end` 중 최댓값, (d) `available_height() - 밴드높이 >= 이_문단_첫줄_높이` 이면 새 밴드(zone_y_offset 진행, col 0 리셋, col_count 유지), 아니면 새 페이지.
- `process_multicolumn_break` 는 변경하지 않음 (`[다단나누기]` 동작 보존).
- 디버그: `export-svg --debug-overlay` 로 페이지 3 의 두 밴드 경계 확인.

검증: shortcut.hwp 페이지 3 끝에 `<편집 화면 분할에서>` | `화면 이동 ⟶ Ctrl+W,N` 정상 표시, 페이지 4 첫 줄로 안 밀림. 이후 페이지 수/흐름 PDF 정합. `cargo test` 전건. 1차 회귀: exam_* 다단 샘플.

산출물: `mydocs/working/task_m100_846_stage2.md`.

---

## Stage 3 — 광역 회귀 검증

목표: 다단/단일 단/표분할 전 영역에 회귀 없음 확정 (메모리 `feedback_essential_fix_regression_risk` 정통 적용 대상).

작업:
- 다단 샘플 전수: exam_* 류, k-water-rfp, 다단+TopAndBottom 표, 다단+목차, 다단+각주 — Stage 2 전/후 SVG diff (`rhwp export-svg`), 변화 시 의도 여부 판정.
- 단일 단(col_count==1) 샘플 일부 — 본 변경이 `col_count==1` 경로를 안 건드리는지 확인.
- shortcut.hwp 1~8페이지 SVG ↔ `pdf/basic/shortcut-2022.pdf` (한글 2022, macOS 1차 기준) 시각 비교. 가능 시 한컴 2010/2020 동작과 대조 (PDF 는 절대 기준 아님 — 메모리 `feedback_pdf_not_authoritative`).
- `cargo test` 전건 + 신규 clippy 경고 0.

산출물: `mydocs/working/task_m100_846_stage3.md`.

---

## Stage 4 — 종합 검증 및 최종 보고서

작업:
- `cargo test` 전건 최종 통과, 신규 clippy 경고 0.
- shortcut.hwp 1~8페이지 SVG ↔ PDF 최종 정합 확인, before/after 캡처.
- 회귀 대상 샘플 최종 점검.
- 최종 보고서 `mydocs/report/task_m100_846_report.md` — 닫힌 #768 과의 관계, 잔존 한계(있으면) 명기.
- merge 전 `git status` 미커밋 확인. `local/task846` → PR (`planet6897:pr-task846` → `edwardkim/rhwp:devel`, 메모리 `feedback_pr_target_devel` / `feedback_per_task_pr_branch`).

---

## 커밋 규약
- 각 Stage 소스 + `working/task_m100_846_stage{N}.md` 함께 커밋, 메시지 `Task #846: ...`.
- 본 계획서(수행 + 구현) 1커밋. 최종 보고서 커밋 후 PR.
- `mydocs/orders/` 변경 금지 (메모리 `feedback_no_orders_modification`).
