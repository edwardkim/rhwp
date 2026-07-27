# PR #3423 검토 기록 — 변경 범위별 검증 게이트 발견성

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3423](https://github.com/edwardkim/rhwp/pull/3423) — `Task #3422: [docs] 부트로더에 변경 범위별 검증 게이트 포인터 추가` |
| 작성자·검토자 | `@lpaiu-cs` (external contributor) · `@jangster77` (collaborator) |
| base / source head | `devel` / `66c1d9ec6dbd0b231785e008f6688e9469d2551a` (`docs/validation-gate-pointer`) |
| 원 변경 규모 | 2 files, +4 / -1, 2 commits(문서 1 + devel merge 1) |
| 통합 검토 | `review/lpaiu-cs-20260727`; `upstream/devel` `7779e737ac5c5df3428d1a06f1099be16375be49` 기준 |
| 원 변경 적용 | `4d814823c8dec55c65937279e25c7afdad6c1573`→`dcc0e212cb2ba12b3937a39cf2b17dc0e927e1f9`; devel merge 제외 |
| collaborator 보정 | `0b58a0d4497d2154b37e797ce49b8eca79357fd2` 중 CLAUDE anchor 정합 |
| 관련 이슈 | [#3422](https://github.com/edwardkim/rhwp/issues/3422); 통합 PR에서 `Closes #3422` |
| 작성 시점 source 상태 | `MERGEABLE` / `BLOCKED`, draft 아님; stale-run reaper만 `SKIPPED`, Build & Test 없음 |
| 라우팅 | base: `collaborator_external_pr`; modifiers: `intake_and_review`, `local_validation`, `multi_pr_update_branch`, `review_only_fast_pass` |

Loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`collaborator_external_pr.md`, `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`, `review_only_fast_pass.md`.

## 변경 범위와 판정

원 PR은 `CLAUDE.md`와 `AGENTS.md`의 부트스트랩 목록에서
`mydocs/manual/pr_review/local_validation.md` 4.3을 직접 가리켜, parser·renderer·studio·문서 범위별
검증 표를 신규 기여자와 에이전트가 놓치지 않게 한다. 명령을 부트로더에 복제하지 않고 canonical
자식 문서로 라우팅하는 방향은 현재 문서 구조와 일치한다.

원 `CLAUDE.md` 링크는 파일까지만 가리켜 4.3 표를 직접 찾는다는 PR 목적을 완전히 고정하지 못했다.
`0b58a0d44`에서 링크를
`mydocs/manual/pr_review/local_validation.md#43-변경-범위별-기본-검증`으로 보정했다. 실제 파일과
anchor가 존재함을 확인했으며 다른 매뉴얼 내용은 바꾸지 않았다.

## 검증과 CI 판정

- `git diff --check`: 통과.
- `CLAUDE.md`의 exact anchor와 대상 파일 존재: 확인.
- 문서 전용 원 PR이므로 Cargo·renderer·browser·visual sweep은 대상이 아니다.
- 새 fixture·baseline·golden 변경이 없어 IR field sweep baseline trigger도 없다.
- source head에는 Build & Test가 없고 `cancel-stale-runs` 한 건만 `SKIPPED`다. 이를 녹색 CI로 간주하지
  않는다.

이 변경만 독립 PR이었다면 review-only fast-pass의 최신 aggregate를 확인해야 한다. 현재는 source·test·CI
action을 포함하는 다수 PR 통합 후보에 포함되므로 통합 PR 전체가 full CI fallback이며, 그 최신 결과가
이 문서 변경도 함께 검증한다.

## Risk와 최종 권고

링크가 구체적인 절을 가리키고 부트로더에 명령을 중복하지 않아 정보구조 회귀가 없다. **보정 후 기술적으로
수용 가능**하다. 통합 PR 본문에는 `Closes #3422`를 사용한다. 최종 merge 조건은 최신 통합 head의 full CI,
mergeable 상태와 작업지시자 승인이다.
