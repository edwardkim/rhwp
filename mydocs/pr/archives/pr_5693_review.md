---
kind: review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-20
---

# PR #5693 검토: 마스터 페이지 이미지가 본문 입력을 가로채지 않게 한다

## Metadata

| 항목 | 값 |
| --- | --- |
| PR | [#5693](https://github.com/edwardkim/rhwp/pull/5693) |
| 작성자·self-review | `jangster77` |
| base / head | `devel` / `task_m100_5692` |
| code candidate | `04c91180d6c42fc9ed49628546ba03c706415753` |
| 규모 | 4 files, +63/-1 (문서 포함) |
| 관련 이슈 | [#5692](https://github.com/edwardkim/rhwp/issues/5692) |
| 작성 시점 상태 | `MERGEABLE`, `BLOCKED`; merge 전에 최신 head와 CI 재확인 필요 |

## 범위와 판정

- `plane: 1`의 header/footer가 아닌 master-page 장식은 직렬화된
  `inFrontOfText` 값과 관계없이 본문 그림 hit-test 후보에서 제외한다.
- CanvasKit 재생의 master-page behind-text 처리와 입력 hit-test를 일치시켜,
  물리 37쪽의 전체 배경 이미지가 본문 캐럿을 가로채지 않게 한다.
- 일반 본문 foreground 및 header/footer 그림은 선택 후보로 유지한다.
- 렌더 출력·페이지네이션·fixture를 변경하지 않는다. PDF/SVG visual sweep은 적용하지 않으며,
  Studio TypeScript·전체 test와 hit-test 정책 회귀 테스트로 입력 경로를 검증했다.

## 완료한 로컬 검증

- `cd rhwp-studio && npx tsc --noEmit`이 통과했다.
- `cd rhwp-studio && node --test tests/master-page-picture-hit.test.ts`가 2건 통과했다.
- `cd rhwp-studio && npm test`가 1,021 passed, 1 skipped로 통과했다.
- `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`이 통과했다.

## 결론

**merge 권고.** 최신 PR head의 GitHub Actions가 모두 통과하고 작업지시자 승인을 받은 뒤에만
merge한다. merge 뒤 #5692가 자동 종료됐는지와 remote/local branch 정리 대상은
`post_merge.md` 절차로 확인한다.
