# PR #1701 리뷰 처리 계획

## 적용 경로

PR #1701은 collaborator self-merge 후보 경로다.

- PR 작성자: @postmelee
- PR head: `postmelee:task1664-docs-policy-measurement`
- review 문서 경로: `mydocs/pr/archives/`
- merge 전 조건: 최신 PR head 기준 GitHub checks 확인 + 작업지시자 승인

## 처리 단계

### Stage 1 — 메타데이터 정렬

완료.

- labels: `ci`, `documentation`
- milestone: `v1.0.0`
- assignee: @postmelee
- self review request는 적용하지 않음

### Stage 2 — 문서 범위 수정

완료.

- 문제: PR #1701 문서 일부가 workflow 변경이 이미 `devel`에 반영된 것처럼 읽힐 수 있었음
- 조치: 수정 커밋 `ed074885` push
- 결과: PR #1701은 문서 전용 PR, #1702는 후속/draft 코드 PR이라는 경계를 명확히 기록

### Stage 3 — 재검토

완료.

- 별도 worktree에서 최신 head 재검토
- `git diff --check upstream/devel...HEAD` 통과
- 변경 범위 `mydocs/**` 유지 확인
- PR #1702 CI 관측값 교차확인
- PR #1701 최신 checks fast-pass 상태 확인

### Stage 4 — 리뷰 문서 push

이 문서와 `pr_1701_review.md`를 PR head에 별도 문서 커밋으로 push한다.

대상 파일:

- `mydocs/pr/archives/pr_1701_review.md`
- `mydocs/pr/archives/pr_1701_review_impl.md`

### Stage 5 — merge 전 확인

작업지시자 승인 후 merge 직전에 다음을 다시 확인한다.

- latest head SHA
- `mergeable` / `mergeStateStatus`
- latest checks 상태
- PR diff에 review 문서 2건 포함 여부
- #1664 / #1668 이슈 close 필요 여부

## merge 후 후속

PR #1701은 `Refs #1664, #1668` 성격의 문서 PR이다. 따라서 merge 후에도 #1664와 #1668을 자동 또는 수동 close
하지 않는다.

후속 확인:

- #1702 workflow 코드 PR의 ready/merge 여부는 별도 판단
- #1702 merge 후 `devel` push run에서 trusted branch cache save 동작 확인
- #1664 close 여부는 #1702 및 측정 확인 후 작업지시자 승인으로 판단
