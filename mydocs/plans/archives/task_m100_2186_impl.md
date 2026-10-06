# Task M100 #2186 구현 기록 — 임베드 MessageChannel v1

## 목표

기존 `@rhwp/editor` 공개 method를 유지하면서 iframe transport를 version/session 협상형
`MessageChannel` v1으로 강화한다. 구버전 Studio는 exact origin/source를 확인하는 legacy 경로로
자동 전환한다.

## 결정

- `@rhwp/editor` 안에 dependency-free transport를 추가했다. 별도 SDK package는 공개 API 중복이라 제외했다.
- binary는 복사본만 transferable로 보내 caller buffer 소유권을 유지한다.
- Studio는 parent window와 유효한 HTTP(S) origin을 확인한 뒤 port/session을 결합한다.
- allowlist/JWT는 배포자가 신뢰 경계를 정의해야 하므로 이번 transport 변경에 포함하지 않았다.

## 검증

- `node --test npm/editor/tests/transport.test.mjs rhwp-studio/tests/embed-protocol.test.ts`
- `npm --prefix rhwp-studio test`
- `npm --prefix rhwp-studio run build`
- `npm --prefix rhwp-studio run e2e:embed`

Refs #2186
