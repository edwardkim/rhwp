# 3층 계획서 — `rhwp run`

이 문서는 rhwp-safe-edit 의 **여러 편집을 원자적으로** 적용하는 경로다.
새 action 을 발명하지 않는다. `run_plan_engine`(CLI `run` 과 MCP `hwp_run_plan` 이
공유하는 본체)이 이미 받는 계획서만 조립한다.

권위:

- 스키마 단일 출처: `rhwp export-plan-schema --json` (`planSchemaVersion` 1.1)
- 구현: `src/plan_schema.rs`, `src/main.rs` `run_plan_engine`
- 실측: [`mydocs/manual/agent_task_playbook.md`](../../../../mydocs/manual/agent_task_playbook.md) §12
- 계약 테스트: `tests/plan_schema_contract.rs`

1층 단건은 [single_edit.md](single_edit.md). 검증 루프는 [verify_loops.md](verify_loops.md).
실패 봉투는 [failure_envelopes.md](failure_envelopes.md).

---

## 0. 왜 3층인가

`edit` 를 이어 붙이면:

```
원본 ──fill──► a.hwp ──replace──► b.hwp
                 ▲
                 └ fill 은 성공, replace 가 실패하면 a.hwp 가 반쪽 산출물
```

`run` 은 그 중간 상태를 만들지 않는다.

```
원본 ──(메모리에서 fill+replace)──► 단언 통과 시에만 output 한 번
실패(선검증·실행·단언) ──► 디스크 무변경, 원본 그대로
```

세 층:

1. **선검증 (check)** — 전 step 의 실행 가능성을 입력 문서 기준으로 판정.
   위반은 `invalid[]` 에 **전부** 모은다. 실행 0, 저장 0, exit 2.
2. **원자 실행** — 전 step 을 인메모리 IR 에만 적용. 디스크는 아직 그대로.
3. **저널 + 저장** — `assertions.verify` 가 참이면 산출 바이트를 재파싱해
   인메모리 IR 과 대조. 통과할 때만 `output` 에 **한 번** 쓴다.

`--dry-run` / 계획서 `dryRun: true` 는 1층만 수행하고 `preview[]` 를 낸다.

---

## 1. 계획서를 쓰기 전에 스키마를 읽는다

필드명을 지어내면 `invalid[]` 왕복이 생긴다. 스키마가 정답지다.

```bash
rhwp export-plan-schema --json      # 봉투: schemaVersion, planSchemaVersion, dialect, definitionCount, schema
rhwp export-plan-schema --bare      # JSON Schema 본문만 ($ref: #/$defs/Plan)
rhwp export-plan-schema -o plan.schema.json --json
```

봉투 계약 (`tests/plan_schema_contract.rs`):

- `schemaVersion`: `"1.0"` (봉투 판)
- `planSchemaVersion`: `"1.1"` (#4378 — `preconditions.inputSha256` 추가)
- `dialect`: `https://json-schema.org/draft/2020-12/schema`
- `definitionCount` == `$defs` 키 수 (11: Plan, Preconditions, Assertions, Step,
  FillFieldsStep, ReplaceTextStep, SetCellStep, SetCheckboxStep, StepCondition,
  FieldEqualsCondition, PreviewStep)
- `untrustedContent`: false — 문서를 열지 않는 명령

`--bare` 는 봉투 키와 출처 표지를 싣지 않는다. 검증기에 그대로 먹이는 본문이다.

계획서 본체·step 은 `additionalProperties: true` (추가-전용 진화).
**조건절은 닫혀 있다** (`additionalProperties: false`, minProperties=1, maxProperties=1).
스키마가 실행기보다 관대하면 "검증은 통과하는데 실행은 거부"가 생긴다.

---

## 2. 계획서 봉투 — 필수 키 넷

```json
{
  "planVersion": "1.0",
  "input": "서식.hwp",
  "output": "완성본.hwp",
  "steps": [ { "action": "fill_fields", "data": { "기관명": "한국수자원공사" } } ]
}
```

| 키 | 필수 | 값 | 오타로 자주 쓰는 것 |
|----|:----:|----|---------------------|
| `planVersion` | 예 | 문자열 `"1.0"` 만 | 누락, `1.0` 숫자, `"2.0"` |
| `input` | 예 | 원본 경로. 실행기는 읽기만 | `source`, `file`, `path` |
| `output` | 예 | 산출 경로. 단언 통과 후 한 번 씀 | `out`, `dest`, `-o` |
| `steps` | 예 | 비어 있지 않은 배열 | `ops`, `actions`, `[]` |
| `assertions` | 아니오 | `notFoundEmpty`·`verify` | |
| `preconditions` | 아니오 | `{ "inputSha256": "<64 hex>" }` | 빈 객체, 다른 키 |
| `dryRun` | 아니오 | bool. CLI `--dry-run` 이 덮어씀 | |

`planVersion` 이 `"1.0"` 이 아니면:

```json
{"error":"planVersion \"1.0\" 이 필요합니다","schemaVersion":"1.0"}
```

exit 2. 실행 0.

`steps` 가 없거나 빈 배열이면 `"steps 는 비어 있지 않은 배열이어야 합니다"`.
아무것도 하지 않는 계획은 의도가 아니라 실수다.

픽스처: [../fixtures/plans/valid_fill_fields.json](../fixtures/plans/valid_fill_fields.json),
[../fixtures/plans/invalid_missing_plan_version.json](../fixtures/plans/invalid_missing_plan_version.json),
[../fixtures/plans/invalid_wrong_keys_source_op.json](../fixtures/plans/invalid_wrong_keys_source_op.json),
[../fixtures/plans/invalid_empty_steps.json](../fixtures/plans/invalid_empty_steps.json).

---

## 3. CLI 입구

```
rhwp run <계획.json> [--json] [--dry-run]
rhwp run --plan-json '<JSON>' [--json] [--dry-run]
```

- 파일 경로와 `--plan-json` 인라인 둘 다 받는다. 인라인이 있으면 파일을 무시한다.
- `--dry-run` 은 계획서에 `dryRun: true` 를 **덮어쓴다**. 계획서가 `false` 여도
  플래그가 이긴다. 의도의 단일 출처는 계획서이고 CLI 는 편의 입구다.
- 알 수 없는 옵션은 exit 2, 계획 파일을 읽기 전에 끊는다.
- 계획 파일 읽기 실패는 exit 1 (`오류: 계획 파일을 읽을 수 없습니다`).
- JSON 파싱 실패는 exit 2 (`오류: 계획 JSON 파싱 실패`).

`--json` 이면 저널이 stdout. 사람 모드에서 성공이면 한 줄 요약 + step 미리보기,
실패면 저널을 **stderr** 로 남긴다 (판정 근거를 버리 지 않기 위함).
에이전트는 항상 `--json` 을 붙인다.

---

## 4. action 4종 — 이 스킬이 배선하는 전부

`steps[].action` 은 스네이크다. 카멜(`fillFields`)·케밥(`fill-fields`)·
1층 CLI 이름(`fill-fields`)을 그대로 넣으면 `알 수 없는 action` 이다.

### 4.1 `fill_fields`

필수: `data` 객체. 값 타입은 string / number / boolean.
문자열이 아니면 JSON 표기 그대로 문자열화한다.

```json
{"action": "fill_fields", "data": {"회사명": "페타플로", "목차1[0]": "개요"}}
```

선검증:

- `data` 가 객체가 아니면 invalid.
- 각 키를 `이름` 또는 `이름[순번]` 으로 분해.
- 그 이름의 동명 개수가 0 이거나 순번 ≥ 개수 이면
  `필드 '…' 이(가) 없거나 순번이 범위 밖입니다 (동명 N개)`.

미리보기 `preview[]` 원소:

```json
{"step": 0, "action": "fill_fields", "targets": [
  {"name": "회사명", "occurrence": 0, "sameNameCount": 1, "value": "페타플로"}
]}
```

실행 저널:

```json
{
  "step": 0,
  "action": "fill_fields",
  "filledCount": 2,
  "filled": [{"name": "회사명", "occurrence": 0, "value": "페타플로"}],
  "notFound": [],
  "ambiguous": [],
  "confusable": []
}
```

선검증이 없는 필드를 이미 걸렀으므로 저널의 `notFound` 는 항상 `[]` 이다.
`assertions.notFoundEmpty` 기본 true 가 이 구조적 보장을 계약 표기로 남긴다.

순번 없이 동명 키를 주면 첫 칸을 채우고 `ambiguous: [{name, matched:1, total:N}]`.
화면상 구별되지 않는 이름(유니코드 혼동)은 `confusable` 로 경고한다.
사람 모드면 stderr 에 한 줄 경고가 나간다. 채우기는 막지 않는다.

### 4.2 `replace_text`

필수: `find`(비어 있지 않은 문자열), `replace`(문자열, 빈 문자열 = 삭제).
선택: `caseSensitive`(기본 true), `occurrence`(0 기준 한 건).

선검증:

- `find` 없거나 빈 문자열 → `find (비어 있지 않은 문자열)가 필요합니다`
- `replace` 가 문자열 아님 → `replace (문자열)가 필요합니다`
- `occurrence` 가 일치 건수 이상 → `occurrence N 이(가) 범위 밖입니다 ('…' 일치 M건)`
- occurrence 없이 일치 0건 → `'…' 일치 0건 — 치환할 곳이 없습니다`

1층 `edit replace-text` 는 0건을 exit 0 + 산출 없음으로 보고한다.
3층은 0건을 **계획 오류** 로 거부한다. 조용히 0건 성공을 만들지 않는다.

미리보기:

```json
{"step": 1, "action": "replace_text", "find": "2025년", "matches": 7, "willReplace": 7}
```

`occurrence` 가 있으면 `willReplace` 는 1.

### 4.3 `set_cell`

필수: `table`·`row`·`col`(0 이상 정수), `text`(한 줄 문자열).
선택: `keepStyle`(기본 false — 검정으로 기록).

선검증:

- 네 키 중 하나라도 빠지거나 타입 불일치 →
  `table·row·col (정수)과 text (문자열)가 필요합니다`
- `text` 에 `\r` `\n` `\t` → `text 에 줄바꿈·탭은 넣을 수 없습니다 (한 줄 값 기록)`
- `row`/`col` 이 65535 초과 → `0..65535 범위를 벗어났습니다` (HWP 격자 주소는 u16)
- 병합으로 덮인 칸·없는 표·중첩 표 → `resolve_table_cell` 의 안내 문장 그대로

미리보기:

```json
{"step": 2, "action": "set_cell", "table": 0, "row": 2, "col": 1,
 "currentText": "", "newText": "1,234"}
```

좌표는 **실행 시점**에 다시 해석한다. 앞 step 이 표를 밀어도 격자 주소는 같다.
그래도 조건절은 입력 문서 기준이므로, "앞 step 이 만든 칸"을 `if` 로 가리키지 마라.

### 4.4 `set_checkbox`

필수: `occurrence`(0 기준 빈 체크박스 □ 순번).

선검증:

- 키 없음 → `occurrence (0 기준 순번)가 필요합니다`
- `n >= grep("□")` → `occurrence N 이(가) 범위 밖입니다 (빈 체크박스 □ M건)`

미리보기:

```json
{"step": 3, "action": "set_checkbox", "occurrence": 0, "available": 4}
```

실행은 그 □ 를 ☑ 로 바꾼다. 1층 `replace-text --find □ --replace ☑ --occurrence k` 와
같은 코어다. 계획서에만 별칭이 있다.

### 4.5 없는 action

```json
{"step": 0, "action": "insert_image",
 "reason": "알 수 없는 action: insert_image (fill_fields·replace_text·set_cell·set_checkbox)"}
```

`fillFields`, `replace`, `setCell`, `op`, `edit` 도 전부 이 갈래다.
이 스킬은 다섯 번째 action 을 문서에 추가하지 않는다.

픽스처: [../fixtures/plans/invalid_unknown_action.json](../fixtures/plans/invalid_unknown_action.json),
[../fixtures/plans/invalid_camel_action.json](../fixtures/plans/invalid_camel_action.json).

---

## 5. 조건절 `if` (#3719 §6-8)

모든 action 이 선택 필드 `if` 를 받는다. 조건이 거짓이면 그 step 만 건너뛰고
저널/미리보기에 `skipped: true` + `reason` 을 남긴다.

### 5.1 정확히 한 종류

허용 키 셋, **한 객체에 하나**:

| 키 | 피연산자 | 참인 때 |
|----|----------|---------|
| `fieldExists` | 비어 있지 않은 문자열 | 그 이름(또는 `이름[N]`)의 누름틀이 있다 |
| `fieldEquals` | `{name, value}` | 그 누름틀의 **현재 값**이 value 와 완전 일치 |
| `textFound` | 비어 있지 않은 문자열 | 본문에 그 문자열이 한 건이라도 있다 (대소문자 구별) |

둘 이상 주면 and/or 가 계획서에 없으므로 거부한다.
빈 객체도 거부 (`minProperties: 1`).
모르는 키도 거부 (조건절은 닫힌 객체).

### 5.2 판정 시점

조건은 **입력 문서 기준으로 실행 전에 한 번** 판정한다.
앞 step 의 채움·치환이 뒤 step 의 `if` 를 바꾸지 않는다.

이유: 선검증과 실행이 같은 판정을 보게 하기 위함이다.
실행 중에 다시 보면 "선검증은 통과시켰는데 실행에서 조건을 잃는" 상태가 생긴다.

따라서 이런 계획은 의도와 다르게 움직인다:

```json
{"action": "fill_fields", "data": {"상태": "완료"}},
{"action": "replace_text", "find": "미완료", "replace": "완료",
 "if": {"fieldEquals": {"name": "상태", "value": "완료"}}}
```

두 번째 `if` 는 입력 문서의 `상태` 를 본다. 첫 step 이 방금 쓴 값이 아니다.

"채운 뒤에 치환"은 조건이 아니라 **step 순서** 로 표현한다. 둘 다 조건 없이 나열하면
선검증이 둘 다 실행 가능하다고 판정한 뒤 메모리에서 순서대로 적용한다.

### 5.3 건너뛴 step 은 선검증 면제

조건이 거짓인 step 은 실행 가능성 검사를 하지 않는다.
없는 필드를 채우는 step 이라도 `if` 가 거짓이면 위반이 아니다.

이것이 없으면 조건절은 "쓸 수는 있으나 쓰면 계획이 통과하지 않는" 장식이 된다.

건너뛴 step 도 `preview[]`/`steps[]` 에서 **자리를 지킨다**.
인덱스로 계획서 항목과 짝지을 수 있다. 조용히 사라지면 "왜 그 칸이 안 바뀌었는지"
저널만 봐서는 재구성되지 않는다.

픽스처: [../fixtures/plans/valid_conditional_field_exists.json](../fixtures/plans/valid_conditional_field_exists.json),
[../fixtures/plans/valid_conditional_field_equals.json](../fixtures/plans/valid_conditional_field_equals.json),
[../fixtures/plans/valid_conditional_text_found.json](../fixtures/plans/valid_conditional_text_found.json),
[../fixtures/plans/invalid_two_conditions.json](../fixtures/plans/invalid_two_conditions.json).

---

## 6. assertions

```json
"assertions": { "notFoundEmpty": true, "verify": true }
```

| 키 | 기본 | 뜻 |
|----|------|-----|
| `notFoundEmpty` | **true** | 지목한 대상이 하나도 빠지지 않았음. 선검증이 구조적으로 보장. 저널에 계약 표기 |
| `verify` | **false** | 저장 직전 산출 바이트를 재파싱해 인메모리 IR 과 대조 |

`verify: true` 이고 차이가 있으면 **exit 3, 디스크 무변경**.
1층 `edit --verify` 가 산출물을 남기는 것과 **반대**다.

단언 실패는 계획이 스스로 검증 조건을 들고 다녀서
"성공했다고 보고했는데 산출물이 깨진" 상태를 구조적으로 없앤다.

생략하면 기본값으로 판정한다. `assertions` 키 자체가 없어도 된다.

픽스처: [../fixtures/plans/valid_assertions_verify.json](../fixtures/plans/valid_assertions_verify.json).

---

## 7. preconditions.inputSha256 (#4378 R22)

```json
"preconditions": { "inputSha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef" }
```

계획이 세워진 시점의 입력 바이트를 고정한다. 실행 시점 해시가 다르면
다른 에이전트/사람이 그 사이에 문서를 바꾼 것이다.

계약:

- `preconditions` 를 쓰면 `inputSha256` **하나만** 있어야 한다.
- 값은 64자리 16진 (대소문자 무관, 내부는 소문자로 정규화).
- 빈 객체, 다른 키, 비문자열, 길이 오류는 exit 2 (사용법).
- 해시는 `rhwp` 가 파일을 읽은 직후 `sha256_hex_of` 로 계산한 값과 대조한다.

**불일치 판정은 exit 3 이지 exit 2 가 아니다.**

- 계획서 문법도 의미도 옳다. 틀린 것은 세상 쪽이다.
- `invalid[]` 는 **빈 배열**이다. "invalid 가 비어 있지 않으면 exit 2" 불변식을
  CAS 가 흔들지 않게 하기 위함이다.
- 봉투에 `preconditionFailed: {kind:"inputSha256", expected, actual}` 와
  `nextCall` 이 실린다.

`nextCall` 은 같은 의도를 **새 지문**으로 다시 선검증하는 실행 가능한 호출이다.

```json
{
  "name": "run",
  "arguments": ["--plan-json", "<기대 해시를 실제로 갈아 끼운 계획>", "--dry-run", "--json"],
  "why": "계획 수립 후 입력 문서가 바뀌었습니다. …"
}
```

`--dry-run` 이라 디스크를 건드리지 않는다. 통과하면 `--dry-run` 만 빼고 다시 부르고,
`invalid[]` 가 나오면 문서를 다시 읽고 재계획한다.

픽스처: [../fixtures/plans/valid_preconditions.json](../fixtures/plans/valid_preconditions.json),
[../fixtures/envelopes/run_precondition_failed.json](../fixtures/envelopes/run_precondition_failed.json).

---

## 8. dry-run 미리보기

```bash
rhwp run plan.json --dry-run --json
```

성공 봉투:

```json
{
  "schemaVersion": "1.0",
  "planVersion": "1.0",
  "dryRun": true,
  "input": "서식.hwp",
  "output": "완성본.hwp",
  "preview": [ {"step": 0, "action": "fill_fields", "targets": […]} ],
  "invalid": [],
  "assertions": {"notFoundEmpty": true, "verify": true}
}
```

사람 모드:

```
검사 통과: 2 step 실행 가능 · 1 step 건너뜀 예정 (디스크 무변경, 산출 예정 완성본.hwp)
  - step 0: 누름틀 1칸 채움
  - step 1 건너뜀 예정: …
  - step 2: '2025년' 7건 중 7건 치환
```

건너뛸 step 은 "실행 가능" 개수에 넣지 않는다.
dry-run 이 예고하는 실행 개수와 실제 `run` 이 보고할 적용 개수가 같아야 한다.

dry-run 은 산출 경로를 touch 하지 않는다. `output` 키는 "예정 경로"일 뿐
파일이 생겼다는 뜻이 아니다.

워크스루: [../examples/07_run_atomic_fill_replace.md](../examples/07_run_atomic_fill_replace.md),
[../examples/08_run_conditional.md](../examples/08_run_conditional.md).

---

## 9. 실행 저널 (저장 성공)

```json
{
  "schemaVersion": "1.0",
  "planVersion": "1.0",
  "input": "samples/field-01.hwp",
  "output": "out/plan_result.hwp",
  "outputFormat": "hwp5",
  "changedPages": [0, 1],
  "steps": [
    {
      "step": 0, "action": "fill_fields",
      "filledCount": 2,
      "filled": [
        {"name": "작성자", "occurrence": 0, "value": "홍길동"},
        {"name": "회사명", "occurrence": 0, "value": "페타플로"}
      ],
      "notFound": [], "ambiguous": [], "confusable": []
    },
    {
      "step": 1, "action": "fill_fields",
      "filledCount": 1,
      "filled": [{"name": "목차1", "occurrence": 0, "value": "개요"}],
      "notFound": [], "ambiguous": [], "confusable": []
    }
  ],
  "assertions": {"notFoundEmpty": true, "verify": true},
  "verify": {"diffCount": 0, "identical": true}
}
```

`changedPages` 는 재조판 후 0 기준 쪽 번호. 눈검증은 이 쪽만 렌더한다.
dry-run 저널에는 이 키가 없거나 확정할 수 없다 — 저장 전에 조판하지 않는다.

CAS 를 켠 성공 저널은 `inputSha256` 을 싣는다. 계획서의 기대 해시와
실행기가 읽은 해시가 같은 함수(`sha256_hex_of`)를 쓴다.

---

## 10. 선검증 실패 저널

위반을 **한 번에** 모은다. 하나 고치면 다음이 나오는 두더지잡기를 피한다.

```json
{
  "schemaVersion": "1.0",
  "planVersion": "1.0",
  "input": "samples/field-01.hwp",
  "output": "out/plan_result.hwp",
  "invalid": [
    {"step": 1, "action": "replace_text",
     "reason": "'여기에 입력' 일치 0건 — 치환할 곳이 없습니다"}
  ]
}
```

exit 2. 산출 파일이 생기지 않는다. 원본 불변.

`invalid[]` 원소는 최소 `step` + `reason`. `action` 은 있을 때 싣는다
(`action` 자체가 빠진 step 은 `action` 키가 없을 수 있다).

에이전트 분기:

```
stdout 가 비어 있지 않고 invalid 가 비어 있지 않으면
  → 호출 조립을 고친다. 같은 계획을 재시도하지 않는다.
  → step 인덱스로 계획서를 고치고 다시 --dry-run.
```

워크스루: [../examples/09_run_invalid_collected.md](../examples/09_run_invalid_collected.md).
봉투 픽스처: [../fixtures/envelopes/run_invalid_replace_zero.json](../fixtures/envelopes/run_invalid_replace_zero.json).

---

## 11. MCP `hwp_run_plan` 주의

같은 `run_plan_engine` 이다. 그러나 MCP 경로에서 선검증 실패는
JSON-RPC `isError: false` 일 수 있다. 도구 실행이 실패한 것이 아니라
**판정 봉투**를 돌려준 것이기 때문이다.

판정:

```
structuredContent.invalid == []  이고  preconditionFailed 가 없다
```

`isError` 만 보고 성공으로 오독하지 마라. MCP 세션 스킬의 본문을 이 파동에서
고치지 않는다. 여기서는 "같은 엔진, 같은 저널, 다른 수송" 만 적는다.

`hwp_export_plan_schema` 가 `export-plan-schema --json` 과 같다.
문서를 입력으로 받지 않는다.

---

## 12. 바인딩

Node `@rhwp/node` 와 Python `rhwp` 의 `Plan(...).check()` / `.run()` 이
있으면 그것은 각각 `run --dry-run` 과 `run` 이다. 새 판정 로직이 아니다.

이 devel 스냅샷에 바인딩 README 가 없을 수 있다. 없어도 CLI 두 호출이 정본이다.
바인딩이 exit 3/4 를 예외로 올리면 계약 위반이다 — 판정 필드로 돌려야 한다.

---

## 13. 1층과의 대응표

| 1층 CLI | 계획 action | 0건 처리 | 비고 |
|---------|-------------|----------|------|
| `edit fill-fields --data` | `fill_fields.data` | 1층은 notFound+exit 0, 3층은 선검증 거부 | 3층이 더 엄격 |
| `edit replace-text --find/--replace` | `replace_text` | 1층 0건=산출 없음, 3층 0건=invalid | 3층이 더 엄격 |
| `edit replace-text --occurrence` | `replace_text.occurrence` | 범위 밖은 둘 다 실패 | |
| `edit replace-text --ignore-case` | `caseSensitive: false` | 기본이 반대 이름 | 기본은 구별 |
| `edit set-cell --keep-style` | `keepStyle: true` | | |
| `edit replace-text □→☑ --occurrence` | `set_checkbox` | | 별칭 |
| `edit insert-image` | **없음** | | 1층만 |
| `edit redact` | **없음** | | 1층만 |
| `edit sanitize` | **없음** | | 1층만 |
| `csv-to-table` | **없음** | | 1층만 |
| `batch fill` | **없음** | | 메일머지 |

`caseSensitive` 기본 true = CLI 기본(구별). `--ignore-case` 를 쓰려면
계획서에 `caseSensitive: false` 를 적는다. 키 이름을 지어내 `ignoreCase` 로 쓰지 마라.

---

## 14. 권장 작성 순서

1. `rhwp export-plan-schema --bare` 를 읽어 action 필수 키를 확인한다.
2. `fields` / `export-tables` / `search` 로 주소를 얻는다.
3. 계획서를 쓴다. `input` 은 원본, `output` 은 새 경로.
4. 원본이 경합에 노출되면 `preconditions.inputSha256` 을 넣는다.
5. `rhwp run plan.json --dry-run --json` → `invalid[]` 가 비고 `preview[]` 가 맞는지.
6. `assertions.verify: true` 를 켜고 `rhwp run plan.json --json`.
7. 저널의 `steps[]` 와 `changedPages` 를 읽고 재독·눈검증은
   [verify_loops.md](verify_loops.md) 로 간다.

---

## 15. 계획서 작성 함정 (실측)

1. `source`/`op`/`ops`/`file` — 실행기가 모르는 키는 본문에선 무시되거나
   (`additionalProperties: true`) 필수 키 누락으로 떨어진다. 필수 키를 대체하지 못한다.
2. `planVersion` 숫자 `1.0` — 문자열만 받는다.
3. action 카멜/케밥.
4. `fill_fields` 의 data 를 배열로 쓰기.
5. `set_cell.text` 에 여러 줄.
6. `if` 에 조건 두 개.
7. `if` 로 "앞 step 의 결과"를 읽으려 하기.
8. `insert_image` 를 steps 에 넣기.
9. 빈 `steps: []`.
10. `output` 을 원본과 같은 경로로 두기 — 엔진이 원본을 읽기만 한다고 해도,
    성공 시 그 경로에 쓴다. 에이전트는 원본과 다른 `-o` 와 같은 이유로 분리한다.
11. dry-run 없이 바로 실행. 선검증이 디스크를 안 건드린다는 것을 알고도
    preview 를 안 보면 `ambiguous`/`confusable` 을 놓친다.
12. CAS 불일치를 exit 2 로 분류해 인자를 고치려 하기.

---

## 16. 다음 문서

- 저장 후 `--verify` · 재독 · `changedPages` → [verify_loops.md](verify_loops.md)
- exit 코드와 봉투 사전 → [failure_envelopes.md](failure_envelopes.md)
- 원자 워크스루 → [../examples/07_run_atomic_fill_replace.md](../examples/07_run_atomic_fill_replace.md)
