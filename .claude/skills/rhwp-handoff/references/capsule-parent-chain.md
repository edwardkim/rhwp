# 세션 체인과 --parent

세션 N 의 산출이 세션 N+1 의 입력이 될 때, 기존 `rhwp replay --capsule` 에
`--parent` 를 붙인다. 산출 파일은 그대로 `workCapsule` 이다. 이 파일은 **세션 운영**만 적는다. 캡슐 불변·상대 경로·
같은 파일 거절·`planText` 해시는 work-receipt 정본이다.

정본 포인터:
[`.claude/skills/rhwp-work-receipt/SKILL.md`](../../rhwp-work-receipt/SKILL.md)
절차 2. 이 스킬은 그 문서를 복사하지 않는다.

## 세션에서 쓰는 한 줄

```bash
rhwp run plan-session-N.json --json
rhwp replay --plan-json '<계획 N+1, input 은 N 의 실산출>' \
  --capsule output/handoff/t-sN1/session.capsule.json \
  --parent output/handoff/t-sN1/parent.capsule.json --json
```

부모 파일을 같은 폴더에 `parent.capsule.json` 으로 복사해 두면 상대 경로가
`parent.capsule.json` 한 토큰이다. `--parent` 와 `--capsule` 이 같은 파일이면
도구가 거부한다 (부모 덮어쓰기 방지).

## 세션 핸드오프가 검사하는 것

1. 자식 `parent.capsule` 이 상대 경로인가 (절대경로 금지)
2. 그 상대 경로는 **캡슐 파일 기준**(호출 cwd 아님)으로 푼다. 파일이 있는가
3. 그 파일 바이트 SHA-256 이 `parent.sha256` 과 같은가
4. 자식 `receipt.inputSha256` 이 부모 `receipt.outputSha256` 과 같은가
   (`lineageOk` 의 정의 — 계보 스킬의 단어. 여기서 재구현하지 않는다)

1~3 이 깨지면 [`exception-parent-hash.md`](exception-parent-hash.md) 또는
[`exception-missing-capsule.md`](exception-missing-capsule.md).
4 가 깨져도 후속 `--parent` 를 붙이지 않는다. 단건 증명이 필요하면
work-receipt 의 `rhwp lineage` 로 보낸다.

## 세션 머리

incoming 은 폴더에서 **가장 최근 세션 캡슐**을 머리로 삼는다. working doc 이
파일 이름을 적는 것이 정본이다. 이름 규칙 권장:

```
session.capsule.json          # 이번 세션 (머리)
parent.capsule.json           # 직전 세션 사본
```

긴 체인(`s01`…`s24`)은 `fixtures/capsules/` 가 보여 준다. 운영에서는 폴더를
얕게 유지하고, 오래된 세션은 보관 폴더로 옮긴 뒤 working doc 경로를 고친다.
`rhwp audit` 는 비재귀 `*.capsule.json` 이다 — 보관 서브폴더는 세지 않는다
(work-receipt 계약. 여기서 재설명하지 않는다).

## 불변

캡슐을 에디터로 열어 저장하면 부모 해시 대조가 깨진다. 의도된 검출이다.
세션 메모를 고치고 싶으면 캡슐이 아니라 working doc 을 고친다.

## 하지 않는 것

- `rhwp session-chain` 같은 새 명령
- 캡슐에 `sessionTrigger` 필드를 추가 (스키마 발명)
- work-receipt 의 `fixtures/capsules/` 를 이 스킬이 덮어쓰기
- `toolVersion` 이 다른데 재현이 안 된다고 코어를 고친다
