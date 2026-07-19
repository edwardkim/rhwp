# Task M100 #2072 Stage 10 - 열린 font ownership 작업 분리

## 목표

열린 #2125의 font ownership inventory·migration 계획을 장기 canonical 문서와 구분한다. 이슈가
열려 있다는 사실은 유지하되, task 문서를 `tech` 루트에 두지 않는다.

## 이동 대상과 근거

- `task_m100_2125_font_ownership.md`
  - #2125의 Stage 1 inventory이며 기준 source commit·계획 commit·당시 font bytes와 migration 목표를 기록한다.
  - #2125는 열려 있어 source-of-truth 이전이 최종 확정된 장기 계약이 아니다.
  - GitHub issue/PR 본문과 코멘트의 기존 파일명 검색 결과는 0건이다.

따라서 `mydocs/tech/investigations/issue-2125/`로 이동한다. 새 디렉터리 README는 현재 contract가
이슈 구현 완료 전까지 검증 대상임을 명시한다. redirect stub은 만들지 않고 저장소 내부 참조를 직접 갱신한다.

## 제외 범위

- `assets/fonts` 이전 구현과 package/extension/VS Code runtime 경로 변경
- `font_fallback_strategy.md`의 장기 fallback 정책 변경
- #2125 issue 상태 변경

## 검증 계획

- 기본 Markdown 링크 검사
- Stage 4~10의 이전 경로를 지정한 `--forbid-path` 검사
- Documentation Link Check YAML 문법과 `git diff --check`

## 결과

- 열린 #2125의 font ownership inventory·migration 제안을 `investigations/issue-2125/`으로 이동했다.
- 이슈 README에 구현 완료 전까지 canonical 계약이 아님을 명시하고, 기존 내부 참조를 새 경로로 갱신했다.
- 기본 링크 검사와 CI workflow에서 추출한 Stage 4~10의 이전 경로 금지 인수 검사가 각각 276개 문서에서
  통과했다. YAML 파싱과 `git diff --check`도 통과했다.

## 다음 단계

문서 이동을 마친 뒤에는 issue #2072와 해당 코멘트의 완료 조건을 다시 대조한다. 남은 root `tech` 문서는
canonical·reference·장기 설계·active guide 중 하나의 역할을 가져야 하며, 새 task 문서는 루트에 만들지 않는다.
