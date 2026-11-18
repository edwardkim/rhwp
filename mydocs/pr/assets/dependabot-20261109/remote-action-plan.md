# Dependabot 19건 원격 처리안

이 문서는 원격 승인 요청 시점의 계획이다. 당시 원격 push·새 PR·공개 댓글·merge는 수행하지 않았다. 원 PR #8811–#8829의 원 head와
기여 이력을 보존한 로컬 branch `review/integration-20261109-dependabot-updates`를 대상으로 한다.
제출할 정확한 SHA는 로컬 검토·증거 commit 확정 뒤 승인 질문과 PR 본문에 기록한다.

## 승인 요청 범위

1. 원 head 19개·현재 devel·검증 source의 동일성과 본인 계정의 열린 PR을 재확인한다.
2. upstream의 동명 통합 branch에 push하고 devel 대상 Open PR 한 건을 생성한다. 원 PR 19건을
   별도로 갱신·재실행하지 않는다. CodeQL init/analyze 동시 적용, 버전 주석·서드파티 고지와
   기존 확장 smoke의 글꼴 선택 대기 순서 보정을 포함한다.
3. 새 통합 PR의 정확한 head에서 GitHub 필수 CI·CodeQL·Render Diff·Adapter·Proptest·최종
   정책 결과를 확인한다. 원 head의 성공을 새 결합 source의 CI 성공으로 표시하지 않는다.
4. self-review를 공개하고 최신 base의 merge-tree·mergeability·required check를 재확인한 뒤
   정확한 head를 고정한 merge commit으로 통합한다. 원 head 19개의 조상 관계를 확인한다.
5. CI workflow 변경이 포함되어 **옵션 2**를 적용한다. 최신 devel의 별도 후속 기록 branch에서
   실제 head CI·self-review·merge SHA·duration 결과, archive review와 오늘할일을 준비한다.
   후속 기록 PR의 실제 preflight·required check·mergeability를 확인한 뒤 병합한다. 원 코드 PR에
   문서 trailing push를 추가해 Full CI를 반복하지 않는다.
6. 기본 devel을 fast-forward하고 원 PR 19건의 MERGED 상태를 확인한다. 간접 MERGED가 표시되지
   않은 경우에만 이력을 보존한 통합 수용 근거를 기록해 완료 처리한다. closing issue는 19건 모두 없다.
7. [개별 댓글 초안](comment-drafts)에 실제 통합 PR·merge·증거 commit·CI URL을 채우고 공개한다.
   이미 게시된 동일 댓글은 중복하지 않으며 API로 UTF-8 본문·링크·이미지를 재확인한다.
8. 이번에 만든 통합 branch·worktree·보존 ref와 임시 remote head를 안전 조건에 따라 정리한다.
   사용이 끝난 기본 작업공간의 `target/pr-review/debug`와 루트 `output/` 전체를 삭제하고 잔여와
   확보 용량을 확인한다. 기본 Studio JS/WASM·사용자 Vite·다른 #8296 worktree·설치 도구·
   `local/crates-io-consumption`은 보존한다.

CI 재실행·취소·탐지 설정·보안 경고 분류 변경은 이 안의 승인 범위와 구분한다. 실제 새로운
실패가 발생하면 원인과 이미 승인된 범위를 대조한다.

## 로컬 완료 근거

- [공통 원장](README.md), [19건 개별 판정](../../archives/dependabot_20261109_review.md).
- 전체 회귀 13,005건·Native 공식 4,077건·Studio 2,778건, 모든 적용 lint·build와 실제 브라우저 통과.
- Native/fresh WASM 각 9쪽 raw 최저 100.00%, base/후보 PNG 동일. 편람 Native 383쪽 raw 최저 99.96%.
- 원 CI 14건 성공. 나머지 5건은 CodeQL 버전 불일치 또는 Native의 30분 제한으로 실제 로그·annotation을 확인했다.
- 검토 코드의 실제 변경은 의존성·Action ref·CodeQL 주석, 배포 고지와 기존 smoke 대기 순서다.
  증거를 포함해 1,000줄을 넘으므로 코드 검토·simulation·전쪽 시각 검증을 완료한 뒤 작업지시자의
  원격 범위 판단과 새 head CI·self-review를 거쳐 통합한다.

실제 통합·CI·self-review·원 PR 완료 상태는 [원격 결과](remote-completion.md)를 따른다.
