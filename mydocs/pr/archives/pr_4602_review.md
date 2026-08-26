---
kind: pr-review
status: pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4602 리뷰 - Subsecond 핫패치 4개 원 PR 통합

## 라우팅과 접수

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, multi_pr_update_branch.md
```

| 항목 | 문서 작성 시점 참고값 |
| --- | --- |
| PR | [#4602](https://github.com/edwardkim/rhwp/pull/4602) |
| base | `devel` / `4f9e4ae694d53162a1a8fd2e2606562d7635085d` |
| code candidate | `5af35460e` |
| trailing review head | 이 문서와 오늘할일을 포함한 후속 docs-only commit |
| source branch | `pr/devel-subsecond-hotpatch-integration-20260811` |
| 통합 원 PR | [#4584](https://github.com/edwardkim/rhwp/pull/4584), [#4590](https://github.com/edwardkim/rhwp/pull/4590), [#4594](https://github.com/edwardkim/rhwp/pull/4594), [#4597](https://github.com/edwardkim/rhwp/pull/4597) |
| 작성 시점 상태 | Open; trailing review head의 GitHub CI 진행 중 |

## 범위와 적용 순서

원 PR을 `#4584 -> #4590 -> #4594 -> #4597` 순으로 최신 `devel` 위에 체리픽했다. 파생 렌더 상태
전면 재구성, patch 결과 진단, watcher 수명 회복, 편집 중 부분 재도색 경계를 하나의 Subsecond 개발
경로로 통합한다. Node test runtime에서 기본 브라우저 진단을 직접 초기화하던 결함은 별도 메인터너
보정 `ed8e0387a`로 처리했다.

원 PR별 수용 판단·개별 한계는 [#4584](pr_4584_review.md), [#4590](pr_4590_review.md),
[#4594](pr_4594_review.md), [#4597](pr_4597_review.md)를, commit 순서와 보정 범위는
[통합 실행 기록](pr_4584_4590_4594_4597_review_impl.md)을 따른다.

## 완료한 검증과 현재 게이트

- 최초 기준선 `a70797db` 위 rebase 전 누적 후보에서 release-test nextest 5,764건, Native Skia 3종,
  WASM build, Studio TypeScript 및 Studio test 847건을 통과했다.
- 최신 `upstream/devel` `4f9e4ae6`으로 rebase는 충돌 없이 완료했고, `git diff --check`와 archive
  review 문서의 내부 Markdown 링크 검사를 통과했다.
- 기준선 전진분 뒤 시작한 전체 nextest는 작업지시자 지시에 따라 중단했다. 종료 코드 `130`은 성공
  결과로 취급하지 않는다.
- 따라서 code candidate와 그 뒤 trailing review head의 GitHub CI·CodeQL·필요한 Render Diff가
  최종 검증 게이트다.
  실제 browser hot-patch apply의 장시간 end-to-end 검증은 아직 자동화하지 않았다.

## 최종 권고

**현재는 CI 대기.** 최신 head의 required checks가 통과하고 작업지시자가 승인하면 수용한다. merge 뒤에는
#4576, #4577, #4578, #4579 자동 close 상태와 원 PR #4584/#4590/#4594/#4597의 close/comment, branch 정리를
`post_merge.md` 절차로 처리한다.
