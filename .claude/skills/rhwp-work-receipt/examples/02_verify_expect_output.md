# 02 — 제3자 검증 (`--expect-output-sha256`)

단: 영수증. 목표: 상대가 준 64hex 가 같은 계획의 재실행 산출과 같은지 판정한다.

권위: [replay-attest.md](../references/replay-attest.md).
픽스처: [../fixtures/envelopes/replay_verify_match.json](../fixtures/envelopes/replay_verify_match.json).

## 1. 요구할 것

상대에게 **같은 계획 원문** 과 **outputSha256** 과 **toolVersion** 을 받는다.
계획 공백이 다르면 `planSha256` 이 달라 다른 작업이다.

## 2. 호출

```bash
rhwp replay --plan-json '<같은 계획>' --expect-output-sha256 <64hex> --json
```

## 3. 판정

| 봉투 | exit | 다음 |
|------|-----:|------|
| `reproduced: true` | 0 | 주장 채택 |
| `reproduced: false` | 3 | 주장 기각. 03 편. **재시도 금지** |
| 짧은 해시 / 비hex | 2 | 호출 조립을 고친다 |

`mode` 는 `verify`. `expectedOutputSha256` 이 요청 값이다.

## 4. 하지 않는 것

- 불일치를 도구 고장으로 승격하지 않는다.
- 상대 해시를 계획 없이 믿지 않는다. 재실행이 증명이다.
