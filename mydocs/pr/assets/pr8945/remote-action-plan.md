# PR #8945 원격 처리안 — 승인 후 실행

원 head `06f47c8a518f542a80d38e5a398ed73fed7dcc64`, 최신 검증 base
`99b9d7ec5ca36884ea8fd981ac815c1edc34c653`. 추가 기여자 commit이나 제품 보정 없이 원 PR을 처리한다.

## 필요한 결정

현재 base와 후보의 보호 문서 PNG 3,542쌍은 모두 같다. 목표 8쪽은 Native/WASM 각각
100.00%, 최소 글꼴 집중 14건·필수 HWPX Native 편람 383쪽·원 head CI가 통과했다.
다만 과거 정본 공식 값보다 낮은 911행과 기존 글꼴·간격·괘선 차이가 있다.
[전쪽 표와 직접 비교](protection/README.md)를 근거로 **이번 PR에서만 이 기존 차이를 수용**할지
결정한다. 보호 TSV·90% 기준·branch protection은 변경하지 않는다. 새 회귀·새 결함·미검증은
이 예외 대상이 아니다. 승인이 없으면 merge를 보류한다.

## 승인 뒤 실행 순서

1. 원 head·CI 38개·fresh devel과 개별 merge-tree를 다시 확인한다. head 변경이나 새 회귀가 있으면
   멈추고 실제 변경을 검토한다. 성공한 CI를 이유 없이 재실행하지 않는다.
2. [승인 리뷰 초안](approval-review-draft.md)을 exact head에 게시한 뒤 원 PR을 merge commit으로
   병합한다. merge SHA와 병합 후 duration 결과의 성공 또는 증거 부족을 확인한다.
   제품 CI·CodeQL을 새로 실행하지 않는다. fork에 보정 push하거나 새 통합 PR을 만들 필요가 없다.
   fork branch는 삭제하지 않는다.
3. merge SHA·결정과 최종 검토를 archive review·asset·오늘할일에 보존한다. maintainer 직접 반영 범위인
   `mydocs/pr/archives/`·`mydocs/pr/assets/`·`mydocs/orders/`만 한 운영 기록 commit으로 devel에 push한다.
   실제 검증한 로컬 source와 원 PR/최종 merge의 runtime 입력 동등성도 연결한다.
4. 기본 작업공간 devel을 fast-forward한다. asset의 원격 존재·해시를 확인한다.
5. #7620에는 부분 개선 결과를 설명하고 **OPEN을 유지**한다. 원 PR에 결과·감사 댓글을 게시하고 API로 확인한다.
6. 이번 검토 전용 worktree·branch·ref와 루트 `output/` 전체·사용 종료한 `target/pr-review/debug`를
   정본 순서로 정리한다. 다른 WIP worktree·기본 작업공간·contributor fork branch는 보존한다.

일반 역할별 후속 절차와 실제 실행 순서는 `mydocs/manual/pr_review/post_merge.md`를 따른다.
2026-11-11 작업지시자가 직전 예외·원격 처리 요청에 “PR 승인, 머지, 후속 처리까지 진행하세요”라고 지시했다. 원 head의 최신 CI·fresh base·merge-tree 재확인을 완료하고 실행한다.
