# 구현계획서 — Task #849 (M100) — v2 (Stage 1 진단 반영, 간소화)

대상: #846 의 `start_new_column_band` 가 단 유형을 구분하지 않아 "일반"(신문형) 단에서 회귀. → `start_new_column_band` 호출을 "배분"(Distribute) 단으로 한정. Issue edwardkim/rhwp#849. 브랜치 `local/task849` (← `local/task846`).

> **v1 → v2**: v1 은 "다단 밴드 높이 산출 정합(공유 헬퍼)"을 가정했으나, Stage 1 진단(`working/task_m100_849_stage1.md`)에서 실제 원인이 단 유형 게이트 누락임이 확인됨. 밴드 높이 정합·`layout.rs` 연동은 불요. 본 v2 가 유효.

---

## Stage 1 — 진단 (완료, 소스 수정 없음)

산출물: `mydocs/working/task_m100_849_stage1.md`. 결과: 원인 = `start_new_column_band` 가 `ColumnType` 무관하게 적용 → 신문형(`Normal`) zone 회귀. 수정 = 호출을 `Distribute` 로 한정. (`exam_math` PDF 20 / shortcut 7 / 21언어 15 / `cargo test` 1232 통과 — 시범 적용 검증 완료.)

---

## Stage 2 — 구현: `start_new_column_band` 를 배분 단으로 한정

작업:
- `paginate` 의 명시적 `Column` break 경로: `start_new_column_band` 호출 조건에 `st.current_zone_column_type == ColumnType::Distribute` 추가. `Normal`(신문형) 은 기존 `advance_column_or_new_page` 유지. `Parallel`(평행) 도 현 동작 유지(범위 밖).
- (그 외 #846 의 `start_new_column_band` 본체·`upcoming_band_has_floating_object`·밴드 높이 산출은 그대로.)

검증: shortcut.hwp 7페이지 + pi=94/95 페이지 3, exam_math 20페이지, 21언어 15페이지, `cargo test` 전건(특히 `test_exam_math_page_count`/`test_539`/`test_548`).

산출물: `mydocs/working/task_m100_849_stage2.md`.

---

## Stage 3 — 광역 회귀 검증

작업:
- 다단 샘플 전수 SVG diff (Stage 2 전/후 — 즉 #846+#849 vs #846 단독, 및 vs baseline): exam_* 류, k-water-rfp, 21언어, shortcut, 다단+TopAndBottom 표, 다단+목차, 다단+각주. `Distribute` zone 의 마지막-단 `[단나누기]` 케이스 외엔 무변화 기대.
- 단일 단(col_count==1) 샘플 일부 — 무변화 확인.
- exam_math/21언어/shortcut SVG ↔ 한컴 PDF 시각 비교 (macOS → 2022 1차, 가능 시 2010/2020 대조; PDF 절대 기준 아님 — 메모리 `feedback_pdf_not_authoritative`).
- `cargo test` 전건 + 신규 clippy 0.

산출물: `mydocs/working/task_m100_849_stage3.md`.

---

## Stage 4 — 종합 검증 및 최종 보고서

작업:
- `cargo test` 전건 최종 통과, 신규 clippy 0.
- exam_math/21언어/shortcut PDF 정합 최종 확인, before/after 캡처.
- 회귀 대상 샘플 최종 점검. #846 합류 가능 상태(`local/task846` + `local/task849` 합치면 회귀 0) 확인.
- 최종 보고서 `mydocs/report/task_m100_849_report.md` — #846 과의 의존·합류 순서 명기.
- merge 전 `git status` 미커밋 확인. 머지 순서: `local/task846` → `local/task849` → `local/devel` → `devel`. PR 대상 `devel` (메모리 `feedback_pr_target_devel`).

---

## 커밋 규약
- 각 Stage 소스 + `working/task_m100_849_stage{N}.md` 함께 커밋, 메시지 `Task #849: ...`.
- 본 v2 계획서는 Stage 1 커밋에 포함. 최종 보고서 커밋 후 PR.
- `mydocs/orders/` 변경 금지 (메모리 `feedback_no_orders_modification`).
