---
name: rhwp-safe-edit
description: rhwp CLI로 HWP/HWPX 문서를 원본 훼손 없이 편집하는 안전 규약입니다. 편집 1건은 1층 edit 하위명령(-o 산출 분리·--dry-run 선확인·--verify 자기검증), 여러 편집은 3층 run 계획서(선검증→원자 실행→저널)로 수행하고, exit 3/4 판정을 예외가 아니라 봉투 데이터로 읽습니다. 트리거 — 사용자가 "누름틀 채워", "표 셀 값 바꿔/갱신", "문구 일괄 치환", "체크박스 켜", "도장/서명 붙여", "개인정보 마스킹", "여러 편집을 한 번에/원자적으로", "안전하게 편집", "dry-run으로 먼저", "run 계획서" 등을 요청할 때. 전체 명령 레퍼런스는 mydocs/manual/cli_commands.md.
---

# rhwp-safe-edit — 안전 편집 규약 Skill

## 목적

`rhwp` 로 문서를 **고칠 때** 지키는 규약이다. 핵심은 넷이다.

1. **원본 불변** — 어떤 편집도 실패 시 원본을 건드리지 않는다.
2. **산출 분리** — 산출물은 항상 별도 경로(`-o`)로 낸다.
3. **선확인** — 파일을 쓰기 전에 `--dry-run` 으로 변경 예정을 본다.
4. **판정은 예외가 아니라 봉투** — exit 3/4 와 `notFound`·`ambiguous`·`overflow` 는
   고장이 아니라 **읽고 분기해야 하는 데이터**다.

이 스킬은 **새 편집 로직을 만들지 않는다.** 이미 devel 에 있는 `edit` 하위명령과
`run` 계획 실행기(`run_plan_engine`)를 에이전트가 원본을 깨지 않고 부르게 배선한다.

권위 출처: [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md)
(§edit 각 절·§run·§종료 코드 #2707)와
[`mydocs/manual/agent_knowledge_map.md`](../../../mydocs/manual/agent_knowledge_map.md) §1-1(라).

## 자식 문서 (이 스킬의 본문)

SKILL.md 는 라우터다. 작업 종류에 맞는 자식을 **읽고 나서** 명령을 조립한다.

| 작업 | 읽기 | 경로 |
|------|------|------|
| 편집 1건 (누름틀·치환·셀·그림·마스킹·메타 제거) | 1층 단건 | [references/single_edit.md](references/single_edit.md) |
| 여러 편집을 한 번에·원자적으로 | 3층 계획서 | [references/run_plans.md](references/run_plans.md) |
| dry-run → 저장 → --verify → 재독 | 검증 루프 | [references/verify_loops.md](references/verify_loops.md) |
| exit 3/4 · invalid[] · notFound 를 데이터로 | 실패 봉투 | [references/failure_envelopes.md](references/failure_envelopes.md) |

실측 워크스루(명령 + 기대 봉투)는 [examples/](examples/README.md) 다.
기계가 읽는 계획서·봉투 픽스처는 [fixtures/](fixtures/catalog.json) 다.

## 실행

```bash
cargo build --release        # 최초 1회 또는 소스 변경 후
./target/release/rhwp <명령> [옵션]
```

계획서 문법을 지어내지 마라. 쓰기 전에 스키마를 읽는다.

```bash
rhwp export-plan-schema --json     # 봉투 + 스키마 (MCP 리소스 rhwp://schemas/plan 과 동일)
rhwp export-plan-schema --bare     # JSON Schema 본문만 — 검증기에 그대로 먹인다
```

## 요청 → 명령 매핑

| 사용자 요청 | 명령 | 판정 필드 |
|------------|------|----------|
| "누름틀 채워" | `edit fill-fields <파일> --data <JSON\|@파일> [-o] [--dry-run] [--json]` | `filledCount`·`notFound`·`ambiguous` |
| "같은 이름 N번째만" | `--data '{"이름[N]":"값"}'` (0 기준 순번, #3476) | `filled[].occurrence` |
| "문구 일괄 치환" | `edit replace-text --find <A> --replace <B> [--ignore-case]` | `replacedCount` |
| "k번째만 치환 / 체크박스" | `edit replace-text --occurrence k` (□→☑ 치환이 체크박스) | `occurrence`·`replacedCount:1` |
| "표 칸에 값 기록" | `edit set-cell --table N --row R --col C --text <값>` | `oldText`/`newText`·`overflow` |
| "표 전체를 CSV로 덮어쓰기" | `csv-to-table --csv <경로> --table N` | `changedCount`·`invalid[]` |
| "도장·서명 붙여" | `edit insert-image --image <그림> [--page N --x --y]` | `binDataId`·`overflow` |
| "개인정보 마스킹" | `edit redact [--kind …] [--no-raw]` — `-o` 또는 `--in-place` **필수** | `findingCount`·`redactedCount` |
| "메타데이터 제거" | `edit sanitize` | `removedCount`·`removed[]` |
| "여러 편집을 한 번에(원자)" | `run <계획.json> [--dry-run] [--json]` | `invalid[]`·`steps[]`·`verify` |
| "서식 1 + 데이터 N명분" | `batch fill --form … --data <행.jsonl\|csv> --out-dir …` | 행별 `notFound`·`filledCount` |

`run` 계획서의 action 은 넷뿐이다. **이 스킬은 다섯 번째 action 을 만들지 않는다.**

- `fill_fields{data}`
- `replace_text{find,replace[,occurrence][,caseSensitive]}`
- `set_cell{table,row,col,text[,keepStyle]}`
- `set_checkbox{occurrence}`

그림 삽입·마스킹·메타 제거는 1층 `edit` 로 남는다. 계획서에 `insert_image` 를 지어내지 마라.

## 판단 트리 — 1층이냐 3층이냐

```
편집이 1건인가?
├─ 예 → 1층: edit 하위명령 + --dry-run 선확인 + -o 산출 분리 + --verify
└─ 아니오(여러 스텝, 반쪽 편집이 남으면 안 됨)
   → 3층: run 계획서 — 선검증(check) → 원자 실행 → 저널
```

`edit` 를 이어 붙이면 중간 실패 시 **반쯤 채워진 파일**이 남는다. `run` 은 전 step 을
정적 선검증하고(위반 시 실행 0·`invalid[]`·exit 2), 인메모리로 적용해 단언
(`assertions.verify`) 통과 시에만 **단 한 번** 저장한다 — 실패 시 디스크 무변경(#3703).

```bash
cat > plan.json <<'EOF'
{ "planVersion": "1.0", "input": "서식.hwp", "output": "완성본.hwp",
  "steps": [
    {"action": "fill_fields", "data": {"기관명": "한국수자원공사"}},
    {"action": "replace_text", "find": "2025년", "replace": "2026년"}
  ],
  "assertions": {"notFoundEmpty": true, "verify": true} }
EOF
rhwp run plan.json --dry-run --json     # ① check 선검증만 — 디스크 무변경
rhwp run plan.json --json               # ② 통과했을 때만 저장
```

- 계획서 문법의 단일 출처는 `rhwp export-plan-schema --json`(MCP 리소스 `rhwp://schemas/plan`).
- 각 step 은 선택 필드 `if` 로 조건을 달 수 있다(`fieldExists`·`fieldEquals`·`textFound`,
  #3719 §6-8). 조건이 거짓이면 그 step 만 건너뛰며 저널에 `skipped:true` 로 남는다.
- **위반은 데이터다**: 선검증 실패는 `invalid[]` 에 **전부 모아** 온다(두더지잡기 방지).
  exit 2 지만 봉투는 나온다 — 함정 ⑥.

자세한 조립은 [references/run_plans.md](references/run_plans.md).

## 원본 불변·산출 분리 원칙

- **실패 시 원본 불변**: `edit` 전 명령이 필드 설정·치환·직렬화·쓰기 하나라도 실패하면
  출력 파일을 쓰지 않고 exit 1 로 끝난다. 원본은 그대로다.
- **산출 분리**: `-o` 생략 시 `<입력명>_filled/_replaced/_cell/_image/_sanitized.<확장자>`
  로 분리 저장된다. 예외는 `edit redact` — 기본 이름조차 만들지 않고 `-o` 또는
  `--in-place` 가 없으면 **실행 자체를 거부**(exit 2)한다. `-o` 가 원본 자신이어도 거부.
- **입력 형식 보존(#3383)**: HWPX 입력 → HWPX 산출, HWP5/HWP3 입력 → HWP5 산출(어댑터 경유).
  실제 저장 형식은 봉투의 `outputFormat` 이 보고한다. 예외 하나 — HWPX 입력에
  `-o ….hwp` 를 **명시**하면 경로를 존중해 HWP5 로 저장하되 형식 변경과 이미지·차트
  유실 가능성을 stderr 로 경고한다(형식 변환의 정식 통로는 `export-hwpx`).
- **무변경 산출물 금지**: 치환 0건·탐지 0건이면 출력 파일을 만들지 않는다(`output` 키 부재).
- **스타일 계약(#3391)**: `set-cell` 기본은 검정·비이탤릭·비진하게 기록 — 파란 안내문
  스타일을 실값이 상속하지 않게 한다. 안내문 모양을 유지하려면 `--keep-style`.

## 판정은 예외가 아니라 봉투 — exit 3/4

| exit | 뜻 | 대응 |
|---:|---|---|
| 0 | 성공 | 봉투의 데이터 신호(`notFound` 등)는 **여전히** 확인한다 |
| 1 | 런타임 실패(읽기·파싱·쓰기) | 환경·입력을 본다. stdout 은 0바이트 |
| 2 | 사용법 오류 — **호출 조립 버그** | 재시도 금지, 인자를 고친다 |
| 3 | `--verify` IR 차이 검출 **또는** `run` CAS 전제 실패 | 고장이 아니라 **판정**. 1층은 산출물이 남고, `run` 단언/CAS 는 디스크 무변경 |
| 4 | `--verify-pages` 쪽 수 불일치 | 위와 같음 (`convert`/`export-hwpx` 전용) |

바인딩도 같은 규약이다 — exit 3/4 를 예외로 올리지 않고 **반환값의 판정 필드**로 준다.
예외로 다루면 호출자가 봉투의 근거(`diffCount`·`status`)를 읽지 않게 되기 때문이다.

**1층 `--verify` 와 3층 `assertions.verify` 는 같은 숫자가 아니다.**

- 1층 `edit … --verify`: 산출물은 **남기고** exit 3. `verify.diffCount` 를 읽는다.
- 3층 `run` + `assertions.verify: true`: 차이가 있으면 **저장하지 않고** exit 3.
- `run` + `preconditions.inputSha256` 불일치: `invalid[]` 는 비어 있고
  `preconditionFailed` + `nextCall` + **exit 3**. 사용법 오류(2)가 아니다.

표와 분기는 [references/failure_envelopes.md](references/failure_envelopes.md).

## 절차 (권장 루프)

1. **발견**: `fields --json`(누름틀 이름·안내문) / `export-tables --json`(표 격자 좌표).
2. **선확인**: 같은 명령줄에서 `--dry-run` 만 붙여 변경 예정·`notFound`·`overflow` 를 본다.
   선검증이 "실행과 같은 명령줄에서 `--dry-run` 하나만 빼면 되는 것"이라야 뜻이 있다.
3. **실행**: `--dry-run` 을 떼고 `-o`·`--verify` 를 붙여 저장한다.
4. **눈검증**: 봉투의 `changedPages`(0 기준) 쪽만 `export-svg -p N` 으로 렌더한다.
5. **재독 대조**: 치환은 `search`(→ `matchCount:0` 확인), 셀은 `export-tables` 재독.

```bash
# 발견 → 선확인 → 기록 → 재독 검증 (전 과정 CLI, 매뉴얼 실측 예)
rhwp export-tables 양식.hwpx --json | jq '.tables[0].cells[:4]'
rhwp edit set-cell 양식.hwpx --table 0 --row 2 --col 1 --text "1,234" --dry-run --json
rhwp edit set-cell 양식.hwpx --table 0 --row 2 --col 1 --text "1,234" -o 작성본.hwpx --json
rhwp export-tables 작성본.hwpx --json | jq '.tables[0].cells[] | select(.row==2 and .col==1).text'

rhwp edit replace-text 공문.hwp --find "2025년" --replace "2026년" -o 개정본.hwp --json
rhwp search 개정본.hwp "2025년" --json | jq .matchCount     # → 0 이어야 함
```

루프 전체와 `changedPages: null` 함정은 [references/verify_loops.md](references/verify_loops.md).

## 함정 (실측된 것만)

1. **`filledCount` 성공 ≠ 완료.** `ambiguous` 가 비어 있지 않으면 순번 없는 이름이
   여러 곳에 해당해 **일부만 채운 것**이다(`{name,matched,total}`). `notFound` 는 오타·
   범위 밖 순번인데 **exit 0** 이다 — 봉투를 읽지 않으면 덜 채운 산출물이 완성본이 된다.
2. **병합으로 덮인 칸에 `set-cell` 하면 exit 2** + 앵커 좌표 안내가 온다(보호 동작,
   stdout 0바이트). `--table` 값은 배열 순번이 아니라 `export-tables` 의 `index` 다.
3. **`overflow` 는 채우기를 막지 않는다**(#3480). 신호를 무시하면 표 밖으로 넘친 문서를
   완성본으로 오판한다. `--dry-run` 에서도 검사된다.
4. **`changedPages: null` 은 "확정 불가"다** — 빈 배열(바뀐 쪽 없음)과 다르다.
   dry-run 은 항상 `null`. 눈검증은 실제 저장 후에 한다.
5. **`run` 계획서 키 오조립이 흔하다**: `source`/`op` 가 아니라 `input`/`steps[].action`,
   action 은 스네이크(`fill_fields`·`replace_text`·`set_cell`·`set_checkbox`).
   `planVersion` 누락은 `{"error":"planVersion \"1.0\" 이 필요합니다"}` + exit 2.
6. **exit 2 라고 stdout 을 버리지 마라.** 단건 명령 실패는 stdout 0바이트가 계약이지만,
   `run` 과 `csv-to-table` 은 **exit 2 에서도 `invalid[]` 봉투를 낸다**. 비어 있지 않으면 읽는다.
7. **`--data @파일` 은 UTF-8 이어야 한다.** CP949 저장본은
   `stream did not contain valid UTF-8` 로 exit 1.
8. **`batch fill` 은 행별 `notFound` 가 있어도 exit 0** 이다 — 데이터 품질 문제는 실행
   실패가 아니라 레코드 판정이다. 행 단위로 확인하지 않으면 조용히 덜 채운 산출물 N개가 나온다.
9. **`redact` 의 `findings[].raw` 는 원문 개인정보 그 자체**다. 봉투를 로그·이슈에 붙일
   거면 `--no-raw` 로 애초에 뺀다.
10. **`--verify` 통과 ≠ 완전 동일.** 자기 재파싱 게이트일 뿐이며, 무손실이 계약이면
    `ir-diff <원본> <산출> --json`(차이 시 exit 3)을 별도로 돌려 `categories` 를 읽는다.
11. **CAS 해시 불일치는 exit 3 이지 exit 2 가 아니다.** `invalid[]` 는 비어 있고
    `preconditionFailed` + `nextCall` 이 온다. 호출 조립을 고치는 자리가 아니다.
12. **조건절은 입력 문서 기준으로 실행 전에 한 번만 판정한다.** 앞 step 이 채운 값이
    뒤 step 의 `if` 를 바꾸지 않는다. "채운 뒤에 치환"을 `if` 로 표현하지 마라.

## 하지 않는 것

- 새 `edit` 하위명령·새 `run` action 을 이 스킬에서 발명하지 않는다.
- gym 과제·채점·pack 을 이 스킬 경로에 끌어들이지 않는다.
- 온보딩·MCP 세션·출처 표지·문서 트리아지 스킬 본문을 이 파동에서 고치지 않는다.
- `--in-place` 를 현황 조사 없이 기본값으로 쓰지 않는다. `redact` 에서만, 그리고
  사용자가 원본 덮어쓰기를 **명시**했을 때만.
- 계획서 키를 예시에서 베껴 쓰지 않고 `export-plan-schema` 를 먼저 읽는다.

## 상세 레퍼런스

- 1층 단건: [references/single_edit.md](references/single_edit.md)
- 3층 계획: [references/run_plans.md](references/run_plans.md)
- 검증 루프: [references/verify_loops.md](references/verify_loops.md)
- 실패 봉투: [references/failure_envelopes.md](references/failure_envelopes.md)
- 워크스루: [examples/README.md](examples/README.md)
- 픽스처 목록: [fixtures/catalog.json](fixtures/catalog.json)
- 전체 명령·옵션·종료 코드: [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md)
- 서식 채움 심화(반복 필드·병합 셀·치환): [`mydocs/manual/form_filling_guide.md`](../../../mydocs/manual/form_filling_guide.md)
- `run` 실측 저널·실패 사례: [`mydocs/manual/agent_task_playbook.md`](../../../mydocs/manual/agent_task_playbook.md) §12
- dry-run→저장→changedPages 루프 실측: [`mydocs/manual/agent_surface_playbook.md`](../../../mydocs/manual/agent_surface_playbook.md) §9
- 봉투 필드 사전·`null` 사전: [`mydocs/manual/agent_knowledge_map.md`](../../../mydocs/manual/agent_knowledge_map.md) §2
- 이 파동의 작업 기록: [`mydocs/working/agent_safe_edit.md`](../../../mydocs/working/archives/agent_safe_edit.md)
