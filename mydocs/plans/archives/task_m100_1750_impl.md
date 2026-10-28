# Task #1750 구현계획서 — 분할 진입 가드 spacing_before 반영 (3단계)

수행계획서: `mydocs/plans/task_m100_1750.md`

## Stage 1 — 재현 샘플 동결 + 실패 재현
- `samples/task1750/split_guard_spacing_before.hwp` 로 재현 파일(3024019) 복사 + README.
- `rhwp dump-pages` 로 실패 기록 (1쪽 PartialParagraph pi=22 lines=0..1, used 1010.9px).

## Stage 2 — 수정 + 단위테스트
- `src/renderer/typeset.rs` 분할 진입 가드: `remaining < first_line_h`
  → `remaining < first_line_h + fmt.spacing_before`.
- 단위테스트: 진입 가드 판정을 함수화하기 어려우면 통합 스냅샷(재현 파일 dump-pages 기반
  기존 테스트 컨벤션) 또는 spacing_before 경계 시나리오 단위테스트 추가.
- 재현 파일 pi22 → 2쪽 전체 배치 확인.

## Stage 3 — 회귀 검증 + 최종보고
- `cargo test --lib`, byeolpyo1/4 · 승강기(#1718) · task1700 페이지 게이트.
- 한글 OLE 대조: 재현 파일 + 잔여 mismatch 38건 + MATCH 표본 150건 (악화 0).
- rustfmt(변경 파일)/clippy.
- 최종보고서 `mydocs/report/task_m100_1750_report.md` → squash → PR.
