# 구현계획서 — Task #849 (M100)

대상: 다단 밴드/단 실제 채움 높이 산출 정합 (인라인·떠다니는 큰 박스가 vpos 에 안 잡혀 밴드 높이 과소추정 → 다단 페이지 과압축). Issue edwardkim/rhwp#849. 브랜치 `local/task849` (← `local/task846`).

핵심: `typeset.rs` 가 다단 밴드를 한 페이지에 적층할 때 쓰는 밴드 높이 산출(`process_multicolumn_break`, `start_new_column_band`)을 "그 밴드 각 단의 실제 점유 높이의 최댓값" 으로 정합하고, 두 경로가 그 산출을 공유하게 한다.

---

## Stage 1 — 진단 (소스 수정 없음)

목표: 단별 "실제 채움 높이" 가 typeset 단계 어디에/어떻게 산출되는지 정리하고, `exam_math.hwp` 에서 산출(≈710px)과 실제(≈1100px) 차이가 정확히 어디서 새는지 규명. 수정 범위 확정.

작업:
- `typeset.rs` 정독: `current_height` 누적 경로(`height_for_fit` vs `total_height`), `flush_column` 의 `used_height` 기록, `process_multicolumn_break` 의 `vpos_zone_height`(직전 문단 한 개의 `vpos_end`), `start_new_column_band` 의 밴드 높이(밴드 각 단의 마지막 문단 `vpos_end` 중 max), `pending_body_wide_top_reserve` / body-wide TopAndBottom 표 처리.
- `rhwp dump-pages -p 2` (exam_math 페이지 3) + `rhwp dump -s 0 -p 69~96` — 단 0 의 각 문단 LINE_SEG vpos 추이, 떠다니는 표(`Table ... wrap=TopAndBottom tac=false vpos=...`)·인라인 도형의 vpos·박스 높이 확인. 산출 710px 가 어느 문단의 어느 vpos 에서 나오고, 실제 ~1100px 와의 ~390px 차이가 (a) 떠다니는 TopAndBottom 표 영역이 LINE_SEG vpos 에 미반영, (b) 인라인 도형 박스가 line_height 에 미반영, (c) 단의 마지막 항목 선정 오류 중 무엇인지 추적.
- `renderer/layout.rs` 의 ColumnContent 좌표 산출 — 밴드 높이 정합 후 단 영역 y 가 정확히 따라가는지(연동 수정 필요 여부) 확인.
- 수정 위치 확정: 밴드 높이 산출 공유 헬퍼를 `TypesetEngine` 메서드(또는 자유 함수)로 둘지, 입력으로 무엇이 필요한지(현 페이지의 `column_contents` 슬라이스 + `paragraphs` + dpi). LINE_SEG 가 이미 박스 높이를 반영하면 헬퍼는 "단의 마지막 항목까지 정확히 집계" 만 하면 됨; 미반영이면 떠다니는 표/도형 박스 높이를 별도 합산하는 로직 필요 — Stage 1 에서 결정.
- 회귀 대상 샘플 목록 확정 (exam_*, k-water-rfp, 21언어, shortcut, 다단+표분할/목차/각주).
- **단일 룰 판정** — 분기/허용오차 필요 시 자문 (메모리 `feedback_rule_not_heuristic`).

산출물: `mydocs/working/task_m100_849_stage1.md` (산출 경로 정리표 + exam_math 과소추정 지점 + 수정 범위 + 회귀 목록 + 단일 룰 판정). → 승인 요청.

> Stage 1 결과 수정 범위가 `layout.rs` 대수술 등으로 커지면 범위 재조정 재제안.

---

## Stage 2 — 구현: 밴드 높이 공유 헬퍼

목표: 밴드 높이 = "그 밴드 각 단에 배치된 모든 항목의 실제 점유 vpos 끝(인라인 수식·도형·표 박스 + 떠다니는 TopAndBottom 표 영역 포함)의 최댓값" 으로 산출하는 공유 헬퍼 도입.

작업 (Stage 1 확정안 기준, 대략):
- 신규 헬퍼 `fn band_filled_height(&self, page_column_contents: &[ColumnContent], zone_y_offset: f64, paragraphs: &[Paragraph]) -> f64` (또는 동등) — 같은 `zone_y_offset` 의 단들에 대해 각 단의 실제 점유 끝을 산출, max 반환. (Stage 1 에서 "단의 실제 점유 끝" 정의 확정 — LINE_SEG vpos_end 만으로 충분한지, 떠다니는 표 높이 별도 합산이 필요한지.)
- `process_multicolumn_break`: 현 `vpos_zone_height`(직전 문단 한 개 기준) → flush 후 `band_filled_height(...)` 로 대체. `[다단나누기]` 동작 회귀 점검.
- `start_new_column_band` (#846): 현 밴드 높이 산출(밴드 각 단 마지막 문단 max) → `band_filled_height(...)` 로 대체. 떠다니는 개체 가드·`available_height` 비교 로직은 유지.
- `renderer/layout.rs` 의 단 영역 y 좌표가 갱신된 zone_y_offset 을 반영하는지 확인 (이미 반영되면 무수정).

검증: `exam_math.hwp` 페이지 3 단 0 밴드 높이가 ~1100px 로 산출, 페이지 수가 PDF(20) 에 근접. shortcut.hwp 7페이지 정합 유지. `cargo test` 전건 (특히 `test_exam_math_page_count`/`test_539`/`test_548` 복구). 1차 회귀: exam_* / 21언어.

산출물: `mydocs/working/task_m100_849_stage2.md` (수정 내역 + 밴드 높이 before/after + 페이지 수 비교).

---

## Stage 3 — 광역 회귀 검증

목표: 다단/단일 단/표분할 전 영역 회귀 없음 확정 (메모리 `feedback_essential_fix_regression_risk`).

작업:
- 다단 샘플 전수: exam_* 류, k-water-rfp, 21언어, 다단+TopAndBottom 표, 다단+목차, 다단+각주 — Stage 2 전/후 SVG diff, 변화 시 의도 여부 판정.
- 단일 단(col_count==1) 샘플 일부 — 본 변경이 단단 경로를 안 건드리는지 확인.
- exam_math / 21언어 / shortcut SVG ↔ 한컴 PDF 시각 비교 (macOS 환경 → 2022 PDF 1차 기준, 가능 시 2010/2020 대조; PDF 절대 기준 아님 — 메모리 `feedback_pdf_not_authoritative`).
- `cargo test` 전건 + 신규 clippy 0.

산출물: `mydocs/working/task_m100_849_stage3.md` (회귀 점검표 + 잔여 차이 설명).

---

## Stage 4 — 종합 검증 및 최종 보고서

작업:
- `cargo test` 전건 최종 통과, 신규 clippy 0.
- exam_math / 21언어 / shortcut PDF 정합 최종 확인, before/after 캡처.
- 회귀 대상 샘플 최종 점검. #846 합류 가능 상태(`local/task846` 의 `start_new_column_band` 가 본 헬퍼 사용, 회귀 0) 확인.
- 최종 보고서 `mydocs/report/task_m100_849_report.md` — #846 과의 의존·합류 순서 명기.
- merge 전 `git status` 미커밋 확인. 머지 순서: `local/task846` → `local/task849` → `local/devel` → `devel` (메모리 워크플로우). PR 대상 `devel` (메모리 `feedback_pr_target_devel`).

---

## 커밋 규약
- 각 Stage 소스 + `working/task_m100_849_stage{N}.md` 함께 커밋, 메시지 `Task #849: ...`.
- 본 계획서(수행 + 구현) 1커밋. 최종 보고서 커밋 후 PR.
- `mydocs/orders/` 변경 금지 (메모리 `feedback_no_orders_modification`).
