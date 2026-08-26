# 구현 계획서 — Task M100-3414: 모달 종료 포커스 복원

- 이슈: https://github.com/edwardkim/rhwp/issues/3414
- 수행 계획서: `mydocs/plans/task_m100_3414.md`
- 작성일: 2026-08-13
- 브랜치: `local/task_m100_3414`

## 설계

`ModalDialog`는 editor에 직접 의존하지 않는다. 마지막 `.modal-overlay`가 사라진 시점에
문서 이벤트 `rhwp-modal-dialog-closed`를 발행하고, 애플리케이션 조립 지점인 `main.ts`가
현재 활성 `InputHandler`에만 `focus()`를 호출한다.

이 경계는 다음을 보장한다.

- UI 기반 클래스는 editor 구현을 import하지 않는다.
- 입력 handler가 아직 없거나 비활성이면 포커스를 강제하지 않는다.
- 중첩 모달의 자식 종료는 부모 오버레이가 남아 이벤트를 발행하지 않는다.
- 기존 `afterClose` 훅은 이벤트 전에 그대로 실행돼 개별 대화상자의 후처리와 호환된다.

## 수정 파일

- `rhwp-studio/src/ui/dialog.ts`
- `rhwp-studio/src/main.ts`
- `rhwp-studio/tests/issue-3414-modal-focus.test.ts` 신규

## 검증

계획서의 focused test, TypeScript, 전체 Studio test를 순차 실행한다. 브라우저 연결이 없으면
UI 스모크를 통과했다고 기록하지 않고 사유를 stage 보고서에 남긴다.
