# Task M100 #1488 구현 계획

- 이슈: #1488 [HWPX] Rowbreak 표 페이지네이션 여분 페이지/겹침
- 브랜치: `local/task_m100_1488`
- 작성일: 2026-06-25
- 수행계획서: [`task_m100_1488.md`](task_m100_1488.md) (승인 완료)

## 기본값 (수행계획서 승인 시 확정)

- 목표 페이지 수: 정답지 `pdf/rowbreak-problem-pages-2024.pdf` = **18페이지** 기준 수렴
- 7p 셀 텍스트 겹침 / 12p 콜아웃 잘림: Stage 1 분류 결과에 따라 포함/분리 결정

## 대상 코드

- `src/renderer/layout/table_layout.rs` — `cell_units`, `advance_row_cut`, `row_cut_content_height`,
  `row_cut_range_has_visible_content`
- `src/renderer/typeset.rs` — 표 분할 walk 루프(약 10795~11200, Task #993/#1022/#1025 계보)
- 회귀 테스트: `src/renderer/pagination/tests.rs` 또는 `tests/` 통합 테스트

## 단계

### Stage 1 — 진단 계측 및 결함 분류 (코드 무수정)

- `RHWP_TABLE_DRIFT` 등 기존 진단으로 sec1 pi=28 분할의 fragment 별 소비/유닛 경계 확정.
- `advance_row_cut` 가 가용 954px 중 32~85px 소비 후 컷을 끊는 직접 원인 식별:
  vpos 리셋 hard-break 처리인지, 유닛 높이 오버레이 합산 오판인지 구분.
- 6개 시각 결함(2/7/10/12/16/23p + 17~22 여분)을 근본 원인별로 분류 →
  7p/12p 본 타스크 포함 여부 확정.
- 산출: `mydocs/working/task_m100_1488_stage1.md` (진단 결과 + 결함 매핑 표)
- 승인 요청.

### Stage 2 — cut 분할 패킹 보정 (여분 페이지 제거)

- `advance_row_cut`/`cell_units` 가 vpos 리셋·오버레이 빈 유닛을 가용 예산까지 정상 패킹하도록 보정.
  (조기 컷 종료 제거 — 한 페이지에 묶일 유닛을 페이지마다 1~2개씩 흩뿌리지 않도록)
- 기존 정상 분할(보호 블록, 단일행 intra-split, 반복 제목행) 동작 불변 보장.
- 검증: sec1 pi=28 분할이 used=0.0px 빈 페이지 0, 페이지 수 18~20 수렴.
- 산출: 소스 커밋 + `mydocs/working/task_m100_1488_stage2.md`
- 승인 요청.

### Stage 3 — overflow/잔여 시각 결함 보정

- PartialTable 하단 잘림(LAYOUT_OVERFLOW: 10p/23p 등) 보정.
- 페이지 2 본문·도식 겹침이 Stage 2 분할 보정으로 해소되는지 확인, 잔여 시 추가 보정.
- (Stage 1 판정 시) 7p/12p 결함 처리.
- 대상 페이지 `LAYOUT_OVERFLOW` 미발생(허용 오차 내) 확인.
- 산출: 소스 커밋 + `mydocs/working/task_m100_1488_stage3.md`
- 승인 요청.

### Stage 4 — 회귀 테스트 및 전체 검증

- 회귀 테스트 추가: 본 샘플 페이지 수 + 핵심 페이지 overflow 부재 가드.
- `cargo test --test hwpx_roundtrip_baseline` 무회귀.
- 전체 `cargo test` 통과 ([[feedback_full_cargo_test_before_pr]] 정합).
- 정답지 PDF(18p) 시각 대조(export-svg/png).
- 산출: `mydocs/report/task_m100_1488_report.md` + 최종 커밋
- 승인 요청.

## 위험 요소

- 표 cut 분할은 Task #993/#1022/#1025/#474/#713/#1022 등 다수 회귀 가드가 누적된 민감 영역.
  단일 게이트 수정이 타 샘플 회귀를 유발할 수 있어 전체 cargo test + baseline 필수.
- vpos 리셋/오버레이는 [[tech_trailing_model_no_ssot]] 처럼 문서별로 정답이 다를 수 있으므로
  광범위 통일 대신 조건부 게이트로 최소 변경 지향.
