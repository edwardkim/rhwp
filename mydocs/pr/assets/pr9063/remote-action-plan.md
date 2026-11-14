# PR #9063 원격 수용과 후속 처리안

작업지시자는 후보 `fafc84f71187`의 원격 처리 전체를 승인했다. 아래는 승인한 계획의 보존 기록이다.
수용 PR #9069의 CI·self-review·merge와 원 PR/연결 이슈 자동 완료를 확인했으며, 최신 실행 결과와 댓글 계획은 archive 검토를 따른다.
최종 제출 SHA와 검증 결과는 [검토 기록](../../archives/pr_9063_review.md) 및 [실행 원장](evidence.md)을 따른다.

## 대상과 이력 보존

- 원 PR: https://github.com/edwardkim/rhwp/pull/9063
- 원 head: `b43050954c10512705d714114f5af23f1ee98ac6`, 작성자 `hyunmyungJaneLee`.
- 원 fork `kidsnote/rhwp`의 실제 `push=false`를 확인했다. 원 fork와 그 branch를 변경·삭제하지 않는다.
- 수용 경로: upstream `review/pr-9063-restricted-object-next-page` → devel 대상 Open PR.
- 원 기여 여섯 commit과 최신 devel을 부모 이력으로 보존하며 squash하지 않는다.
- 제품 보정은 `1720c96477`·`e2d7fb08b5`, 정식 회귀 보강은 `5b028b3f03`이다.

## 승인 후 실행 순서

1. 원 PR head·최신 devel·본인 열린 PR을 다시 조회한다. 다른 reviewer가 맡은 PR은 갱신하지 않는다.
   제출 후보와 대상 devel의 `git merge-tree --write-tree`가 충돌 없이 성공해야 한다.
   새 base가 생기면 유입 기능 보존과 영향받은 필수 검증을 완료한 뒤 진행한다.
2. 검증한 후보를 upstream의 위 branch에 push하고 devel 대상 **Open 수용 PR**을 만든다.
   [본문 초안](submission-pr-body.md)의 증거 URL에서 branch를 실제 제출 SHA로 치환해 영구 링크로 게시한다.
3. 해당 head의 모든 필수 CI와 정책 check를 확인하고 self-review를 남긴다.
   기존 원 PR의 성공 CI를 보정 후보의 CI로 표시하지 않는다. 실패는 원인을 조사하며,
   workflow 취소·재실행 또는 실행 정의 변경은 이번 계획만으로 자동 승인되지 않는다.
4. 실제 head·CI·mergeability·최신 base 병합 시뮬레이션을 다시 확인하고
   `--match-head-commit`으로 정확한 head를 지정해 원 기여를 보존하는 merge commit으로 통합한다.
5. merge SHA와 duration 갱신 결과를 확인한다. review·보완 계획을 archive로 옮기고
   오늘할일에 확정값을 반영한다. 이 PR에는 CI 정의 변경이 없으므로, 승인 범위에 포함된
   mydocs 운영 기록을 devel에 반영하고 로컬 devel을 최종 upstream/devel로 동기화한다.
   소스·테스트·기준값을 운영 기록 직접 push에 섞지 않는다.
6. #9063의 간접 MERGED 상태를 확인한다. 반영되지 않았다면 수용 PR·merge SHA를 근거로
   통합 수용 완료 댓글과 close를 수행한다. #8660·#9059는 해당 이월·공백 증상이 해결됐음을
   설명하고 완료 댓글·close 상태를 확인한다.
7. 실제 devel에 존재하는 증거 URL로 아래 감사 댓글을 게시한다. 원 PR/issue 상태를 API로 재확인한다.
8. 원 fork branch는 보존하고 이번 검토 전용 upstream branch·clean한 local branch/worktree만 정리한다.
   [종료 정리](../../../manual/pr_review/post_merge.md#77-branch-worktree-검토-전용-target-정리)에 따라
   사용이 끝난 root output 전체와 기본 작업공간 target/pr-review/debug를 삭제하고 부재를 확인한다.

## 원 PR 감사 댓글 초안

기여해 주셔서 감사합니다. 제한 개체의 다음 쪽 이월과 부동 앵커 앞 꼬리 공백 처리를 수용했습니다.
원 기여 commit을 보존했고, 메인테이너 검토 중 발견한 편집 후 이월 해제·호스트 높이 중복 예약·
저장된 합성 줄 원점의 재사용 경계를 공통 조판 경로에서 보완했습니다.

같은 공개 입력과 실제 편집·반복 저장본을 독립 한컴 Desktop Print와 대조했습니다.
Native/fresh WASM 원본·파생본 58쪽씩은 전쪽 raw 100.00%이며, 보호 3,542쪽의 공식값 하락은 없습니다.
최종 전체 회귀·Native 공식 검사와 새 head의 GitHub CI 결과는 수용 PR의 검토 기록에 연결합니다.

게시할 때 실제 수용 PR 번호·merge SHA·devel의 비교 이미지와 전쪽 TSV URL을 추가한다.
이 초안의 마지막 안내 문장은 게시하지 않는다.

## 연결 이슈 완료 댓글 초안

PR #9063의 원 기여와 메인테이너 보완을 수용 PR에서 통합했습니다.
#8660의 제한 개체는 일반 편집 직후와 HWP/HWPX 반복 저장·재열기에서도 다음 쪽 소유를 유지하며,
#9059의 부동 앵커 앞 꼬리 공백과 본문 간격은 독립 한컴 Desktop Print와 대조했습니다.
최종 검토 기록·실제 merge SHA·해당 이슈의 비교 이미지 URL을 첨부하고 이 해결 범위로 완료 처리합니다.

두 이슈에는 각 이슈에 해당하는 증상·입력·쪽의 증거만 선택해 게시한다.
