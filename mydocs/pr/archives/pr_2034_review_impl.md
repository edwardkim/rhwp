# PR #2034 처리 계획

- PR: #2034
- 관련 이슈: #1994
- 작성자: `planet6897` (`CONTRIBUTOR`)
- 기준 브랜치: `devel`
- head: `planet6897:fix/1994-behindtext-table-overlap`
- 문서 작성 시점 참고 head: `a11997ea444736d654db3e8a7cffa41c62a0f610`
- merge commit: `e5f9e00864f2b0423c9292fc170a2477b2882504`
- 처리 방식: 옵션 2. 코드 PR merge 후 docs-only 후속 PR 로 review 문서, 기준 PDF, 대표 visual asset 반영

## Stage 1. 검토 완료

- reviewer `jangster77` 지정.
- PR 메타 확인: base `devel`, draft 아님, maintainer edit 가능, 작성자 관계 `CONTRIBUTOR`.
- 코드 diff 확인: `src/renderer/typeset.rs`, `tests/issue_1994_behindtext_table_overlap.rs`,
  `samples/basic/issue1994_behindtext_table_20200830.hwp`, 계획/보고 문서.
- issue 댓글 첨부 기준 PDF 확인:
  https://github.com/edwardkim/rhwp/issues/1994#issuecomment-4903928794
- 첨부 PDF 보존:
  - `samples/issue1994/issue_1994.pdf`
  - `pdf/issue1994/issue_1994.pdf`
- 대표 visual asset 보존:
  - `mydocs/pr/assets/pr_2034_issue1994_review_p003.png`
- 로컬 검증과 visual sweep 완료.
- approve review 게시 완료.
- 원 코드 PR #2034 merge 완료: `e5f9e00864f2b0423c9292fc170a2477b2882504`.
- #1994 는 GitHub closing keyword 로 자동 close 됨.

## Stage 2. docs-only 후속 PR

원 코드 PR 에 maintainer review 기록을 섞지 않기 위해 옵션 2로 처리한다. docs-only 브랜치에 다음만 포함한다.

- `mydocs/pr/archives/pr_2034_review.md`
- `mydocs/pr/archives/pr_2034_review_impl.md`
- `mydocs/pr/assets/pr_2034_issue1994_review_p003.png`
- `samples/issue1994/issue_1994.pdf`
- `pdf/issue1994/issue_1994.pdf`
- `mydocs/orders/20260708.md`

이 후속 PR 은 review 문서, 기준 PDF, 대표 visual asset, 오늘할일 기록만 바꾸는 fast-pass 후보 PR 이다.

## Stage 3. merge 후 후속 처리

후속 docs-only PR merge 뒤 `mydocs/manual/pr_review_workflow.md` 7장을 따른다.

1. 후속 docs-only PR merge commit 확인.
2. `devel` 을 `upstream/devel` 로 fast-forward sync.
3. #1994 closed 상태를 다시 확인하고, 원 PR merge commit, GitHub Actions, 로컬 검증, 기준 PDF,
   visual asset 링크를 후속 코멘트로 남긴다.
4. PR #2034 에 감사 및 검증 요약 코멘트를 남긴다.
5. contributor fork branch 는 작성자 권한/설정에 맡기고, 로컬 `pr2034-review` 브랜치는 review 종료 후 정리한다.
6. 후속 docs-only 브랜치와 원격 head 브랜치를 정리한다.
