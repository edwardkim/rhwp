# PR #1717 처리 계획 — #1716 반복 제목행 overhead 누적 폭주 수정

## 대상

- PR: #1717
- 작성자: @planet6897
- Base: `devel`
- Head: `pr/devel-1716`
- 검토 head: `3c449d3935b0d2e3b79f3c9367656e313712cb85` (문서 작성 시점 참고값)
- 관련 이슈: #1716

## 대상 커밋

| 커밋 | 내용 |
|---|---|
| `7f6c770b84ad` | #1716 코드 수정, 단위테스트, 샘플, 작업 문서 추가 |
| `8d259161e781` | `devel` 최신화 merge commit |
| `3c449d3935b0` | `devel` 최신화 merge commit |

## 처리 단계

1. PR head fetch — 완료
   - `git fetch upstream devel pull/1717/head:local/pr1717 --prune`
2. 임시 worktree 생성 — 완료
   - `/private/tmp/rhwp-pr1717-review`
3. merge 시뮬레이션 — 완료
   - `git merge upstream/devel --no-commit --no-ff`
   - 결과: 충돌 없음 (`Already up to date`)
4. 로컬 동작 검증 — 완료
   - release-test binary build
   - 재현 샘플 `dump-pages` 페이지 수 및 `pi=12` 분할 확인
   - `leading_header_rows` 단위테스트
   - 기존 rowbreak 통합 테스트
   - `cargo fmt --check`
   - `git diff --check`
5. 리뷰 문서 작성 — 완료
   - `mydocs/pr/pr_1717_review.md`
   - `mydocs/pr/pr_1717_review_impl.md`
6. merge 전 최종 확인 — 완료
   - GitHub Actions `Build & Test` 완료 확인
   - 최신 head SHA 변동 여부 확인
7. 승인 후 merge — 완료
   - `gh pr merge 1717 --repo edwardkim/rhwp --merge --admin`
8. 후속 처리 — 완료
   - #1716 close 상태 확인
   - 필요 시 수동 close + 감사 코멘트
   - PR #1717 감사 코멘트
   - 리뷰 문서 `mydocs/pr/archives/` 이동
   - 오늘할일 갱신
9. worktree 정리 — 완료
   - `/private/tmp/rhwp-pr1717-review` 제거는 작업지시자 승인 후 진행

## 검증 기록

```text
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo build --profile release-test --bin rhwp
```

- 결과: 통과

```text
/Users/tsjang/rhwp/target/release-test/rhwp dump-pages samples/task1716/table_scattered_header_rowbreak.hwpx
```

- 결과: 53 pages
- `pi=12` 분할:
  - page 10 rows `0..6`
  - page 11 rows `6..40`
  - page 12 rows `40..86`
  - page 13 rows `86..132`
  - page 14 rows `132..176`
  - page 15 rows `176..183`

```text
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo test --profile release-test --lib leading_header_rows -- --nocapture
```

- 결과: 4 passed

```text
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo test --profile release-test --test issue_rowbreak_chart_overlap -- --nocapture
```

- 결과: 20 passed

```text
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo fmt --check
git diff --check upstream/devel...HEAD
```

- 결과: 통과

## Merge 전 조건

- PR head 최신 SHA 재확인: `3c449d3935b0d2e3b79f3c9367656e313712cb85`
- GitHub Actions `Build & Test` success 확인: 완료
- 작업지시자 merge 승인: 완료

## Merge 및 후속 처리 기록

- merge 시각: 2026-07-01 08:49:02Z
- merge commit: `5e3b1ec652fda14a74af7cf9afd77962e3bb7903`
- #1716 close 시각: 2026-07-01 08:49:33Z
- #1716 close comment: https://github.com/edwardkim/rhwp/issues/1716#issuecomment-4852331846
- PR 후속 comment: https://github.com/edwardkim/rhwp/pull/1717#issuecomment-4852334183
- `/private/tmp/rhwp-pr1717-review`: 작업지시자 승인 후 제거 완료

## 게시한 PR 후속 코멘트

```text
@planet6897 감사합니다. PR #1717 머지 완료했습니다.

검증 결과 요약:
- 최신 head `3c449d3935b0d2e3b79f3c9367656e313712cb85` 기준 GitHub Actions 통과 확인
- merge simulation 충돌 없음
- `table_scattered_header_rowbreak.hwpx`: 53 pages 확인
- `pi=12` RowBreak 표가 page 10~15에 정상 분할되는 것 확인
- `leading_header_rows` 단위테스트 4건 통과
- 기존 rowbreak 통합 테스트 20건 통과

#1716은 auto-close가 되지 않아 merge 후 수동으로 close 처리했습니다.
merge commit: 5e3b1ec652fda14a74af7cf9afd77962e3bb7903

감사합니다.
```
