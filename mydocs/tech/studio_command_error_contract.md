---
kind: canonical
status: active
canonical: mydocs/tech/studio_command_error_contract.md
last_verified: 2026-10-24
---

# Studio 명령 오류 계약 v1

Issue: [#8104](https://github.com/edwardkim/rhwp/issues/8104).
호스트 구현 트랙은 [alhangeul-macos #594](https://github.com/postmelee/alhangeul-macos/issues/594)다.

## 지원 범위와 발견

`command-error-v1`은 Studio `InputHandler.executeOperation` 및 undo/redo의 실패 계약이다.
동기 명령 호출은 `StudioCommandError`를 throw한다. `error.report`는 **실패 보고**를 반환하는
Promise이며 성공한 명령 응답이 아니다. dispatcher도 기존처럼 `ok: false`를 반환한다.
상태 복원·실제 visible page render·선택 갱신을 마친 뒤 보고한다. 갱신이 10초 안에 완료되지
않으면 `unknown`이다. 동일 실패의 반복 throw는 같은 보고와 이벤트 ID를 사용한다.

- 독립 Studio/WKWebView: `window.rhwpCommandErrors.capability`를 확인하고
  `getState()`로 `documentSessionId`와 `ready/recovering/blocked`를 조회한다.
  `null`은 아직 초기화되지 않았음을 뜻한다. `rhwp-command-error` CustomEvent의 `detail`이 payload다.
- MessagePort: 기존 embed protocol v1 connect에서 `command-error-v1`을 요청한다.
  connected의 capabilities에도 있어야 지원된다. `getCommandErrorState` RPC로 현재 문서 세션을
  얻는다. `rhwp-event`의 `event: "commandError"`, `payload`로 보고한다. 해당 RPC가 typed 실패를
  전파하면 `rhwp-response.error.commandError`에도 같은 payload를 싣고 `result`는 싣지 않는다.
- 협상하지 않은 연결에는 오류 이벤트와 확장 error payload를 보내지 않는다.
  `getCommandErrorState` 호출도 `UNSUPPORTED_CAPABILITY`로 거절한다.
- 이 변경은 `npm/editor` SDK의 새 공개 메서드나 호스트 앱 번들 배포를 포함하지 않는다.
  직접 EventTarget 또는 MessagePort를 소비하는 호스트가 위 계약을 사용한다.

## Payload

```json
{
  "schemaVersion": 1,
  "capability": "command-error-v1",
  "eventId": "<instance UUID>:<document generation>:<sequence>",
  "documentSessionId": "<instance UUID>:<document generation>",
  "operation": "pasteInternal",
  "code": "COMMAND_ROLLED_BACK",
  "recovery": "succeeded",
  "checks": {
    "content": true,
    "editing": true,
    "history": true,
    "dirty": true,
    "ready": true
  }
}
```

ID는 opaque 문자열이다. 호스트가 분해하거나 순서를 추정하지 않는다. 문서 이름·오류 문자열·
번들 파일명·줄 번호를 판정에 사용하지 않는다. payload는 문서 내용과 원 오류 메시지를 포함하지
않는다. 직접 호출자의 `error.cause`는 로컬 진단용 원 오류를 보존한다.

| code | recovery | 의미 |
| --- | --- | --- |
| `COMMAND_REJECTED` | `succeeded` | 소유자가 mutation 전 거부를 확인했고 모든 복구 검사를 통과 |
| `COMMAND_ROLLED_BACK` | `succeeded` | snapshot 복원·정리와 모든 복구 검사를 통과 |
| `ROLLBACK_FAILED` | `failed` | snapshot rollback 또는 정리가 완료되지 않음 |
| `INTEGRITY_UNKNOWN` | `unknown` | 부분 변경, 미지원 복구 경로, trap, 편집 상태·history·dirty·화면 검증 실패 등 |

`checks.content`는 검증된 mutation 전 거부 또는 before snapshot 복원 계약에 근거한다.
`editing`은 본문·셀·머리말/꼬리말·각주 cursor/selection, 객체·셀 블록 선택, 선택 확장 단계와
예약 글자 모양·누름틀 경계 상태의 복원이다. geometry는 저장 값을 재사용하지 않고 다시 조회한다.
`history`는 undo와 redo의 엔트리 정체성·순서를 확인한다. `dirty`는 진입 시 값과 같음을 확인한다.
`ready`는 위 상태와 실제 화면 render, 문서·커서 질의가 성공했음을 뜻한다. 임의의 모든 미래
명령이 성공한다는 보장이 아니다. 회귀 검사는 뒤이은 편집과 undo도 실제로 실행한다.

history를 이미 변경한 뒤 난 오류, 일반 역연산 명령의 실패, IME 조합·지연 pagination 실행 중
실패는 v1에서 성공으로 인증하지 않는다. rollback이 성공해도 원 오류가 WASM RuntimeError이면
`unknown`이다. 알 수 없는 부분 변경을 숨기거나 이전 dirty 값으로 덮어쓰지 않는다.

## 전역 오류와 중복·오래된 세션

Studio는 소유한 `StudioCommandError`만 `error/unhandledrejection`에서 `preventDefault`한다.
명령의 실패 신호는 typed exception/실패 응답과 구조화 이벤트에 남는다. 일반 오류, 초기화·로드·
리소스 실패와 이 경계를 거치지 않는 오류는 기존 전역 경로를 유지한다. 개발 서버의 별도 진단
리스너는 여전히 오류를 기록할 수 있다.

호스트의 전역 오류 리스너는 `window.rhwpCommandErrors?.isHandledError(error)`가 true이면
구조화 보고를 기다린다. 이 함수는 payload 모양이 아니라 Studio가 실제 만든 오류 객체를 확인한다.
`preventDefault`가 다른 리스너의 실행을 중단하지는 않으므로, 호스트도 이 소유권 검사를 적용해야
전역 오류를 먼저 치명적으로 처리하는 경합을 막을 수 있다. 그 외 오류에 이 예외를 적용하지 않는다.

1. 현재 문서의 세션 ID를 조회한 뒤 통지를 처리한다. 문서를 교체하면 다시 조회한다.
   MessagePort 연결의 `sessionId`와 문서의 `documentSessionId`는 다른 식별자다.
2. schema/capability, 필수 필드와 code/recovery/checks의 일관성을 검사한다.
   `window.rhwpCommandErrors.inspect(payload, currentSessionId, seenIds)`가 이 검사와 중복 검사를
   제공한다. 다른 호스트는 `src/core/command-error.ts`의 검사와 같은 규칙을 구현할 수 있다.
3. `stale`는 현재 세션에 적용하지 않는다. 이전 문서의 늦은 완료는 새 문서 상태를 복원하거나
   차단하지 않는다. `duplicate`도 다시 알리거나 세션을 폐기하지 않는다.
4. 미지원 버전·계약 누락·잘못된 payload는 성공 증거가 아니다. 기존 보수적 보호 정책을 유지한다.
   `seenIds`는 문서 교체 시 비우며, 오래된 ID는 새 세션에 포함하지 않는다.
5. 정확히 검증된 `succeeded`만 비차단 알림으로 소비한다. 나머지는 호스트가 편집·저장을 차단하고
   사전에 확보한 유효한 복구본과 재시작 UX를 제공한다. 무결성 불명인 현재 모델의 export를
   안전한 복구본이라고 부르지 않는다.

Studio의 공통 명령 경계는 `recovering/blocked` 동안 추가 실행을 거절한다. 이것은 raw WASM·
플러그인·별도 저장 API까지 차단하는 호스트 보안 경계가 아니다. 호스트는 자체 편집·저장 진입점을
모두 보호해야 한다. 기존 document-agent `recovered` boolean은 이 v1의 다섯 검사와 동등하지 않다.
초기화 실패·WebContent 프로세스 종료에는 이 명령 계약의 복구 성공을 추정하지 않는다.

## 검증과 배포 구분

`rhwp-studio/tests/command-error-contract.test.ts`는 snapshot/history와 payload·transport를 실행한다.
`rhwp-studio/e2e/command-error-contract.test.mjs`는 실제 WASM/Chrome에서 문서 바이트·선택·history·
dirty·후속 편집, rollback 실패·부분 변경·trap·render 실패, 문서 교체 중 늦은 완료, 실제 window
error 경계 및 MessagePort 호스트를 검증한다. 실행 결과는
[#8104 결과보고](../report/task_m100_8104_report.md)에 기록한다.

이 저장소의 구현·브라우저 검증 완료는 실제 WKWebView 호스트의 소비 구현, 호스트 번들 반영,
서명된 앱 배포 완료와 다르다. 그 세 항목은 #594에서 별도로 확인한다.
