# PR #1371 리뷰 처리 구현 계획서

## 현재 상태

| 항목 | 값 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1371 |
| 이슈 | #1363 |
| base | `devel` |
| head | `planet6897:task1363` |
| head SHA | `36a474ed6479f66a02bdd59a2605f24027874115` |
| mergeable | `MERGEABLE` |
| merge state | `CLEAN` |
| GitHub CI | 전체 통과 |
| 로컬 사전 검증 | 빌드/테스트/Clippy/Doc-test/svg snapshot/`diff --check` 통과 |
| merge commit | `0f0c7319cd88ecd2b78136df1907f1d541ed180c` |
| 이슈 상태 | #1363 closed 확인 |

## PR 커밋

| # | SHA | 내용 |
|---|---|---|
| 1 | `251b4c5a` | Task #1363: 미주 높이 모델 측정 SSOT - scratch 전-단 순차 렌더 (A3 opt-in) |
| 2 | `670b6a29` | Merge branch 'devel' into task1363 |
| 3 | `36a474ed` | Task #1363: trailing whitespace 정리 |

## Stage 구성

### Stage 1 - 리뷰 문서 작성 - 완료

- `mydocs/pr/pr_1371_review.md` 작성
- `mydocs/pr/pr_1371_review_impl.md` 작성
- 메인터너 코멘트, GitHub CI, 로컬 사전 검증 결과 기록
- 메인터너 코멘트의 trailing whitespace 지적을 `36a474ed`로 직접 정리

### Stage 2 - 로컬 사전 검증 - 완료

임시 worktree:

```bash
git worktree add -B pr1371-whitespace-fix /tmp/rhwp-pr1371-fix local/pr1371
```

검증:

```bash
git merge upstream/devel --no-commit --no-ff
git diff --check upstream/devel...HEAD
cargo build --lib
cargo test --lib
cargo clippy -- -D warnings
cargo test --doc
cargo test --test svg_snapshot
```

결과:

- merge 시뮬레이션: `Already up to date`
- `git diff --check`: 통과
- `cargo build --lib`: 통과
- `cargo test --lib`: 1724 passed, 0 failed, 6 ignored
- `cargo clippy -- -D warnings`: 통과
- `cargo test --doc`: 통과
- `cargo test --test svg_snapshot`: 8 passed, 0 failed

### Stage 3 - GitHub Actions 최종 확인

최종 확인:

- `Build & Test`: 통과, 14m 58s
- `Analyze (javascript-typescript)`: 통과
- `Analyze (python)`: 통과
- `Analyze (rust)`: 통과
- `Canvas visual diff`: 통과
- `CodeQL`: 통과
- `WASM Build`: skipped

### Stage 4 - merge

작업지시자가 GitHub Actions와 local test 완료 후 merge를 지시했고, 두 조건을 모두
충족했으므로 진행한다.

```bash
gh pr merge 1371 --repo edwardkim/rhwp --merge
```

결과:

- `gh pr merge --admin`은 사용하지 않고 일반 merge로 처리했다.
- merge commit: `0f0c7319cd88ecd2b78136df1907f1d541ed180c`
- #1363 closed 상태를 확인했다.

### Stage 5 - 후속 문서 정리

merge 후 처리:

```bash
git fetch upstream
git checkout local/devel
git rebase upstream/devel
```

정리 대상:

- `mydocs/pr/pr_1371_review.md` -> `mydocs/pr/archives/`
- `mydocs/pr/pr_1371_review_impl.md` -> `mydocs/pr/archives/`
- task #1363 active 문서 중 archive 정리 대상
- `git diff --check`에서 잡힌 trailing whitespace
- 닫힌 PR #1396의 누락 문서:
  - `mydocs/orders/20260612.md`
  - `mydocs/pr/archives/pr_1395_review.md`
  - `mydocs/pr/archives/pr_1395_review_impl.md`

결과:

- `local/devel`을 `upstream/devel` 위로 rebase하면서 #1396 누락 문서 커밋을 보존했다.
- `mydocs/pr/pr_1371_review.md`와 `mydocs/pr/pr_1371_review_impl.md`를 archives로 이동했다.
- `mydocs/orders/20260612.md`에 PR #1371 처리 내역을 추가했다.

## 위험 요소 요약

| 위험 | 판단 |
|---|---|
| GitHub CI | 통과 |
| 로컬 빌드/테스트 | 통과 |
| trailing whitespace | 해소됨 |
| 문서 중복/active 재도입 | merge 후 정리 필요 |
| #1374 스택 | #1371 선처리 후 재검토 필요 |

## 권고

#1371은 GitHub Actions, 로컬 테스트, 메인터너 코멘트 반영을 모두 통과해 merge 완료됐다.
문서 정리와 #1396 누락 문서 반영도 같은 후속 처리 흐름에 포함했다.
