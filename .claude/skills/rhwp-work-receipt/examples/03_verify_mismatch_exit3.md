# 03 — 검증 불일치 (exit 3)

단: 영수증. 목표: `reproduced:false` 를 판정 데이터로 읽고 멈춘다.

권위: [exit-codes.md](../references/exit-codes.md).
픽스처: [../fixtures/envelopes/replay_verify_mismatch.json](../fixtures/envelopes/replay_verify_mismatch.json).

```bash
rhwp replay --plan-json '<계획>' --expect-output-sha256 0000000000000000000000000000000000000000000000000000000000000000 --json
```

기대:

- exit **3**
- `mode: "verify"`
- `reproduced: false`
- `outputSha256` 는 재실행 실측 (64hex)
- `expectedOutputSha256` 는 주장 값

이 숫자는 도구 크래시(1)도 사용법(2)도 아니다. 봉투를 사용자에게 보여 주고
주장을 기각한다. 같은 명령을 그대로 다시 돌리지 않는다 — 결정론이면 같은 판정이다.

선대조: `toolVersion` 이 상대와 다르면 19 편.
