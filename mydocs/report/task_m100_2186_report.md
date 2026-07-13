# Task M100 #2186 최종 보고서 — 임베드 MessageChannel v1

## 결과

기존 `@rhwp/editor` 공개 API를 유지하면서 iframe 통신을 version/session 협상형
`MessageChannel` v1으로 강화했다. binary는 number array 대신 transferable로 전달하고,
호출자 입력은 복사본을 전송해 원본 `ArrayBuffer` 소유권을 보존한다.

## 구현 범위

- SDK: exact Studio origin bootstrap, capability/version 협상, method별 timeout, pending cleanup
- Studio: parent source와 HTTP(S) origin 확인, 최초 session/port binding, runtime schema guard
- 호환성: 구버전 Studio/client용 legacy request/response 유지, source/origin 제한
- 생명주기: 연결·요청 실패와 `destroy()`에서 iframe, port, listener, timer 정리
- 계약: 50 MiB binary 전송, malformed message, forged sibling, surplus port를 회귀 테스트로 고정

## 설계 결정

- 새 `@rhwp/embed-sdk`는 기존 `@rhwp/editor`와 역할이 겹쳐 만들지 않았다.
- 전체 임베드 스펙은 한 PR로 합치지 않았다. transport 보안과 binary 계약을 먼저 독립적으로
  검토·배포할 수 있는 수직 슬라이스로 제한했다.
- host allowlist/JWT는 issuer, JWKS, 배포 정책이 정해지지 않은 상태에서 가짜 인증을 만들 수
  없으므로 후속 설계로 남겼다. exact origin binding은 host 인증을 의미하지 않는다.

## 검증

- `npm --prefix rhwp-studio test`: Studio와 SDK 계약 테스트 통과
- `npm --prefix rhwp-studio run build`: fresh WASM binding 기반 production build 통과
- `npm --prefix rhwp-studio run e2e:embed`: 실제 공개 SDK load/export/destroy 통과
- `playwright-cli`: 실제 iframe UI를 포함한 동일 flow 두 차례 통과, console error 0건
- `cargo fmt --all -- --check`: 통과
- `cargo test --profile release-test --tests`: 통과
- `cargo clippy -- -D warnings`: 통과

## 최신 devel review follow-up

- 최종 검증 직전 최신 `upstream/devel@48c33455`까지 rebase했다. `rhwp-studio/package.json` 충돌은 최초 새 base의
  `e2e:canvaskit-font-coverage`·`metrics:frontend`와 #2186의 `e2e:embed`를 모두 보존해 해결했다.
- Phase 0 contract는 수정 전 `window.removeEventListener is not a function`으로 RED였다. harness를
  public `createEditor` + 실제 `MessageChannel` 기반으로 보정해 exact origin, v1 binary와 제한된 legacy
  fallback을 함께 검증한다.
- 공식 Phase 0 metrics snapshot은 수정하지 않았다. 최종 별도 compare 결과는 reported function +32,
  total CC +74이며 top 20 합, CC>25 합, CC>100 합, max CC delta는 모두 0이다. 증가는 #2186
  transport와 final review guard/test surface에서 발생했고 기존 고복잡도 지표는 변하지 않았다.
- #2183 frontend CI gate는 별도 이슈이므로 구현하지 않았다. 최종 collaborator review의 외부
  선행 조건으로만 남긴다.

## 후속 범위

저장 handler와 revision 충돌, autosave 세대 관리, 필드·누름틀, command/policy, PDF export,
Web Component, JWT/allowlist, `hwpctl` 호환 adapter, Firefox/Safari CI는 별도 이슈와 PR로
분리한다. 50 MiB 계약은 Node MessageChannel에서 검증했으며 실제 브라우저 memory peak는
아직 증명하지 않았다.

Refs #2186
