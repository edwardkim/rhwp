# PR #7970 self-review 후 통합 순서

## 현재까지 완료

공동 이슈 #7957–#7963·#7966·#7971의 구현·범위 확장은 작업지시자가 승인했다.
승인된 원격 반영 뒤 code head `ba1de40d331769de664c90bfdcd34ca62464c716`의 Full CI·
CodeQL·Render Diff·Adapter·Proptest가 성공했고 2026-10-17 다음 절차 진행 승인을 받았다.
구체적 결과·코드 검토·예외는 [self-review](pr_7970_review.md)에 기록했다.

주요 commit은 `145446111a25fa5e2f3072ab04f68fa31218a00e`(최초 공동 처리 제출),
`121e1094e`(#7971 계획), `3b7a3662b3fbabac2e8544ce364497bd4c454b19`(JPEG 실제 예약),
`ba1de40d331769de664c90bfdcd34ca62464c716`(동일 runtime 최종 검증·test/공개 증거)다.
Native/Docker producer는3b7a3662b이고 code head의 실행 test bytes도 대조했다.

## 후행 기록과 최종 게이트

1. 정확한 code head CI 성공 뒤 archive review·이 문서·오늘할일만 single-parent trailing
   commit으로 추가한다. code/test/fixture/workflow/PNG를 다시 바꾸지 않는다.
2. latest `upstream/devel`의 오늘할일을 읽고 source에 없는 타 PR 기록을 복사하지 않는다.
   양쪽에서 바뀌지 않은 경계에 #7970만 추가한다. source를 반복 merge/rebase하지 않는다.
3. push 전 base/head를 고정해 `git merge-tree --write-tree` exit0, merge tree의 whitespace·
   변경 문서 내부 링크·base 오늘할일 및 source 기존 기록의 무삭제·무중복을 확인한다.
   원격 base/head를 직전에 다시 조회하고 변경됐으면 이 작은 게이트만 다시 수행한다.
4. 승인된 다음 절차로 trailing commit을 push하고 본문의 대표 이미지 URL을 새 PR head에
   고정한다. code candidate·runtime producer·시각 기준 SHA는 실제 검증값을 유지한다.
5. trailing head의 실제 CI preflight·trusted 재사용 candidate·required aggregate를 확인한다.
   문서만 바뀌었다는 이유로 fast-pass나 성공을 추정하지 않고, 대기 중은 대기로 보고한다.
6. 최신 exact head·MERGEABLE/CLEAN·required checks를 재조회하고 작업지시자의 병합 승인을
   받는다. 그 뒤 승인된 normal merge를 수행한다. `--admin` 우회 지시는 받지 않았다.

로컬 사전 게이트 JSON·실제 trailing CI 결과는 ignored
`output/pr-review/issue7957/extraction-buffer-20261017/self-review-20261017/`에 보존한다.
같은 runtime의 전체회귀·Native·fresh Docker WASM·370쪽 산출을 다시 실행하지 않는다.
실제 코드나 필수 통합 조건이 바뀌면 그때 영향 검증을 별도로 판단한다.

## 병합 후 후속 처리 — 아직 실행 전

병합과 후속 처리 승인 뒤 [post_merge](../../manual/pr_review/post_merge.md)에 따라 merge SHA·
devel 상태·duration 결과와 아홉 이슈의 실제 종료를 확인한다. devel 대상 PR은 default branch
자동 종료를 가정하지 않고 REST 상태를 재조회한다. 남은 B1·전체 할당 감사 등을 원래 이슈의
해결 범위로 넓히지 않는다. 공개 증거는 merge SHA URL과 한글 본문으로 게시하고 API로 확인한다.

소유·clean·최종 SHA·증거 보존·프로세스 사용 여부가 확인된 이번 전용 branch/worktree와
임시 remote branch만 승인된 후속 처리 범위에서 정리한다. 비공개 원본·필수 증거는 안정적
로컬 경로에 보존하고 main 작업공간·사용자 WIP·7700 Studio·공용 target/pr-review는 보호한다.
병합 실패 시 source를 강제로 덮어쓰거나 보호 규칙을 변경하지 않는다. 실제 문제와 해소 조건을
기록하고, devel에 들어간 코드의 rollback이 필요하면 별도 revert PR로 진행한다.
