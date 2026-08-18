# 09 — 선검증 위반을 한 번에 (`invalid[]`)

층: 3. 목표: 잘못된 계획을 실행하지 않고, 위반을 전부 모아 고친다.
하나 고치고 다시 돌려 다음 위반을 만나는 두더지잡기를 하지 않는다.

권위: [run_plans.md](../references/run_plans.md) §10,
[failure_envelopes.md](../references/failure_envelopes.md) §6.3.

## 1. 표본

픽스처 [../fixtures/envelopes/run_invalid_collected.json](../fixtures/envelopes/run_invalid_collected.json),
[../fixtures/envelopes/run_invalid_replace_zero.json](../fixtures/envelopes/run_invalid_replace_zero.json).

실측 (playbook §9-2):

```
$ rhwp run out/plan.json --dry-run --json
{"input":"samples/field-01.hwp",
 "invalid":[{"action":"replace_text","reason":"'여기에 입력' 일치 0건 — 치환할 곳이 없습니다","step":1}],
 "output":"out/plan_result.hwp","planVersion":"1.0","schemaVersion":"1.0"}
exit=2
```

`output` 키는 예정 경로일 뿐 파일이 생겼다는 뜻이 아니다.
이 경로를 `test -e` 하면 없어야 한다 (이전 잔재를 미리 지운다).

## 2. 여러 위반

엔진은 전 step 을 훑어 `invalid` 에 push 한다. 첫 위반에서 멈추지 않는다.

```json
"invalid": [
  {"step": 0, "action": "fill_fields",
   "reason": "필드 '없는필드' 이(가) 없거나 순번이 범위 밖입니다 (동명 0개)"},
  {"step": 1, "action": "replace_text",
   "reason": "'여기에 입력' 일치 0건 — 치환할 곳이 없습니다"},
  {"step": 2, "action": "insert_image",
   "reason": "알 수 없는 action: insert_image (fill_fields·replace_text·set_cell·set_checkbox)"}
]
```

세 줄을 한 번에 고친다. 첫 줄만 고치고 재실행하지 않는다.

## 3. 문법 오류는 `error`

`planVersion` 누락은 `invalid[]` 가 아니라:

```
{"error":"planVersion \"1.0\" 이 필요합니다","schemaVersion":"1.0"}
exit=2
```

픽스처 [../fixtures/envelopes/run_error_plan_version.json](../fixtures/envelopes/run_error_plan_version.json).
분기: FixPlanKeys. `steps` 를 보기 전에 끊긴다.

## 4. 에이전트 루프

```
run --dry-run --json
if exit 2 and invalid:
    for item in invalid: fix plan[item.step]
    repeat dry-run
if exit 2 and error:
    fix keys (planVersion/input/output/steps)
    repeat dry-run
if exit 0 and dryRun:
    run without --dry-run
```

같은 계획 바이트를 재시도하지 않는다.

## 5. 체크리스트

- [ ] stdout 을 exit 2 에서도 파싱했다
- [ ] invalid 전 원소를 고쳤다
- [ ] 산출 경로에 파일이 생기지 않았음을 확인했다
