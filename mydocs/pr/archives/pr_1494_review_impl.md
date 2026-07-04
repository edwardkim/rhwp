# PR #1494 처리 계획

## 커밋 목록

문서 작성 시점 참고값:

- `0001aead` task 1491: 셀 너비 균등화 로컬 힌트 보정
- `dea8e608` task 1491: 병합 셀과 마우스 resize 회귀 보정
- `b3054086` task 1491: 표 resize 런타임 캐시 보정
- `719f7d27` task 1491: 복구본 Shift 셀 resize 보정
- `e5880112` task 1491: 세로 Shift resize 복구본 분석 보강
- `804c6ea0` task 1491: 세로 Shift resize 행 기준 전환
- `b1ff3be2` task 1491: 표 셀 리사이즈와 균등화 보정
- review 문서 추가 커밋은 PR #1494 생성 후 같은 head branch에 추가한다.

## Stage 구성

### Stage A: PR head 준비

- `local/task_m100_1491`을 `upstream/task_m100_1491`로 push한다.
- base `devel`, head `task_m100_1491` Open PR을 생성한다.
- PR 본문에 `Closes #1491`을 포함한다.

### Stage B: review 문서 포함

- PR 번호 확정 후 `mydocs/pr/archives/pr_1494_review.md`와 이 문서를 작성한다.
- 두 문서를 별도 커밋으로 추가하고 `upstream/task_m100_1491`에 push한다.
- PR diff에 review 문서 2건이 포함되는지 확인한다.

### Stage C: merge 전 확인

- PR head 최신 커밋 기준 GitHub Actions 결과를 확인한다.
- `mergeable`, `mergeStateStatus`, head SHA는 merge 직전 최신 상태를 다시 확인한다.
- 작업지시자 승인 상태를 확인한다.

### Stage D: merge 및 후속 처리

- 조건 충족 시 `gh pr merge 1494 --repo edwardkim/rhwp --merge --admin`로 merge한다.
- #1491 auto-close 여부를 확인하고, 열려 있으면 수동 close한다.
- 필요 시 PR 후속 코멘트에 로컬 검증과 merge 결과를 요약한다.
- `local/devel`을 `upstream/devel`과 동기화한다.
- PR head 원격 브랜치 `task_m100_1491` 정리 여부를 확인한다.

## 작업지시자 확인 필요 사항

- 작업지시자는 로컬 검증 완료 후 `pr_review_workflow.md`에 따른 merge 및 후속 처리를 승인했다.
- merge 전 최종 조건은 PR head 최신 커밋 기준 GitHub Actions 통과, review 문서 PR diff 포함, 작업지시자 승인이다.
- PR 후속 코멘트는 과장하지 않고 검증 결과와 후속 처리 사실 중심으로 작성한다.

