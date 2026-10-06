# Task M100 #2072 Stage 9 - 확정 직렬화·분할 대응과 종료 기준선 분리

## 목표

남은 task 문서를 해결 상태와 현재성으로 분류한다. 확정 원인과 수정·검증을 갖춘 #1637·#1860은
`troubleshootings/`으로, 미해결 충돌 또는 종료된 기준선은 `tech/investigations/issue-####/`으로 이동한다.

## 분류 결과

### Troubleshootings

- `task1637_pagination_hidefirstemptyline.md`
  - `hideFirstEmptyLine`과 표 `flowWithText` 직렬화 드롭의 원인, PR #1642 수정, IR diff gate 보강,
    재현 샘플과 최종 보고서를 갖춘다.
- `task_m100_1860_split_budget.md`
  - deferred co-anchored RowBreak 표에서 예산만 참 `para_start`를 사용하도록 분리한 원인·수정·
    권위 PDF 대조·회귀 범위를 갖춘다.

### Investigations

- `task_m100_1472_rootcause.md`
  - HWP3 variant indent 보정이 IR 정확성과 페이지네이션 parity에서 충돌하며 재설계가 필요한 미해결 조사다.
- `task_m100_1904_baseline_manifest.md`
  - 종료된 #1904의 특정 기준 커밋·도구·오라클 freeze snapshot이다.

## Redirect 판단

각 기존 파일명은 GitHub issue/PR 본문과 코멘트에서 외부 참조 0건이다. redirect stub을 만들지 않고
저장소 내부 참조를 새 경로로 직접 갱신한다.

## 제외 범위

- 열린 #2125의 `task_m100_2125_font_ownership.md`: 현재 source-of-truth 계약을 정하는 active 작업이므로
  canonical 승격 또는 이슈 분리는 구현·계약 완료 뒤 별도로 판단한다.
- #1472의 pagination 재설계와 #1637·#1860의 코드·테스트 변경

## 검증 계획

- 기본 Markdown 링크 검사
- Stage 4~9의 이전 경로를 지정한 `--forbid-path` 검사
- Documentation Link Check YAML 문법과 `git diff --check`

## 결과

- #1637의 HWPX visibility·flowWithText 직렬화 보존과 #1860의 deferred RowBreak 예산 교정을
  `troubleshootings/`으로 이동하고 해당 지도에 추가했다.
- #1472의 HWP3 variant indent 충돌 조사와 종료된 #1904 baseline freeze를 각각 이슈별 조사로
  분리했다.
- 기존 파일명은 외부 이력 참조가 없어 redirect stub을 남기지 않았고, 내부 참조와 CI 금지 경로를
  새 위치에 맞췄다.
- 기본 링크 검사와 CI workflow에서 추출한 Stage 4~9의 이전 경로 금지 인수 검사가 각각 275개 문서에서
  통과했다. YAML 파싱과 `git diff --check`도 통과했다.

## 다음 단계

열린 #2125의 font ownership 문서는 계약 구현 완료 뒤 canonical 승격 또는 이슈 조사 보관을 판단한다.
그 외 `tech` 루트 문서는 현재 canonical 문서·reference·장기 설계인지 재감사한다.
