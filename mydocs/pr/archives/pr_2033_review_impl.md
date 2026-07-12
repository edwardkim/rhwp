# PR #2033 처리 계획

- PR: #2033
- 관련 이슈: #2032
- 작성자: `lpaiu-cs` (`FIRST_TIME_CONTRIBUTOR`)
- 기준 브랜치: `devel`
- head: `lpaiu-cs:fix/picture-offpage-loss`
- 문서 작성 시점 참고 head: `c8c9ed1eec46ff1806f84b7b2783baaaf780d130`
- merge commit: `2b213ab706f7824f32bbb64b06831d9b794bce5d`
- 처리 방식: 옵션 2. 원 코드 PR merge 후 docs-only 후속 PR 로 review 문서와 asset 을 반영

## Stage 1. 검토 완료

- reviewer `jangster77` 지정.
- PR 메타 확인: base `devel`, draft 아님, maintainer edit 가능.
- 동일 작성자 PR 누적 확인: #2028 merge 완료, #2033/#2039/#2040 open. GitHub `author_association` 기준
  #2033 은 `FIRST_TIME_CONTRIBUTOR`.
- 코드 diff 확인: `src/renderer/layout/picture_footnote.rs`, `tests/issue_2032_picture_offpage_restrict_loss.rs`.
- 로컬 검증과 visual sweep 완료.
- approve review 게시 후 원 PR merge 완료.

## Stage 2. docs-only 후속 PR

원 코드 PR 에 review 문서/asset 을 섞지 않기 위해 후속 브랜치 `docs/pr2033-review-record` 에 다음만 포함한다.

- `mydocs/pr/archives/pr_2033_review.md`
- `mydocs/pr/archives/pr_2033_review_impl.md`
- `mydocs/pr/assets/pr_2033_ta_pic_restrict_on_review_p001.png`
- `mydocs/pr/assets/pr_2033_ta_pic_restrict_off_review_p001.png`
- `mydocs/orders/20260708.md`

이 후속 PR 은 review 문서/asset/오늘할일만 바꾸는 기록 PR 이다.

## Stage 3. merge 후 후속 처리

후속 docs-only PR merge 뒤 `mydocs/manual/pr_review_workflow.md` 7장을 따른다.

1. 후속 PR merge commit 확인.
2. `devel` 을 `upstream/devel` 로 fast-forward sync.
3. #2032 close 상태 확인. 원 PR 본문에 `Closes #2032` 가 있으므로 auto-close 가 동작할 수 있다.
4. #2032 에 원 PR merge commit, GitHub Actions, 로컬 검증, visual sweep 요약을 후속 코멘트로 남긴다.
5. PR head 는 fork branch 이므로 원격 branch 삭제는 contributor 권한/설정에 맡긴다. 로컬 `pr2033-review` 브랜치는
   merge 후 정리한다.
6. 후속 docs-only 브랜치와 원격 head 브랜치를 정리한다.
