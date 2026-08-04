# PR #3132 메인터너 보정 실행 계획

## 1. 대상과 원칙

- 원 PR: [#3132](https://github.com/edwardkim/rhwp/pull/3132)
- contributor source head: `myeolinmalchi/rhwp:feat/export-doclang@587e786d`
- 로컬 메인터너 보정 branch: `review/pr3132-maintainer-20260723`
- 로컬 보정 commit: `0afa775b3 fix(doclang): export 안전성과 EqEdit doctest 보정`

보정은 contributor의 마지막 commit 위에 새 commit을 추가하는 방식이다. contributor commit을 rebase,
amend, force-push하거나 변경 내용을 체리픽해 별도 통합 branch로 옮기지 않는다. 이 문서는 9.2 절의
archive 경로로 PR head에 포함하는 운영 기록이다.

## 2. 커밋 구성

1. `0afa775b3` — `export-doclang` 동일 파일 보호를 기존 identity helper로 통일하고 symlink/hard link
   CLI 회귀 테스트 및 EqEdit doctest import를 보정한다.
2. 이 운영 문서 commit — `mydocs/pr/archives/pr_3132_review.md`, 이 계획서,
   `mydocs/manual/pr_review_workflow.md`의 contributor PR head 직접 보정·remote push 절을 추가한다.

## 3. 실행 단계

1. **완료** — contributor source head `587e786d`에서 보정 branch를 만들고 1번 commit을 작성했다.
2. **완료** — `cargo fmt --check`, `cargo test --test doclang_export`,
   `cargo test --profile release-test --tests --no-fail-fast`, `cargo test --doc`,
   `cargo clippy --all-targets -- -D warnings`, `cargo check --target wasm32-unknown-unknown --lib`를
   전용 target에서 통과시켰다.
3. **완료(로컬)** — archive review 문서와 contributor PR head 직접 보정·remote push 절을 2번 운영 문서
   commit으로 준비했다. remote에는 아직 push하지 않았다.
4. **승인 대기** — source branch SHA와 PR `headRefOid`를 다시 확인하고, LFS 객체 변경이 없음을 확인한다.
   `GIT_LFS_SKIP_PUSH=1 git push https://github.com/myeolinmalchi/rhwp.git HEAD:feat/export-doclang`으로
   두 collaborator 추가 commit을 source head에 직접 push한다.
5. **승인 대기** — 최신 code SHA 기준 GitHub Actions를 확인한다. code/test 변경이 있으므로 문서-only
   fast-pass를 적용하지 않는다.
6. **승인 대기** — 최신 mergeability/CI를 확인하고 GitHub review 또는 contributor 코멘트를 남긴 뒤 merge
   여부를 결정한다.

## 4. 롤백 경계

push 전에는 local collaborator commit만 존재하므로 해당 local branch/worktree만 정리하면 된다. push 후 문제가
발견되면 contributor의 원 commit을 rewrite하지 않고, source head 위에 되돌림 또는 정정 commit을 추가한다.
