# PR #1455 처리 계획 - rhwp-studio 쪽 테두리 미리보기 버튼 토글 복구

- PR: https://github.com/edwardkim/rhwp/pull/1455
- 관련 이슈: #1426
- base: `devel`
- head: `task_m100_1426`
- 처리 경로: collaborator self-merge 후보 예외 경로

## 1. 처리 원칙

이 PR은 collaborator self-merge 후보 예외 경로를 적용한다.

- review 문서와 처리 계획서를 PR diff에 포함한다.
- `draft`, `mergeable`, `head SHA`, `CI 상태`는 merge 후 낡는 값이므로 확정 사실처럼 기록하지 않는다.
- 최종 merge는 PR head 최신 커밋 기준 GitHub Actions 통과와 작업지시자 승인 후 진행한다.
- merge 전에는 PR diff에 `mydocs/pr/archives/pr_1455_review.md`와
  `mydocs/pr/archives/pr_1455_review_impl.md`가 포함되어 있는지 확인한다.

## 2. 커밋 구성

PR head는 다음 작업 단위로 구성한다.

1. `Task #1426 Plan: Add execution and implementation plans`
   - 수행계획서와 구현계획서 추가
2. `Task #1426 Stage 1: Implement page border state rules`
   - 개별/전체 버튼 토글 로직 구현
   - `테두리 사용 안 함` 체크 시 내부 방향 상태 전체 해제
   - 활성 방향에만 선 모양 바로 적용
   - 1단계 완료보고서 추가
3. `Task #1426 Stage 2: Add regression coverage`
   - 개별/전체 버튼 토글 E2E 추가
   - none 상태에서 전체/개별 버튼 적용 E2E 추가
   - 2단계 완료보고서 추가
4. `Task #1426 Stage 3: Final report and daily order update`
   - 3단계 완료보고서, 최종 보고서, 오늘할일 갱신
5. `PR #1455 Review: Add self-merge review records`
   - 본 review 문서와 처리 계획서 추가

## 3. 최종 조작 규칙

| 조작 | 규칙 |
|------|------|
| 개별 방향 버튼 | 해당 방향이 켜져 있으면 해제, 꺼져 있으면 현재 선 속성으로 적용 |
| 전체 버튼 | 네 방향이 모두 켜져 있으면 전체 해제, 하나라도 꺼져 있으면 전체 적용 |
| `테두리 사용 안 함` 체크 | 네 방향 내부 상태를 모두 해제하고 preview 선을 숨김 |
| none 상태에서 개별 방향 클릭 | 클릭한 방향만 적용 |
| none 상태에서 전체 클릭 | 네 방향을 한 번에 적용 |
| `선 모양 바로 적용` | 현재 켜진 방향에만 선 속성 변경 반영 |

## 4. 검증 전략

코드 변경은 rhwp-studio 대화상자 UI에 한정된다. 다음 검증을 기준으로 한다.

- `cd rhwp-studio && node --check e2e/page-border-toggle.test.mjs`
- `cd rhwp-studio && npm run build`
- `cd rhwp-studio && CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" node e2e/page-border-toggle.test.mjs --mode=headless`
- `cd rhwp-studio && CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" node e2e/dialog-theme.test.mjs --mode=headless`
- `git diff --check`

## 5. GitHub 처리 순서

1. `task_m100_1426`을 최신 `upstream/devel` 기준 커밋 이력으로 정리한다.
2. 기능 로직, 회귀 테스트, 최종 보고, PR review 문서 커밋을 분리한다.
3. PR 본문을 최종 조작 규칙과 검증 기준 중심으로 갱신한다.
4. 정리된 PR head를 `upstream/task_m100_1426`에 push한다.
5. GitHub Actions가 PR head 최신 커밋 기준으로 통과하는지 확인한다.
6. 작업지시자 승인에 따라 draft 해제 여부를 결정한다.
7. 작업지시자 승인 후 collaborator 권한으로 merge한다.
8. merge 후 #1426 close 여부를 확인한다.

## 6. merge 전 확인 조건

merge 직전에는 다음 조건을 최신 상태로 확인한다.

- PR head 최신 커밋 기준 GitHub Actions 통과
- PR diff에 `mydocs/pr/archives/pr_1455_review.md`와 `pr_1455_review_impl.md` 포함
- PR merge 가능 상태
- 작업지시자 승인

## 7. merge 후 추가 문서 커밋 방지

이 PR의 review 문서와 처리 계획서는 PR head에 포함한다. 따라서 merge 후 별도 저장소 문서 커밋은 만들지 않는다.

필요한 후속 작업은 저장소 변경 없이 다음 상태 확인으로 제한한다.

- PR #1455 merged 상태 확인
- #1426 closed 상태 확인
- auto-close 실패 시 작업지시자 승인 후 수동 close
