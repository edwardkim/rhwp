# 07 — 원자 편집 (`rhwp run` fill + replace)

층: 3. 목표: 누름틀 채움과 연도 치환을 **전부 되거나 전혀 안 되게** 한다.
1층 `edit` 를 두 번 이어 붙이지 않는다.

권위: [run_plans.md](../references/run_plans.md),
playbook §12, `tests/plan_schema_contract.rs`.

## 0. 하지 않는 것

- `source`/`op` 키. `input`/`steps[].action` 이다.
- action 카멜 (`fillFields`). 스네이크 네 이름만.
- `insert_image` 를 steps 에 추가.
- dry-run 없이 바로 저장.
- `planVersion` 생략.

## 1. 스키마

```bash
rhwp export-plan-schema --json | jq '{planSchemaVersion, definitionCount, dialect}'
```

기대: `planSchemaVersion` `"1.1"`, `definitionCount` 11,
`dialect` `https://json-schema.org/draft/2020-12/schema`.

계획을 쓰기 전에 `--bare` 로 `FillFieldsStep` / `ReplaceTextStep` 필수 키를 확인한다.

```bash
rhwp export-plan-schema --bare | jq '.["$defs"].FillFieldsStep.required'
```

## 2. 계획서

픽스처 [../fixtures/plans/valid_multi_step.json](../fixtures/plans/valid_multi_step.json)
과 같은 골격:

```json
{
  "planVersion": "1.0",
  "input": "samples/field-01.hwp",
  "output": "out/plan_result.hwp",
  "steps": [
    {"action": "fill_fields", "data": {"회사명": "페타플로", "작성자": "홍길동"}},
    {"action": "fill_fields", "data": {"목차1[0]": "개요"}}
  ],
  "assertions": {"notFoundEmpty": true, "verify": true}
}
```

`목차1` 을 순번 없이 쓰면 `ambiguous` 가 저널에 남고 첫 칸만 찬다.
선검증은 통과한다 (칸은 존재하므로). 완료 조건에 `ambiguous == []` 를 넣는다.

## 3. 선검증

```bash
rhwp run out/plan.json --dry-run --json
```

기대 골격: [../fixtures/envelopes/run_dry_run_preview.json](../fixtures/envelopes/run_dry_run_preview.json).

- `dryRun: true`
- `invalid: []`
- `preview[]` 길이 = steps 길이
- `out/plan_result.hwp` 가 생기지 않음

사람 모드면 `검사 통과: N step 실행 가능 (디스크 무변경, 산출 예정 …)`.

`invalid` 가 비지 않으면 09 편. 디스크는 그대로다.

## 4. 실행

```bash
rhwp run out/plan.json --json
```

실측 저널 골격 (playbook §9-2):

```json
{
  "assertions": {"notFoundEmpty": true, "verify": true},
  "changedPages": [0, 1],
  "input": "samples/field-01.hwp",
  "output": "out/plan_result.hwp",
  "outputFormat": "hwp5",
  "planVersion": "1.0",
  "steps": [
    {"action": "fill_fields", "ambiguous": [], "confusable": [],
     "filledCount": 2, "notFound": [], "step": 0}
  ],
  "verify": {"diffCount": 0, "identical": true}
}
```

`verify.identical == false` 면 exit 3 이고 **파일이 없다**. 10 편.

## 5. 재독

```bash
rhwp fields out/plan_result.hwp --json \
  | jq -c '[.fields[]|select(.value!="")|{name,value}]'
```

각 step 의 `filled[]` 와 대조한다.

## 6. 왜 1층 체인이 아닌가

```
edit fill-fields -o a.hwp
edit replace-text a.hwp -o b.hwp    # replace 가 실패하면 a.hwp 가 반쪽
```

`run` 은 a.hwp 를 만들지 않는다. 실패 시 원본만 남는다. 15 편.

## 7. 체크리스트

- [ ] `export-plan-schema` 를 읽었다
- [ ] `--dry-run` 후 `invalid[]` 가 비었다
- [ ] `assertions.verify: true`
- [ ] `output` 이 `input` 과 다르다
- [ ] 재독이 저널과 같다
