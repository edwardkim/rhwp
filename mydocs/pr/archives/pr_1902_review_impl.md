# PR #1902 처리 계획

## Stage 1. PR 생성 및 코드 검증

완료.

- #1638 구현 커밋: `2451e240dc9d9eb2b597b7ddb0c159b1f3e9387c`
- PR: https://github.com/edwardkim/rhwp/pull/1902
- PR 본문에 focused 검증, self PR full CI equivalent, Linux/Windows 교차 환경 검증 결과를 포함했다.

## Stage 2. 옵션 1 운영 문서 반영

진행.

- `mydocs/pr/archives/pr_1902_review.md`
- `mydocs/pr/archives/pr_1902_review_impl.md`
- `mydocs/orders/20260704.md`

이번 PR은 CLI 검증 게이트 변경으로 visual sweep 대상이 아니므로 asset은 추가하지 않는다.

## Stage 3. GitHub Actions 대기

예정.

- 최신 PR head 기준 Build & Test / CodeQL 등 required check 통과를 확인한다.
- 이전 SHA run이 남아 required check와 섞이면 `pr_review_workflow.md`에 따라 이전 SHA run만 force-cancel 한다.

## Stage 4. Merge 및 후속 처리

예정.

- PR merge 후 `devel`을 `upstream/devel`로 fast-forward sync 한다.
- #1638 auto-close 여부를 확인한다.
- auto-close가 이미 동작했더라도 이슈에 merge commit, 검증 요약, 남은 후속 과제 없음 여부를 코멘트한다.
- self PR head 브랜치와 로컬 작업 브랜치를 `pr_review_workflow.md` 7.7 절에 따라 정리한다.
