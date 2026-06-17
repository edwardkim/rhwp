# PR #1430 리뷰 처리 계획

## 목적

Task 1428 구현 PR을 Collaborator 절차에 맞게 검증하고, 리뷰 문서와 오늘할일을 처음부터 archive 경로 및 orders 경로에 포함한 상태로 PR head에 반영한다.

## 처리 단계

1. PR 생성 및 메타데이터 확인
   - PR #1430: https://github.com/edwardkim/rhwp/pull/1430
   - base/head: `edwardkim/rhwp:devel` ← `edwardkim/rhwp:task_m100_1428`
   - Draft 아님.
   - `MERGEABLE`, GitHub Actions pending으로 `BLOCKED`.

2. 로컬 검증 결과 반영
   - PR 생성 전 개발 clone에서 전체 PR 준비 검증을 완료했다.
   - Rust release build/lib tests/integration tests, Clippy, WASM build, Studio type/test/build를 통과했다.

3. Collaborator merge 시뮬레이션
   - `review/pr-1430`에서 PR head를 checkout했다.
   - `pr1430-merge-test` 임시 브랜치에서 `upstream/devel` merge 시뮬레이션을 수행했다.
   - 결과는 `Already up to date`, 충돌 없음.

4. 리뷰 문서/오늘할일 커밋
   - `mydocs/pr/archives/pr_1430_review.md`
   - `mydocs/pr/archives/pr_1430_review_impl.md`
   - `mydocs/orders/20260618.md`
   - 문서 전용 추가 변경이므로 `git diff --check`와 변경 파일 범위 확인으로 검증한다.

5. 원격 push
   - Collaborator 규칙에 따라 fork `origin`이 아니라 원본 저장소 remote의 PR head로 직접 push한다.
   - 대상: `git push upstream HEAD:task_m100_1428`

6. GitHub Actions 재확인
   - 문서/오늘할일 push 뒤 CI가 다시 실행된다.
   - 단순히 CI 통과 결과를 문서에 추가하기 위한 재-push는 하지 않는다.
   - 모든 required check가 통과하면 merge 가능 상태로 판단한다.

7. merge 후 후속 처리
   - PR 상태와 merge commit 확인.
   - #1428 close 여부 확인. 자동 close가 실패하면 작업지시자 승인 후 수동 close한다.
   - `devel` 동기화와 로컬 브랜치 정리를 수행한다.

## 주의 사항

- PR 리뷰 문서는 active `mydocs/pr/` 경로를 거치지 않고 archive 경로에 바로 작성한다.
- PR/issue 코멘트가 필요하면 초안을 먼저 보여주고 승인받은 뒤 등록한다.
- 오늘할일과 리뷰 문서는 이번 PR head push에 포함되어야 하며, merge 이후 별도 커밋으로 남기지 않는다.
