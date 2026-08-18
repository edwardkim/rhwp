# replay — 단건 영수증

기존 CLI: `rhwp replay [--plan-json <json> | <계획.json>] [--expect-output-sha256 <hex>]
[--capsule <파일>] [--parent <캡슐>] [--sign-key <키>] [--json]`

## attest

기대를 주지 않으면 `mode=attest`. 임시 산출로 재실행해
`inputSha256` · `planSha256` · `outputSha256` 을 발급한다. 계획의
`output` 경로는 만들어지지 않는다. `reproduced` 와
`expectedOutputSha256` 은 JSON null.

`planSha256` 은 **원문 바이트**의 SHA-256 이다. pretty-print 하거나
키 순서를 바꾸면 해시가 바뀐다.

## verify

`--expect-output-sha256` 이 있으면 `mode=verify`. 값은 trim 후
소문자화하고 64자리 ascii hex 여야 한다. 짧거나 16진이 아니면
**exit 2** (엔진 진입 전).

- 일치: `reproduced=true`, exit 0
- 불일치: `reproduced=false`, exit 3, 주장 해시는
  `expectedOutputSha256` 에 에코

ZERO64 (`0`×64) 는 유효한 hex 이므로 usage 가 아니라 판단 실패다.

## 캡슐

`--capsule` 은 계획(원본 output 보존) + 영수증의 자기완결 교환 파일.
`kind=workCapsule`. `--parent` 는 부모 **파일 바이트** SHA-256 을
저장한다. 상대 경로는 **캡슐 파일 기준**이지 cwd 가 아니다.

같은 파일을 `--capsule` 과 `--parent` 로 주면 거부(부모 덮어쓰기 방지).
`--sign-key` 는 `--capsule` 없이 쓸 수 없다.

## 픽스처

- `fixtures/replay/cases/*.json` — 가족·장르별로 attest/verify 3경로
- `fixtures/capsules/*.capsule.json` — 발급형 + 변조형
- `fixtures/exceptions/replay/` — usage / IO / 판단

결정론: 같은 `planText` 의 두 attest 는 같은 `outputSha256` 을 낸다.
이 전제가 제3자 재현을 가능하게 한다.
