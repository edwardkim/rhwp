# PR #8379 메인터너 검토·최소 보정 계획

## 범위와 기준

원 기여 `34b51b70098230bfc568a7c566ab73057ad6a140`의 연속 형광펜 모드를 보존한다.
최신 base `de2295d430b588f0741021f66fae086eb4ff4009`와 충돌 없이 병합한 로컬 후보는
`503e53e9463d242ee26b5aedf4bda251538cbe49`다. 검토 branch는
`review/pr-8379-continuous-highlighter`다. 계획 작성 당시에는 원격 head를 변경하지 않았다.

기본 경로는 `maintainer_general`, 보조 경로는 `intake_and_review`, `local_validation`,
`first_time_contributor`다. 기존 #4162 → #4271 → merge `d4834d7375`의 셀 블록 즉시
서식 적용 계약을 확인했다. 과거 회귀의 기대값이나 Rust·WASM 구현은 변경하지 않는다.

## 재현과 보정

실제 Studio의 새 문서·표 만들기·실키 F5·팔레트 클릭에서 셀 글자의 배경이
`#ffffff`로 남고 history가 증가하지 않았다. 선택 길이가 0인 본문 range에서도 모드 진입이
실패했다. 일반 텍스트의 anchor 존재만 확인하는 새 `highlight-pick` 분기가 원인이다.

1. 본문과 머리말·꼬리말은 기존 공통 non-empty selection helper로 실제 선택을 판정한다.
2. F5 셀 블록도 기존 `applyCharFormat` 즉시 적용 경로로 전달한다. 각주의 기존 지원 경계는
   유지한다.
3. 원 E2E에 실제 F5 셀 블록·축소된 빈 선택의 회귀를 추가한다. 실키와 팔레트 클릭, 실제
   WASM 속성·history·undo 결과를 검사한다. 원 source에서 실패하고 보정 뒤 통과하는지 확인한다.
4. TypeScript, 전체 Studio 검사, build, 기존 #6635와 새 연속 형광펜 E2E를 실행한다.
   최적화 fresh WASM의 실제 초기 로딩 응답을 빌드 해시와 대조한다.

문서 전환 뒤 모드 유지와 양식 모드의 표시 상태는 원 구현의 관찰 사항으로 기록한다.
이 보정에서 새로운 종료 정책이나 각주 서식 지원을 추가하지 않는다.

## 원격 처리 경계

로컬 검토·보정·필수 검증 뒤 원 기여와 보정 commit, 실제 결과, 게시 문안을 제시한다.
push·PR 본문 변경·GitHub review·merge·댓글은 승인된 원격 범위에서만 수행한다.
원 PR을 정상 merge해 첫 기여 이력을 보존하는 경로를 우선 검토하며 contributor fork는
삭제하지 않는다. 검증된 후보와 최신 devel의 tree·CI를 merge 전에 다시 확인한다.

## 완료 결과

최소 보정 source `e6439bd3ca7e90bd3894ac7f8524df41c6553c93`를 원 기여 branch에 push했다. 새 head의 CI·정책·self-review를
통과한 뒤 원 PR을 `ba9a3c5f222a151736a4a21f9743bf8d9ddfbd8a`로 정상 merge했다. source·test의 추가 변경은 없으며
[최종 검토](pr_8379_review.md)에 실제 검증·원격 수행과 후속 정리 경로를 기록했다.
