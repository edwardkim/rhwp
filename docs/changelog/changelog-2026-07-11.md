# 변경 기록 — 2026-07-11

## Issue #2186: iframe MessageChannel v1

- `@rhwp/editor`가 exact Studio origin으로 v1 `MessageChannel` version/capability를 협상하고 일반 10초·load/export 60초 timeout을 관리한다.
- load/export binary는 transferable로 전달하되 caller 입력 buffer는 복사해 detach하지 않는다.
- Studio protocol guard, RPC router, runtime을 분리해 version/session과 최초 parent source/origin/port binding을 검증한다.
- 협상·RPC 오류는 구조화된 v1 envelope로 전달하고 malformed response는 완료 응답으로 처리하지 않는다.
- 연결 또는 초기화 실패 시 생성 중인 iframe, port, listener, timer를 정리한다.
- 거부된 malformed/unsupported 연결과 foreign-origin 재연결의 transferred port도 즉시 닫고, runtime 종료 시 port handler를 해제한다.
- foreign source, non-connect, surplus transferred port도 즉시 닫아 수신 측 소유권을 남기지 않는다.
- SDK와 Studio는 HTTP(S) origin만 허용하고 `file:`·`data:`·브라우저 확장 등 opaque origin을 명시적으로 거부한다.
- 동기 `postMessage` 실패도 request timer와 pending map을 즉시 정리한다.
- 기존 method/return type과 legacy client message shape을 유지한다.
- browser E2E는 protocol을 재구현하지 않고 실제 `@rhwp/editor` entry로 load/export/destroy를 호출하며 caller buffer, forged sibling, legacy 경로를 함께 검증한다.
- 50 MiB `Uint8Array`도 v1 `loadFile`에서 number array 변환 없이 전달하며 caller의 원본 backing buffer를 분리하지 않음을 Node `MessageChannel` 계약 테스트로 검증한다.
- session ID는 `crypto.randomUUID()` 또는 `crypto.getRandomValues()`로만 만들고 안전하지 않은 `Math.random()` fallback은 제거했다.

별도 package와 caller buffer 직접 transfer는 각각 API 중복과 기존 소유권 변경 때문에 채택하지 않았다.

## Issue #2186 review follow-up

- 검증 직전 최신 `upstream/devel@48c33455`까지 rebase하고 Studio package의 기존 CanvasKit font coverage·frontend metrics
  script와 #2186 embed E2E script를 함께 보존했다.
- Phase 0 embed contract를 public `createEditor`의 exact-origin MessageChannel v1 transferable 계약과
  handshake timeout 뒤 제한된 legacy request/response fallback으로 갱신했다.
- Phase 0 metrics snapshot은 기준선 보존을 위해 재생성하지 않았다. 최종 별도 compare 결과는 reported
  function +32, total CC +74, top 20/CC>25/CC>100/max delta 0이다.
- #2183 frontend CI gate는 별도 이슈이므로 이번 변경에서 제외했다.
- port binding 뒤 같은 session의 v1 `rhwp-request`가 safe integer ID를 가졌지만 full envelope 검증에
  실패하면 `INVALID_REQUEST` response를 반환한다. wrong-session과 unsafe-ID traffic은 계속 응답 없이
  거부해 신뢰하지 않은 입력에 protocol response를 반사하지 않는다.
- bound session과 safe integer ID로 상관관계를 확인한 request는 version까지 별도로 검증한다. 지원하지
  않는 정수 version은 `UNSUPPORTED_VERSION`과 `supportedVersions: [1]`을, version 누락·비숫자 값은
  malformed `INVALID_REQUEST`를 반환한다.

PR head를 그대로 두면 최신 base 충돌과 stale contract가 남고, Phase 0 metrics를 재생성하면 기준선이
훼손되므로 두 방안 모두 채택하지 않았다.

모든 malformed port message에 오류를 반환하는 방식은 foreign session과 unsafe ID까지 반사하므로
기각했다. 빈 method를 router의 `RPC_ERROR`로 처리하거나 request guard에서 허용하는 방식도 malformed
envelope와 정상 RPC 실행 실패를 구분하지 못하므로 채택하지 않았다.

version이 일치해야 request attempt로 인정하는 방식은 unsupported/missing version을 모두 조용히 버려
#2186의 explicit failure 계약을 위반하므로 기각했다. 반대로 version 누락·비숫자 값까지 protocol
mismatch로 분류하면 지원 가능한 version 협상과 malformed shape를 섞으므로 `INVALID_REQUEST`로 분리했다.

reviewer가 확인한 SDK response 경계도 같은 신뢰 기준으로 보정했다. `exportHwpVerify`는 다른 load/export
작업과 동일한 60초 timeout을 사용한다. port의 `rhwp-response`가 현재 session과 pending safe-integer ID로
상관관계가 확인되면, 지원하지 않는 정수 version은 즉시 `UNSUPPORTED_VERSION`과
`supportedVersions: [1]`로 거부하고 malformed v1 envelope는 `INVALID_RESPONSE`로 거부한다. foreign
session과 unsafe ID는 기존처럼 응답 없이 무시한다.

malformed v1을 원래 request timeout까지 무시하는 방식은 구조화된 explicit failure 계약을 지키지 못해
기각했다. 반대로 첫 malformed envelope에서 동기적으로 pending을 제거하는 방식은 같은 event-loop turn에
이미 전송된 valid response로 회복하는 기존 계약을 깨므로 채택하지 않았다. 한 turn 동안만 거부를
유예하고 그 사이 valid envelope가 pending을 완료하지 않은 경우에만 `INVALID_RESPONSE`를 반환한다.
