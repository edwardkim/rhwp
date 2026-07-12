# PR #1906 처리 계획

## Stage 1. PR 생성 및 코드 검증

완료.

- 구현 커밋:
  - `f708a3cb9f58e5598a56eef86957a65d890aa9b1` — `build-from-ingest` HWPX 직렬화 복구
  - `775c9012220494cecb0207a95326e2b96fc760b9` — 4종 시험지 ingest 검증 문서화
  - `7bfd89ed6127661405435ebbd690131e405318e1` — 시각 정합 범위 정정
- PR: https://github.com/edwardkim/rhwp/pull/1906
- PR 본문에 focused 검증과 4종 시험지 텍스트 보존 결과를 포함했다.

## Stage 2. 옵션 1 운영 문서 반영

완료.

- `mydocs/pr/archives/pr_1906_review.md`
- `mydocs/pr/archives/pr_1906_review_impl.md`
- `mydocs/orders/20260704.md`

이번 PR 은 ingest 텍스트 보존과 CLI 산출 경로 검증이 범위이며 원본 PDF 시각 정합 검증 PR 이 아니다.
따라서 visual sweep asset 은 추가하지 않는다.

## Stage 3. GitHub Actions 대기

예정.

- 최신 PR head 기준 Build & Test / CodeQL / Render Diff required check 통과를 확인한다.
- 옵션 1 문서 커밋 이후 새 head 로 CI 가 다시 도는지 확인한다.
- 이전 SHA run 이 required check 와 섞이면 `pr_review_workflow.md` 에 따라 이전 SHA run 만 force-cancel 한다.

## Stage 4. Merge 및 후속 처리

예정.

- PR merge 후 `devel` 을 `upstream/devel` 로 fast-forward sync 한다.
- #666 auto-close 여부를 확인한다.
- auto-close가 이미 동작했더라도 이슈에 merge commit, 검증 요약, 남은 후속 과제를 코멘트한다.
- self PR head 브랜치와 로컬 작업 브랜치를 `pr_review_workflow.md` 7.7 절에 따라 정리한다.
