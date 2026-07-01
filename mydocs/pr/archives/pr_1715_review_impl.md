# PR #1715 처리 계획 — #1705 wrap 빈 문단 first/last 페이지 귀속 정정

## 대상

- PR: #1715
- 작성자: @planet6897
- Base: `devel`
- Head: `task/m100-1705`
- 검토 head: `17fee877b0deed7c4c5d72c9d63fd4527c67c4c4` (문서 작성 시점 참고값)
- 관련 이슈: #1705

## 대상 커밋

| 커밋 | 내용 |
|---|---|
| `dcb37eef178f` | #1705 코드 수정, 샘플, 보고서 추가 |
| `e0c740d6c10c` | `devel` 최신화 merge commit |
| `9d0b98b1da4c` | `devel` 최신화 merge commit |
| `17fee877b0de` | `devel` 최신화 merge commit |

## 처리 단계

1. PR head fetch — 완료
   - `git fetch upstream pull/1715/head:local/pr1715`
2. 임시 worktree 생성 — 완료
   - `/private/tmp/rhwp-pr1715-review`
3. merge 시뮬레이션 — 완료
   - `git merge upstream/devel --no-commit --no-ff`
   - 결과: 충돌 없음
4. 로컬 동작 검증 — 완료
   - release-test binary build
   - 동봉 샘플과 #1700 대비 샘플 `dump-pages` 확인
   - 관련 query/wrap 회귀 subset 실행
5. 중간 커밋 Actions 정리 — 완료
   - `9d0b98b1da4cb4c296ee4c5e2d76f9d890419198`의 stale Actions run cancel/force-cancel
6. 리뷰 문서 작성 — 완료
   - `mydocs/pr/pr_1715_review.md`
   - `mydocs/pr/pr_1715_review_impl.md`
7. merge 전 최종 확인 — 완료
   - GitHub Actions `Build & Test` / CodeQL 완료 확인
   - 최신 head SHA 변동 여부 확인
8. 승인 후 merge — 완료
   - `gh pr merge 1715 --repo edwardkim/rhwp --merge --admin`
9. 후속 처리 — 완료
   - #1705 close 상태 확인
   - 필요 시 수동 close + 감사 코멘트
   - PR #1715 감사 코멘트
   - 리뷰 문서 `mydocs/pr/archives/` 이동
   - 오늘할일 갱신

## 검증 기록

```text
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo build --profile release-test --bin rhwp
```

- 결과: 통과

```text
/Users/tsjang/rhwp/target/release-test/rhwp dump-pages samples/task1705/wrap_empty_para_anchor_page.hwp
```

- 결과: 2 pages, page 1에 `WrapAroundPara pi=2 table_pi=1 "(빈)"` 확인

```text
/Users/tsjang/rhwp/target/release-test/rhwp dump-pages samples/task1700/myeonjeok_wrap_10page.hwp
```

- 결과: 10 pages, page 10에 `WrapAroundPara pi=2 table_pi=1 "(빈)"` 유지 확인

```text
/Users/tsjang/rhwp/target/release-test/rhwp dump-pages samples/task1700/byeolpyo1_uujeong_wrap_singlepage.hwp
```

- 결과: 1 page, 단일 페이지 표의 `WrapAroundPara pi=2` 유지 확인

```text
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo test --profile release-test \
  --test issue_1139_inline_picture_duplicate \
  --test issue_1488_rowbreak_empty_overlay_pages \
  --test issue_643 \
  --test page_number_propagation \
  -- --nocapture
```

- 결과: 89 passed

```text
cargo fmt --check
git diff --check upstream/devel...HEAD
```

- 결과: 통과

## Actions 정리 기록

`9d0b98b1da4cb4c296ee4c5e2d76f9d890419198` 기준 stale run:

| Workflow | Run ID | 처리 |
|---|---:|---|
| Render Diff | `28500880064` | cancelled |
| CodeQL | `28500880275` | cancelled |
| CI | `28500880237` | force-cancel 후 cancelled |

최신 head `17fee877b0deed7c4c5d72c9d63fd4527c67c4c4`의 CI run `28500953207`은 rerun 처리했다.

## Merge 전 조건

- PR head 최신 SHA 재확인: `17fee877b0deed7c4c5d72c9d63fd4527c67c4c4`
- GitHub Actions `Build & Test` / CodeQL success 확인: 완료
- 작업지시자 merge 승인: 완료

## Merge 및 후속 처리 기록

- merge 시각: 2026-07-01 08:01:47Z
- merge commit: `3f22b8dd0919d82dbd9c89a8ade73249fe7a04b9`
- #1705 close 시각: 2026-07-01 08:04:08Z
- #1705 close comment: https://github.com/edwardkim/rhwp/issues/1705#issuecomment-4851881453
- PR 후속 comment: https://github.com/edwardkim/rhwp/pull/1715#issuecomment-4851883890

## 게시한 PR 후속 코멘트

```text
@planet6897 감사합니다. PR #1715 머지 완료했습니다.

검증 결과 요약:
- 최신 head `17fee877b0deed7c4c5d72c9d63fd4527c67c4c4` 기준 GitHub Actions 통과 확인
- merge simulation 충돌 없음
- `wrap_empty_para_anchor_page.hwp`: `WrapAroundPara pi=2`가 page 1에 귀속 확인
- `myeonjeok_wrap_10page.hwp`: 전체폭 대비 케이스가 page 10 유지 확인
- 관련 query/wrap 회귀 subset 89건 통과

중간 커밋 `9d0b98b1da4cb4c296ee4c5e2d76f9d890419198`의 stale Actions run은 모두 cancel 처리했습니다.
#1705는 auto-close가 되지 않아 merge 후 수동으로 close 처리했습니다.
merge commit: 3f22b8dd0919d82dbd9c89a8ade73249fe7a04b9

감사합니다.
```
