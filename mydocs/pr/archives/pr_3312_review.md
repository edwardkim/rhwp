# PR #3312 검토 기록

## 라우팅

```text
base route: collaborator_self_merge
modifiers: intake_and_review, local_validation, review_only_fast_pass
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_self_merge.md, intake_and_review.md, local_validation.md,
  review_only_fast_pass.md
current head: 문서 trailing commit push 전 참고값 7fed99fd2c0e294d41ef2d868e67cd26c54f9290
```

## PR metadata

| 항목 | 문서 작성 시점 참고값 |
| --- | --- |
| PR | [#3312](https://github.com/edwardkim/rhwp/pull/3312) |
| 제목 | `ci: 문서 후속 기록의 직전 green PR head 재사용` |
| 작성자 | `jangster77` |
| base / head | `devel` / `task_m100_3309` |
| 관련 이슈 | [#3309](https://github.com/edwardkim/rhwp/issues/3309) |
| 구현 SHA | `7fed99fd2c0e294d41ef2d868e67cd26c54f9290` |
| 규모 | 8 files, +422/-218 (문서 trailing commit 전) |
| 상태 | `MERGEABLE`, `CLEAN`; 최종 판단 전 최신 head 재확인 필요 |

## 변경 범위와 판단

- CI·CodeQL·Render Diff의 preflight가 trailing review-only commit을 최신순으로 candidate로 탐색한다.
  current base를 포함하는 가장 최근 green candidate를 선택하고, check/workflow가 missing·pending이면 더 이전
  후보를 확인한다.
- 최신 완료 후보의 실패, current-base 불일치, 비허용 merge 형태, 허용 경로 밖 변경은 fast-pass하지 않고 full
  CI fallback한다. branch protection과 aggregate 이름은 변경하지 않는다.
- #3304의 `bcff621 → 2042ee0`를 재현 근거로 삼았다. 기존에는 마지막 비문서 SHA `a60ae32`만 조회해 full CI가
  재실행됐고, 수정 뒤에는 직전 green PR head도 candidate가 된다.
- `multi_pr_update_branch.md`는 Update branch 뒤 stale SHA의 queued/pending/in-progress run을 일반 cancel 없이
  force-cancel API로 즉시 정리하도록 명확히 했다.
- renderer, Rust source/test, fixture, golden/baseline은 변경하지 않았다. 시각 검증과 Cargo 재실행은 대상이
  아니며, workflow 자체가 바뀌므로 GitHub full CI가 구현 SHA에서 필수다.

## 구현 SHA 검증

- `git diff --check`, workflow YAML parse, 각 inline GitHub Script의 `node --check`,
  `actionlint -ignore SC2086`를 통과했다. Render Diff의 SC2086은 변경 범위 밖 기존 경고다.
- candidate mock으로 최신 문서 후보가 미완료이면 이전 green 후보를 선택하고, 최신 완료 후보 실패와
  current-base 불일치에서는 full CI로 fallback하는 것을 세 workflow에서 확인했다.
- `7fed99fd2`의 CI Build & Test, CodeQL, Render Diff가 모두 성공했다. CI heavy job과 8개 default-feature
  shard도 성공했다.
- `mydocs/**`는 세 preflight에서 파일 상태·확장자 검사보다 먼저 허용한다. 따라서 `mydocs/pr/assets`의 PDF,
  HWP/HWPX, PNG 증적도 문서-only 범위다.

## 문서 trailing commit 검증 계획

이 문서·implementation 기록·오늘할일만 포함하는 후속 commit을 push한다. 최신 head에서는 구현 SHA
`7fed99fd2`를 candidate로 재사용해 CI·CodeQL·Render Diff heavy job이 skip되고, required aggregate는
success여야 한다. base가 바뀌거나 최신 required check가 실패하면 fast-pass하지 않고 full CI 결과를 따른다.

## 최종 권고

문서 trailing commit의 최신 head CI 성공, mergeable 재확인, `edwardkim` review와 작업지시자 merge 승인을
조건으로 merge를 권고한다. merge 뒤 #3309를 close하고 `upstream/devel` 동기화와 task branch 정리를 수행한다.
