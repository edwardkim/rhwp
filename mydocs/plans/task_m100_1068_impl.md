# 구현계획서 — Task #1068: treat_as_char 표 새 페이지 이월

- 이슈: edwardkim/rhwp#1068
- 브랜치: `local/task1068` (stream/devel 기준)
- 수행계획서: `task_m100_1068.md` (승인 완료)

## 관련 코드 (정독 출발점)

- atomic TAC top-fit 가드: `typeset.rs:1851` `is_atomic_tac_singleton` (Picture/Shape 한정).
- treat_as_char Table 분기: `typeset.rs:1100` 등.
- 분할(split) 경로: `typeset.rs` 줄 단위 분할 (line_count>0 이후).
- 합성 픽스처: `gen-table` CLI (`src/main.rs:43`).

## 단계 (4단계)

### Stage 1 — 조사 + 공개 픽스처 (소스 무변경)
- typeset.rs 분할 경로에서 **treat_as_char 표 줄**이 어떻게 배치되는지 정독 + 비공개 문서로
  실제 흐름(`LAYOUT_OVERFLOW`/dump-pages) 확인. atomic TAC 가드가 표를 왜 제외하는지 확정.
- **공개 합성 픽스처 생성**: near-full-page treat_as_char 표 1개 + 전후 쪽나누기 문단.
  `gen-table` 또는 별도 생성기로 `.hwp`/`.hwpx` 산출 → `samples/`(공개) 또는 테스트 픽스처로 추가.
  비공개 문서와 동일 증상(표 줄 본문 하단 overflow) 재현 확인.
- 산출물: `working/task_m100_1068_stage1.md` + 픽스처 파일. 커밋: 보고서 + 공개 픽스처(비공개 금지).

### Stage 2 — 설계 + 페이퍼 검증 (소스 무변경)
- atomic TAC 이월 가드를 treat_as_char **Table**(본문보다 작은 단일 표 줄)로 확장하는 정합안.
  - 조건: 표 줄 높이 ≤ 본문 높이(이월 시 들어감) & 현재 페이지에 안 들어감 → 새 페이지 이월.
  - page-larger(본문보다 큰 표, #874) 케이스와 구분(이월해도 안 들어가면 분할).
- 비회귀 케이스(tac-case-*, tac-img-*, table-in-tbox) 예상 동작 표로 모순 점검.
- 산출물: `working/task_m100_1068_stage2.md`.

### Stage 3 — 구현
- typeset.rs 분할/atomic 경로에 표 이월 조건 반영. 최소 변경. 단위 TDD(픽스처 기반).
- 산출물: 소스 + `working/task_m100_1068_stage3.md`.

### Stage 4 — 회귀 검증
- 합성 픽스처 + 비공개 문서 해당 overflow 0, 한컴 PDF 정합(독립 페이지).
- 비회귀 전수(tac-*, table-*), 골든 SVG 8종, `cargo test --release`, 251 합계(769) 무회귀,
  `cargo fmt`(변경 파일).
- 산출물: `working/task_m100_1068_stage4.md` → `report/task_m100_1068_report.md`.

## 완료 기준
treat_as_char 표 본문 하단 overflow 해소(이월 배치) + 비회귀 0 + 골든 회귀 0.

## 리스크
- page-larger(본문보다 큰 표) 와 이월 가능 표의 구분 오류 → 무한 이월/누락. Stage 2에서 경계 명시.
- 합성 픽스처가 비공개 문서 증상을 정확히 재현 못 할 가능성 → Stage 1에서 재현 확인 후 진행.
