# PR #1702 리뷰 처리 계획

## 적용 경로

PR #1702는 collaborator self-merge 후보 경로다.

- PR 작성자: @postmelee
- PR head: `postmelee:task1664-ci-cache-workflow`
- review 문서 경로: `mydocs/pr/archives/`
- merge 전 조건: 최신 PR head 기준 GitHub checks 확인 + review 문서 diff 포함 + 작업지시자 승인

## 처리 단계

### Stage 1 - PR head 동기화

완료.

- `origin/task1664-ci-cache-workflow` fetch
- `upstream` PR ref fetch: `pull/1702/head:local/pr1702`
- 로컬 `task1664-ci-cache-workflow`를 `origin/task1664-ci-cache-workflow`로 fast-forward
- 동기화 후 head: `924b9b4e80fe46a663d4af0e66542144dee78517`

### Stage 2 - 변경 범위 및 정적 검증

완료.

- `upstream/devel...HEAD` 기준 변경 파일: `.github/workflows/ci.yml`
- `git diff --check upstream/devel...HEAD` 통과
- `actionlint .github/workflows/ci.yml` 통과
- 코드, 테스트, fixture, golden, profile 변경 없음 확인

### Stage 3 - 리뷰 보강 반영 확인

완료.

- PR body에 `Build & Test` cargo cache 한정 범위 명확화 반영
- CodeQL Rust analyze cache는 #1667 또는 별도 후속 PR에서 분리 검토한다고 명시
- #1664 issue comment에 before/after 측정표와 범위 보강 반영
- 1차 검토 open question 해소 확인

### Stage 4 - 리뷰 문서 push

이 문서와 `pr_1702_review.md`를 PR head에 별도 문서 커밋으로 push한다.

대상 파일:

- `mydocs/pr/archives/pr_1702_review.md`
- `mydocs/pr/archives/pr_1702_review_impl.md`

push 후 확인:

- PR head SHA가 문서 커밋으로 변경됐는지 확인
- PR diff에 위 review 문서 2건이 포함됐는지 확인
- PR diff에 의도하지 않은 파일이 추가되지 않았는지 확인
- 문서 전용 후속 커밋이므로 GitHub Actions fast-pass 또는 latest checks 상태를 확인

### Stage 5 - merge 전 확인

작업지시자 승인 후 merge 직전에 다음을 다시 확인한다.

- latest head SHA
- `mergeable` / merge state
- latest checks 상태
- PR diff에 review 문서 2건 포함 여부
- GitHub review 또는 PR comment 상태
- #1664 / #1668 이슈 close 필요 여부

## merge 후 후속

PR #1702는 `Refs #1664, #1668` 성격의 workflow 코드 PR이다. merge 후에도 #1668은 tracking/RFC 이슈이므로
close하지 않는다.

후속 확인:

- `devel` push run에서 trusted branch cache save 경로 확인
- exact cache hit이면 save skipped도 정상 상태로 기록
- #1664 close 여부는 #1702 merge 및 trusted branch run 관측 후 작업지시자 승인으로 판단
- #1666, #1667 후속 비교 기준으로 #1664 raw 측정값을 유지
