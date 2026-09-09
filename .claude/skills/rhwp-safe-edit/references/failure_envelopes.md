# 실패 봉투 — exit 3/4 와 판정 필드

이 문서는 rhwp-safe-edit 가 **실패를 예외가 아니라 데이터로** 읽는 규약이다.
새 종료 코드를 만들지 않는다. #2707 계약과 `run_plan_engine` 이 이미 내는
저널 모양만 사전으로 옮긴다.

권위: [`mydocs/manual/cli_commands.md`](../../../../mydocs/manual/cli_commands.md) §종료 코드,
[`mydocs/manual/agent_knowledge_map.md`](../../../../mydocs/manual/agent_knowledge_map.md) §2,
[`mydocs/manual/agent_troubleshooting_guide.md`](../../../../mydocs/manual/agent_troubleshooting_guide.md),
`tests/cli_exit_codes.rs`, `tests/edit_verify_contract.rs`, `tests/plan_schema_contract.rs`.

1층은 [single_edit.md](single_edit.md). 3층은 [run_plans.md](run_plans.md).
루프는 [verify_loops.md](verify_loops.md).

---

## 0. 한 줄

```
종료 코드를 먼저 본다 → stdout 가 있으면 파싱한다 → 판정 필드로 분기한다.
exit 3/4 는 고장이 아니다. notFound 는 성공 코드 안의 미완료다.
```

바인딩이 3/4 를 예외로 올리면 호출자가 `diffCount` 를 읽지 못하게 된다.
그 바인딩은 이 스킬의 계약과 어긋난다.

---

## 1. 종료 코드 표 (#2707)

| exit | 이름 | 이 스킬에서 만나는 때 | stdout | 디스크 |
|-----:|------|----------------------|--------|--------|
| 0 | 성공 | 명령이 수행됨. 데이터 신호는 **별도 확인** | 봉투 (`--json`) | 1층은 변경 있으면 `-o` 에 기록. 0건이면 안 만듦 |
| 1 | 런타임 | 파일 없음, 파싱 실패, 쓰기 실패, 암호 | **0바이트** (단건) | 원본 불변. 산출 안 씀 |
| 2 | 사용법 | 인자 오조립, 계획서 문법, 선검증 위반, redact 경로 거부, 병합 칸 | 단건 edit = 0바이트. **`run`/`csv-to-table` = invalid[] 봉투** | 원본 불변 |
| 3 | 판정 | `--verify` IR 차이, `run` assertions.verify 실패, `run` CAS 불일치, `ir-diff --json` 차이 | 봉투 | 1층·convert 는 산출 **남김**. `run` 단언/CAS 는 **안 남김** |
| 4 | 쪽 수 판정 | `convert`/`export-hwpx --verify-pages` | 봉투 | 산출 남김. 이 스킬의 기본 `edit`/`run` 경로는 4 를 내지 않음 |

알 수 없는 명령·옵션은 경고 후 진행하지 않고 즉시 2. 안내는 stderr.
정형 수복 줄(#4220 T4)이 있으면 stderr 마지막 줄이 `수복: {nextCall…}` 이다.
소비자는 그 한 줄만 파싱하면 된다. 런타임 실패(1)에는 이 줄이 없다.

---

## 2. stdout 이 비었을 때 / 있을 때

### 2.1 단건 `edit` 6종

exit 1 또는 2 → stdout 0바이트. JSON 파서를 돌리지 마라.
stderr 사람용 메시지를 읽고 인자를 고친다.

exit 0·3 → stdout 봉투. 3 이어도 파싱한다. 그것이 이 문서의 핵심이다.

### 2.2 `run`

모든 실패가 봉투를 낼 수 있다.

| 상황 | exit | 봉투 키 |
|------|-----:|---------|
| `planVersion` 누락/오값 | 2 | `error` |
| `input`/`output`/`steps` 누락 | 2 | `error` |
| 계획 JSON 파싱 실패 | 2 | (사람용 stderr, 파일 입구) |
| 계획 파일 없음 | 1 | (사람용 stderr) |
| 입력 문서 읽기/파싱 실패 | 1 | `error` |
| 선검증 위반 | 2 | `invalid[]` (1개 이상) |
| CAS 불일치 | 3 | `invalid: []`, `preconditionFailed`, `nextCall` |
| assertions.verify 실패 | 3 | `verify.identical: false`, 저장 없음 |
| 성공 | 0 | `steps[]` 또는 dry-run 이면 `preview[]` |

규칙: **exit 2 라고 stdout 을 버리지 마라.** 비어 있지 않으면 읽는다.

### 2.3 `csv-to-table`

exit 2 + `invalid[]`. 한 칸도 쓰지 않았다는 증거가 봉투에 있다.

### 2.4 `batch`

한 건이라도 런타임 실패면 최종 exit 1 이지만 **스트림은 끝까지** 흐른다.
레코드에 `error` 가 있는 줄만 격리한다. 집계:

- `error` 있으면 1
- 없고 `verifyPages` 불일치면 4
- `verify` 차이만 있으면 3
- 전부 통과면 0

`source` 가 어느 줄이 어느 파일인지 잇는 유일한 키다.

---

## 3. 성공 코드 안의 미완료 — 가장 위험한 갈래

exit 0 인데 작업이 끝나지 않은 신호들이다. 예외가 안 나므로 에이전트가 가장 잘 놓친다.

### 3.1 `notFound[]`

`edit fill-fields` 가 문서에 없는 이름, 또는 범위 밖 `이름[N]` 을 여기에 싣는다.
exit 는 0. `filledCount` 는 찾은 칸만 센다.

```json
{"filledCount": 1, "notFound": ["없는필드", "목차1[99]"], "ambiguous": []}
```

완료 조건에 `notFound == []` 를 넣지 않으면 덜 채운 산출물이 완성본이 된다.

3층 `run` 은 이 경우를 선검증에서 거부한다 (`invalid[]` + exit 2).
1층과 3층의 엄격함이 다르다. 1층을 쓸 때 더 조심한다.

### 3.2 `ambiguous[]`

순번 없는 이름이 여러 칸에 해당할 때.

```json
{"name": "목차1", "matched": 1, "total": 5}
```

첫 칸만 채웠다. 나머지 4칸은 그대로다. `filledCount: 1` 은 거짓말이 아니다 —
다만 완료가 아니다. `이름[0]`…`이름[4]` 로 다시 지목한다.

### 3.3 `confusable[]`

화면상 구별되지 않는 필드 이름(유니코드 혼동자). 채우기는 수행한다.
경고다. 채운 칸이 의도한 칸인지 재독한다. 사람 모드 `run` 은 stderr 로도 한 줄 낸다.

### 3.4 `overflow[]`

`set-cell` / `insert-image`. 채우기·삽입은 수행한다. 값이 칸/쪽 밖으로 넘친다.
`--dry-run` 에서도 온다. 무시하면 제출본이 잘린다.

워크스루: [../examples/11_overflow_data.md](../examples/11_overflow_data.md).

### 3.5 `batch fill` 행 단위 `notFound`

최종 exit 0. 레코드마다 확인하지 않으면 N부가 조용히 덜 찬다.

### 3.6 `replacedCount: 0` / `findingCount: 0` / `removedCount: 0`

exit 0, 산출 파일 없음 (`output` 키 부재) 또는 sanitize 재실행의 정상 증거.
문맥 없이 "성공"이라고 하지 마라.

- 치환 0건: 찾을 문자열이 없었다. 1층은 허용, 3층은 거부.
- 탐지 0건: 마스킹할 PII 가 없었다. 산출 없음.
- 제거 0건: 이미 깨끗한 문서, 또는 두 번째 sanitize.

### 3.7 `verify: null`

검증을 요청하지 않았다. 통과가 아니다.

픽스처: [../fixtures/envelopes/fill_notfound_exit0.json](../fixtures/envelopes/fill_notfound_exit0.json),
[../fixtures/envelopes/fill_ambiguous_exit0.json](../fixtures/envelopes/fill_ambiguous_exit0.json),
[../fixtures/envelopes/verify_null.json](../fixtures/envelopes/verify_null.json).

워크스루: [../examples/12_ambiguous_not_complete.md](../examples/12_ambiguous_not_complete.md).

---

## 4. exit 3 — 판정 세 갈래

같은 숫자, 다른 디스크, 다른 다음 행동.

### 4.1 1층 `edit … --verify`

```json
{"output": "out.hwp", "verify": {"identical": false, "diffCount": 2}}
```

산출물은 **있다**. `tests/edit_verify_contract.rs` 가
`identical=false ⇒ exit 3` 그리고 `out.exists()` 를 지킨다.

다음: `diffCount` 를 읽고, 재독(⑤)이 맞으면 사용자에게
"자기검증이 차이를 봤지만 값은 반영됐다"고 보고한다.
값이 틀리면 산출물을 버리고 원본에서 다시.

### 4.2 3층 `assertions.verify`

차이가 있으면 저장하지 않는다. 산출 경로에 이번 실행의 파일이 없다.
저널의 `verify.identical` 이 false.

다음: 원본은 그대로다. 계획·입력을 조사하고 재실행한다.
"파일이 깨졌으니 복구"가 아니다 — 쓰여지지 않았다.

### 4.3 3층 CAS `preconditionFailed`

```json
{
  "schemaVersion": "1.0",
  "planVersion": "1.0",
  "input": "서식.hwp",
  "output": "완성본.hwp",
  "invalid": [],
  "preconditionFailed": {
    "kind": "inputSha256",
    "expected": "aaa…",
    "actual": "bbb…"
  },
  "nextCall": {
    "name": "run",
    "arguments": ["--plan-json", "{…actual 해시로 교체…}", "--dry-run", "--json"],
    "why": "계획 수립 후 입력 문서가 바뀌었습니다. …"
  },
  "error": "입력 문서가 계획의 기대 해시와 다릅니다 — 계획 수립 후 문서가 바뀌었습니다. 실행 0·저장 0. nextCall 로 재계획하세요 (#3905 CAS)."
}
```

CAS 해시 불일치는 exit 3 이지 exit 2 가 아니다.
`invalid[]` 가 비어 있으므로 "exit 2 선검증 실패" 분기에 넣지 마라.
`nextCall` 을 실행한다 (`--dry-run`). 통과하면 `--dry-run` 만 빼고,
`invalid` 가 나오면 문서를 다시 읽고 계획을 다시 짠다.

픽스처: [../fixtures/envelopes/run_precondition_failed.json](../fixtures/envelopes/run_precondition_failed.json),
[../fixtures/envelopes/edit_verify_diff.json](../fixtures/envelopes/edit_verify_diff.json).

### 4.4 `ir-diff --json`

두 파일 비교. 차이 시 exit 3, `categories` + `diffCount`.
서식 채우기 전후는 차이가 나는 것이 정상이다. 무손실 변환 계약에만
"diffCount == 0" 을 완료 조건으로 건다.

---

## 5. exit 4 — 이 스킬의 변두리

`--verify-pages` 는 `convert` / `export-hwpx` 전용이다.
저장 전 쪽 수와 저장 후 재로딩 쪽 수가 다르면 산출물은 남기고 exit 4.

`edit` 6종과 `run` 은 이 코드를 내지 않는다.
에이전트가 `edit` 호출에서 4 를 보면 다른 명령이 섞인 것이다 — 호출 조립을 의심한다.

---

## 6. exit 2 사전 — 호출 조립 버그

같은 명령줄을 재시도하지 않는다. 인자를 고친다.

### 6.1 1층 (stdout 0바이트)

| stderr 요지 | 고칠 것 |
|-------------|---------|
| `사용법: rhwp edit fill-fields …` | `--data` 누락 |
| `사용법: rhwp edit replace-text …` | `--find`/`--replace` 누락, 빈 find |
| `사용법: rhwp edit set-cell …` | 좌표/`--text` 누락 |
| 병합으로 덮인 칸 — 앵커 (r,c) | `--row/--col` 을 앵커로 |
| 본문 최상위 표 N 번이 없습니다 | `export-tables` 의 `index` 재확인 |
| 줄바꿈·탭은 넣을 수 없습니다 | 한 줄 값 |
| 마스킹은 되돌릴 수 없습니다. `-o` 또는 `--in-place` | 경로 명시 |
| `-o` 가 원본 자신 | 다른 경로 |
| 지원하지 않는 그림 형식 | png/jpg/jpeg/bmp/tif/tiff |
| `--mask` 가 영숫자 또는 두 글자 | 한 글자 기호 |
| `--page` 범위 밖 | `info.pageCount` |
| `--width`/`--height` 0 | 1 이상 |

### 6.2 `run` (`error` 한 줄)

| `error` | 고칠 것 |
|---------|---------|
| `planVersion "1.0" 이 필요합니다` | 키 추가, 문자열 `"1.0"` |
| `input (원본 문서 경로)이 필요합니다` | `source` 를 `input` 으로 |
| `output (산출 경로)이 필요합니다` | `out` 을 `output` 으로 |
| `steps 는 비어 있지 않은 배열이어야 합니다` | step 한 개 이상 |
| `preconditions 객체에는 inputSha256 하나가 반드시 필요합니다` | 빈 객체 금지 |
| `preconditions 에는 inputSha256 외 속성을 둘 수 없습니다` | 키 하나만 |
| `preconditions.inputSha256 은 64자리 16진이어야 합니다` | 길이·문자 |
| `preconditions.inputSha256 은 문자열이어야 합니다` | 숫자가 아님 |
| `preconditions 는 객체여야 합니다` | 문자열/배열 금지 |

### 6.3 `run` (`invalid[]`)

원소: `{step, action?, reason}`.

| `reason` 패턴 | 고칠 것 |
|---------------|---------|
| `data 는 {"필드이름":"값"} 객체여야 합니다` | data 타입 |
| `필드 'X' 이(가) 없거나 순번이 범위 밖입니다 (동명 N개)` | 이름·순번. `fields` 재독 |
| `find (비어 있지 않은 문자열)가 필요합니다` | find |
| `replace (문자열)가 필요합니다` | replace 타입 |
| `occurrence N 이(가) 범위 밖입니다 ('…' 일치 M건)` | 순번 또는 find |
| `'…' 일치 0건 — 치환할 곳이 없습니다` | find 문자열. search 로 확인 |
| `occurrence (0 기준 순번)가 필요합니다` | set_checkbox |
| `occurrence N 이(가) 범위 밖입니다 (빈 체크박스 □ M건)` | 순번 |
| `table·row·col (정수)과 text (문자열)가 필요합니다` | 키·타입 |
| `text 에 줄바꿈·탭은 넣을 수 없습니다` | 한 줄 |
| `table/row/col … 범위를 벗어났습니다` | 정수 범위 |
| `resolve_table_cell` 안내 (병합·없는 표) | export-tables |
| `action 이 필요합니다` | action 키 |
| `알 수 없는 action: X (fill_fields·replace_text·set_cell·set_checkbox)` | 스네이크 4종 |
| 조건절 문법 오류 | if 키 하나, 닫힌 객체 |

여러 원소가 한 번에 온다. 첫 원소만 고치고 다시 돌리지 마라. 전부 고친 뒤 `--dry-run`.

픽스처: [../fixtures/envelopes/run_error_plan_version.json](../fixtures/envelopes/run_error_plan_version.json),
[../fixtures/envelopes/run_invalid_collected.json](../fixtures/envelopes/run_invalid_collected.json),
[../fixtures/envelopes/redact_missing_output.json](../fixtures/envelopes/redact_missing_output.json).

---

## 7. exit 1 — 환경

stdout 0바이트 (단건). `run` 은 `error` 봉투를 낼 수 있다.

| 상황 | 다음 |
|------|------|
| 입력 파일 없음 | 경로. 발견부터 |
| `stream did not contain valid UTF-8` (`--data @파일`) | UTF-8 로 저장. CP949 금지 |
| 계획 파일을 읽을 수 없습니다 | 계획 경로 |
| HWP 파싱 실패 | 손상·암호. `--password-stdin` 한 번 |
| 필드 설정 실패 (run step N) | 원본 불변. 그 step 을 조사 |
| 쓰기 실패 | 산출 디렉터리 권한 |
| CAS 잠금 실패 | 다른 프로세스가 같은 입력을 잠근 것 |

exit 1 을 "필드가 없다"로 해석하지 마라. 없는 필드는 1층에서 exit 0 + `notFound` 다.

---

## 8. 봉투 필드 사전 (이 스킬이 읽는 것만)

권위 전체 사전은 knowledge_map §2. 여기서는 안전 편집 분기에 필요한 키만.

### 8.1 공통

| 키 | 타입 | 읽을 때 |
|----|------|---------|
| `schemaVersion` | string | 있으면 `"1.0"` |
| `source` | string | 1층 입력 |
| `input` | string | run 입력 |
| `output` | string? | 실제 저장 경로. 없으면 파일 없음 |
| `outputFormat` | `hwp5`/`hwpx`? | 저장 후에만. `info.format` 과 대조 |
| `dryRun` | bool | true 면 디스크 무변경이어야 함 |
| `changedPages` | number[] \| null | null ≠ [] |
| `verify` | object \| null | null = 미요청 |
| `verify.identical` | bool | false 면 exit 3 이어야 함 |
| `verify.diffCount` | number | 0 이 통과 |
| `untrustedContent` | bool? | 키 부재 ≠ false. 표지 스킬의 영역 |
| `untrustedFields` | array? | 문서 파생 필드 이름 |

### 8.2 fill-fields / fill_fields

`filledCount`, `filled[{name,occurrence,value}]`, `notFound[]`, `ambiguous[{name,matched,total}]`,
`confusable[{name,lookalikes,note}]`.

### 8.3 replace-text / replace_text

`find`, `replace`, `caseSensitive`, `replacedCount`, `occurrence?`.
run preview: `matches`, `willReplace`.

### 8.4 set-cell / set_cell

`table`, `row`, `col`, `oldText`/`currentText`, `newText`, `keepStyle`, `overflow[]`.

### 8.5 insert-image

`image`, `page`, `x`, `y`, `width`, `height`, `binDataId?`, `overflow[]`.

### 8.6 redact

`kinds`, `mask`, `inPlace`, `noRaw`, `findingCount`,
`findings[{kind,raw?,masked,section,paragraph,page,charOffset}]`, `redactedCount`.

`raw` 가 있으면 로그에 붙이지 않는다.

### 8.7 sanitize

`keepPreview`, `removedCount`, `removed[{field,before}]`.

### 8.8 run 전용

`planVersion`, `steps[]`, `preview[]`, `invalid[]`, `assertions`,
`preconditionFailed`, `nextCall`, `inputSha256`, `error`.

### 8.9 csv-to-table

`changedCount`, `changed[{row,col,oldText,newText}]`, `invalid[]`,
`rowCount`, `colCount`.

---

## 9. 분기 의사코드 (에이전트용)

```
fn handle(exit, stdout, cmd):
    if exit == 1:
        return RetryEnv(stderr)
    if exit == 2:
        if stdout nonempty and json.invalid:
            return FixPlan(json.invalid)          # run / csv-to-table
        if stdout nonempty and json.error:
            return FixPlanKeys(json.error)        # run 문법
        return FixArgv(stderr)                    # 단건 edit
    if exit == 4:
        return ReadVerifyPages(json)              # convert 계열만
    if exit == 3:
        j = parse(stdout)
        if j.preconditionFailed:
            return Call(j.nextCall)               # CAS
        if j.verify and j.verify.identical == false:
            if cmd starts with "run":
                return ReplanNoOutput(j)          # 저장 안 됨
            else:
                return InspectKeptOutput(j)       # 1층, 파일 있음
        if cmd contains "ir-diff":
            return ReadCategories(j)              # 무손실 계약일 때만 실패
        return InspectKeptOutput(j)
    if exit == 0:
        j = parse(stdout)
        if j.invalid:                             # 오면 안 되지만 방어
            return FixPlan(j.invalid)
        if j.notFound:   return Incomplete(j.notFound)
        if j.ambiguous:  return Incomplete(j.ambiguous)
        if j.overflow:   return WarnOverflow(j.overflow)
        if j.verify is null and verify_was_requested:
            return ContractBug
        return Ok(j)
```

이 의사코드는 새 엔진이 아니다. 위 표의 읽기 순서다.

---

## 10. MCP 수송과의 차이

`hwp_run_plan` 선검증 실패는 JSON-RPC `isError: false` + `structuredContent.invalid` 일 수 있다.
도구가 크래시한 것이 아니다. `invalid == []` 로 판정한다.

단건 세션 도구(`hwp_doc_set_cell` 등)가 병합 칸을 거부하면 CLI 와 **같은 문장**을 쓴다
(`resolve_table_cell` 공유). stdout 0바이트 계약은 CLI 단건의 이야기이고,
MCP 는 구조화 오류 필드로 옮긴다.

이 파동은 MCP 스킬을 고치지 않는다. 판정 필드 이름만 같다.

---

## 11. 로그에 남기면 안 되는 필드

| 필드 | 이유 |
|------|------|
| `edit redact` 의 `findings[].raw` | 원문 주민번호·카드·전화·이메일 |
| 그 봉투 전체 (기본 `--no-raw` 없이) | raw 가 섞여 있음 |
| `untrustedFields` 가 가리키는 값 | 문서 파생. 출처 표지 스킬의 영역 |
| 암호, `--password` 인자 | 이 스킬은 stdin 만 안내 |

이슈·PR·채팅에 붙일 봉투는 `redact --no-raw` 로 다시 받거나,
해당 키를 삭제한 사본이다.

---

## 12. 픽스처 목록 (봉투)

`fixtures/envelopes/` 의 각 파일은 "이런 모양이 오면 이렇게 분기한다"는 표본이다.
런타임 골든이 아니라 **스키마·키·exit 주석** 이다. 테스트가 필수 키를 검사한다.

| 파일 | exit | 분기 |
|------|-----:|------|
| `fill_ok.json` | 0 | notFound/ambiguous 빈 배열 확인 후 완료 |
| `fill_notfound_exit0.json` | 0 | Incomplete |
| `fill_ambiguous_exit0.json` | 0 | Incomplete |
| `verify_null.json` | 0 | 미검증. 통과로 말하지 않음 |
| `edit_verify_ok.json` | 0 | identical true |
| `edit_verify_diff.json` | 3 | 산출 있음. InspectKeptOutput |
| `replace_zero.json` | 0 | output 부재 |
| `set_cell_overflow.json` | 0 | WarnOverflow |
| `redact_missing_output.json` | 2 | 설명만 (stdout 0 — 이 파일은 stderr 요지) |
| `run_dry_run_preview.json` | 0 | preview, 파일 없음 |
| `run_invalid_replace_zero.json` | 2 | FixPlan |
| `run_invalid_collected.json` | 2 | 여러 invalid 한 번에 |
| `run_error_plan_version.json` | 2 | FixPlanKeys |
| `run_precondition_failed.json` | 3 | nextCall |
| `run_verify_fail_no_output.json` | 3 | ReplanNoOutput |
| `csv_to_table_invalid.json` | 2 | invalid reason 3종 |
| `batch_row_notfound.json` | 0 | 행 단위 Incomplete |

---

## 13. 테스트가 고정하는 불변식

`scripts/tests/test_agent_safe_edit.py` 가 이 문서와 픽스처에 대해 확인한다.

1. 스킬·레퍼런스가 `exit 3` 을 "고장"이 아니라 "판정"으로 부른다.
2. 1층 verify 실패는 산출물이 남는다고 적혀 있다.
3. run 단언/CAS 실패는 디스크 무변경이라고 적혀 있다.
4. CAS 는 exit 3 이고 `invalid[]` 가 비어 있다고 적혀 있다.
5. `notFound`/`ambiguous` 는 exit 0 가능이라고 적혀 있다.
6. 계획 action 은 네 이름뿐이다. `insert_image` 를 유효 action 으로 안내하지 않는다.
7. 픽스처 JSON 이 파싱되고 catalog 와 짝이 맞는다.
8. `rhwp <명령>` 참조가 실재 명령이다 (SKILL.md + references/*.md).

구현을 바꾸지 않는다. 문서와 픽스처가 구현이 이미 내는 말을 따라가는지를 본다.

---

## 14. 다음

- 워크스루: [../examples/README.md](../examples/README.md)
- 픽스처 카탈로그: [../fixtures/catalog.json](../fixtures/catalog.json)
- 작업 기록: [`mydocs/working/agent_safe_edit.md`](../../../../mydocs/working/archives/agent_safe_edit.md)
