# #7032 Stage 2 — 빈 셀 문단 배치 구현

- 일자: 2026-09-11 (KST)
- 승인 근거: 메인테이너의 구현계획 승인
- 계획: [구현계획서](../plans/task_m100_7032_impl.md)
- 현재 범위: **R1 온전한 셀·반복 제목행. 검증 진행 중.**

## R1 변경 원리

저장 LINE_SEG가 없고 합성 줄도 없는 실제 빈 문단을, 온전한 셀에서는 누락시키지 않는다.
기존 `empty_no_lineseg_paragraph_metrics`의 대상 판정을 재사용해 빈 TextLine/TextRun 및
캐럿을 만드는 `layout_composed_paragraph`로 전달한다. y만 직접 가산하지 않는다.

controls가 있는 문단, `char_count=0`, 저장 LINE_SEG가 있는 문단, HWP3에는 새 경로를 적용하지
않는다. 세로쓰기는 기존 전용 경로를 유지한다. 파서·IR·스타일 해석·전역 합성은 변경하지 않는다.

## 검증 기록

- RED 테스트 commit: `c98c7f8e3` (제품 소스는 baseline 그대로)
- review worktree: `/home/edward/mygithub/rhwp-review-7032`
- 고정 target: `/home/edward/mygithub/rhwp-shared-review-target`
- 새 원본: `tests/cases/issue_7032_cell_empty_paragraph_flow.rs`
- 파생 suite는 review worktree에서만 준비. source checkout 및 커밋에는 포함하지 않음.
- `output/7032/r1-red.log`: 수정 전 실행 기록. 결과는 실행 완료 후 기록한다.

## 남은 승인 범위

R1에서 실제 cut 창의 의미를 변경하지 않는다. **R2는 미완료**이며, 현재 선택된 빈 atom의
높이·가시 마지막 문단·split 여부를 일관되게 판정하는 구현과 창 안/밖·마지막/연속 빈 문단
반례 검증이 남아 있다. R1의 실제 샘플 통과를 전체 구현 완료로 간주하지 않는다.

R1 SVG/PDF 대조 및 메인테이너 판정 후 R2, 이후 Stage 3 전체 검증·Docker WASM 순서다.
