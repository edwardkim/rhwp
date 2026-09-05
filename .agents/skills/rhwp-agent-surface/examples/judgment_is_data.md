# 레시피 — 판정을 오류로 읽지 않기

픽스처: [`../fixtures/envelopes/`](../fixtures/envelopes/).

## `identical:false`

```bash
rhwp ir-diff samples/추진일정.hwp out/추진일정.hwpx --json
# exit=3, identical:false, diffCount:2
```

MCP `hwp_ir_diff` 는 `isError:false`.
차이를 실패로 재시도하지 않는다. `categories` 를 읽고 의도한 차이인지 본다.

## `replacedCount:0`

```bash
rhwp edit replace-text samples/hwp3-sample.hwp --find 존재하지않는문자열ZZZ --replace X -o out/rep0.hwp --json
# exit=0, replacedCount:0, 파일 없음, output 키 없음
```

`output` 을 열기 전에 `replacedCount > 0`.

## `notFound` / `ambiguous`

```bash
rhwp edit fill-fields samples/field-01.hwp --data '{"회사명":"페타플로","목차1":"개요","없는필드":"x"}' --dry-run --json
# exit=0, filledCount:2, notFound:["없는필드"], ambiguous:[{name:목차1,total:5}]
```

완료 조건: 두 배열이 빈다. `목차1[2]` 처럼 순번으로 지목.

## `invalid[]` 계획

```bash
rhwp run out/plan.json --dry-run --json
# exit=2, invalid:[{step,reason}], 실행 0
```

MCP `hwp_run_plan` 은 같은 내용을 `isError:false` 로 줄 수 있다.
`invalid != []` 를 성공으로 읽지 마라.
