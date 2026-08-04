# PR #3299 검토 기록

## 라우팅

```text
base route: collaborator_self_merge
modifiers: intake_and_review, local_validation, review_only_fast_pass, rework_and_exceptions
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_self_merge.md, intake_and_review.md, local_validation.md,
  review_only_fast_pass.md, rework_and_exceptions.md
current head: 작성 시점 참고값 cfcc6c501ae8730d57d63ac1fd2f57c3e7b886e9
```

## PR metadata

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#3299](https://github.com/edwardkim/rhwp/pull/3299) |
| 제목 | docs: PR 검토 워크플로를 조건별 가이드로 분리 |
| 작성자 | `jangster77` |
| base / head | `devel` / `task_m100_3296_pr_review_routing` |
| 관련 이슈 | [#3296](https://github.com/edwardkim/rhwp/issues/3296) |
| 규모 | 19 files, +1,274/-1,435, 3 commits |
| 상태 | `MERGEABLE`; required check와 review 대기로 `BLOCKED` |
| reviewer | `edwardkim` 요청 완료 |

위 상태와 head SHA는 작성 시점 참고값이다. review 기록 commit이 추가되므로 merge 전 최신 head,
mergeable, required check를 다시 확인한다.

## 변경 범위

- `mydocs/manual/pr_review_workflow.md`는 공통 계약·조건별 라우팅·CI 병렬 구조·기존 절 번호 대응만 남긴
  canonical 진입 문서로 축소했다.
- `mydocs/manual/pr_review/`에는 접수, 역할별 처리, 로컬 검증, 시각·fixture 증적, 다수 PR, review-only
  fast-pass, merge 후속 처리, 재작업·예외를 조건별 자식 문서로 분리했다.
- `AGENTS.md`, `CLAUDE.md`, `mydocs/README.md`, `mydocs/manual/README.md`와 관련 문서 지도에서 모 문서와
  선택표를 먼저 읽고 필요한 자식 문서를 적용하도록 연결했다.
- 실제 CI 구현과 대조해 Lint/Frontend, archive/Native Skia, 8개 test shard의 병렬 의존성을 기록했다.
  같은 checkout·target·Cargo cache의 로컬 Cargo와 wasm-pack은 계속 순차 실행한다.
- contributor PR head 보정의 명시적 branch 전환·SHA 확인, fast-pass A/B 구분, merge 후 remote branch
  소유권·worktree·검토 전용 target 정리를 보강했다.
- maintainer review 제안에 따라 배정 불가 runner의 좀비 `queued` run과 concurrency 연쇄 고착, close/reopen
  대신 head push의 `synchronize` 이벤트로 merge ref를 재계산하는 복구 조건을 다수 PR 가이드에 추가했다.
- devel의 신규 테스트 규약이 열린 PR 여러 건에 일괄 파급될 때의 메인터너 보정·기여자 안내·PR별 재검증
  절차와 신규 CLI 통합 테스트의 런타임 우선 `rhwp_bin()` 규약을 명시했다.

source, test, CI workflow, Cargo.lock, golden/baseline, sample 변경은 없다.

## 대형 문서 PR 판정

줄 수는 1,000줄을 넘지만 기존 단일 매뉴얼의 내용을 조건별 파일로 이동한 정보구조 변경이다. 작업지시자의
심층 검토에서 누락된 branch switch, cleanup 잔여 검사, 전체 로컬 gate, legacy 절 번호와 CI fast-pass
조건을 찾아 보정했다. 변경은 문서와 bootstrap 링크에 한정되며, rollback 단위도 이 PR 하나이므로 별도
`pr_3299_review_impl.md`는 작성하지 않는다.

## 검증

- `git diff --check`: 통과
- `python3 scripts/check_document_metadata.py`: 398개 문서, 이상 없음
- `python3 scripts/check_markdown_links.py --changed-from upstream/devel --forbid-redirect-references`:
  405개 문서, 변경 20개, redirect stub 31개, 내부 상대 링크 이상 없음
- 모 문서와 `pr_review/README.md`의 자식 문서 라우팅 완전성: 누락 0건
- 기존 단일 매뉴얼과 분리 문서의 절 번호·주요 명령 대조: 필요한 명령과 2.3, 2.4, 4.1.1, 4.3.1,
  7.7.1, 7.8, 8.2.1, 9.3.2, 10.1, 11.1–11.3, 13 보존 확인
- maintainer review comment의 보강 후보 A/B/C를 각각 `multi_pr_update_branch.md`,
  `maintainer_general.md`, `local_validation.md`에 반영
- 문서-only 변경이므로 Cargo, frontend, 시각 검증은 로컬에서 생략했다.

## CI와 fast-pass 판정

내용은 문서 변경이지만 PR diff에 fast-pass 허용 범위인 `mydocs/**` 외의 `AGENTS.md`와 `CLAUDE.md`가
포함된다. 따라서 `all-review-only-no-code-impact` B 경로로 단정하지 않고 full CI fallback 결과를 기다린다.
작성 시점에는 CI preflight와 CodeQL preflight가 성공했고 나머지 required check는 진행 중이다.

## 위험과 후속 확인

- 에이전트가 모 문서만 읽고 세부 절차를 추정하면 누락이 재발할 수 있다. bootstrap과 모 문서 모두
  `base route`, `modifiers`, `loaded documents` 기록을 요구한다.
- 공통 규칙과 자식 문서의 중복을 다시 만들면 drift가 생긴다. 새 규칙은 공통 여부에 따라 한 위치에만
  추가하고 라우팅 표·문서 지도를 함께 갱신해야 한다.
- 과거 archive 문서의 절 번호는 이관만을 이유로 수정하지 않고 모 문서의 대응표로 해석한다.

## 최종 권고

최신 PR head의 full GitHub Actions 성공, `edwardkim` review, mergeable 재확인과 작업지시자 merge 승인을
조건으로 merge를 권고한다. merge 뒤에는 #3296 close 상태 확인, `devel` 동기화와 작업 branch·검토 전용
target 정리를 수행한다.
