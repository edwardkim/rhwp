# PR #6813 실행 계획

## 범위와 기록

[Self-review](pr_6813_review.md), [22단계 최종 보고서](../../working/task_m100_6662_stage22_pr_readiness.md)를 따른다.
코드 후보 `6dcd46608`, 증적 `2b75a052c`, 링크 보정 및 최초 PR head `964ef10a6`이다.

- #6712/#6708/#6714를 완료 대상으로 제출하고 #6699/#6662는 Ref로 유지한다.
- 작업지시자가 지정한 #6712까지의 범위를 유지하고 새로운 이슈/외부 PR을 추가하지 않는다.
- 23개 기존 일반 commit은 분석/수정/검증 checkpoint이며 임의 amend 또는 squash로 재작성하지 않는다.

## 단계

1. 완료: 로컬 전체 검증, Native Skia, WASM/browser 및 대표 시각 증적을 고정했다.
2. 완료: upstream/devel을 fetch하고 충돌 없는 merge tree와 diff check를 확인했다.
   source head는 리베이스하지 않았다. `964ef10a6`을 upstream 작업 branch에 push해 Open PR #6813을 생성했다.
3. 완료: 초기 exact PR head `964ef10a6`의 CI, CodeQL, Render Diff, Adapter, Proptest가 모두 성공했다.
   GitHub merge commit `c0687162210c4277d44a506bdff325ec3e548861`을 검증 기준으로 고정했다.
4. 로컬 완료, push 진행: 문서 commit `d7f165819` 뒤 exact current base bridge `359f4d3ea`에서
   오늘할일 add/add 충돌만 양쪽 기록을 보존해 해소했다. verifier는 mydocs-only, exit 0이었다.
   초기 검증된 GitHub merge commit 대비 코드 diff 0, 최종 tree의 세 문서 링크/diff check도 통과했다.
5. 대기: trailing head CI와 mergeability를 다시 확인한 뒤 결과를 보고한다. 이번 지시를 admin merge나
   issue 선행 close 승인으로 확대하지 않는다.
6. 별도 merge 승인 뒤: `post_merge.md`를 다시 읽고 exact head를 merge한다. merge SHA와 devel 포함을
   확인하고 self-review의 이미지 코멘트 계획을 이슈/PR에 적용한다. #6699/#6662는 닫지 않는다.
7. 해당 merge/후속 처리 승인 범위에서 전용 branch/worktree만 정리한다. 기본 작업공간,
   `target/pr-review`, 사용자 변경, contributor fork는 제거하지 않는다.

## 복구 경계

merge 전 회귀가 발견되면 최신 성공 head와 실패 commit 범위를 분리해 보정한다. 임의 force push,
CI 우회, baseline 완화 또는 golden 변경으로 통과시키지 않는다. merge 뒤 회귀라면 이번 PR diff를 근거로
일반 후속 PR 또는 승인된 revert를 준비하며 devel에 직접 push하지 않는다.
