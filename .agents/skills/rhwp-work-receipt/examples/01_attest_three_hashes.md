# 01 — 단건 영수증 발급 (attest)

단: 영수증. 목표: 계획 하나를 임시 재실행해 입력·계획·산출 SHA-256 을 받는다.
사용자 `output` 경로는 **생기지 않는다**.

권위: [replay-attest.md](../references/replay-attest.md).
픽스처: [../fixtures/envelopes/replay_attest.json](../fixtures/envelopes/replay_attest.json).

## 0. 하지 않는 것

- 새 `receipt` 명령을 만들지 않는다. 기존 `rhwp replay` 다.
- 산출 파일을 사용자 경로에 쓰지 않는다. 필요하면 10 편 `run`.
- 해시를 손으로 지어내지 않는다.

## 1. 계획

```bash
rhwp replay --plan-json '{"planVersion":"1.0","input":"samples/basic/issue2007_nested_cell_pagination_42065.hwp","output":"out/notice.hwp","steps":[{"action":"replace_text","find":"2025년","replace":"2026년"}]}' --json
```

## 2. 읽는 필드

| 키 | 뜻 |
|----|----|
| `mode` | `attest` |
| `inputSha256` | 입력 문서 바이트 |
| `planSha256` | **계획 원문** 바이트. 공백이 바뀌면 해시가 바뀐다 |
| `outputSha256` | 임시 재실행 산출 바이트 |
| `toolVersion` | 재현 조건. 19 편 |
| `reproduced` | attest 에서는 `null` |
| `steps` | 숫자. `run` 저널 배열과 동명 다른 타입 |

## 3. 전달

영수증 JSON 을 산출물(있다면 `run` 이 쓴 파일)과 함께 넘긴다.
제3자는 02 편으로 검증한다.

## 4. 명령 체크리스트

- [ ] `rhwp replay` 이다
- [ ] `--json` 으로 3해시를 읽었다
- [ ] 사용자 경로에 파일이 생기지 않았음을 확인했다
- [ ] `toolVersion` 을 같이 적었다
