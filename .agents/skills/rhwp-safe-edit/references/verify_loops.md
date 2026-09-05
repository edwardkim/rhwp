# 검증 루프 — dry-run · `-o` · `--verify` · 재독

이 문서는 rhwp-safe-edit 가 **한 편집을 끝냈다고 말하기 전에** 돌리는 루프다.
새 검증 엔진을 만들지 않는다. 이미 있는 `--dry-run`, `-o`, `--verify`,
`ir-diff`, `search`, `fields`, `export-tables`, `export-svg` 를 순서대로 배선한다.

권위: [`mydocs/manual/agent_surface_playbook.md`](../../../../mydocs/manual/agent_surface_playbook.md) §9,
[`mydocs/manual/cli_commands.md`](../../../../mydocs/manual/cli_commands.md) §종료 코드,
`tests/edit_verify_contract.rs`, `tests/changed_pages_contract.rs`.

1층 조립은 [single_edit.md](single_edit.md). 3층 계획서는 [run_plans.md](run_plans.md).
exit 3/4 를 데이터로 읽는 법은 [failure_envelopes.md](failure_envelopes.md).

---

## 0. 루프 한 장

```
① 발견        fields / export-tables / search / info
② 선확인      같은 명령줄 + --dry-run --json     → 디스크 무변경
③ 실행        --dry-run 을 떼고 -o · --verify     → 원본과 다른 경로
④ 판정        종료 코드 + 봉투 필드               → 예외로 throw 하지 않음
⑤ 재독        쓰기와 쌍인 읽기 명령               → 값이 맞는지
⑥ 눈검증      changedPages 쪽만 export-svg        → null 이면 아직 하지 않음
⑦ (선택)      ir-diff 원본 vs 산출                → 무손실 계약일 때만
```

어느 단계에서든 실패하면 원본은 그대로다. 산출 경로만 버리거나 덮어쓴다.

"실행과 같은 명령줄에서 `--dry-run` 하나만 빼면 되는 것"이 선확인의 정의다.
dry-run 과 실행의 인자가 갈라지면 선확인이 아니다.

---

## 1. ① 발견 — 주소를 쓰기 전에 읽는다

쓰기는 발견이 준 좌표만 받는다. 이름을 기억해서 넣지 않는다.

| 목표 | 발견 명령 | 쓰는 필드 |
|------|-----------|-----------|
| 누름틀 채우기 | `rhwp fields <파일> --json` | `fields[].name`, 동명 개수 = 순번 범위 |
| 표 칸 | `rhwp export-tables <파일> --json` | `tables[].index`, `cells[].row/col` |
| 문구 치환 | `rhwp search <파일> <검색어> --json` | `matchCount`, 쪽·문단 |
| 쪽 범위 | `rhwp info <파일> --json` | `pageCount`, `format` |
| 체크박스 | `rhwp search <파일> □ --json` | `matchCount` = `set_checkbox` 범위 |

발견 결과를 프롬프트에 통째로 붙여 다음 도구 이름을 바꾸지 않는다.
이름·좌표·건수만 골라 명령 인자를 만든다.

암호 문서는 `--password-stdin`. 발견이 exit 1 이면 쓰기를 시작하지 않는다.

---

## 2. ② 선확인 — `--dry-run --json`

### 2.1 1층

```bash
rhwp edit fill-fields 서식.hwp --data @row.json --dry-run --json
rhwp edit replace-text 공문.hwp --find "2025년" --replace "2026년" --dry-run --json
rhwp edit set-cell 양식.hwpx --table 12 --row 1 --col 1 --text "1,234" --dry-run --json
rhwp edit insert-image 서식.hwp --image 도장.png --page 0 --x 50000 --y 70000 --dry-run --json
rhwp edit redact 계약서.hwp --dry-run --no-raw --json
```

읽는 것:

- `dryRun` 이 true 인가. 아니면 실행해 버린 것이다 — 중단.
- `notFound` / `ambiguous` / `overflow` / `findingCount` / `replacedCount`.
- `output` 키가 **없어야** 한다. 있으면 dry-run 이 아니다.
- 산출 경로를 `test -e` 로 확인한다. 파일이 생겼으면 계약 위반이다. 사용자에게 알린다.

`edit sanitize` 에는 `--dry-run` 이 없다. 선확인 대신 복사본에 적용하고
`removed[]` 를 읽은 뒤, 본문 `export-text` 전후가 같은지 본다.

### 2.2 3층

```bash
rhwp run plan.json --dry-run --json
```

읽는 것:

- `invalid[]` 가 비었는가. 아니면 계획을 고치고 ② 로 돌아간다.
- `preview[]` 의 step 수 = 계획서 steps 수 (건너뛴 step 포함).
- `skipped: true` 인 step 이 의도한 것인가.
- `preview[].targets[].sameNameCount` > 1 이면 순번을 붙일지 결정한다.
- `willReplace` / `matches` 가 기대와 같은가.
- 산출 경로에 파일이 생기지 않았는가.

dry-run 의 exit 는 성공이면 0, 선검증 실패면 2. exit 2 여도 stdout 에 봉투가 있다.

### 2.3 `changedPages: null`

dry-run 은 재조판을 하지 않으므로 `changedPages` 는 항상 `null` 이다.
`null` 은 "바뀐 쪽 없음"이 아니다. 빈 배열 `[]` 이 "바뀐 쪽 없음"이다.

눈검증(⑥)을 dry-run 직후에 하지 마라. 저장 후 배열이 온 다음에 한다.

---

## 3. ③ 실행 — `-o` 와 `--verify`

### 3.1 산출 분리

```bash
rhwp edit fill-fields 서식.hwp --data @row.json -o 완성본.hwp --verify --json
rhwp run plan.json --json
```

`-o` / 계획서 `output` 은 입력과 **다른 경로**다.
작업 디렉터리 루트에 `*_filled.hwp` 기본 이름을 만들지 않으려면 항상 명시한다.

`redact` 는 `-o` 또는 `--in-place`. 에이전트 기본은 `-o`.

### 3.2 `--verify` 를 붙이는 때

에이전트 기본은 붙인다. 빠지면 봉투의 `verify` 가 `null` 이고,
그것은 통과가 아니라 **검증을 안 한 것**이다.

```
$ rhwp edit fill-fields samples/field-01.hwp \
    --data '{"회사명":"페타플로","작성자":"홍길동"}' -o out/field-filled.hwp --json
{ … "output":"out/field-filled.hwp","outputFormat":"hwp5","verify":null}
```

붙이면:

```
"verify": {"diffCount": 0, "identical": true}
```

`run` 은 계획서 `assertions.verify: true` 가 같은 스위치다.
CLI 플래그 `--verify` 가 `run` 에 있는 것이 아니다. 계획서에 적는다.

### 3.3 1층 verify 와 3층 verify 의 디스크

| 경로 | 차이 시 산출물 | exit |
|------|----------------|-----:|
| `edit … --verify` | **남는다** | 3 |
| `run` + `assertions.verify` | **남기지 않는다** | 3 |
| `convert` / `export-hwpx --verify` | 남는다 | 3 |
| `convert` / `export-hwpx --verify-pages` | 남는다 | 4 |

1층에서 exit 3 이 나왔어도 산출물은 있다. 버리기 전에 `verify.diffCount` 와
재독(⑤)을 보고 판단한다. "exit 3 = 실패 = 파일 없음" 으로 지우지 마라.

3층에서 exit 3 이면 산출 경로를 열지 마라. 파일이 없거나 이전 잔재다.

워크스루: [../examples/10_verify_exit3.md](../examples/10_verify_exit3.md).

---

## 4. ④ 판정 — 종료 코드를 먼저

```
code = process.exitCode
if code == 1:
    stdout 는 0바이트. 환경·경로·암호를 본다. 재시도는 원인을 고친 뒤.
elif code == 2:
    단건 edit 는 stdout 0바이트. run / csv-to-table 은 invalid[] 를 읽는다.
    같은 명령줄을 재시도하지 않는다.
elif code in (0, 3, 4):
    stdout JSON 을 파싱한다. 아래 완료 식을 평가한다.
```

완료 식은 명령마다 다르다. 공통으로:

- `schemaVersion` 이 있다.
- 1층이면 `dryRun` 이 false (실행했는데 true 면 잘못 붙인 것).
- `notFound` / `ambiguous` / `invalid` 가 있으면 비었는지 본다.
- `verify` 가 객체면 `identical` 과 `diffCount` 를 읽는다.
  `identical == false` 인데 code == 0 이면 계약 위반이다 (`tests/edit_verify_contract.rs`).

`filledCount > 0` 만으로 완료하지 않는다.

---

## 5. ⑤ 재독 — 쓰기와 쌍

저장이 끝난 산출물에 대해 **같은 주소로** 다시 읽는다.

### 5.1 fill-fields

```bash
rhwp fields 완성본.hwp --json \
  | jq -c '[.fields[]|select(.name=="회사명" or .name=="작성자")|{name,value}]'
```

기대: data 에 넣은 값이 `value` 에 있다. 동명 필드는 `이름[N]` 으로 지목한
순번만 바뀌고 나머지는 입력과 같다.

### 5.2 replace-text

```bash
rhwp search 개정본.hwp "2025년" --json | jq .matchCount
```

전건 치환이면 0. `--occurrence k` 이면 원본 `matchCount - 1`.
`replacedCount` 와 `matchCount` 감소가 맞아야 한다.

### 5.3 set-cell

```bash
rhwp export-tables 작성본.hwpx --json \
  | jq -r '.tables[]|select(.index==12)|.cells[]|select(.row==1 and .col==1).text'
```

기대: 봉투의 `newText`. 이웃 칸은 `old` 그대로.

### 5.4 insert-image

```bash
rhwp info 제출본.hwp --json | jq '{pageCount, format}'
rhwp export-svg 제출본.hwp -o /tmp/svg -p 0 --json
```

`overflow` 가 비었는지, 해당 쪽 SVG 가 생겼는지. 픽셀 대조는 이 스킬의 일이 아니다.
시각 회귀는 rhwp-visual-regression 스킬로 넘긴다.

### 5.5 redact

```bash
rhwp edit redact 공개본.hwp --dry-run --no-raw --json | jq .findingCount
```

같은 `--kind` 로 0 이어야 한다. 원문에 `--no-raw` 없이 재스윕하지 마라.

### 5.6 sanitize

```bash
rhwp edit sanitize 배포본.hwp -o /tmp/재확인.hwp --json | jq .removedCount
```

기대 0. 본문은

```bash
rhwp export-text 원본.hwp
rhwp export-text 배포본.hwp
```

두 결과가 같아야 한다.

### 5.7 run 다단계

저널 `steps[]` 를 순회하며 각 action 에 위 재독을 적용한다.
`skipped: true` 인 step 의 대상은 원본과 같아야 한다.

한 step 의 재독이 실패하면 산출물을 완성본으로 넘기지 않는다.
원본은 살아 있으므로 계획서를 고치고 ② 부터.

---

## 6. ⑥ 눈검증 — `changedPages` 만

편집 봉투는 재조판 후 **0 기준 쪽 번호**를 준다.

실측 (`agent_surface_playbook` §9-3):

```
$ rhwp edit replace-text samples/hwp3-sample.hwp --find 의 --replace 의 -o out/rep1.hwp --json
{"changedPages":[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14],"replacedCount":276, …}

$ rhwp edit replace-text samples/hwp3-sample.hwp --find 의 --replace ★ --occurrence 3 -o out/rep2.hwp --json
{"changedPages":[0],"occurrence":3,"replacedCount":1, …}
```

전건은 15쪽, occurrence 3 은 0쪽 하나. **그 쪽만** 렌더한다.

```bash
rhwp export-svg out/rep2.hwp -o out/svg -p 0 --json
```

규칙:

- `changedPages` 가 배열이면 그 원소만 `-p`.
- `[]` 이면 바뀐 쪽이 없다. 렌더할 필요가 없다. 재독(⑤)만 확인한다.
- `null` 이면 확정 불가. dry-run 이거나 조판을 못 한 것. 전체를 보지 말고
  먼저 왜 null 인지 본다. 저장 전 null 이면 저장 후에 다시 받는다.
- 쪽 번호는 0 기준이다. `export-svg -p` 도 0 기준이다. 사용자에게 말할 때
  "1쪽"으로 바꾸려면 +1 하고, 명령에는 0 을 유지한다.

세션 도구 `hwp_doc_render_page` 는 같은 루프를 상수 비용으로 닫는다.
이 파동은 MCP 스킬을 고치지 않는다. CLI `export-svg -p` 가 동등한 1층이다.

---

## 7. ⑦ `ir-diff` — 무손실이 계약일 때만

`--verify` 통과가 "원본과 완전 동일"이 아니다. 자기 재파싱 게이트다.

실측:

```
$ rhwp export-hwpx samples/추진일정.hwp out/추진일정.hwpx --verify --json
{"verify":{"diffCount":0,"identical":true}, …}
exit=0

$ rhwp ir-diff samples/추진일정.hwp out/추진일정.hwpx --json
{"categories":{"cc":1,"char_offsets[0]: A=32 vs B=16":1},"diffCount":2,"identical":false, …}
exit=3
```

두 판정은 같은 대상을 보지 않는다.

- `--verify`: 저장 직후 산출물을 재파싱해 **자기 자신**과 대조.
- `ir-diff`: 두 파일을 직접 비교.

무손실이 계약이면 **둘 다** 돌리고 `categories` 를 읽어 판단한다.
일반 서식 채우기는 원본과 산출이 달라야 한다 (`ir-diff` 차이 존재 = 정상).
이 경우 `ir-diff` 를 "실패"로 보고하지 마라.

`ir-diff` 는 `--json` 없이 차이가 있어도 exit 0 일 수 있다.
자동화에서는 반드시 `--json` 을 붙인다.

---

## 8. `batch fill` 루프

대량은 전행 dry-run 이 선확인이다.

```
$ rhwp batch fill --form samples/field-01.hwp --data out/rows.jsonl \
    --out-dir out/merge --name-field 회사명 --dry-run --json
{"dryRun":true,"filledCount":3,"notFound":[],"output":"out/merge\\페타플로.hwp","row":0, …}
{"dryRun":true,"filledCount":3,"notFound":[],"output":"out/merge\\가나다.hwp","row":1, …}
{"dryRun":true,"filledCount":2,"notFound":["없는필드"],"output":"out/merge\\라마바.hwp","row":2, …}
batch fill: 3행 중 3 성공, 0 실패 (…, dry-run)
exit=0
```

세 번째 행에 `notFound` 가 있는데 **exit 0**. 데이터 품질 문제는 실행 실패가 아니다.
행 단위로 `notFound` 를 확인하지 않으면 조용히 덜 채운 산출물 N개가 나온다.

루프:

1. `--dry-run --json` 으로 NDJSON 을 받는다.
2. `notFound` 가 있는 `row` 를 고친다 (데이터 또는 매핑).
3. 전행이 깨끗해진 뒤에 `--dry-run` 을 떼고 실행한다.
4. 실행 NDJSON 을 다시 훑어 같은 완료 식을 적용한다.
5. 표본 한 행만 `fields` 재독 + `changedPages` 눈검증.

워크스루: [../examples/14_batch_fill_row_judgment.md](../examples/14_batch_fill_row_judgment.md).

---

## 9. 원본 불변을 루프에서 증명하기

에이전트가 "원본을 안 건드렸다"고 말하려면 주장이 아니라 대조가 있어야 한다.

```bash
# 실행 전
cp 서식.hwp /tmp/orig.hwp
# 또는
sha256sum 서식.hwp > /tmp/orig.sha

# ② ③ 수행

# 실행 후
cmp 서식.hwp /tmp/orig.hwp          # 바이트 동일
# 또는
sha256sum -c /tmp/orig.sha
```

Windows 에서는 `Get-FileHash` 로 같은 대조를 한다.
테스트는 이 불변식을 이미 지킨다 (`edit_fill_fields_contract.rs` —
dry-run 은 산출 경로를 만들지 않고, 실패 시 출력 파일을 쓰지 않는다).

`run` 선검증 실패·단언 실패·CAS 실패는 산출 경로를 만들지 않는다.
이전 실행의 잔재가 그 경로에 있으면 이번 실패의 산출물이 아니다.
돌리기 전에 산출 경로를 지운다.

워크스루: [../examples/15_original_untouched.md](../examples/15_original_untouched.md).

---

## 10. 세션 루프 (참고, 이 스킬은 CLI 가 정본)

```
hwp_open → hwp_doc_fill_fields / hwp_doc_replace_text / …
         → changedPages 를 읽어 hwp_doc_render_page
         → hwp_doc_save {verify:true}
         → hwp_close
```

`hwp_doc_save` 전에는 디스크가 바뀌지 않는다.
저장 후에도 핸들은 열려 있어 이어서 편집·재저장할 수 있다.
세션 안의 연속 편집은 메모리에서 이어지므로 1층을 이어 붙이는 반쪽 파일 문제가
디스크에는 없다. 그러나 저장 한 번에 단언을 걸려면 결국 `hwp_run_plan` 또는
`hwp_doc_save --verify` 다.

이 파동은 MCP 스킬 본문을 수정하지 않는다. 위 줄은 CLI 루프와 동형이라는
표지일 뿐이다.

---

## 11. 루프 픽스처

기계가 읽는 루프 정의는 [../fixtures/loops/](../fixtures/loops/) 다.

| 파일 | 뜻 |
|------|-----|
| `layer1_fill.json` | 발견→dry-run→실행→재독 (fill-fields) |
| `layer1_replace.json` | 치환 + search 재독 |
| `layer1_set_cell.json` | export-tables 왕복 |
| `layer3_run.json` | export-plan-schema→dry-run→run→재독 |
| `verify_null_vs_object.json` | verify null ≠ 통과 |
| `changed_pages_null.json` | null ≠ [] |

각 파일은 단계 배열과 각 단계의 `command`·`expectExit`·`readFields` 를 담는다.
테스트가 이 스키마를 고정한다. 런타임에 rhwp 를 반드시 띄우지는 않는다 —
문서와 픽스처가 같은 단어를 쓰는지가 이 파동의 시험이다.

---

## 12. 에이전트가 루프를 건너뛰는 징후

다음이 보이면 루프를 되감는다.

1. 첫 도구 호출이 `edit` 또는 `run` 이고 그 앞에 `fields`/`export-tables`/`search` 가 없다.
2. `--dry-run` 없이 `-o` 로 바로 저장했다.
3. 저장 후 `fields`/`search`/`export-tables` 재독이 없다.
4. `verify: null` 을 "검증 통과" 로 사용자에게 말했다.
5. `changedPages: null` 인 채로 `export-svg` 전 페이지를 돌렸다.
6. exit 3 을 예외로 삼켜 봉투를 읽지 않았다.
7. `filledCount` 만 보고 완료라고 했다.
8. 원본 경로에 썼다 (`-o` 가 입력과 같거나 `--in-place` 를 묻지 않고 사용).

이 징후가 0건인 것이 K8 DoD 의 "무계획 실행 0" 이다.
스킬 판단 트리가 구조적으로 배제하고, 예제·픽스처·시험이 그 배제를 표본으로 남긴다.

---

## 13. 단계별 중단 표

| 단계에서 본 것 | 다음 |
|----------------|------|
| 발견 exit 1 (파일 없음·암호) | 쓰기 금지. 암호는 stdin 재시도 한 번 |
| dry-run `notFound` 비지 않음 | data/키 수정 후 ② |
| dry-run `ambiguous` 비지 않음 | `이름[N]` 으로 고친 후 ② |
| dry-run `overflow` 비지 않음 | 더 짧은 값 또는 사용자에게 알림 |
| dry-run `invalid[]` 비지 않음 | 계획서 수정 후 ②. 재시도 금지 |
| 실행 exit 1 | 원본 확인. 산출 경로 잔재 삭제 |
| 실행 exit 2 (1층) | 인자 수정. 같은 줄 재시도 금지 |
| 실행 exit 3 (1층 verify) | 산출물은 있음. diffCount 읽고 ⑤ |
| 실행 exit 3 (run verify/CAS) | 산출물 없음. 저널 읽고 재계획 |
| 실행 exit 4 | convert/export-hwpx 전용. 이 스킬 기본 경로 아님 |
| 재독 불일치 | 산출물 폐기. 원본에서 다시 |
| `changedPages` 쪽 렌더 실패 | 시각 스킬로 인계. 편집 자체는 재독이 맞으면 유지 |

---

## 14. 다음 문서

- 종료 코드·봉투 필드 사전 → [failure_envelopes.md](failure_envelopes.md)
- 루프 워크스루 모음 → [../examples/README.md](../examples/README.md)
