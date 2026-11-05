# 다음 10건의 승인된 원격 실행과 후속 절차

작업지시자의 전체 원격 처리 승인을 받았다. 통합 PR #8640을 head `4cf202d2aba44b0cfdd33465b03e37d1826a3fdc`·merge `fd598250b7d8abee39eba1b1c68e604a326faa17`로 병합했고 원 PR 10건도 자동 MERGED를 확인했다. 아래 절차와 댓글 계획에 따라 후속 기록·댓글·동기화·정리를 완료한다. [최종 기록](../../archives/pr_8640_review.md)을 참조한다.

## 대상과 보존

원 PR은 [#8368](https://github.com/edwardkim/rhwp/pull/8368)·[#8369](https://github.com/edwardkim/rhwp/pull/8369)·[#8385](https://github.com/edwardkim/rhwp/pull/8385)·[#8484](https://github.com/edwardkim/rhwp/pull/8484)·[#8488](https://github.com/edwardkim/rhwp/pull/8488)·[#8490](https://github.com/edwardkim/rhwp/pull/8490)·[#8492](https://github.com/edwardkim/rhwp/pull/8492)·[#8494](https://github.com/edwardkim/rhwp/pull/8494)·[#8500](https://github.com/edwardkim/rhwp/pull/8500)·[#8504](https://github.com/edwardkim/rhwp/pull/8504)다.

원 head 10개를 merge 조상으로 보존했다. 대상 원격 branch는 본 저장소의 `review/integration-20261105-semanticist21-next10`, PR base는 `devel`이다. contributor fork branch에는 push하거나 삭제하지 않는다. 제품 source와 시각 producer, 새 base의 기능 보존·전체 회귀는 [최종 검증](validation.md)에서 구분한다.

## 승인 후 실행 순서

1. 원 PR의 head·상태와 본인 계정의 열린 PR/지정 reviewer·실패 CI를 다시 조회한다. 최신 upstream/devel과 충돌·추가 기능 보존을 확인한다. 후보가 달라지면 영향 검증을 완료한 뒤 제출한다.
2. 승인한 후보를 본 저장소 통합 branch에 push하고 [PR 본문](pr-body.md)으로 Open PR을 만든다. 로컬 미완료를 Draft나 CI로 넘기지 않는다.
3. 해당 head의 실제 필수 CI·CodeQL·Render Diff·Adapter·Proptest·정책 감사 등 required 결과와 실제 로그를 확인한다. 실패는 원인을 판단하며, 취소/재실행·workflow 변경은 현재 규칙의 별도 승인 범위를 따른다.
4. 같은 head로 self-review를 기록한다. source가 그대로이고 CI가 성공한 뒤에는 이미 완료한 로컬 전체 검증을 반복하지 않는다. 실제 변경이 생긴 경우에만 영향 검사와 새 head CI를 갱신한다.
5. merge 직전 head·checks·최신 base/mergeability를 재조회하고 `git merge-tree --write-tree`를 확인한다. `--match-head-commit`과 merge commit으로 통합해 원 기여 SHA를 유지한다.
6. merge SHA·원 PR 10개의 완료 상태를 확인한다. GitHub가 자동 MERGED로 반영하지 않은 원 PR은 승인된 통합 수용 범위에서 완료 처리하고, 실제 상태를 기록한다. 닫힌 PR을 직접 merge한 것으로 바꿔 부르지 않는다.
7. 개정된 [후속 절차](../../../manual/pr_review/post_merge.md)에 따라 archive review·오늘할일·확정 SHA·후속 기록을 반영하고 devel을 fast-forward한다. 현재 후보 diff에는 CI/workflow/판정 정책 변경이 없어 옵션 2 의무 적용 대상은 아니다. 최종 diff가 달라지면 다시 분류한다. 기록 범위를 넘으면 별도 기록 PR 경로를 따른다.
8. 원 PR별 검토 문서의 `Merge 후 contributor PR comment 계획`과 아래 초안을 대조한다. merge SHA에 실제 들어간 이미지의 raw 응답 형식·SHA-256을 확인한 뒤 한국어 존댓말로 게시한다. 게시 뒤 API 본문·이미지 표시를 확인한다. 같은 댓글을 중복 게시하지 않는다.
9. 관련 issue의 귀속·close 상태와 후속 comment 필요성을 확인한다. 이 후보와 무관한 issue를 자동으로 닫지 않는다. merge 뒤 제품 CI를 새로 시작하지 않고 duration refresh의 실제 결과 또는 증거 부족을 확인한다.
10. 실제 빌드·검증 사용 종료 후 이번 소유의 clean branch/worktree·임시 원격 통합 branch를 안전 조건에 따라 정리한다. 기본/다른 작업공간과 contributor fork는 보존한다. 저장소 루트 `output/` 전체와 기본 작업공간 `target/pr-review/debug`를 추가 백업 없이 자동 삭제하고 잔여를 확인한다.

## 원 PR별 댓글 초안

각 초안은 아직 게시하지 않았다. `<MERGE_PR_NUMBER>`·`<MERGE_SHA>`·`<CI_RUN_ID>`은 승인된 실제 통합 뒤의 확정값으로 바꾼다. 이미지 형식·해시와 입력/쪽 대응이 없으면 게시를 보류한다. 각 초안의 경로는 `comments/pr-<원 번호>.md`다.

최종 현재-base 확인: a95d1d5afa와의 [실제 병합 호환](current-base-compatibility.md)을 완료했다. strict=false 정책 아래 검증한 code head를 유지하며, push/merge 직전의 계산 tree가 실제 검증한 제품 subtree와 같은지 다시 확인한다. CI/workflow 변경은 이번 PR의 고유 diff에 없다.
