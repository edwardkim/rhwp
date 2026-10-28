# #8104 Studio 명령 오류·복구 계약 구현 계획

Issue: #8104 — https://github.com/edwardkim/rhwp/issues/8104

기준: `upstream/devel` `6097253468`. 사용자 지시 범위는 로컬 구현·검증이며 원격 push와 PR 생성은 별도 승인 대상이다.

## 선행 분석

- #3350의 snapshot rollback은 통합 PR #3661, `e5d3049199973b059ce3cf4e642a978d3db38f27`에 반영됐다.
  현재 `SnapshotCommand.execute`는 before 복원과 ID 해제 뒤 원 오류를 다시 던진다.
- #5769의 after 지연 저장 및 #2328의 history 예산·실패 처리 계약을 유지한다.
- #8102는 #8180으로 반영됐으며 셀 주소 보정과 이번 오류 전달 계약은 별개다.
- `CommandHistory.execute`는 실행 실패 시 기본적으로 스택을 유지하지만 부분 서식 실패는
  복원용 명령을 기록한다. 따라서 모든 예외를 복구 성공으로 분류할 수 없다.
- `InputHandler.executeOperation`의 snapshot 예외는 상위로 전파되고 명령 오류 이벤트가 없다.
  embed protocol v1은 capability 확장과 `rhwp-event`를 이미 지원한다.

## 구현

1. 버전·안정 코드·operation·문서 세션·이벤트 ID·복구 결과와 검증 항목을 정의한다.
2. snapshot rollback의 결과를 명령 소유자가 기록하고 history 경계가 전달한다.
   커서·선택·history·dirty 및 실제 화면 갱신을 확인한 경우에만 복구 성공을 통지한다.
3. 실패를 typed exception으로 유지하고 별도 구조화 이벤트를 제공한다. 해당 예외만 전역
   오류에서 중복 처리하지 않으며 일반 전역 오류·초기화 실패는 기존 경로에 남긴다.
4. 직접 임베딩 및 MessagePort capability 경로를 제공하고 문서 세대 변경·중복·미지원
   payload의 소비 규칙을 문서화한다. 호스트 앱 수정·번들 반영·배포는 이 저장소 작업과 구분한다.

## 검증

TypeScript, 전체 npm test, production build, 실제 브라우저에서 정상 편집·거부·rollback 성공·
rollback 실패·부분 변경·trap·refresh 실패·문서 교체·중복 통지·미지원 협상을 확인한다.
내용·cursor/selection·history·dirty와 후속 편집/undo를 함께 검사한다. Rust 조판·저장 속성·
출력 규칙은 변경하지 않는다. 검증 중 발견하는 필수 실패는 보정 후 재실행한다.
