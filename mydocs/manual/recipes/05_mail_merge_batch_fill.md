---
kind: guide
status: active
canonical: mydocs/manual/recipes/05_mail_merge_batch_fill.md
last_verified: 2026-08-03
---

# 레시피 5 — 서식 하나에 여러 사람 데이터를 한 번에 채우기 (메일머지)

**목표 한 줄**: 누름틀이 있는 서식(`.hwp`/`.hwpx`) 하나와, 사람 수만큼 행이 있는
데이터(CSV/JSONL)를 받아서, 행마다 산출물 파일을 하나씩 만든다 — 참석자 명단,
안내문 발송 대상, 계약서 상대방 목록처럼 "같은 서식 + 다른 값"이 반복되는
작업이다.

이 레시피는 [레시피 1](01_fill_form_and_submit.md)의 `edit fill-fields`를 행 단위로
반복하는 것과 결과는 같지만, `batch fill`은 이 반복을 명령 하나로 처리하고
스레드를 나눠 병렬로 돈다. 값 하나를 채우는 절차가 궁금하면 레시피 1을 먼저 본다
— 여기서는 "여러 행"에서만 달라지는 부분에 집중한다.

`batch`의 다른 하위 명령(`export-text`·`info`·`export-structure`·
`export-tables`·`fields`·`search`·`convert`)은 표준입력으로 **파일 경로
목록**을 받지만, `fill`만은 다르다 — 서식은 하나 고정이고 대신 **데이터 행
목록**을 `--data`로 받는다. 이 차이를 헷갈리면 "파일 목록을 stdin으로 파이프
했는데 아무 일도 안 일어난다"는 증상을 겪는다 — `fill`은 stdin을 아예 읽지
않는다.

절차 요약:

```
fields (누름틀 이름 확인 — 레시피 1의 0단계와 동일)
  → 데이터 준비 (JSONL 또는 CSV, 행마다 필드명=값)
  → batch fill --dry-run (커밋 전 미리보기)
  → batch fill (실제 산출)
  → batch fill --verify (재파싱 대조까지 원하면)
```

모든 명령·출력은 이 저장소의 `samples/form-01.hwp`(누름틀 `myMsg01` 1개 포함,
레시피 1과 동일 표본)로 실제 실행해서 얻었다. 지어낸 값은 없다.

## 0단계 — 채울 필드 이름을 먼저 확인한다

레시피 1의 0단계와 동일하다 — `fields --json`은 문서를 읽기만 한다.

```bash
rhwp fields --json samples/form-01.hwp
```

실측 출력:

```json
{"fieldCount":1,"fields":[{"command":"Clickhere:set:48:Direction:wstring:6:여기에 입력 HelpState:wstring:0:  ","editableInForm":true,"fieldId":2110609883,"fieldType":"ClickHere","guide":"여기에 입력","location":{"nested":[],"paragraph":10,"section":0},"memo":"","name":"myMsg01","value":""}],"schemaVersion":"1.0","source":"samples/form-01.hwp","textSecurity":{"status":"clean"}}
```

`fields[].name`: `"myMsg01"` — 데이터 파일의 컬럼/키 이름으로 그대로 쓴다.
여러 사람에게 보낼 서식이라면, 이 시점에 `textSecurity.status`도 확인해 둔다 —
서식 자체(빈 템플릿)는 대개 `"clean"`이지만, 재사용해온 서식이라면 이전에 채운
값이 안내문에 남아 있을 수 있다.

## 1단계 — 데이터를 준비한다: JSONL 한 줄당 한 행

`--data`는 확장자로 형식을 판별한다 — `.jsonl`은 한 줄에 JSON 객체 하나, `.csv`는
첫 줄이 헤더다. 필드명은 0단계에서 확인한 `name`과 정확히 일치해야 한다.

```bash
printf '%s\n' \
  '{"myMsg01":"김철수 귀하"}' \
  '{"myMsg01":"이영희 귀하"}' \
  > row1.jsonl
```

```bash
rhwp batch fill --form samples/form-01.hwp --data row1.jsonl --out-dir batch_out --json
```

실측 출력 — 행마다 한 줄씩 JSON, 마지막에 사람이 읽는 요약:

```json
{"ambiguous":[],"changedPages":[0],"confusable":[],"dryRun":false,"filled":[{"name":"myMsg01","occurrence":0,"value":"김철수 귀하"}],"filledCount":1,"notFound":[],"output":"batch_out\\0001.hwp","outputFormat":"hwp5","row":0,"schemaVersion":"1.0","source":"samples/form-01.hwp","verify":null}
{"ambiguous":[],"changedPages":[0],"confusable":[],"dryRun":false,"filled":[{"name":"myMsg01","occurrence":0,"value":"이영희 귀하"}],"filledCount":1,"notFound":[],"output":"batch_out\\0002.hwp","outputFormat":"hwp5","row":1,"schemaVersion":"1.0","source":"samples/form-01.hwp","verify":null}
```

```
batch fill: 2행 중 2 성공, 0 실패 (3ms, threads=32)
```

읽어야 할 것 셋:

- `row`: 0, 1 — 0부터 시작하는 행 번호. `--data`의 몇 번째 줄에서 왔는지 그대로
  대응한다.
- `output`: `batch_out\0001.hwp`, `batch_out\0002.hwp` — 이름을 따로 지정하지
  않으면 행 번호로 4자리 0채움 파일명이 자동 생성된다.
- 요약 줄의 `2행 중 2 성공, 0 실패` — 파이프라인에서 성공/실패 건수를 사람이
  빠르게 확인하는 용도다. 기계 판정에는 각 행 JSON의 `notFound`/`ambiguous`를
  쓴다(레시피 1과 동일한 판정 규칙).

## 2단계 — CSV로도 같은 결과를 얻는다

외부 스프레드시트에서 명단을 관리한다면 CSV가 더 편하다. 첫 줄이 헤더이고, 헤더
이름이 곧 필드명이다.

```bash
printf 'myMsg01\n박민수 귀하\n최지은 귀하\n' > rows.csv
```

```bash
rhwp batch fill --form samples/form-01.hwp --data rows.csv --out-dir batch_out2 --json
```

실측 출력(값만 다름을 확인):

```json
{"ambiguous":[],"changedPages":[0],"confusable":[],"dryRun":false,"filled":[{"name":"myMsg01","occurrence":0,"value":"박민수 귀하"}],"filledCount":1,"notFound":[],"output":"batch_out2\\0001.hwp","outputFormat":"hwp5","row":0,"schemaVersion":"1.0","source":"samples/form-01.hwp","verify":null}
{"ambiguous":[],"changedPages":[0],"confusable":[],"dryRun":false,"filled":[{"name":"myMsg01","occurrence":0,"value":"최지은 귀하"}],"filledCount":1,"notFound":[],"output":"batch_out2\\0002.hwp","outputFormat":"hwp5","row":1,"schemaVersion":"1.0","source":"samples/form-01.hwp","verify":null}
```

CSV와 JSONL은 동등한 입력이다 — 이미 JSON을 다루는 파이프라인이면 JSONL을,
사람이 스프레드시트로 명단을 관리하면 CSV를 고르면 된다.

## 3단계 — 산출 파일 이름을 사람이 알아볼 수 있게: `--name-field`

기본 `0001.hwp`/`0002.hwp` 대신, 데이터의 특정 컬럼 값을 파일명으로 쓸 수 있다.

```bash
printf 'myMsg01,outname\n김철수 귀하,chulsoo\n이영희 귀하,younghee\n' > named_rows.csv
```

```bash
rhwp batch fill --form samples/form-01.hwp --data named_rows.csv \
  --out-dir named_out --name-field outname --json
```

실측 출력:

```json
{"ambiguous":[],"changedPages":[0],"confusable":[],"dryRun":false,"filled":[{"name":"myMsg01","occurrence":0,"value":"김철수 귀하"}],"filledCount":1,"notFound":["outname"],"output":"named_out\\chulsoo.hwp","outputFormat":"hwp5","row":0,"schemaVersion":"1.0","source":"samples/form-01.hwp","verify":null}
{"ambiguous":[],"changedPages":[0],"confusable":[],"dryRun":false,"filled":[{"name":"myMsg01","occurrence":0,"value":"이영희 귀하"}],"filledCount":1,"notFound":["outname"],"output":"named_out\\younghee.hwp","outputFormat":"hwp5","row":1,"schemaVersion":"1.0","source":"samples/form-01.hwp","verify":null}
```

**함정**: `output`이 `named_out\chulsoo.hwp`로 정확히 원하는 대로 나왔지만,
`notFound`에 `"outname"`이 같이 찍힌다 — `--name-field`로 지정한 컬럼이 파일명
용도로만 쓰이는 게 아니라, 채울 필드 후보로도 함께 검사되기 때문이다. 서식에
`outname`이라는 이름의 누름틀이 없으니 그 컬럼은 "채우기엔 못 찾음"으로
보고된다. **이건 실패가 아니다** — `filledCount`가 원하는 값(1)과 일치하고
`output` 경로가 의도한 파일명이면 정상이다. 다만 자동화 게이트를 짤 때
`notFound`를 무조건 실패로 처리하면 이 경우 오탐이 난다 — `--name-field`로
지정한 컬럼명은 `notFound` 판정에서 미리 빼고 비교한다.

## 4단계 — 커밋 전에 미리보기: `--dry-run`

디스크에 아무것도 쓰지 않고 행별로 채움 가능 여부만 판정한다. 명단이 수백 행일
때, 필드 이름 오타나 데이터 형식 문제를 실제 산출 전에 잡는다.

```bash
rhwp batch fill --form samples/form-01.hwp --data row1.jsonl --out-dir dry_out --dry-run --json
```

실측 출력 — `dryRun: true`, `changedPages: null`(아무것도 안 바뀜):

```json
{"ambiguous":[],"changedPages":null,"confusable":[],"dryRun":true,"filled":[{"name":"myMsg01","occurrence":0,"value":"김철수 귀하"}],"filledCount":1,"notFound":[],"output":"dry_out\\0001.hwp","outputFormat":"hwp5","row":0,"schemaVersion":"1.0","source":"samples/form-01.hwp"}
{"ambiguous":[],"changedPages":null,"confusable":[],"dryRun":true,"filled":[{"name":"myMsg01","occurrence":0,"value":"이영희 귀하"}],"filledCount":1,"notFound":[],"output":"dry_out\\0002.hwp","outputFormat":"hwp5","row":1,"schemaVersion":"1.0","source":"samples/form-01.hwp"}
```

```
batch fill: 2행 중 2 성공, 0 실패 (1ms, threads=32, dry-run)
```

요약 줄 끝의 `dry-run` 표시로 사람이 로그만 봐도 "이건 실제 산출이 아니다"를
바로 알 수 있다.

## 5단계(선택) — `--verify`로 행마다 재파싱 대조한다

레시피 1의 2단계와 같은 개념을 행 단위로 확장한다 — 각 산출물을 저장 직후 다시
읽어 요청한 값이 실제로 들어갔는지 확인한다.

```bash
rhwp batch fill --form samples/form-01.hwp --data row1.jsonl --out-dir verify_out --verify --json
```

실측 출력 — 행마다 `verify.identical: true`:

```json
{"ambiguous":[],"changedPages":[0],"confusable":[],"dryRun":false,"filled":[{"name":"myMsg01","occurrence":0,"value":"김철수 귀하"}],"filledCount":1,"notFound":[],"output":"verify_out\\0001.hwp","outputFormat":"hwp5","row":0,"schemaVersion":"1.0","source":"samples/form-01.hwp","verify":{"diffCount":0,"identical":true}}
{"ambiguous":[],"changedPages":[0],"confusable":[],"dryRun":false,"filled":[{"name":"myMsg01","occurrence":0,"value":"이영희 귀하"}],"filledCount":1,"notFound":[],"output":"verify_out\\0002.hwp","outputFormat":"hwp5","row":1,"schemaVersion":"1.0","source":"samples/form-01.hwp","verify":{"diffCount":0,"identical":true}}
```

명단이 커질수록 재파싱 비용도 같이 커진다 — 대량 발송 직전 한 번, 표본만 뽑아
검증하는 절충도 가능하다(예: 전체는 `--verify` 없이 돌리고, 처음 몇 건만 별도로
`--verify`를 켜서 재확인).

## 6단계 — 처리 속도 조절: `--threads`

기본은 CPU 코어 수에 맞춰 스레드를 자동으로 정한다(이 머신에서는 `threads=32`로
나왔다). 다른 작업과 CPU를 나눠 써야 하면 상한을 직접 건다.

```bash
rhwp batch fill --form samples/form-01.hwp --data row1.jsonl --out-dir batch_out_t2 --threads 2 --json
```

실측 출력(요약 줄만 다름을 확인 — 각 행 JSON은 위와 동일 구조):

```
batch fill: 2행 중 2 성공, 0 실패 (2ms, threads=2)
```

## 7단계 — CSV 안에 쉼표가 들어간 값도 안전하다

이름 뒤에 직함을 붙이는 등 값 자체에 쉼표가 들어가는 경우, CSV 표준대로 큰따옴표로
감싸면 정상 처리된다.

```bash
printf 'myMsg01\n"김철수, 대표"\n' > comma_rows.csv
```

```bash
rhwp batch fill --form samples/form-01.hwp --data comma_rows.csv --out-dir comma_out --json
```

실측 출력 — `value`에 쉼표가 그대로 살아 있다:

```json
{"ambiguous":[],"changedPages":[0],"confusable":[],"dryRun":false,"filled":[{"name":"myMsg01","occurrence":0,"value":"김철수, 대표"}],"filledCount":1,"notFound":[],"output":"comma_out\\0001.hwp","outputFormat":"hwp5","row":0,"schemaVersion":"1.0","source":"samples/form-01.hwp","verify":null}
```

값을 직접 조립해 CSV를 만드는 스크립트를 짤 때는 큰따옴표 이스케이프를 CSV
표준 라이브러리(파이썬 `csv` 모듈, Node `csv-stringify` 등)에 맡기고 문자열을
손으로 이어붙이지 않는다 — 손으로 이어붙이면 값 안의 쉼표·줄바꿈·따옴표가 열
경계를 깨뜨릴 수 있다.

## 실패했을 때 무엇을 보고 어떻게 고칠지

| 신호 | 원인 | 처방 |
|---|---|---|
| `오류: 서식을 읽을 수 없습니다 - ...` (종료 코드 1) | `--form` 경로가 잘못됨 | 경로를 다시 확인 — 이 저장소 표본으로 재현: `rhwp batch fill --form samples/no-such-form.hwp --data row1.jsonl --out-dir x --json` → `지정된 파일을 찾을 수 없습니다.` |
| `오류: --data 에 데이터 행이 없습니다 - ...` (종료 코드 2) | CSV/JSONL에 헤더만 있고 실제 데이터 행이 0개 | 데이터 파일에 최소 1행 이상 있는지 확인 |
| 행 JSON의 `notFound`에 필드명이 남음(단, `--name-field`로 지정한 컬럼 제외) | 데이터 컬럼/키가 실제 필드 이름과 다름 | `fields --json`의 `name`을 그대로 복사해 쓴다 — 레시피 1 참고 |
| `--name-field`로 쓴 컬럼이 매 행 `notFound`에 뜸 | 정상 동작 — 그 컬럼은 파일명 용도로만 쓰였을 뿐 서식 필드가 아님 | 자동화 게이트에서 그 컬럼명은 `notFound` 비교 대상에서 제외 |
| `ambiguous`가 비어 있지 않음 | 같은 이름의 필드가 문서에 여러 번 있음 | [서식 자동화 심화 가이드 1-1절](../form_filling_guide.md#1-1-반복-필드--같은-이름이-여러-번-나올-때-이름n) |
| `--verify`의 `identical: false` | 특정 행에서만 재파싱 결과가 다름 | 해당 행만 `--verify` 없이 재실행 후 `export-svg`로 육안 확인, [레시피 6](06_visual_regression_before_after.md)의 `render-diff`로 정량화 |
| 전체 처리 시간이 예상보다 김 | 스레드 상한이 낮거나 문서가 큼 | `--threads`를 늘리거나(코어 수 이내), 우선 `--dry-run`으로 병목이 채움 자체인지 디스크 I/O인지 먼저 좁힌다 |
| `--data`에 헤더만 있고 실제 값 행이 0개 | 빈 CSV/JSONL을 그대로 넘김 — 이 저장소 표본으로 재현: `printf 'myMsg01\n\n' > empty.csv` 후 `batch fill --data empty.csv` | 데이터 생성 스크립트가 빈 결과를 냈는지 먼저 확인 — 명단 조회 쿼리가 0건을 돌려줬을 가능성 |

## 파이프라인 게이트로 묶기

행 전체가 성공했는지 사람이 로그를 읽지 않고 기계로 판정하려면, NDJSON
출력을 `jq`로 좁힌다:

```bash
rhwp batch fill --form samples/form-01.hwp --data row1.jsonl --out-dir final_out --verify --json \
  | jq -es '
      map(select(.notFound != ["outname"] and (.notFound|length>0 or .ambiguous|length>0 or (.verify != null and .verify.identical==false))))
      | if length==0 then "OK" else error("실패 행 \(length)건") end
    '
```

이 게이트는 `notFound`/`ambiguous`가 있거나 `--verify`가 `identical: false`를
반환한 행만 걸러내고, 하나도 없으면 `"OK"`를 반환한다. 실패 행이 있으면 어떤
행(`row` 번호)이 실패했는지 원본 NDJSON을 다시 훑어 확인한다 — 요약 줄의 "N행
중 M 성공"만 보고 넘어가면 어떤 행이 문제였는지 알 수 없다.

## 이 레시피가 다루지 않는 것

- **서식 자체를 행마다 바꾸는 것**은 지원 범위 밖이다 — `--form`은 명령 하나당
  서식 파일 하나만 받는다. 서식이 여러 종류라면 서식별로 `batch fill`을 따로
  실행하고 `--out-dir`을 분리한다.
- **도장/서명 이미지 삽입이나 메타데이터 제거는 이 명령이 하지 않는다** — 각
  산출물에 그런 후처리가 필요하면, `batch fill`이 만든 파일 목록(각 행 JSON의
  `output`)을 후속 스크립트로 순회하며 개별 명령을 적용한다. 예를 들어
  산출물을 한 번 더 재파싱해 페이지 수·표 구조를 정량 비교하고 싶다면 [레시피
  6](06_visual_regression_before_after.md)의 `render-diff`를 그 목록에 대해
  반복 실행한다.
- **출력 폴더 정리는 호출자 몫이다** — `--out-dir`이 이미 존재하고 같은
  이름의 파일이 있으면 덮어쓴다(멱등하게 재실행 가능하다는 뜻이기도 하다).
  행 번호가 아니라 `--name-field`로 이름을 지정했는데 데이터에 중복 값이
  있으면 나중 행이 먼저 행의 산출물을 덮어쓴다 — 명단에 중복이 없는지는 이
  명령이 검사해주지 않는다.

## 요약

`batch fill`은 "서식 하나 + 데이터 N행"을 산출물 N개로 바꾸는 명령이다. 판정
규칙은 [레시피 1](01_fill_form_and_submit.md)의 `edit fill-fields`와 완전히
같다 — 행별 JSON의 `notFound`/`ambiguous`/`verify`를 그대로 게이트에 쓰면
된다. 이 저장소 표본으로 실제 확인한 것: JSONL·CSV 둘 다 동일 결과, 쉼표가
든 값도 CSV 표준 인용으로 안전, `--name-field`는 파일명 용도지만 `notFound`
판정에 같이 잡히는 함정이 있음, 빈 데이터 파일은 종료 코드 2로 즉시 거부됨,
`--dry-run`/`--verify`/`--threads`는 [레시피 1](01_fill_form_and_submit.md)의
단일 파일 옵션과 의미가 같다.

## 관련 문서

- [레시피 1 — 서식 문서를 채워서 제출용으로 만들기](01_fill_form_and_submit.md) —
  단일 문서 기준 `edit fill-fields`의 판정 규칙(`notFound`/`ambiguous`/
  `--verify`)의 canonical 문서. `batch fill`의 행별 결과도 같은 규칙을 따른다.
- [레시피 4 — 출처를 모르는 문서를 처음 열 때](04_safety_check_untrusted_doc.md) —
  대량 발송 전 서식 자체를 재사용한다면, `fields --json`의 `textSecurity`부터
  확인한다.
- [레시피 6 — 편집 전후를 눈이 아니라 숫자로 비교하기](06_visual_regression_before_after.md) —
  `--verify`가 못 잡는 렌더링 차이를 `render-diff`로 정량화할 때.
- [서식 자동화 심화 가이드](../form_filling_guide.md) — 반복 필드·`ambiguous`
  판정 규칙의 canonical 문서.
- [CLI JSON 파이프라인 가이드](../cli_json_pipeline_guide.md) — stdout
  NDJSON 규약과 `batch` 파이프라인의 실측 시나리오.
- [CLI 명령어 매뉴얼](../cli_commands.md) — `batch fill`의 전체 옵션과 종료
  코드 계약.
- [에이전트 실무 대체 예제집](../agent_task_playbook.md) — 사람 업무 관점에서
  "명단 → 개별 문서 N개"가 실제 업무 시나리오 어디에 대응하는지 더 넓은
  맥락.
