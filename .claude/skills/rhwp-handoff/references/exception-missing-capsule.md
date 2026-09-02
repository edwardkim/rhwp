# 예외: 캡슐 부재

## 신호

working doc 또는 incoming 규약이 가리키는 `session.capsule.json` /
`parent.capsule.json` 이 없다. `result.json` 만 있거나 working doc 만 있는
상태.

## 하지 않는 것

- 빈 `{"kind":"workCapsule"}` 를 날조해 머리를 채운다
- 대화 기억으로 `plan` 을 재구성해 `--capsule` 없이 다음 `run` 을 잇는다
- 다른 세션 폴더의 캡슐을 이름만 바꿔 복사한다 (해시 체인이 거짓이 된다)

## 하는 것

1. `result.json` 이 있으면 그것만으로 **위임 결과**를 소비할지 판단한다
   (`outcome == accepted` 이면 수거물은 쓸 수 있다)
2. 세션 체인(`--parent`)은 **중단**한다. 다음 캡슐은 parent 없이 새 뿌리로만
   발급할 수 있고, 그 결정은 working doc 에 "체인 단절, 새 뿌리" 로 적는다
3. 단건 증명이 필요하면 사람에게 알리고 work-receipt 로 보낸다
4. `_skillMeta.exit` 표본은 1 (IO: 머리 파일 없음) —
   `fixtures/exceptions/missing_capsule.json`,
   `fixtures/envelopes/missing_capsule.json`

## 워크스루

[`../examples/08_missing_capsule.md`](../examples/08_missing_capsule.md).
레이아웃: `fixtures/layouts/missing-capsule/`.
