# 승인된 원격 처리 계획 — semanticist21 2026-11-05

2026-11-05 작업지시자가 #8331·#8332의 변경 전부터 있던 동일 출력 차이에 한정한 수용 예외와 10건의 통합 push·Open PR·해당 head CI·self-review·merge·원 PR 완료/감사 댓글·동기화·정리를 승인했다. 새 회귀·검사 실패·쪽수 차이·누락·90% 미만의 새 렌더링 회귀 추가는 예외 대상이 아니다. [승인 기록](maintainer-approval.json). 아직 실행하지 않은 단계를 완료로 기록하지 않는다.

## 현재 검증 후보와 선행 조건

- branch: `review/integration-20261105-semanticist21-batch`.
- runtime source: `2f44d7d0ab19c743089ae309a8ca64eee57efae2`; 제출 HEAD는 최종 검토 기록 및 최신 devel의 문서 전용 bridge를 담은 branch HEAD로 고정해 재확인한다.
- 비교 base `22947f6a90c998e95fa51238d5b907a0ebd37162`, 최신 devel `e96b500e3ea259c821f05440f0388355916a9e9b`는 이전 PR 검토 기록 3개만 추가됐다. 제품 코드·테스트·설정은 같다.
- 메인테이너 계정의 열린 PR은 준비 시 0건이었다. 원 PR 10건은 고정한 head·Open·non-draft·reviewer를 다시 확인했다.
- 기능 8건은 전쪽 Native/fresh WASM 각각 75쪽 모두 90% 이상이다. 성능 2건은 39쪽씩 base/최종 출력이 모두 같지만 독립 Print의 기존 미달이 남아 **승인된 기존 차이 한정 예외**를 적용한다.
- 예외 제안은 그 두 성능 문서의 기존 차이만 대상으로 한다. 새 회귀·누락·쪽수 차이·검사 실패·새 저품질 golden·미달 렌더링 회귀 추가를 허용하지 않는다.

## 승인된 원격 범위

1. 선택한 예외/수정 결정을 기록하고 제출 HEAD·최신 devel·원 PR 10개 head·계정의 기존 PR을 다시 확인한다. 제품 코드가 바뀌면 영향 검증을 먼저 완료한다.
2. 원 기여 commit 10개와 메인터너 보정을 보존한 branch를 본 저장소에 push하고 devel 대상 Open 통합 PR을 만든다. 원 fork branch를 force-push하지 않는다.
3. 해당 제출 head의 필수 GitHub CI·CodeQL·Render Diff·Adapter·Proptest·정책 집계를 확인한다. 실패·미실행을 성공으로 바꾸거나 검증 기준을 완화하지 않는다. 별도 승인이 필요한 workflow 취소·재실행·설정 변경을 이 제안만으로 실행하지 않는다.
4. 실제 source/head와 최신 base의 기능·충돌 상태를 다시 확인하고 self-review를 마친다. 모두 충족했을 때만 exact head를 지정해 merge commit으로 통합한다.
5. 원 PR 10개가 간접 MERGED로 반영됐는지 확인한다. 반영되지 않은 원 PR은 통합 수용 근거와 함께 완료 처리하는 범위까지 승인받는다. 감사 댓글에는 통합 PR·검증·보정 내용을 연결하고, CI 시간 초과를 기여자 소스 결함으로 쓰지 않는다.
6. 검토 기록·archive·오늘할일의 필요한 후속 반영을 canonical 역할 절차로 마치고 devel 동기화·검토 전용 branch/worktree·output 전체·검토 debug를 정리한다. contributor fork branch와 다른 작업은 보존한다.

관련 issue와 별도 PR은 해결 범위를 확인하며, 성능 문서의 기존 조판 차이나 별도 PR #8368 등 미처리 범위를 자동 종료하지 않는다. 공개 댓글의 구체 문구는 아래 초안을 바탕으로 실제 integration PR·merge SHA·CI URL을 확정한 뒤 채운다.

## 원 PR 후속 댓글 공통 초안

> 이 PR의 기여 commit을 보존하고 최신 devel의 기존 계약과 함께 통합 PR `<INTEGRATION_PR_URL>`에서 수용했습니다. 메인터너 보정과 Native/fresh WASM·저장/편집·전체 회귀를 완료했고, 해당 제출 head의 GitHub CI와 self-review를 확인한 뒤 merge했습니다. `<PR별 보정·검증 핵심>` 상세 기록: `<FINAL_REVIEW_URL>`. 기여해 주셔서 감사합니다.

#8331·#8332의 댓글에는 승인된 경우에만 “성능 검증 문서의 변경 전부터 있던 동일 출력 차이에 한정해 메인테이너 예외를 적용했으며, 해당 기존 조판을 해결했다고 주장하지 않는다”는 사실을 추가한다.
