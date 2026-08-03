---
kind: guide
status: active
canonical: mydocs/manual/cli_commands.md
last_verified: 2026-08-03
---

# 에이전트 실무 대체 예제집 — 따라하면 업무가 줄어드는 시나리오

rhwp 를 **에이전트 활용 도구**로 쓰는 실무자용 카탈로그다. 각 항목은
"① 목표 → ② 명령 시퀀스 → ③ 기계 검증 → ④ 실패하면 무엇을 보나" 4단 구성이며,
모든 시퀀스는 저장소 `samples` 실문서로 실행해 검증한 것이다(수치·출력은 실측).
옵션의 canonical reference 는 [CLI 명령어 매뉴얼](cli_commands.md),
파이프라인 계약(JSON/NDJSON·종료 코드)은
[CLI JSON 파이프라인 가이드](cli_json_pipeline_guide.md)를 따른다.
증상별 처방은 [에이전트 실패 사전](agent_troubleshooting_guide.md).

## 원칙 — 눈 검증을 계약 검증으로

에이전트 루프에서 사람이 스크린샷을 보며 고치는 단계가 남아 있으면 업무는 줄지 않는다.
모든 예제는 결과 확인을 **기계 계약**으로 닫는다:

- **종료 코드** (#2707): 0/1/2(+검증 게이트 3/4) — `&&`·CI 게이트로 무인 판정
- **재독 대조**: 쓴 것을 다시 읽어(JSON) 값·구조를 프로그램으로 비교 — "됐다는 보고"가
  아니라 산출물 자체를 근거로
- **자기서술** (`capabilities`): 호출 전에 도구·플래그·가용성을 확인해 exit 2 지뢰 회피

시각 확인이 정말 필요한 순간(최종 납품 확인 등)에만 렌더를 쓰고, 그때도 대상 페이지만
좁혀 렌더한다(시나리오 3).

**exit 0 을 성공으로 읽지 마라.** `fill-fields` 의 `notFound`, `replace-text` 의
`replacedCount`, `inspect *` 의 `clean` 처럼 **판정이 봉투 안에만 있는 명령**이 여럿이다.
각 시나리오의 ④ 가 그 자리를 짚는다. 전체 목록은
[실패 사전 §1](agent_troubleshooting_guide.md).

## 이 문서와 `mydocs/manual/recipes/` 의 경계

레시피 모음(`mydocs/manual/recipes/`, #3835)이 들어오면 둘의 역할이 이렇게 갈린다.
**같은 내용을 두 곳에 쓰지 않는다.**

| | 본 예제집 | `recipes/` |
|---|---|---|
| 단위 | **명령 3~8줄**로 끝나는 시퀀스 | 여러 단계·분기·재시도가 있는 **작업 서사** |
| 목적 | "이 업무에 어떤 명령을 쓰나"를 **찾게** 한다 | "이 작업을 처음부터 끝까지" **따라하게** 한다 |
| 포함 | 시퀀스·검증 게이트·실패 시 볼 필드 | 입력 준비, 데이터 정제, 예외 처리, 산출물 정리 |
| 분량 | 시나리오당 20~40줄 | 레시피당 문서 하나 |
| 예 | "표 → CSV → 되돌리기"(시나리오 13) | "실적 취합 파이프라인 구축기" |

경계 규칙:

1. **시퀀스가 8줄을 넘거나** 조건 분기·재시도 루프가 필요하면 → 레시피로 옮기고
   여기에는 한 줄 요약 + 링크만 남긴다.
2. 본 문서의 시나리오는 **실행 가능한 최소형**이어야 한다. 준비물 설명이 길어지면
   그것 자체가 레시피 신호다.
3. 레시피가 새로 들어오면 해당 시나리오의 "관련" 줄에 링크를 추가하고, 중복된
   본문은 **지운다**.

## 시나리오 색인

| # | 업무 | 주 명령 | 판정 게이트 |
|---|---|---|---|
| 1 | 서식 채워 제출 | `fields` → `edit fill-fields` | `notFound==[]` |
| 2 | 표 데이터 수확 | `export-tables` / `table-to-csv` | `tableCount`·exit 0 |
| 3 | 근거 찾아 그 쪽만 렌더 | `search` → `export-svg -p` | `matchCount` |
| 4 | 대량 문서 스윕 | `batch info/export-text/export-structure` | 레코드별 `error` |
| 5 | 새 문서 생성 | `build-from-ingest` | 재독 텍스트 대조 |
| 6 | 형식 변환 + 무손실 | `export-hwpx --verify` | exit 0/3/4 |
| 7 | 요약·질의 준비 | `export-text --json` pages | `pageCount` |
| 8 | 배포본 동일성 | `render-diff` + SVG 바이트 | `maxDisp`·바이트 일치 |
| 9 | 대량 발급(메일머지) | `batch fill` | 전 행 `notFound==[]` |
| 10 | 누름틀 이름 감사 | `batch fields` | 이름 집합 |
| 11 | 서식 버전 비교 | `batch fields` + `ir-diff` | 이름 차집합 |
| 12 | 원자적 편집 | `run <계획.json>` | `invalid==[]` |
| 13 | 표 왕복(CSV 편집) | `table-to-csv` → `csv-to-table` | `invalid==[]`·`changedCount` |
| 14 | 실적 수치 갱신 | `edit set-cell` | `overflow==[]` + 재독 |
| 15 | 기관명·연도 일괄 치환 | `edit replace-text` | `replacedCount` |
| 16 | 여러 문서에서 조항 찾기 | `batch search` | `totalMatchCount` |
| 17 | 계약서 날짜·금액 뽑기 | `extract-data` | `normalized != null` |
| 18 | 목차 추출 | `export-structure` | `nodeCount` |
| 19 | 문서 묶음 일괄 변환 | `batch convert` | 집계 exit |
| 20 | 배포용 PDF | `export-pdf` | 쪽수·크기 |
| 21 | 개인정보 마스킹 후 배포 | `edit redact` | `findingCount` → 재독 |
| 22 | 메타데이터 제거 | `edit sanitize` | `removed[]` |
| 23 | 출처 모르는 문서 안전 점검 | `inspect injection` | `clean==true` |
| 24 | 은닉 텍스트·유니코드 기만 | `inspect hidden-text/unicode` | `clean==true` |
| 25 | 도장·서명 삽입 | `edit insert-image` | `overflow==[]`·`binDataId` |
| 26 | 발췌본 만들기 | `extract-pages` | `pagesAfter` |
| 27 | 편집 전후 시각 회귀 | `render-diff A B` | `maxDisp`·`pageCountMismatch` |
| 28 | 개정 전후 텍스트 비교 | `export-text` + diff | 문단 단위 diff |
| 29 | 초소형 모델용 한 번 요약 | `digest` | `nextStep` |
| 30 | 각주·미주 설정 확인 | `dump-note-shape` | 구역별 값 대조 |
| 31 | 썸네일로 문서 식별 | `thumbnail` | PNG 시그니처 |
| 32 | 문서 자체 이상 좁히기 | `info` → `diag` → `dump` | 단계별 산출 |
| 33 | 성능 회귀 감시 | `bench` | 단계별 median |
| 34 | 구조화 교환(DocLang) | `export-doclang` | `lossCount` |
| 35 | 도구 온보딩·표면 캐시 | `capabilities` | `available` 필드 |

---

## A. 서식·발급

### 1. 서식 자동 작성 — 누름틀 채우기

- **① 목표**: 행정 서식(신청서·기획서·보도자료)을 열어 회사명·작성자·연락처… 칸을
  하나씩 타이핑하는 일을 없앤다.
- **② 명령 시퀀스**:
  ```bash
  rhwp fields 서식.hwp --json | jq '[.fields[]|select(.name!="")|.name]'   # ① 서식이 요구하는 것
  rhwp edit fill-fields 서식.hwp --data @row.json --dry-run --json         # ② 먼저 미리보기
  rhwp edit fill-fields 서식.hwp --data @row.json -o 완성본.hwp --json     # ③ 값 채우기
  rhwp fields 완성본.hwp --json                                            # ④ 재독 — 기계 대조
  rhwp export-pdf 완성본.hwp -o 완성본.pdf                                  # ⑤ (필요시) 납품용 PDF
  ```
- **③ 기계 검증**: ④ 의 `fields[].value` 를 ③ 입력과 프로그램으로 대조.
  실측(보도자료 서식, 누름틀 12개):
  ```json
  {"ambiguous":[],"changedPages":[0,1],"confusable":[],
   "filled":[{"name":"기관명","occurrence":0,"value":"한국수자원공사"}],
   "filledCount":1,"notFound":[]}
  ```
  `fields --json` 의 항목마다 `location`(section/paragraph/nested cell)이 붙으므로
  "어느 칸에 들어갔는지"까지 좌표로 확인된다.
- **④ 실패하면 무엇을 보나**:
  - `notFound` — 이름 오타 **또는** `이름[N]` 의 N 이 범위 밖. `fields --json` 으로
    실제 이름·개수를 다시 읽는다.
  - `ambiguous` — 동명 누름틀이 여러 개. 실측 `{"matched":1,"name":"대안제목","total":11}`
    처럼 `total > matched` 면 나머지는 손대지 않은 것이다. `이름[0]`·`이름[1]` 로 지목.
  - `confusable` — 화면상 구별 안 되는 동형자 이름 충돌. **자동 채우기를 멈춘다.**
  - **exit 0 이어도 위 셋이 비지 않으면 실패다.** 파일은 이미 만들어졌으므로
    삭제/재작성 로직을 넣어라.
- **절감**: 서식 1건 수기 기입 → row 데이터만 준비하면 반복. 눈 확인 불필요.
- **관련**: 심화는 [서식 자동화 심화 가이드](form_filling_guide.md), 원자성이 필요하면
  시나리오 12, 대량이면 시나리오 9.

### 9. 대량 발급 — 메일머지(`batch fill`)

- **① 목표**: 같은 서식에 부서/분기/담당자만 바꿔 N건을 발급한다.
- **② 명령 시퀀스** (실측 3행):
  ```bash
  # rows.csv 첫 줄 헤더 = 누름틀 이름 (UTF-8, BOM 허용)
  rhwp batch fill --form 서식.hwp --data rows.csv --out-dir out \
       --name-field 제목명 --dry-run --json          # ① 전 행 채움 가능 여부만
  rhwp batch fill --form 서식.hwp --data rows.csv --out-dir out \
       --name-field 제목명 --verify --json > merge.ndjson   # ② 실제 발급
  jq -c 'select(.notFound|length > 0)' merge.ndjson   # ③ 조용한 유실 잡기
  ```
- **③ 기계 검증**: 행마다 NDJSON 레코드 하나. 실측:
  ```json
  {"filled":[{"name":"기관명",…},{"name":"담당자명",…},{"name":"담당자전화번호",…},
   {"name":"제목명","occurrence":0,"value":"1분기 실적 보고"}],
   "filledCount":4,"notFound":[],"output":"out\\1분기 실적 보고.hwp","row":0,
   "verify":{"diffCount":0,"identical":true}}
  ```
  stderr 요약: `batch fill: 3행 중 3 성공, 0 실패 (12ms, threads=32)`.
- **④ 실패하면 무엇을 보나**:
  - **`notFound` 가 있어도 exit 0 이고 "성공"으로 집계된다.** 헤더 오타 하나가 전 행에서
    조용히 유실된다 — ③ 의 jq 게이트가 필수다(실측 재현:
    헤더 `없는칸` → 2행 모두 `"notFound":["없는칸"]`, 그래도 `2행 중 2 성공`).
  - `--data` 가 cp949 면 `stream did not contain valid UTF-8` (exit 1). 엑셀에서 왔다면
    "CSV UTF-8" 로 다시 저장.
  - 산출 이름이 `0001.hwp` 면 `--name-field` 를 안 준 것. 이름이 겹치면 덮어쓰지 않고
    `_2`·`_3` 이 붙으므로(실측 `동일이름.hwp`, `동일이름_2.hwp`) **파일명으로 행을
    되짚을 수 없다** — `row`↔`output` 매핑을 남긴다.
  - `fill` 축만 **stdin 을 읽지 않는다.** 파이프를 태우면 아무 일도 안 일어난다.
- **절감**: N건 수기 발급 → 데이터 파일 1개. 발급 로그가 NDJSON 으로 그대로 남는다.

### 10. 누름틀 이름 감사 — 서식이 요구하는 것 목록화

- **① 목표**: 부서가 쓰는 서식 수십 개가 각각 무슨 값을 요구하는지 대장으로 만든다.
  이게 있어야 데이터 수집 양식을 한 번에 설계할 수 있다.
- **② 명령 시퀀스**:
  ```bash
  find 서식폴더/ -name '*.hwp*' -type f | rhwp batch fields --json > fields.ndjson
  jq -r '[.source, ((.fields//[])|map(.name)|unique|join("|"))] | @tsv' fields.ndjson
  jq -r 'select((.fields//[])|length == 0) | .source' fields.ndjson    # 누름틀 없는 서식
  ```
- **③ 기계 검증**: `fieldCount` 합계와 서식 수. 실측(353건 스윕):
  `batch: 353건 중 350 성공, 3 실패 (3147ms, threads=8)` — 실패 3건은 전부 암호 문서.
  상위 결과 예: 규제영향분석서 1,070필드 / 보도자료 12필드.
- **④ 실패하면 무엇을 보나**:
  - 레코드에 `error` 가 있으면 그 파일만 격리. `exitClass` 로 재시도 가치를 판단
    (`runtime` + "비밀번호" → 사람에게 암호를 받아 단건 처리).
  - `fieldCount: 0` 인데 서식처럼 보이면 **누름틀이 아니라 표 칸**으로 만든 양식이다 —
    시나리오 14(`set-cell`)로 간다. 실측: 지자체 보고서 양식 hwpx 는 `fieldCount:0`,
    최상위 표 52개.
  - 같은 이름이 수십 번 반복되면(실측 `대안제목` 11개) 발급 시 `ambiguous` 가 뜬다.
    감사 단계에서 미리 `이름[N]` 매핑을 만들어 둔다.
- **절감**: 서식을 하나씩 열어 "여기 뭘 적어야 하나" 확인하는 일이 사라진다.

### 11. 서식 버전 비교 — "이번 개정판에서 칸이 바뀌었나"

- **① 목표**: 작년 서식과 올해 서식 사이에 **없어진 칸/새 칸**을 찾는다. 자동 발급
  스크립트가 조용히 깨지는 지점이다.
- **② 명령 시퀀스**:
  ```bash
  printf '%s\n' 서식_2025.hwp 서식_2026.hwp | rhwp batch fields --json > f.ndjson
  python - <<'PY'
  import json
  s={}
  for l in open('f.ndjson',encoding='utf-8'):
      d=json.loads(l); s[d['source']]=set(f['name'] for f in d.get('fields',[]))
  a,b=list(s)
  print('없어진 칸:', s[a]-s[b]); print('새 칸:', s[b]-s[a])
  PY
  rhwp ir-diff 서식_2026.hwp 서식_2025.hwp --json      # 본문 구조까지 볼 때
  ```
- **③ 기계 검증**: 두 차집합이 모두 공집합이면 발급 스크립트를 그대로 써도 된다.
  실측(보도자료 3판본): 세 파일 모두 12개, 차집합 공집합 → 호환.
- **④ 실패하면 무엇을 보나**:
  - `ir-diff` 는 **`--json` 없이는 차이가 있어도 exit 0** 이다. 자동화에서는 반드시 붙인다.
  - `ir-diff <A> --json` 처럼 파일을 하나만 주면 `--json` 이 파일로 해석돼
    `오류: --json 읽기 실패: …` (exit 1) 이 나온다.
  - `categories` 는 `untrustedFields` 다 — 차이 요약을 그대로 프롬프트에 붙이지 마라.
- **절감**: 개정판 배포 때마다 손으로 서식을 열어 대조하던 일이 사라진다.

### 12. 원자적 편집 — 선언적 계획서(`run`)

- **① 목표**: 여러 편집(칸 채우기 + 문구 치환 + 셀 갱신)을 **전부 되거나 전혀 안 되게**
  한다. `edit` 축을 이어 붙이면 반쯤 채워진 파일이 남을 수 있다.
- **② 명령 시퀀스**:
  ```bash
  cat > plan.json <<'EOF'
  { "planVersion": "1.0",
    "input": "서식.hwp",
    "output": "완성본.hwp",
    "steps": [
      {"action": "fill_fields", "data": {"기관명": "한국수자원공사"}},
      {"action": "replace_text", "find": "봉화댐", "replace": "소양강댐"}
    ],
    "assertions": {"notFoundEmpty": true, "verify": true} }
  EOF
  rhwp run plan.json --dry-run --json     # ① 선검증만 (디스크 무변경)
  rhwp run plan.json --json               # ② 통과했을 때만 저장
  ```
- **③ 기계 검증**: 실측 저널 —
  ```json
  {"changedPages":[0,2,3],
   "steps":[{"action":"fill_fields","filled":[{"name":"기관명",…}],"filledCount":1,
             "notFound":[],"step":0},
            {"action":"replace_text","find":"봉화댐","replacedCount":7,"step":1}],
   "verify":{"diffCount":0,"identical":true}}
  ```
  dry-run 은 `preview[].targets[].sameNameCount` 로 지목의 모호성까지 미리 알려 준다
  (실측: `{"name":"기관명","occurrence":0,"sameNameCount":1,…}`, `"willReplace":7`).
- **④ 실패하면 무엇을 보나**:
  - `invalid[]` + exit 2 → **디스크는 그대로다**(실측: 산출 파일이 생기지 않음).
    `step` 인덱스와 `reason` 이
    `필드 '없는필드' 이(가) 없거나 순번이 범위 밖입니다 (동명 0개)` 처럼 사유를 갈라 준다.
  - 키 이름 오조립이 흔하다: `source`/`op` 가 아니라 **`input`/`steps[].action`**,
    action 값은 스네이크(`fill_fields`, `replace_text`, `set_cell`, `set_checkbox`).
    `planVersion` 을 빠뜨리면 `{"error":"planVersion \"1.0\" 이 필요합니다"}` (exit 2).
  - `assertions.verify: true` 로 exit 3 이면 저장이 취소된 것이다.
  - **MCP 경로 주의**: `hwp_run_plan` 은 선검증 실패에도 `isError:false` 다.
    `structuredContent.invalid == []` 로 판정하라.
- **절감**: "반쯤 편집된 파일" 이라는 실패 상태 자체가 없어진다.

---

## B. 표·데이터

### 2. 표 데이터 수확 — HWP 표 → CSV/스프레드시트

- **① 목표**: 공문·편람의 표를 화면 보고 엑셀에 옮겨 치는 일을 없앤다.
- **② 명령 시퀀스**:
  ```bash
  rhwp export-tables 문서.hwp --json > tables.json                # 병합·중첩 보존 격자
  jq '.tables[] | {index, rows, cols, cellCount}' tables.json     # 어떤 표를 쓸지 고른다
  rhwp table-to-csv 문서.hwp --table 12 -o 표.csv --bom            # RFC 4180 CSV
  rhwp table-to-csv 문서.hwp --table 12                            # 파이프로 흘릴 때
  ```
- **③ 기계 검증**: `tableCount`·`cellCount` 와 원본 표 개수 대조, exit 0.
  실측: 보건소 분장사무 — 표 1개, `147 x 3`, 441셀. 지자체 보고서 양식 — 최상위 표 52개
  (병합 앵커 다수: `(0,0,rowSpan2,colSpan1)` 등).
- **④ 실패하면 무엇을 보나**:
  - `--table` 은 **배열 순번이 아니라 `tables[].index` 값**이다. 틀리면
    `오류: 본문 최상위 표 999 번이 없습니다 (최상위 표 52개; 중첩 표는 v1 범위 밖).` (exit 1)
  - 열이 밀려 보이면 병합 때문이다 — `table-to-csv` 는 병합 격자를 **채워서** 내지만
    `export-tables` 원본 격자는 앵커 셀에만 값이 있다.
  - 엑셀에서 한글이 깨지면 `--bom` 을 빼먹은 것.
- **절감**: 표 옮겨치기 소멸. 아카이브 규모는 `batch export-tables` 로 확장.

### 13. 표 왕복 — CSV 로 편집해서 되돌리기

- **① 목표**: 표를 스프레드시트에서 편하게 고치고 **원래 문서의 그 표에** 되돌린다.
  표 크기·서식은 그대로 두고 값만.
- **② 명령 시퀀스**:
  ```bash
  rhwp table-to-csv 양식.hwpx --table 12 -o t12.csv --json    # ① 뽑기 (실측 12행 × 3열)
  # …엑셀/파이썬으로 값만 수정…
  rhwp csv-to-table 양식.hwpx --csv t12.csv --table 12 --dry-run --json   # ② 예행
  rhwp csv-to-table 양식.hwpx --csv t12.csv --table 12 -o 완성.hwpx --verify --json
  rhwp table-to-csv 완성.hwpx --table 12                       # ③ 재독 대조
  ```
- **③ 기계 검증**: 실측 —
  ```json
  {"changed":[{"col":1,"newText":"1,234","oldText":"","row":1}],"changedCount":1,
   "changedPages":[6,7],"colCount":3,"invalid":[],"outputFormat":"hwpx","rowCount":12}
  ```
  `changed[]` 가 **바뀐 칸만** 정확히 열거하므로 "무엇이 달라졌나"가 데이터로 남는다.
  ③ 재독 결과가 `,"1,234",` 로 나와 반영을 확인했다.
- **④ 실패하면 무엇을 보나**: `invalid[].reason` 세 가지 (전부 exit 2, **한 칸도 안 쓴다**)
  - `rowCountMismatch` — `CSV 행 수 2 가 표 3 의 행 수 5 와 다릅니다 — 표 크기는 바꾸지 않습니다.`
  - `colCountMismatch` — `CSV 0행의 열 수 3 가 표의 열 수 2 와 다릅니다.` (행마다 반복)
  - `coveredCellNotEmpty` — `(0,1) 는 병합으로 덮인 칸이라 쓸 수 없습니다 — 값은 앵커 칸에 두고 이 칸은 비우세요.`
  - **처방 공통**: 손으로 CSV 를 만들지 말고 ① 에서 뽑은 것을 그대로 고쳐라.
    병합 자리를 비워 두는 규칙을 사람이 지키기는 어렵다.
- **절감**: 문서 표를 붙잡고 셀 단위로 타이핑하던 일이 스프레드시트 편집으로 바뀐다.

### 14. 실적 수치 갱신 — 셀 하나만 고치기

- **① 목표**: 보고서 표의 특정 칸(합계·실적·날짜)만 갱신한다. 표 전체를 되돌릴 필요가 없다.
- **② 명령 시퀀스**:
  ```bash
  rhwp export-tables 보고서.hwpx --json | jq '.tables[] | select(.index==12) | {rows,cols}'
  rhwp edit set-cell 보고서.hwpx --table 12 --row 1 --col 1 --text "1,234" --dry-run --json
  rhwp edit set-cell 보고서.hwpx --table 12 --row 1 --col 1 --text "1,234" -o 갱신.hwpx --json
  rhwp export-tables 갱신.hwpx --json \
    | jq -r '.tables[]|select(.index==12)|.cells[]|select(.row==1)|.text'
  ```
- **③ 기계 검증**: 실측 —
  ```json
  {"changedPages":[6,7],"col":1,"newText":"1,234","oldText":"","outputFormat":"hwpx",
   "overflow":[],"row":1,"table":12}
  ```
  재독 결과 `['', '1,234', '']` — 해당 칸만 바뀌었음이 데이터로 확인된다.
  `oldText` 가 함께 오므로 변경 이력을 그대로 로그에 남길 수 있다.
- **④ 실패하면 무엇을 보나**:
  - `overflow` 가 비지 않으면 값이 칸 폭을 넘었다(줄이 늘어 아래가 밀린다).
    실측 형태: `{"cellWidthPx":20.71,"lines":3,"target":"table0[1,1]","textWidthPx":48.0}`.
    **막지 않으므로** 납품 문서면 여기서 멈춰라.
  - `오류: (1,1) 는 병합으로 덮인 칸입니다 — 앵커 (0,1) 를 지정하세요.` (exit 2) → 앵커로 다시.
  - `오류: 좌표가 격자를 벗어났습니다 — 표 0 는 147x3 입니다.` (exit 2) vs
    `오류: 본문 최상위 표 99 번이 없습니다 …` (exit **1**) — 표가 없으면 1, 칸이 없으면 2.
  - `오류: --text 에 줄바꿈·탭은 넣을 수 없습니다 (한 줄 값 기록).` → 값을 `strip()`.
  - `oldText` 는 `untrustedFields` 다(문서에서 온 값).
- **절감**: 분기마다 표를 열어 숫자를 고치던 일이 한 줄 명령이 된다.

### 15. 기관명·연도 일괄 치환

- **① 목표**: 조직 개편·연도 갱신으로 문서 전체의 용어를 바꾼다. 본문 + 표 셀 모두.
- **② 명령 시퀀스**:
  ```bash
  rhwp search 문서.hwp "구 기관명" --json | jq '.totalMatchCount'   # ① 규모 파악
  rhwp edit replace-text 문서.hwp --find "구 기관명" --replace "새 기관명" --dry-run --json
  rhwp edit replace-text 문서.hwp --find "구 기관명" --replace "새 기관명" -o 개정본.hwp --json
  rhwp search 개정본.hwp "구 기관명" --json | jq '.totalMatchCount'   # ② 0 이어야 한다
  ```
- **③ 기계 검증**: `replacedCount` == ① 의 `totalMatchCount`, 그리고 ② 가 0.
  실측(계획서 안 `replace_text` 스텝): `{"find":"봉화댐","replacedCount":7}`.
  `changedPages` 로 어느 쪽이 바뀌었는지도 나온다(실측 `[0,1,2]`).
- **④ 실패하면 무엇을 보나**:
  - `replacedCount: 0` → **출력 파일이 아예 안 만들어진다**(의도된 동작, 실측 확인).
    다음 단계가 "파일 없음"(exit 1)으로 터지면 진짜 원인은 여기다. `--find` 표기를
    `search` 로 먼저 확인하라 — 전각/반각·공백·줄바꿈이 흔한 원인.
  - `오류: --find 는 빈 문자열일 수 없습니다.` (exit 2) → 변수가 비었다.
  - 치환어가 `-` 로 시작하면 `알 수 없는 옵션` (exit 2) — `--` 로 탈출한다.
  - `-o` 를 입력과 같은 경로로 주면 **조용히 덮어쓴다**(실측 exit 0). 호출 전에 경로
    비교를 넣어라.
- **절감**: 문서 수십 건의 기관명 개정이 배치 한 번으로 끝난다(시나리오 19 와 조합).

---

## C. 검색·근거

### 3. 아카이브에서 근거 찾기 — 검색 → 해당 페이지만 렌더

- **① 목표**: "그 조항 어느 문서 몇 쪽이더라" — 문서를 열어 스크롤하는 일을 없앤다.
- **② 명령 시퀀스**:
  ```bash
  rhwp search 문서.hwp "위임전결" --json          # 매치마다 구역·문단·페이지 주소
  rhwp export-svg 문서.hwp -p 35 -o out/ --json   # 근거 페이지만 렌더 (VLM/사람 확인용)
  ```
- **③ 기계 검증**: `matchCount`(0 이면 근거 없음이 판정값), 매치의 `page` 를 그대로 렌더
  대상으로 — 전 페이지 렌더가 아니라 근거 페이지만. `--max-matches` 사용 시에도
  `totalMatchCount` 로 총량 판단. 실측: 편람에서 "위임전결" 19건, 첫 매치 page 35.
  `export-svg -p 1` 실측 봉투: `{"renderedCount":1,"pages":[{"bytes":227749,"page":1,…}]}`.
- **④ 실패하면 무엇을 보나**:
  - `truncated: true` 면 `matchCount` 는 상한이고 총량은 `totalMatchCount` 다
    (실측: `matchCount:3, omittedCount:1275, totalMatchCount:1278`).
  - `-p` 는 **0 기준**이다. `오류: 페이지 번호가 범위를 벗어났습니다 (0~3)` (exit 2)
    가 나오면 사람용 쪽번호를 그대로 넣은 것이다.
  - 검색어가 `-` 로 시작하면 `--` 뒤에 둔다(실측 힌트가 stderr 로 나온다).
  - `matches[].text`·`context` 는 `untrustedFields` 다.
- **절감**: 문서 열람·스크롤 소멸. 렌더 비용은 근거 페이지 수에 비례.

### 16. 여러 문서에서 특정 조항 찾기

- **① 목표**: 폴더 전체에서 "이 조항이 있는 문서만" 골라낸다. 규정 개정 영향 범위 파악.
- **② 명령 시퀀스**:
  ```bash
  find 규정/ -name '*.hwp*' -type f | rhwp batch search --query "위임전결" --json > hits.ndjson
  jq -r 'select(.totalMatchCount > 0) | [.source, .totalMatchCount] | @tsv' hits.ndjson
  jq -r 'select(.totalMatchCount > 0) | .matches[0] | [.page, .text] | @tsv' hits.ndjson
  ```
- **③ 기계 검증**: 실측 3문서 스윕 —
  ```
  편람.hwp                -> 19   (첫 매치 page 35: "결재권자란 행정기관의 장, 법령에 따…")
  국립국어원 업무계획.hwp   -> 0
  보건소 분장사무.hwp       -> 0
  batch: 3건 중 3 성공, 0 실패 (143ms, threads=32)
  ```
  `totalMatchCount > 0` 이 곧 "해당 문서" 판정이다.
- **④ 실패하면 무엇을 보나**:
  - `오류: batch search 는 --query <검색어> 가 필요합니다.` (exit 2)
  - 레코드에 `error` 가 있으면 그 파일만 실패. 배치 전체 exit 1 이어도 나머지 결과는 유효하다.
  - 암호 문서가 섞여 있으면 `batch` 는 비밀번호를 받지 못한다 — 목록에서 빼고 단건 처리.
- **절감**: "이 조항 어디어디에 있죠?" 를 사람에게 묻는 왕복이 사라진다.

### 17. 계약서·공고문에서 날짜·금액 뽑기

- **① 목표**: 계약 기간·납품 기한·금액을 사람이 읽어 옮기지 않는다.
- **② 명령 시퀀스**:
  ```bash
  rhwp extract-data 계약서.hwp --kind date --json | jq '.items[] | {page, raw, normalized}'
  rhwp extract-data 계약서.hwp --kind amount --json | jq '{counts, totalItemCount}'
  rhwp extract-data 계약서.hwp --json --limit 50 > all.json     # date|amount|number 전부
  ```
- **③ 기계 검증**: 값마다 **구역·문단·페이지·문자 오프셋**이 붙어 나오므로 "몇 쪽
  어디서 뽑았나"를 인용할 수 있다. 실측(편람, `--kind date`): `totalItemCount 466`,
  `1961. 9. → 1961-09`, `1949. 7. 15. → 1949-07-15`.
  숫자에는 단위도 붙는다(실측 `{"raw":"5대","normalized":5,"unit":"대"}`).
- **④ 실패하면 무엇을 보나**:
  - `normalized` 가 `null` 이면 **정규화 불가**이고 `raw` 만 있다. 두 자리 연도
    (`26.8.2`)·한글 수사 금액은 **세기·값을 추정하지 않는다** — 사람 확인 대상이다.
  - `totalItemCount` 와 `itemCount` 가 다르면 `--limit` 로 잘린 것이다.
  - 표 안의 값은 `cell` 좌표가 함께 온다 — 표 기반 서식이면 시나리오 2 와 조합해
    행 단위로 재구성한다.
- **절감**: 계약 관리대장 입력이 자동화된다. 근거 쪽까지 함께 저장된다.

### 18. 목차 추출 — 조문·개요 계층

- **① 목표**: 긴 규정·편람의 구조를 파악하고, 요약·RAG 청킹의 뼈대로 쓴다.
- **② 명령 시퀀스**:
  ```bash
  rhwp export-structure 편람.hwp --json > tree.json          # mode auto (조문/개요 자동)
  jq '{nodeCount, mode}' tree.json
  jq -r '.structure.roots[] | .heading' tree.json
  rhwp export-structure 편람.hwp --mode outline -o outline.json   # 방식을 고정할 때
  ```
- **③ 기계 검증**: 실측(2025 행정업무운영 편람) — `{"nodeCount":591,"mode":"clause"}`,
  최상위 5개:
  ```
  제1장 행정업무 운영 개요        1   (자식 0)
  제2장 공문서 관리 등 행정업무의 처리  19  (자식 4)
  제3장 행정업무의 효율적 수행     175 (자식 3)
  제4장 행정업무의 관리           243 (자식 0)
  제5장 질의 및 답변              269 (자식 12)
  ```
- **④ 실패하면 무엇을 보나**:
  - `nodeCount` 가 1~2 면 문서에 개요 번호/조문 표기가 없는 것이다. `--mode outline`
    또는 `--mode clause` 로 바꿔 보고, 그래도 안 나오면 쪽 단위 청킹(시나리오 7)으로 간다.
  - `structure.*` 필드는 전부 `untrustedFields` 다.
- **절감**: 문서를 처음부터 훑어 목차를 만드는 일이 사라진다. 아래 "공무원 업무 커버리지
  체크리스트"도 이 명령으로 편람에서 뽑아 만들었다(도그푸딩).

### 30. 각주·미주 설정 확인

- **① 목표**: 여러 판본 사이에 각주/미주 모양(구분선·간격)이 달라졌는지 기계로 본다.
  변환 회귀에서 가장 자주 어긋나는 축이다.
- **② 명령 시퀀스**:
  ```bash
  rhwp dump-note-shape 문서.hwp   > a.json
  rhwp dump-note-shape 변환본.hwpx > b.json
  diff <(jq -S . a.json) <(jq -S . b.json)
  ```
- **③ 기계 검증**: 구역별 raw 값과 **한컴 UI 의미값**이 함께 나오므로, 차이가 나면
  "UI 에서 뭘 바꾸면 되는지"까지 특정된다.
- **④ 실패하면 무엇을 보나**: `export-hwpx --verify` 가 exit 3 이면서 차이가
  `en.p[..] linesegs [..].vertpos` 축에 몰려 있으면 이 명령으로 좁힌다
  (실측: 교육 통합 문서군에서 이 패턴이 반복됐다).
- **절감**: 미주 간격이 미묘하게 달라진 것을 눈으로 찾던 일이 사라진다.

---

## D. 대량 처리

### 4. 대량 문서 스윕 — 메타/본문/구조를 한 번에

- **① 목표**: 폴더의 문서 수백 건을 하나씩 열어 쪽수·내용 확인하는 일을 없앤다.
- **② 명령 시퀀스**:
  ```bash
  find docs/ -name '*.hwp*' -type f | rhwp batch info --json > meta.ndjson
  jq -r 'select(.pageCount >= 10) | .source' meta.ndjson > targets.txt
  rhwp batch export-text --json < targets.txt > corpus.ndjson
  find docs/ -name '*.hwp*' -type f | rhwp batch export-structure --json > tree.ndjson
  ```
- **③ 기계 검증**: 건별 실패는 `{"error","exitClass"}` 레코드로 격리, 하나라도 실패면
  최종 exit 1 — 성공분 재작업 없이 실패분만 재시도. 요약 줄은 **stderr** 다
  (실측 `batch: 353건 중 350 성공, 3 실패 (3147ms, threads=8)`).
- **④ 실패하면 무엇을 보나**:
  - `exitClass` 로 재시도 가치를 나눈다: `UNSUPPORTED_FILE_FORMAT`·`EMPTY_FILE` 은
    재시도해도 영원히 같다. "비밀번호" 는 사람에게 암호를 받아 단건 처리.
  - **빈 목록도 exit 0** 이다(실측 `batch: 0건 중 0 성공, 0 실패`). 목록 생성 실패를
    따로 잡아라.
  - `-type f` 를 빼면 디렉터리가 섞여 `액세스가 거부되었습니다. (os error 5)` 레코드가 난다.
  - 메모리가 튀면 `--threads 4` 로 낮춘다(기본은 CPU 코어 수, 실측 32).
- **절감**: 문서 1건씩 열람 소멸. RAG/DB 적재 입력이 그대로 나온다.

### 19. 문서 묶음 일괄 변환 (배포본 → 편집본)

- **① 목표**: 읽기전용(배포용) HWP 뭉치를 편집 가능한 HWP 로 한 번에 바꾼다.
- **② 명령 시퀀스**:
  ```bash
  find 배포/ -name '*.hwp' -type f | rhwp batch convert --out-dir 편집본/ \
       --verify --verify-pages --json > conv.ndjson ; echo "exit=$?"
  jq -c 'select(.error != null)' conv.ndjson
  jq -c 'select(.verify.identical == false)' conv.ndjson
  ```
- **③ 기계 검증**: 레코드 키(실측): `bytes, format, output, passwordProtected,
  schemaVersion, source, verify, verifyPages, wasDistribution`.
  `wasDistribution` 이 배포본이었는지, `verify.identical` 이 무손실 여부다.
  실측 181건 스윕: `batch: 181건 중 180 성공, 1 실패 (4253ms, threads=32)` —
  실패 1건은 암호 문서, 나머지는 `verify` 차이 0건.
- **④ 실패하면 무엇을 보나**:
  - `오류: 산출 경로가 겹칩니다 - out\x.hwp ← a/x.hwp · b/x.hwp` (exit 2) —
    **한 건도 쓰지 않고** 먼저 멈춘다(대소문자만 다른 이름도 충돌로 본다).
    입력을 폴더별로 나눠 실행한다.
  - 집계 규칙: `error` 있으면 1 / `verifyPages` 불일치 4 / `verify` 차이만 3 / 전부 통과 0.
  - `batch convert` 는 **MCP 에 노출되지 않는다**(파일을 쓰는 축이라 CLI 전용).
- **절감**: 배포본을 하나씩 열어 "다른 이름으로 저장"하던 일이 사라진다.

### 20. 붙임 일괄 PDF 변환 — 납품·게시용

- **① 목표**: 결재/게시용 PDF 를 문서마다 만든다.
- **② 명령 시퀀스**:
  ```bash
  while read -r f; do
    rhwp export-pdf "$f" -o "pdf/$(basename "${f%.*}").pdf" --font-path ./ttfs \
      || echo "FAIL $f"
  done < 목록.txt
  ```
- **③ 기계 검증**: 사람 모드 마지막 줄이 크기·쪽수를 준다 — 실측:
  `→ .../h.pdf (754KB, 4페이지)` / `PDF 내보내기 완료`. 원본 `info --json` 의
  `pageCount` 와 대조하면 쪽 유실을 잡는다.
- **④ 실패하면 무엇을 보나**:
  - 글꼴이 다르면 대체 폰트로 떨어진 것이다 — **오류가 아니라 exit 0** 이다.
    `--font-path` 로 `ttfs/` 를 주거나 `RHWP_FONT_PATH` 를 건다.
  - 메모리가 부족하면 `--text-as-paths` (텍스트 선택·검색은 포기).
  - `batch` 에 pdf 축은 없다 — 위처럼 셸 루프거나 `xargs -P` 로 병렬화한다(격차 G5).
- **절감**: 문서를 열어 "PDF 로 저장"을 반복하던 일이 사라진다.

---

## E. 생성·변환

### 5. 새 문서 생성 — JSON 명세 → HWPX

- **① 목표**: 시험지·양식 문서를 편집기에서 처음부터 조판하는 일을 없앤다.
- **② 명령 시퀀스**:
  ```bash
  rhwp build-from-ingest 명세.json -o 산출물.hwpx      # 스키마: tools/rhwp-ingest/schema/
  rhwp export-text 산출물.hwpx --json                   # 재독 — 내용 반영 대조
  rhwp export-pdf 산출물.hwpx -o 산출물.pdf             # 납품 확인용
  ```
- **③ 기계 검증**: 재독 텍스트를 명세와 대조.
- **④ 실패하면 무엇을 보나**: 잘못된 명세는 **위치·허용 키가 붙은 오류로 즉시 실패**한다
  (조용한 내용 유실 없음, #3358). 실측:
  ```
  오류: ingest JSON 파싱 실패 - … unknown field `nope`, expected one of `version`, `page_size`,
  `default_font`, `header_text`, `footer_text`, `form_label`, `passages`, `questions` at line 1 column 7
  ```
  → 허용 키 목록이 그대로 나오므로 스키마 문서를 열지 않아도 고칠 수 있다.
  `-o` 를 빠뜨리면 별개로 `오류: -o <출력 경로> 가 누락되었습니다` (exit 2).
  **범위 주의**: 현재 스키마는 **시험지 계열**(passages/questions)이다 — 일반 공문
  기안(결재란·항목체계)은 아직 범위 밖이다(아래 체크리스트 G4).
- **절감**: 반복 양식의 조판 소멸.

### 6. 형식 변환 + 무손실 검증 — "변환했는데 깨졌나?"를 기계로

- **① 목표**: 변환 결과를 원본과 나란히 띄워 눈으로 비교하는 일을 없앤다.
- **② 명령 시퀀스**:
  ```bash
  rhwp export-hwpx 원본.hwp 변환본.hwpx --verify --verify-pages   # IR 차이 3 / 쪽수 불일치 4
  rhwp ir-diff 변환본.hwpx 원본.hwp --json                        # 차이 봉투(카테고리·건수)
  ```
- **③ 기계 검증**: exit 0 이면 무손실 계약 통과 — 눈 비교 불필요. 통과 문구(실측):
  `검증 통과(--verify-pages): 4쪽` / `검증 통과(--verify): IR 차이 없음`.
- **④ 실패하면 무엇을 보나**:
  - **exit 3** — `verify.diffCount` 와 stderr 의 차이 예시. 실측:
    `{"verify":{"diffCount":301,"identical":false}}`,
    `[차이] section[0] paragraph[4]/ctrl[0]tbl.cell[28].p[0] char_shapes: expected=[(0,9),(1,9)] actual=[(0,9)]`
  - **exit 4** — `검증 실패(--verify-pages): 변환 전 4쪽, 재파싱 후 1쪽`.
    **줄면 내용 유실, 늘면 흘러넘침**이다(실측: 수식 문서 4→1, 대형 문서 64→65, 35→36).
  - `-o` 는 없다 — 출력은 **positional** 이다(`알 수 없는 옵션: -o`, exit 2).
  - `--verify` 통과인데 `ir-diff` 가 exit 3 일 수 있다 — **비교자가 다르다**
    (자기 라운드트립 vs 두 파일 비교). 같은 게이트에 섞지 마라.
- **절감**: 나란히 비교 소멸. 대량 변환의 품질 게이트가 CI 로 들어간다.

### 34. 구조화 교환 — DocLang XML

- **① 목표**: 외부 시스템(검색엔진·번역·아카이브)에 **의미 구조를 보존한 채** 넘긴다.
- **② 명령 시퀀스**:
  ```bash
  rhwp export-doclang 문서.hwp -o out.dclg.xml --assets-dir assets/ --json
  ```
- **③ 기계 검증**: 실측 봉투 —
  `{"assetCount":0,"bytes":720092,"doclangVersion":"0.6","format":"doclang","lossCount":37}`
  → **`lossCount` 가 손실 계량이다.** 0 이 아니면 무엇이 안 실렸는지 알고 넘긴다.
- **④ 실패하면 무엇을 보나**: `--assets-dir` 를 생략하면 그림이 base64 data URI 로
  XML 에 인라인되어 파일이 급격히 커진다(위 실측은 인라인, 720KB).
- **절감**: "HWP 라서 못 넣는다"는 시스템 연계 장벽이 사라진다.

---

## F. 요약·질의

### 7. 요약·질의응답 준비 — 페이지 청킹

- **① 목표**: 긴 문서를 읽고 요약하거나, 질문 받을 때마다 다시 찾아 읽는 일을 없앤다.
- **② 명령 시퀀스**:
  ```bash
  rhwp export-text 문서.hwp --json | jq -c '.pages[] | {page, text}'   # 페이지 단위 청크
  rhwp export-text 문서.hwp --json --max-chars 200000                   # 컨텍스트 상한 방어
  ```
- **③ 기계 검증**: `pageCount` == 렌더 쪽수. 청크의 `page` 는 시나리오 3 의 검색 주소·렌더
  대상과 같은 좌표계라 "요약 근거 몇 쪽" 인용이 성립한다. 실측: 편람 393쪽 / 2,618문단.
- **④ 실패하면 무엇을 보나**:
  - `truncated: true` + `omittedCount` (실측 275,330자 생략) — 잘린 것이다.
    `pages` 배열 자체는 전 페이지가 남고 **뒷부분 텍스트만 빈다**는 점에 주의
    (실측: `pages` 393개 유지).
  - 구조가 있는 문서면 쪽이 아니라 **절 단위**가 낫다 → 시나리오 29.
  - `pages[].text` 는 `untrustedFields` 다.
- **절감**: LLM 요약/RAG 입력이 좌표째 나온다 — 인용 검증 가능한 요약.

### 29. 초소형 모델용 한 번 호출 요약 (`digest`)

- **① 목표**: 도구 호출 예산이 적은 에이전트가 **한 번에** 문서를 파악한다.
  메타 + 개요 + 발췌 + 다음 행동 안내를 한 봉투로 받는다.
- **② 명령 시퀀스**:
  ```bash
  rhwp digest 문서.hwp --json --max-chars 2000        # 기본: 첫 페이지 발췌
  rhwp digest 문서.hwp --sections --json              # 절 단위 청크(쪽 주소 보존)
  rhwp digest 문서.hwp --pages 10..20 --json          # 특정 범위만
  ```
- **③ 기계 검증**: 실측(편람) —
  ```json
  {"format":"hwp5","pageCount":393,"paraCount":2618,
   "outline":["제1장 행정업무 운영 개요\t 1","제2장 공문서 관리 등 행정업무의 처리\t 19", …],
   "nextStep":"더 읽으려면 export-text --json -p <쪽>, 찾으려면 search --json","truncated":true}
  ```
  `--sections` 실측: `sectionsMode:"clause"`, 절마다 `{title,page,charCount,excerpt}`
  (`제2장 …` → page 3, charCount 564), `nextStep` 도 절 모드 문구로 바뀐다.
- **④ 실패하면 무엇을 보나**:
  - `sectionsMode` 가 `"page"` 면 구조를 못 찾아 쪽 단위로 폴백한 것이다.
  - `truncated: true` 면 `nextStep` 이 **다음 호출을 문장으로** 알려 준다 — 그대로 따르면 된다.
  - `outline[]`·`excerpt` 는 `untrustedFields` 다.
- **절감**: "무슨 문서인지" 파악에 3~4회 걸리던 호출이 1회가 된다.

### 28. 개정 전후 텍스트 비교

- **① 목표**: 개정판과 원본 사이에 **문장이 어떻게 달라졌는지** 사람이 읽을 형태로 낸다.
  (`ir-diff` 는 구조 차이, 이건 내용 차이다.)
- **② 명령 시퀀스**:
  ```bash
  rhwp export-text 원본.hwp --json  | jq -r '.pages[].text' > a.txt
  rhwp export-text 개정본.hwp --json | jq -r '.pages[].text' > b.txt
  diff -u a.txt b.txt | head -60
  ```
- **③ 기계 검증**: diff 가 비면 본문 동일. 남는 헝크마다 페이지 경계를 유지했으므로
  "몇 쪽에서 뭐가 바뀌었나"를 그대로 보고할 수 있다.
- **④ 실패하면 무엇을 보나**:
  - 전체가 달라 보이면 줄바꿈 위치 차이일 수 있다 — `tr -d '\n'` 후 문장 단위로 나눠 비교.
  - 표 안 텍스트는 셀 순서로 직렬화되므로, 표 중심 문서는 시나리오 13(CSV 대조)이 낫다.
  - 구조(컨트롤·모양)까지 봐야 하면 `ir-diff --json` 을 겹쳐라.
- **절감**: 개정 대비표를 손으로 만들던 일이 사라진다.

---

## G. 검증·비교

### 8. 배포본 동일성 검증 — "이 두 파일, 같은 문서인가?"

- **① 목표**: 배포된 판본과 원본을 나란히 띄워 관인·내용 변조를 눈으로 대조하는 일을 없앤다.
- **② 명령 시퀀스**:
  ```bash
  rhwp render-diff A.hwp B.hwp --json               # 기하 게이트: 변위 px·구조 불일치
  rhwp export-svg A.hwp -o a/ && rhwp export-svg B.hwp -o b/
  for p in a/*.svg; do cmp -s "$p" "b/$(basename "$p" | sed s/^A/B/)"; done   # 바이트 대조
  ```
- **③ 기계 검증**: `maxDisp 0.0` + `pageCountMismatch:false` + 페이지별 SVG 바이트
  일치 — "시각적으로 같은 문서"가 명령 판정값이 된다. 실측 정상 쌍:
  ```json
  {"mode":"pair","maxDisp":0.0,"overPages":0,"pageCountA":4,"pageCountB":4,
   "pageCountMismatch":false,"hardStructPages":0}
  ```
- **④ 실패하면 무엇을 보나**:
  - `--json` 은 회귀를 **exit 3**, 사람 모드는 **exit 1** 로 낸다 — 게이트를 섞지 마라.
  - `maxDisp` 가 0 이어도 `pageCountMismatch:true` 면 회귀다(실측 회귀 사례:
    `pageCountA:4, pageCountB:1, hardStructPages:1`, 사람 모드 `status: PAGE_MISMATCH`).
  - **주의** — render-diff 는 **기하(배치) 게이트**라 같은 자리·같은 크기의 이미지
    내용 교체는 바이트 대조(SVG/원본 BinData)로 잡는다.
- **절감**: 나란히 눈 대조 소멸.

### 27. 편집 전후 시각 회귀 확인

- **① 목표**: `edit` 로 값을 넣은 뒤 **레이아웃이 안 무너졌는지** 확인한다.
  칸 폭 초과·그림 겹침은 값 대조로는 안 잡힌다.
- **② 명령 시퀀스**:
  ```bash
  rhwp edit fill-fields 서식.hwp --data @row.json -o 완성본.hwp --json   # changedPages 확인
  rhwp render-diff 서식.hwp 완성본.hwp --json | jq '{maxDisp, overPages, pageCountMismatch}'
  rhwp export-svg 완성본.hwp -p 0 -o after/ --json     # 넘친 쪽만 눈으로
  ```
- **③ 기계 검증**: `maxDisp` 를 임계로 게이트한다(`--max-disp <px>`). 값이 늘어난 만큼
  줄이 밀리는 것은 정상이므로, 편집 봉투의 **`changedPages`**(실측 `[0,1]`, `[6,7]`)만
  비교 대상으로 좁힌다.
- **④ 실패하면 무엇을 보나**:
  - 편집 봉투의 `overflow[]` 가 원인을 먼저 알려 준다(`cellWidthPx` vs `textWidthPx`).
  - `pageCountMismatch: true` → 값이 길어 쪽이 늘었다. 서식 제출물이면 하드 실패다.
  - `hardStructPages > 0` → 구조 자체가 달라졌다. `typeDeltas` 로 어떤 노드가 늘었는지 본다
    (실측 형태: `Δ Column: 1→4 (+3)  Equation: 1→4 (+3)  TextRun: 2→8 (+6)`).
- **절감**: 채운 뒤 문서를 열어 훑어보던 마지막 눈 검증이 사라진다.

### 26. 발췌본 만들기 — 필요한 쪽만

- **① 목표**: 두꺼운 편람에서 해당 절만 떼어 회람한다.
- **② 명령 시퀀스**:
  ```bash
  rhwp search 편람.hwp "위임전결" --json | jq '.matches[0].page'    # 0 기준 → 35
  rhwp extract-pages 편람.hwp 발췌.hwp --from 36 --to 40 --json     # 1 기준!
  rhwp info 발췌.hwp --json | jq '.pageCount'
  ```
- **③ 기계 검증**: 봉투의 `pagesBefore` / `pagesAfter` / `paragraphsKept` /
  `paragraphsRemoved`. 실측: `{"from":2,"to":3,"pagesBefore":4,"pagesAfter":4,
  "paragraphsKept":1,"paragraphsRemoved":4}`.
- **④ 실패하면 무엇을 보나**:
  - **`pagesAfter` 가 요청 범위와 다를 수 있다** — 쪽 단위로 자르되 **문단 단위로 지우기**
    때문이다. "정확히 N쪽"이 계약이면 `export-pdf -p` / `export-svg -p` 를 써라.
  - 범위가 문서보다 커도 **exit 0** 이다(실측 `--to 99` 통과, `paragraphsRemoved:0`).
  - **이 명령만 1 기준**이다. 0 을 주면
    `오류: 쪽 추출 실패 - 렌더링 오류: 쪽 범위가 잘못됐습니다: 0..2 (1 기준, from <= to)` (exit 1).
  - `--to` 를 빠뜨리면 `오류: --to 가 필요합니다.` (exit 2).
- **절감**: 필요한 쪽만 뽑아 보내는 일이 명령 한 줄이 된다.

---

## H. 보안·공개

### 21. 개인정보 마스킹 후 배포

- **① 목표**: 공개·제3자 제공 전에 주민등록번호·전화·이메일·카드번호를 지운다.
- **② 명령 시퀀스**:
  ```bash
  cp 원본.hwp 사본.hwp                                        # ① 되돌릴 수 없다
  rhwp edit redact 사본.hwp --dry-run --json                  # ② 무엇이 지워지나
  rhwp edit redact 사본.hwp -o 공개본.hwp --verify --json     # ③ 적용
  rhwp export-text 공개본.hwp --json | grep -E '[0-9]{3}-[0-9]{4}' || echo "잔여 없음"
  ```
- **③ 기계 검증**: 실측 왕복 —
  ```json
  // ② dry-run
  {"findingCount":3,"findings":[
    {"kind":"phone","masked":"**-***-****","page":1,"paragraph":24,"raw":"02-123-4567"},
    {"kind":"phone","masked":"***-****-****","raw":"010-1234-5678"},
    {"kind":"email","masked":"****@*******.**","raw":"hong@example.kr"}],
   "kinds":["ssn","card","phone","email"],"mask":"*","redactedCount":0}
  ```
  ③ 이후 재독(`fields --json`) 실측:
  `담당자명 = '홍길동 ***-****-**** ****@*******.**'`, `담당자전화번호 = '**-***-****'`
  → **자릿수를 보존**하며 마스킹됐음이 데이터로 확인된다. `changedPages` 로 어느 쪽이
  바뀌었는지도 나온다.
- **④ 실패하면 무엇을 보나**:
  - **dry-run 의 `redactedCount` 는 항상 0** 이다. 게이트는 **`findingCount`**.
  - `findingCount: 0` 이 "개인정보 없음"을 뜻하지 않는다 — 탐지가 보수적이다
    (주민번호 검증숫자·카드 Luhn 통과·**하이픈 있는** 이동전화/02 번호만).
    하이픈 없는 번호, 031·051 지역번호는 안 잡힌다. 남은 것은 시나리오 15 로 지운다.
  - `오류: 마스킹은 되돌릴 수 없습니다. …` (exit 2) → `-o` 나 `--in-place` 를 명시.
  - `오류: 알 수 없는 --kind 값 - bogus (ssn|phone|email|card|all)` (exit 2).
  - **보안**: `--dry-run` 출력의 `findings[].raw` 에는 원문 개인정보가 그대로 있다.
    로그·티켓·LLM 컨텍스트에 남기지 마라.
- **절감**: 공개 전 검열을 손으로 하던 일이 게이트가 된다. 다만 **최종 책임은 사람**이다.

### 22. 메타데이터 제거 — 작성자·이력 지우기

- **① 목표**: 외부 배포본에서 작성자·최종수정자·작성일·미리보기를 지운다.
  본문만 검열해도 요약정보에 이름이 남는다.
- **② 명령 시퀀스**:
  ```bash
  rhwp edit sanitize 문서.hwp -o 배포본.hwp --json
  rhwp info 배포본.hwp --json | jq '.title'
  ```
- **③ 기계 검증**: `removed[]` 가 **지운 항목과 지우기 전 값**을 열거한다. 실측:
  ```json
  {"removed":[{"field":"title","before":"3"},{"field":"author","before":"edward"},
   {"field":"dateString","before":"2021년 11월 8일 월요일 오전 8:40:50"},
   {"field":"lastSavedBy","before":"edward"},
   {"field":"revisionNumber","before":"8, 0, 0, 466 WIN32LEWindows_7"},
   {"field":"createdAt","before":"2008-09-16T00:00:00Z"},
   {"field":"lastSavedAt","before":"2026-02-09T06:42:33Z"},
   {"field":"preview.text","before":"<  ><보도자료>…"}, …]}
  ```
  → `revisionNumber` 에 **작성 PC 의 OS 정보**까지 들어 있었다는 게 드러난다.
- **④ 실패하면 무엇을 보나**:
  - **본문은 건드리지 않는다.** 본문 개인정보는 시나리오 21 이 따로 필요하다.
  - 미리보기 이미지를 남기려면 `--keep-preview` (기본은 제거) — 시나리오 31 과 순서 충돌 주의.
  - HWPX 입력에 `-o *.hwp` 를 주면 형식 변환 경고가 stderr 로 나온다(실측) —
    `outputFormat` 으로 실제 저장 형식을 확인하라.
- **절감**: 배포 전 문서 정보 창을 열어 하나씩 지우던 일이 사라진다.

### 23. 출처 모르는 문서 안전 점검 — 프롬프트 주입

- **① 목표**: 메일·게시판에서 받은 문서를 **LLM 에 먹이기 전에** 지시문이 숨어 있는지 본다.
- **② 명령 시퀀스**:
  ```bash
  rhwp inspect injection 의심.hwp --json > inj.json
  jq '{clean, signalCount, highestConfidence}' inj.json
  jq -r '.injectionSignals[] | [.confidence, .scope, .page, .kind] | @tsv' inj.json
  rhwp inspect injection 의심.hwp --include-fields --json   # 누름틀·메모까지
  ```
- **③ 기계 검증**: 실측(주입 문구를 넣은 문서) —
  ```json
  {"clean":false,"highestConfidence":"high","signalCount":2,
   "injectionSignals":[{"confidence":"high","kind":"instruction_override",
     "matched":"이전 지시는 무시하","page":0,"paragraph":5,"scope":"tableCell",
     "why":"선행 지시를 무효화하라는 관용구입니다 — '이전/모든' 범위어 + '지시/지침' 목적어 + '무시/폐기' 서술어가 한 창 안에 모두 있습니다"}, …],
   "scanScopes":["body","tableCell","textBox","equation","footnote","endnote","header","footer"]}
  ```
  게이트는 **`clean == true`** 다. 깨끗한 문서는 `{"clean":true,"signalCount":0,…}`.
- **④ 실패하면 무엇을 보나**:
  - **신호를 찾아도 exit 0** 이다. exit 로 판정하면 전부 통과한다.
  - `scanScopes` 밖은 검사하지 않는다 — 요약정보·바탕쪽·OLE 내부·**이미지 속 글자**.
    "clean" 이 안전 보증이 아니라는 뜻이다.
  - 잡음이 많으면 `--min-confidence medium|high`.
  - `excerpt`·`matched` 는 `untrustedFields` 다 — **탐지 결과를 그대로 프롬프트에 붙이면
    탐지한 것을 실행하는 꼴**이 된다. 요약해서 사람에게 보고하라.
- **절감**: 문서 기반 에이전트의 가장 흔한 사고를 처리 전에 차단한다.

### 24. 은닉 텍스트·유니코드 기만 검사

- **① 목표**: "사람 눈엔 안 보이는데 텍스트 추출기는 읽는" 문자열, 그리고 표시 순서를
  뒤집거나 동형자로 속이는 문자를 찾는다.
- **② 명령 시퀀스**:
  ```bash
  rhwp inspect hidden-text 의심.hwp --json                       # 흰 글씨·0pt·쪽 밖
  rhwp inspect hidden-text 의심.hwp --include-offpage --threshold-pt 2.0 --json
  rhwp inspect unicode 의심.hwp --json                           # 제로폭·bidi·태그·동형자
  rhwp inspect unicode 의심.hwp --kind bidi --json               # 축을 좁혀서
  ```
- **③ 기계 검증**: 실측(제로폭 + bidi 를 넣은 문서) —
  ```json
  {"clean":false,"findingCount":2,"findings":[
    {"codepoint":"U+200B","kind":"zero_width","severity":"low",
     "excerpt":"수자원<U+200B>공사<U+202E>보고서","rendered":"수자원공사서고보",
     "location":"cell[0:0].para[0]",
     "why":"사람 눈에 보이지 않는 문자입니다 — 화면에 없는 내용이 LLM 이 읽는 텍스트에는 남습니다"},
    {"codepoint":"U+202E","kind":"bidi_override","severity":"high",
     "why":"표시 순서를 뒤집는 제어문자입니다 — 화면에 보이는 순서와 실제 문자 순서가 다릅니다"}],
   "kindCounts":{"bidi_override":1,"confusable":0,"tag_char":0,"zero_width":1},
   "scannedChars":1610,"severityCounts":{"high":1,"low":1,"medium":0}}
  ```
  `rendered`(화면)와 `raw`(실제 순서)를 **나란히** 주는 것이 이 명령의 핵심이다.
- **④ 실패하면 무엇을 보나**:
  - 세 축 모두 **판정이 `clean` 필드에만** 있다(exit 0).
  - `hidden-text` 가 0건인데 의심스러우면 `--include-offpage` 를 켜고
    `--threshold-pt` 를 올려 본다(기본 1.0).
  - 누름틀 **이름**이 동형자로 충돌하면 `fill-fields` 봉투의 `confusable` 로도 잡힌다 —
    그쪽은 값이 아니라 **이름** 축이다.
- **절감**: 문서 신뢰 판정이 사람의 눈이 아니라 명령 결과가 된다.

---

## I. 편집 기타

### 25. 도장·서명 삽입

- **① 목표**: 결재용 관인·서명 이미지를 정해진 자리에 붙인다.
- **② 명령 시퀀스**:
  ```bash
  rhwp edit insert-image 문서.hwp --image 도장.png \
       --page 0 --x 45000 --y 12000 --width 6000 --dry-run --json
  rhwp edit insert-image 문서.hwp --image 도장.png \
       --page 0 --x 45000 --y 12000 --width 6000 -o 결재본.hwp --json
  rhwp export-svg 결재본.hwp -p 0 -o check/ --json     # 최종 눈 확인(한 쪽만)
  ```
- **③ 기계 검증**: 실측 —
  ```json
  {"binDataId":7,"changedPages":[0],"height":1900,"outputFormat":"hwp5",
   "overflow":[],"page":0,"width":6000,"x":45000,"y":12000}
  ```
  `binDataId` 가 붙으면 실제로 이진 자원이 들어간 것이다(dry-run 은 `null`).
  `overflow: []` 가 "쪽 안에 들어갔다"는 판정. `height` 는 비율 유지로 계산된 값이다.
- **④ 실패하면 무엇을 보나**:
  - **단위가 HWPUNIT(1/7200 inch)** 이다. 픽셀로 착각하면 문서 밖으로 나간다.
    A4 세로 = **59528 × 84188**. 넘치면 `overflow` 가 좌표를 계량해 준다(실측:
    `{"bottomHu":92534,"overflowXHu":38472,"overflowYHu":8346,"paperWidthHu":59528,
    "paperHeightHu":84188,"rightHu":98000}`).
  - `--width` 만 주면 비율 유지, 둘 다 생략하면 원본 픽셀 × 75.
  - `오류: 지원하지 않는 그림 형식입니다 - csv (지원: png, jpg, jpeg, bmp, tif, tiff)` (exit 2)
    vs `오류: 그림 파일을 읽을 수 없습니다 - nope.png: …` (exit 1).
- **절감**: 결재 이미지 붙이기가 배치 가능한 단계가 된다.

### 31. 썸네일로 문서 식별

- **① 목표**: 대량 아카이브에서 "이게 무슨 문서인지" 목록 화면에 미리보기를 붙인다.
- **② 명령 시퀀스**:
  ```bash
  rhwp thumbnail 문서.hwp -o thumb.png
  rhwp thumbnail 문서.hwp --data-uri            # 웹 UI 에 바로 박을 때
  ```
- **③ 기계 검증**: `--base64` 출력이 `iVBORw0KGgo…` (PNG 시그니처)로 시작하면 유효한
  이미지다(실측 확인).
- **④ 실패하면 무엇을 보나**: PrvImage(미리보기)가 없는 문서는 추출할 게 없다.
  그때는 `export-svg -p 0` 또는 `export-png -p 0`(가용시)으로 첫 쪽을 렌더한다.
  `edit sanitize` 는 기본으로 미리보기를 **제거**하므로 순서에 주의(시나리오 22).
- **절감**: 파일명만 보고 문서를 찾던 일이 줄어든다.

---

## J. 운영

### 32. 문서 자체의 이상 좁히기 — info → diag → dump

- **① 목표**: "이 문서만 이상하다" 는 신고를 재현 가능한 사실로 좁힌다.
- **② 명령 시퀀스**:
  ```bash
  rhwp info 문서.hwp --json                    # ① 포맷·버전·쪽/문단 수·폰트
  rhwp diag 문서.hwp                           # ② 번호·글머리표·개요 구조 진단
  rhwp dump-pages 문서.hwp -p 0 --json         # ③ 그 쪽의 배치(문단·표) 목록
  rhwp dump 문서.hwp --section 0 --para 12     # ④ 조판부호 수준
  rhwp dump-records 문서.hwp                   # ⑤ HWP5 raw record (최후)
  ```
- **③ 기계 검증**: 실측 `diag` 출력 —
  ```
  === DocInfo 요약 ===
    Numbering: 1개  [0] start=0, formats: L1="^1.", L2="^2.", L3="^3)", …
  === ParaShape head_type 분포 ===  None: 502개, Outline: 7개, Number: 0개, Bullet: 0개
  === SectionDef 개요번호 ===  구역0: outline_numbering_id=1 → Numbering[0], flags=0x00000000
  ```
  `dump-pages --json` 은 `bodyArea`·`columns[].items[]` 로 배치를 좌표째 준다
  (실측 `{"bodyArea":{"height":933.57,"width":642.53,"x":75.59,"y":94.47},…}`).
- **④ 실패하면 무엇을 보나**: 각 단계에서 처음 이상해지는 지점이 원인 구역이다.
  `info` 가 이미 실패하면 입력 파일 자체의 문제(포맷·빈 파일·암호)다.
- **관련**: [문서 진단 도구 매뉴얼](document_diagnostics_tool_manual.md).
- **절감**: "이상하다"는 신고가 재현 명령 + 좌표로 바뀐다.

### 33. 성능 회귀 감시

- **① 목표**: 변경 전후로 파싱·레이아웃·렌더가 느려졌는지 같은 환경에서 잰다.
- **② 명령 시퀀스**:
  ```bash
  rhwp bench 대표문서.hwp -n 5 --tsv before.tsv
  # …변경 적용…
  rhwp bench 대표문서.hwp -n 5 --tsv after.tsv
  ```
- **③ 기계 검증**: 실측 출력 —
  ```
  파일                        크기KB   쪽    parse   layout   render  serialize    total
  20250130-hongbo.hwp         628.0    4    0.6ms    0.9ms    6.0ms     10.5ms    18.0ms
  ```
  단계가 갈려 나오므로 **어느 단계가 느려졌는지**까지 특정된다.
- **④ 실패하면 무엇을 보나**: 절대 수치는 머신·빌드 의존이다(도구가 스스로 경고한다).
  **같은 머신·같은 빌드의 상대 비교**로만 읽어라. `--batch` 로 폴더 단위 회귀를 본다.
- **절감**: "느려진 것 같다"가 숫자가 된다.

### 35. 도구 온보딩 — 표면을 한 번에 캐시

- **① 목표**: 에이전트가 **추측으로 플래그를 만들어 exit 2 를 밟는 일**을 없앤다.
- **② 명령 시퀀스**:
  ```bash
  rhwp capabilities > caps.json                  # 명령·플래그·봉투 필드·종료 코드
  jq -r '.commands[] | [.name, .category, (.flags|join(" "))] | @tsv' caps.json
  jq '.batch' < caps.json                        # 배치 축·집계 규칙·MCP 노출 여부
  rhwp capabilities --mcp         > mcp.json     # MCP 도구 정의(inputSchema 포함)
  rhwp export-capabilities-schema > schema.json  # 바인딩 코드 생성의 단일 출처
  rhwp export-provenance-map --json > prov.json  # '문서에서 온 값' 필드 지도
  ```
- **③ 기계 검증**: 호출 전에 `commands[].flags` 에 있는 플래그만 쓴다. feature 의존
  명령은 `available` 로 거른다(`export-png` 가 대표). `batch.exitAggregation` 실측 문자열:
  `error 레코드가 하나라도 있으면 1, 없고 verifyPages 불일치가 있으면 4, verify 차이만
  있으면 3, 전부 통과면 0`.
- **④ 실패하면 무엇을 보나**: `알 수 없는 옵션: -o` 류 exit 2 를 만났다면 **온보딩을
  건너뛴 것**이다. 자주 틀리는 조합 대조표는
  [에이전트 실패 사전](agent_troubleshooting_guide.md) §16.
- **절감**: 도구 학습 비용이 1회 호출로 압축된다. MCP 로 연결하면
  `rhwp://capabilities/mcp` 리소스로 같은 것을 받는다(실측 `resources/list` 확인).

---

## 공무원 업무 커버리지 체크리스트 — 편람 기준 (#3370)

기준은 임의 목록이 아니라 **행정안전부 「2025 행정업무운영 편람」**(저장소
`samples/2025 행정업무운영 편람(최종).hwpx`, 제2장 공문서 관리·제3장 효율적 수행)이다 —
이 구조 자체를 `export-structure --json` 으로 추출해 작성했다(도그푸딩. 실측
`nodeCount 591`, mode `clause`, 최상위 5장).
상태는 전부 실측: ○ = 오늘 CLI 로 실행 검증 / △ = 부분 / ✕ = 격차.

| # | 공무원 업무 (편람 근거) | CLI 대응 | 상태 |
|---|---|---|---|
| 1 | **기안문(공문서) 작성** — 두문·본문(항목체계)·결문·결재란 (§2-1절) | ingest 스키마가 시험지 전용(허용 키 `passages`/`questions` 실측 — 표·결재란·항목체계 없음) | **✕ 격차 G4** |
| 2 | 서식 기입(누름틀) (§2-3절 서식) | `fields` → `edit fill-fields` → 재독 대조 (시나리오 1) | ○ |
| 3 | 서식 취합·대량 발급(메일머지) | `batch fields` ○ / `batch fill` ○ (시나리오 9·10) | ○ |
| 4 | 계획서·보고서 구조 파악/검토 | `export-structure --json` (편람 591노드 실측) | ○ |
| 5 | 규정·근거 검색("몇 쪽에 있나") | `search --json` → 해당 쪽 렌더 (시나리오 3·16) | ○ |
| 6 | 표 자료 수확(엑셀화) | `export-tables` / `table-to-csv --bom` (시나리오 2) | ○ |
| 7 | 보고서 표 수치 갱신(실적 취합) | `edit set-cell` / `csv-to-table` 왕복 (시나리오 13·14) | ○ |
| 8 | 문구 일괄 치환(기관명·연도 개정) | `edit replace-text` (시나리오 15) | ○ |
| 9 | 개정 전/후 비교 | `export-text`+diff / `ir-diff --json` / `render-diff` (시나리오 27·28) | ○ |
| 10 | 형식 변환+무손실 게이트(hwp↔hwpx, PDF 배포) | `export-hwpx --verify` / `export-pdf` (시나리오 6·20) | ○ |
| 11 | 대량 문서 대장화(메타 스윕) | `batch info` (실측 353건 3.1s / 변환 181건 4.3s) | ○ |
| 12 | 민원·질의 대응 준비(요약·청킹) | `export-text --json` pages / `digest` (시나리오 7·29) | ○ |
| 13 | 개인정보 탐지→마스킹 | `edit redact` dry-run → 적용 → 재독 (시나리오 21) | ○ (탐지 보수적 — ④ 주의) |
| 14 | 문서 식별·미리보기 | `thumbnail` (시나리오 31) | ○ |
| 15 | 새 양식·시험지 생성 | `build-from-ingest` — 시험지 ○ / 일반 공문 양식 | △ G4 |
| 16 | **배포본 동일성 검증** — 변조·판본 확인 | `render-diff`(기하) + 페이지별 SVG 바이트 대조 (시나리오 8) | ○ |
| 17 | 보도자료 검토 — 제목·수치·일자 추출 | `export-text` + `search` + `extract-data` (시나리오 17) | ○ |
| 18 | 업무 인계·인수 문서 목록화 (편람 4장 1) | `batch info` 대장 + `export-structure` 지도 | ○ |
| 19 | 정책연구 보고서 등록·검색 (편람 3장 2절) | #4·#5·#12 와 동일 프리미티브 | ○ |
| 20 | 회의·행사 자료 취합→요약 배포 | 취합·요약 ○ / 자료 신규 작성은 G4 | △ G4 |
| 21 | 붙임 일괄 PDF 변환·배포 | `export-pdf` 건별 ○ / batch 에 pdf 축 없음(셸 루프) (시나리오 20) | △ G5 |
| 22 | 문서 배포 전 메타데이터 정리 | `edit sanitize` — 작성자·이력·미리보기 제거 실측 (시나리오 22) | ○ |
| 23 | **출처 불명 문서 안전 점검** | `inspect injection` (신호·근거·범위 실측, 시나리오 23) | ○ |
| 24 | 은닉 텍스트·유니코드 기만 검사 | `inspect hidden-text` / `inspect unicode` (시나리오 24) | ○ |
| 25 | 결재용 도장·서명 삽입 | `edit insert-image` (HWPUNIT 좌표, 시나리오 25) | ○ |
| 26 | 편집 원자성 보장(부분 실패 방지) | `run <계획.json>` — 선검증 실패 시 디스크 무변경 실측 (시나리오 12) | ○ |
| 27 | 외부 시스템 연계(구조화 교환) | `export-doclang` (lossCount 계량, 시나리오 34) | ○ |
| 28 | 서식 개정판 호환성 확인 | `batch fields` 이름 차집합 + `ir-diff` (시나리오 11) | ○ |
| 29 | 발췌본 회람 | `extract-pages` (쪽수 계약 주의, 시나리오 26) | ○ |
| 30 | 문서 이상 신고 분류 | `info` → `diag` → `dump-pages` (시나리오 32) | ○ |

**남은 격차 → 로드맵:**

- **G4 공문서 기안 생성 지원** — #1·#15·#20 해제. ingest 스키마에 표(결재란)·
  항목체계(1.→가.→1)→가))·두문/결문 요소가 필요하다. 현재 허용 키가
  `version, page_size, default_font, header_text, footer_text, form_label,
  passages, questions` 라는 것이 오류 메시지로 확인된다 — 시험지 계열 전용이다.
- **G5 배치 렌더/PDF 축** — #21 의 △ 해소. `batch` 에 `export-pdf`·`export-svg` 축이
  없어 셸 루프로 감싸야 한다. 대량 납품 파이프라인에서 반복되는 보일러플레이트다.
- **G6 배치의 credential 계약** — 암호 문서가 섞인 폴더는 배치가 통째로 exit 1 이 된다
  (`capabilities` 스스로 "암호화 batch 의 credential 전달 계약은 아직 정의되지
  않았다"고 밝힌다). 실무 아카이브에는 암호 문서가 섞여 있는 것이 정상이므로
  분리 실행이 강제된다 — 실측 353건 스윕의 실패 3건이 전부 이 사유였다.

새 축이 붙을 때마다 본 체크리스트의 해당 행을 ○ 로 갱신하고 시나리오 절을 추가하는 것을
DoD 로 제안한다 — 기능이 아니라 **줄어든 업무**가 릴리스 노트가 되도록.
