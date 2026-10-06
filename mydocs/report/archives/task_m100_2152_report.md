# 최종 보고서 — Task M100 #2152: per-pi 오라클 미주 문단 정렬

- 이슈: #2152 / 브랜치: `fix/2152-oracle-endnote-pi` / 작성일: 2026-07-10
- 계획서: `mydocs/plans/task_m100_2152.md`

## 수정 내용

1. `src/document_core/queries/rendering.rs` `dump_page_items` — `[미주]` 라벨을
   FullParagraph 한정에서 **전 kind**(PartialParagraph/Table/PartialTable/Shape)로
   확장 (`para_index >= body_len` 동일 기준). per-pi 오라클의 본문/미주 판별 신호.
2. `tools/verify_pi_page_vs_hangul.py` — `[미주]` 라벨 행을 본문 pi 카운트/매핑에서
   제외. 한글 SetPos 문단 공간(본문만)과 정렬.

## 검증

### 정렬 회복 (한글 2022 COM 오라클 재검)

| 파일 | 수정 전 (#2154 스윕) | 수정 후 |
|---|---|---|
| 3-11월_실전_통합_2022.hwpx | PARA_COUNT (1013 vs 451) | **MATCH 21=21쪽, n_mm 0** |
| 3-11월_실전_통합_2022.hwp | PARA_COUNT | **MATCH 21=21** |
| endnote-01.hwp | PARA_COUNT (45 vs 40) | **MATCH 5=5** |
| SO-SUEOP.hwp | PARA_COUNT (1260 vs 1037) | **MATCH 46=46** |
| 2025 행정업무운영 편람 | PARA_COUNT (2617 vs 2618) | PARA_COUNT 잔존 (393 vs 384) |

- 미주 계열 ~30건이 검증 가능 영역으로 회수 — **회수 문서들의 페이지네이션이
  실제로 전부 정합**임이 확인됨 (사각지대였을 뿐 결함 아님).
- 편람 off-by-one은 별축 잔존 (이슈 유지). 재검에서 쪽수도 393 vs 384로 관찰 —
  후속 조사 대상으로 이슈에 기록.

### 게이트

- cargo test 전체 / clippy 0 / rustfmt 0 (rendering.rs·tools 접촉분)
- dump 출력 계약: 본문 항목 라벨 불변 (기존 [미주] FullParagraph 라벨의 일반화라
  본문-only 소비자 무영향)
- 기존 dump 계약 테스트 갱신: `tests/issue_1139_inline_picture_duplicate.rs`의
  미주-공간 pi 항목 assert 리터럴 20건을 새 라벨(`PartialParagraph[미주]`,
  `Shape[미주]`, `Table[미주]`)로 갱신 — 실제 dump 출력으로 전건 확인 후 반영,
  파일 85/85 통과

## 잔존 (#2152 유지 항목)

- 배포용/DRM·HWPML 군 verdict 세분화 (SKIP_DRM) — 도구 후속
- 편람 off-by-one(문단)·393vs384(쪽수) — 별도 조사
- rhwp 저장본 1문단 열화 군 — #2153 판정과 연동
