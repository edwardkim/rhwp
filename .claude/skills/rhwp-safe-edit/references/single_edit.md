# 1층 단건 편집 — `edit` 하위명령

이 문서는 rhwp-safe-edit 의 **편집 1건** 경로다. 새 편집 로직을 설명하지 않는다.
이미 devel 에 있는 `edit` 6종과 인접 쓰기 명령(`csv-to-table`)을 에이전트가
원본을 깨지 않고 부르기 위한 조립 규약만 적는다.

권위: [`mydocs/manual/cli_commands.md`](../../../../mydocs/manual/cli_commands.md) §edit.
실측 루프: [`mydocs/manual/agent_surface_playbook.md`](../../../../mydocs/manual/agent_surface_playbook.md) §9.
회귀 테스트: `tests/edit_fill_fields_contract.rs`, `tests/edit_replace_text_contract.rs`,
`tests/edit_set_cell_contract.rs`, `tests/insert_image_contract.rs`,
`tests/edit_verify_contract.rs`, `tests/edit_format_preserve_contract.rs`,
`tests/edit_fit_check_contract.rs`, `tests/edit_field_occurrence_contract.rs`.

여러 편집을 한 파일에 원자적으로 적용해야 하면 이 문서를 닫고
[run_plans.md](run_plans.md) 로 간다. `edit` 를 이어 붙이지 않는다.

---

## 0. 1층을 고르는 조건

다음을 **모두** 만족할 때만 1층이다.

1. 바꿀 대상이 한 종류다 (누름틀이거나, 문자열이거나, 칸이거나, 그림이거나, PII 이거나, 메타다).
2. 중간 실패 시 반쯤 채워진 `-o` 산출물이 남아도 다음 명령으로 이을 필요가 없다.
3. 사용자가 "한 번에 / 원자적으로 / 여러 칸과 문구를 같이" 라고 하지 않았다.

하나라도 아니면 3층 `run` 이다.

`batch fill` 은 1층도 3층도 아니다. 서식 1 + 데이터 N행의 **메일머지** 다.
행마다 독립 산출물을 내고, 행별 `notFound` 가 있어도 최종 exit 0 일 수 있다.
N명분 발급이면 `batch fill` 이지 `run` 을 N번 돌리는 것이 아니다.

---

## 1. 공통 계약 — 여섯 명령이 같은 말

`edit` 6종은 아래를 공유한다. 명령마다 예외가 있으면 그 절에서만 덮어쓴다.

### 1.1 원본은 읽기만

실행기는 입력 경로를 열어 읽고, 변경은 메모리 IR 에서 끝낸 뒤 **다른 경로**에 쓴다.
입력 파일을 truncate 하지 않는다. 직렬화·쓰기 실패 시 산출 경로에도 부분 파일을
남기지 않는다(원자 쓰기). 원본 바이트는 실패 전후 동일하다.

### 1.2 산출 분리 (`-o`)

| 명령 | `-o` 생략 시 기본 이름 | `-o` 필수? |
|------|------------------------|:----------:|
| `edit fill-fields` | `<입력>_filled.<확장자>` | 아니오 |
| `edit replace-text` | `<입력>_replaced.<확장자>` | 아니오 |
| `edit set-cell` | `<입력>_cell.<확장자>` | 아니오 |
| `edit insert-image` | `<입력>_image.<확장자>` | 아니오 |
| `edit sanitize` | `<입력>_sanitized.<확장자>` | 아니오 |
| `edit redact` | **기본 이름 없음** | **예** (`-o` 또는 `--in-place`) |

에이전트 기본은 **항상 `-o` 를 명시**한다. 기본 이름이 생겨도 작업 디렉터리에
예기치 않은 `*_filled.hwp` 가 쌓이는 것을 막기 위함이다.

`edit redact` 는 `-o` 와 `--in-place` 둘 다 없으면 exit 2 로 **실행 자체를 거부**한다.
`-o` 가 입력 경로와 같아도 거부한다. 되돌릴 수 없는 마스킹을 기본 이름으로
조용히 만들지 않기 위한 보호다.

`--in-place` 는 `redact` 전용이다. 다른 `edit` 에 `--in-place` 를 붙이지 마라.
사용자가 "원본을 덮어써" 라고 **명시**하지 않으면 `--in-place` 를 제안하지 않는다.

### 1.3 `--dry-run` — 같은 명령줄에서 플래그 하나

선확인은 실행과 **같은 인자**에 `--dry-run` 만 더한 것이어야 한다.
dry-run 에서 본 `notFound`·`overflow`·`findingCount` 를 실행에서 다시 해석하지 않는다.

dry-run 계약:

- 산출 파일을 만들지 않는다. `-o` 가 가리키는 경로를 touch 하지 않는다.
- 봉투에 `dryRun: true` 가 실린다.
- `output` / `outputFormat` / `verify` / `binDataId` 는 실제 저장 때만 실린다.
- `changedPages` 는 항상 `null` 이다 (확정 불가 — 저장 전 재조판을 하지 않음).

### 1.4 `--json` — stdout 은 봉투만

`--json` 이면 stdout 은 JSON 한 덩어리다. 진행·경고·사람용 요약은 stderr 다.
stdout+stderr 를 한 버퍼에 붙잡아 파싱하지 마라.

단건 `edit` 가 exit 1 또는 2 로 끝나면 stdout 은 **0바이트**다 (부분 매니페스트 금지).
종료 코드를 먼저 보고 0/3/4 일 때만 파싱한다.

예외는 이 스킬 범위 안에서 `run` 과 `csv-to-table` 뿐이다 — 그 둘은 exit 2 에서도
`invalid[]` 봉투를 낸다. 1층 `edit` 6종은 그 예외가 아니다.

### 1.5 `--verify` — 자기 재파싱 게이트

저장 직후 산출 바이트를 다시 읽어 인메모리 IR 과 대조한다.

- 요청하지 않으면 봉투의 `verify` 는 **`null`** 이다. 통과가 아니라 **안 한 것**.
- `identical: true` · `diffCount: 0` 이면 exit 0.
- `identical: false` 이면 봉투를 출력한 뒤 **exit 3**. 산출 파일은 **남는다**.
- 이것은 `ir-diff <원본> <산출>` 이 아니다. 원본과 산출을 비교하지 않는다.
  저장본이 방금 직렬화한 IR 과 같은지를 본다.

무손실이 계약이면 별도로 `rhwp ir-diff <원본> <산출> --json` 을 돌린다.
차이 시 exit 3, `categories` 를 읽는다.

### 1.6 입력 형식 보존 (#3383)

| 입력 | 기본 산출 | 직렬화 |
|------|-----------|--------|
| `.hwpx` | `.hwpx` (`export_hwpx_native`) | HWPX 네이티브 |
| `.hwp` (HWP5/HWP3) | `.hwp` (HWP5) | 어댑터 경유 `export_hwp_with_adapter` |

봉투 `outputFormat` 은 `info --json` 의 `format` 과 같은 어휘다 (`hwp5` / `hwpx`).
두 봉투를 그대로 대조할 수 있다.

예외 하나: HWPX 입력에 `-o ….hwp` 를 **명시**하면 경로를 존중해 HWP5 로 저장하고
stderr 로 형식 변경·이미지·차트 유실 가능성을 경고한다. 형식 변환의 정식 통로는
`export-hwpx` 이지 `edit -o` 가 아니다.

반대로 HWP 입력에 `-o ….hwpx` 를 줘도 형식은 바뀌지 않는다 (경고만). 변환은
`export-hwpx` 가 담당한다.

### 1.7 무변경 산출물 금지

치환 0건(`replacedCount: 0`), 마스킹 탐지 0건(`findingCount: 0`)이면 출력 파일을
만들지 않고 봉투에 `output` 키 자체가 없다. "빈 산출물을 만들어 두었다"고 보고하지 마라.

`fill-fields` 는 지목한 키가 모두 `notFound` 여도 exit 0 일 수 있다. 이 경우에도
실제로 채운 칸이 0 이면 산출물을 완성본으로 다루지 않는다.

### 1.8 공통 플래그 표

| 플래그 | fill-fields | replace-text | set-cell | insert-image | redact | sanitize |
|--------|:-----------:|:------------:|:--------:|:------------:|:------:|:--------:|
| `-o/--output` | ○ | ○ | ○ | ○ | 필수* | ○ |
| `--dry-run` | ○ | ○ | ○ | ○ | ○ | — |
| `--json` | ○ | ○ | ○ | ○ | ○ | ○ |
| `--verify` | ○ | ○ | ○ | ○ | ○ | — |
| `--in-place` | — | — | — | — | ○ | — |
| `--keep-style` | — | — | ○ | — | — | — |
| `--ignore-case` | — | ○ | — | — | — | — |
| `--occurrence` | data 키 | ○ | — | — | — | — |
| `--no-raw` | — | — | — | — | ○ | — |
| `--keep-preview` | — | — | — | — | — | ○ |

`*` redact 는 `-o` 또는 `--in-place` 중 하나.

sanitize 에 `--dry-run`/`--verify` 가 없는 것은 이 스킬이 빼먹은 것이 아니다.
메타 제거는 본문을 건드리지 않으며, 재실행 `removedCount: 0` 이 적용 증거다.

---

## 2. 발견 — 쓰기 전에 주소를 얻는다

주소를 추측하지 않는다. 쓰기는 발견이 준 좌표만 받는다.

### 2.1 누름틀 — `fields --json`

```bash
rhwp fields 신청서.hwp --json
```

읽는 키:

- `fields[].name` — 채울 때 `--data` 키. 동명이면 선언 순서가 순번이다.
- `fields[].value` — 현재 값. 비어 있으면 미작성.
- `fields[].guide` (있으면) — 안내문. 실값과 섞지 않는다.

같은 이름이 N번 나오면 `--data '{"이름[0]":"…","이름[N-1]":"…"}'` (0 기준, #3476).
순번 없는 키는 **첫 매치만** 채우고 `ambiguous` 로 보고한다.

실측 샘플 `samples/field-01.hwp` 는 누름틀 11개다
(회사명/작성자/부서명/전화번호/이메일/제목/목차1×5). 테스트가 이 목록을
하드코딩하지 않고 `fields --json` 으로 다시 읽는다. 에이전트도 그렇게 한다.

### 2.2 표 격자 — `export-tables --json`

```bash
rhwp export-tables 양식.hwpx --json | jq '.tables[] | {index, rows, cols, cellCount}'
```

읽는 키:

- `tables[].index` — `--table` 에 넣는 값. **배열 순번이 아니다.** 0부터가 아닐 수 있다.
- `tables[].cells[].row` / `col` — `--row` / `--col`.
- 병합 앵커 — 덮인 칸은 값이 없다. 앵커 좌표만 쓸 수 있다.
- `cells[].nested` — 중첩 표. **v1 `set-cell` 범위 밖**.

표 전체를 스프레드시트에서 고칠 거면 `table-to-csv` → 외부 편집 → `csv-to-table` 이다.
그 왕복은 rhwp-table-exchange 스킬의 책임이다. 이 스킬은 칸 하나(`set-cell`)와
원자 계획서(`run` 의 `set_cell`)만 다룬다.

### 2.3 문자열 — `search --json`

```bash
rhwp search 공문.hwp "2025년" --json | jq '{matchCount, matches: [.matches[]|{page,section,paragraph}]}'
```

`matchCount` 가 0 이면 `replace-text` 를 돌리지 않는다. 1층은 0건 치환을 성공(exit 0,
산출 없음)으로 보고하고, 3층 `run` 은 0건을 **선검증 위반**(invalid + exit 2)으로 거부한다.
같은 찾기라도 층이 다르면 판정이 다르다.

### 2.4 그림 자리 — 쪽 좌표는 HWPUNIT

`insert-image` 의 `--x/--y/--width/--height` 는 픽셀이 아니다. 1 HWPUNIT = 1/7200 inch.
A4 세로 ≈ 59528 × 84188. 쪽 왼쪽 위가 (0, 0) 이다.

쪽 번호는 0 기준. `info --json` 의 `pageCount` 로 범위를 확인하고, 밖이면 실행기가
exit 2 로 거부한다.

---

## 3. `edit fill-fields` — 누름틀 채우기 (#3329)

검증된 코어 `set_field_value_by_name` 을 재사용한다. 필드 값만 바꾸므로
레이아웃·구조는 불변이다. 새 편집 로직이 없다.

### 3.1 사용법

```
rhwp edit fill-fields <파일.hwp|파일.hwpx> --data <JSON|@파일> [-o <출력>] [--dry-run] [--verify] [--json]
```

- `--data` — `{"필드이름":"값"}`. `@경로` 면 파일에서 읽는다. **UTF-8**.
  CP949 저장본은 `stream did not contain valid UTF-8` + exit 1.
- 값이 문자열이 아니면 JSON 표현을 문자열로 넣는다.
- 반복 항목: `이름[N]` (0 기준). 범위 밖 순번은 `notFound` 에 실린다.

### 3.2 봉투

```json
{
  "schemaVersion": "1.0",
  "source": "신청서.hwp",
  "dryRun": false,
  "filledCount": 2,
  "filled": [
    {"name": "회사명", "occurrence": 0, "value": "페타플로"},
    {"name": "작성자", "occurrence": 0, "value": "홍길동"}
  ],
  "notFound": [],
  "ambiguous": [],
  "output": "out/filled.hwp",
  "outputFormat": "hwp5",
  "verify": null
}
```

| 필드 | 뜻 | 완료 조건 |
|------|-----|-----------|
| `filledCount` | 실제로 쓴 칸 수 | 성공 판정이 **아님** |
| `filled[]` | 이름·순번·값 | 재독 대조의 기대값 |
| `notFound[]` | 없는 이름 또는 범위 밖 순번 | **반드시 빈 배열**이어야 완료 |
| `ambiguous[]` | `{name,matched,total}` 순번 없는 동명 | **반드시 빈 배열**이어야 완료 |
| `output` | 저장된 경로 | dry-run / 무변경 시 부재 |
| `verify` | 자기검증 또는 `null` | `--verify` 를 붙였는가 |

완료 식:

```
exit ∈ {0,3}  AND  notFound == []  AND  ambiguous == []  AND  filledCount == 지목한 키 수
```

`filledCount == 2` 이고 `ambiguous: [{name:"목차1", matched:1, total:5}]` 이면
5칸 중 1칸만 채운 것이다. 이것을 완성본으로 사용자에게 넘기지 마라.

### 3.3 권장 호출

```bash
rhwp fields samples/field-01.hwp --json | jq -r '.fields[].name'
rhwp edit fill-fields samples/field-01.hwp \
  --data '{"회사명":"페타플로","작성자":"홍길동"}' \
  --dry-run --json
rhwp edit fill-fields samples/field-01.hwp \
  --data '{"회사명":"페타플로","작성자":"홍길동"}' \
  -o out/field-filled.hwp --verify --json
rhwp fields out/field-filled.hwp --json \
  | jq -c '[.fields[]|select(.value!="")|{name,value}]'
```

워크스루: [../examples/01_fill_fields_single.md](../examples/01_fill_fields_single.md).

### 3.4 실패

- 입력 없음·파싱 실패·쓰기 실패 → exit 1, stdout 0바이트, 원본 불변.
- `--data` 누락·JSON 파싱 실패 → exit 2, stdout 0바이트.
- 필드 설정 중 하나라도 실패 → 산출 파일을 쓰지 않고 exit 1.

`notFound` 는 실패가 아니다. exit 0 이다. 봉투를 읽지 않으면 놓친다.

---

## 4. `edit replace-text` — 일괄 치환 (#3373)

검증된 코어 `replace_all` (역순 치환, 오프셋 안전)을 재사용한다. 새 편집 로직이 없다.
본문과 표 셀을 함께 본다.

### 4.1 사용법

```
rhwp edit replace-text <파일> --find <문자열> --replace <문자열> [-o] [--ignore-case] [--occurrence N] [--dry-run] [--verify] [--json]
```

- `--find` 빈 문자열은 exit 2 (문서 전체를 뜻하게 되므로).
- `--replace ""` 는 삭제다.
- `--ignore-case` 기본은 구별한다.
- `--occurrence k` (0 기준) 이면 그 한 건만. 체크박스 □→☑ 도 이 경로다.

### 4.2 봉투

```json
{
  "schemaVersion": "1.0",
  "source": "공문.hwp",
  "find": "2025년",
  "replace": "2026년",
  "caseSensitive": true,
  "dryRun": false,
  "replacedCount": 7,
  "changedPages": [0, 2, 3],
  "output": "개정본.hwp",
  "outputFormat": "hwp5",
  "verify": {"identical": true, "diffCount": 0}
}
```

`replacedCount: 0` 이면 `output` 키가 없고 파일을 만들지 않는다.

전건 치환은 여러 쪽에 걸친다. `--occurrence 3` 은 보통 한쪽이다.
눈검증은 `changedPages` 만 `export-svg -p N` 한다.

### 4.3 권장 호출

```bash
rhwp search 공문.hwp "2025년" --json | jq .matchCount
rhwp edit replace-text 공문.hwp --find "2025년" --replace "2026년" --dry-run --json
rhwp edit replace-text 공문.hwp --find "2025년" --replace "2026년" -o 개정본.hwp --verify --json
rhwp search 개정본.hwp "2025년" --json | jq .matchCount     # → 0
```

체크박스 한 칸:

```bash
rhwp edit replace-text 신청서.hwp --find "□" --replace "☑" --occurrence 0 -o 체크.hwp --json
```

여러 칸을 켜고 다른 편집과 같이 가야 하면 `run` 의 `set_checkbox` 다.
1층에서 `--occurrence` 를 여러 번 이어 붙이지 마라.

워크스루: [../examples/02_replace_text_single.md](../examples/02_replace_text_single.md).

---

## 5. `edit set-cell` — 표 한 칸 (#3381)

`export-tables` 와 같은 좌표계(`index`/`row`/`col`). 누름틀 없는 표 양식의
발견 → 기록 → 재독 을 한 주소로 닫는다. v1 범위는 본문 최상위 표와 셀 첫 문단.

### 5.1 사용법

```
rhwp edit set-cell <파일> --table <N> --row <R> --col <C> --text <값> [-o] [--keep-style] [--dry-run] [--verify] [--json]
```

- 좌표는 0 기준. `--table` 은 `export-tables` 의 `index`.
- 빈 `--text` 는 비우기다. 줄바꿈·탭은 v1 에서 거부 (exit 2).
- `--keep-style` 없으면 검정·비이탤릭·비진하게 (#3391). 파란 안내문 스타일을
  실값이 상속하지 않게 한다. 안내문 모양을 유지할 때만 `--keep-style`.

### 5.2 봉투

```json
{
  "schemaVersion": "1.0",
  "source": "양식.hwpx",
  "table": 12,
  "row": 1,
  "col": 1,
  "oldText": "",
  "newText": "1,234",
  "dryRun": false,
  "keepStyle": false,
  "overflow": [],
  "changedPages": [6, 7],
  "output": "작성본.hwpx",
  "outputFormat": "hwpx"
}
```

`oldText` 가 변경 이력이다. 로그에 그대로 남길 수 있다.

### 5.3 `overflow` (#3480)

넣은 값이 칸 폭을 넘치면 원소가 실린다.

```json
[{
  "target": "table0[2,3]",
  "text": "…",
  "cellWidthPx": 214.63,
  "textWidthPx": 440.0,
  "lines": 3
}]
```

- 채우기를 **막지 않는다**. 여러 줄이 정상인 칸(주소·사유)이 있다. 판단은 소비자 몫.
- `--dry-run` 에서도 검사된다. 파일을 만들기 전에 알 수 있다.
- 칸 폭은 `Cell.width` − 안여백, 글자 폭은 첫 문단 `CharShape.base_size` 기준
  한글 전각·ASCII 반각 **근사**다. 정밀 조판이 아니다.
- 조판 엔진이 있어야 한다. 에이전트는 렌더를 보지 않으므로 이 신호가 없으면
  표 밖으로 넘친 문서를 완성본으로 오판한다.

`overflow` 가 비지 않으면 사용자에게 알리고, 짧은 값으로 다시 dry-run 할지 묻는다.
무시하고 제출본으로 넘기지 마라.

### 5.4 병합·범위

병합으로 덮인 칸은 앵커 좌표를 안내하며 exit 2, stdout 0바이트.

```
(0,2) 는 병합으로 덮인 칸입니다 — 앵커 (0,1) 를 지정하세요.
```

격자 밖 좌표도 exit 2. 중첩 표는

```
본문 최상위 표 0 번이 없습니다 (최상위 표 0개; 중첩 표는 v1 범위 밖)
```

`--table 999` 처럼 없는 번호는 배열 길이로 추측하지 말고 `export-tables` 를 다시 읽는다.

### 5.5 권장 호출

```bash
rhwp export-tables 양식.hwpx --json | jq '.tables[] | select(.index==12) | {rows,cols}'
rhwp edit set-cell 양식.hwpx --table 12 --row 1 --col 1 --text "1,234" --dry-run --json
rhwp edit set-cell 양식.hwpx --table 12 --row 1 --col 1 --text "1,234" -o 작성본.hwpx --json
rhwp export-tables 작성본.hwpx --json \
  | jq -r '.tables[]|select(.index==12)|.cells[]|select(.row==1 and .col==1).text'
```

워크스루: [../examples/03_set_cell_single.md](../examples/03_set_cell_single.md).

---

## 6. `edit insert-image` — 도장·서명 (#3719 §6-5)

채워 넣은 서식에 직인을 얹는 제출의 마지막 조각. 인자 파싱·저장·봉투·`--verify`·
`changedPages` 형태는 `set-cell` 과 같다.

### 6.1 사용법

```
rhwp edit insert-image <파일> --image <그림> [--page N] [--x N --y N] [--width N --height N] [-o] [--dry-run] [--verify] [--json]
```

- `--image` 필수. 허용 확장자 **그리고** 내용: `png` `jpg` `jpeg` `bmp` `tif` `tiff`.
  그 밖은 문서를 읽기 전에 exit 2.
- 단위는 전부 HWPUNIT. `--page` 생략 = 0. 범위 밖이면 exit 2.
- `--width`/`--height` 둘 다 생략이면 원본 픽셀을 96dpi 로 환산.
  한쪽만 주면 원본 비율로 다른 쪽을 계산. `0` 은 exit 2.
- 쪽 밖으로 나가도 **자르지 않는다.** `overflow` 로만 알린다.
- 대체 텍스트는 삽입한 파일명을 그대로 쓴다.

### 6.2 봉투 요지

- `binDataId` — 실제 저장 때만. 방금 삽입한 BinData 참조.
- `overflow[]` — 넘칠 때만 원소 1개, 아니면 `[]`.
  `{page, paperWidthHu, paperHeightHu, rightHu, bottomHu, overflowXHu, overflowYHu}`.
- `changedPages` — 저장 후 쪽 번호(0 기준). dry-run 은 `null`.

```bash
rhwp edit insert-image 신청서_filled.hwp --image samples/images/moogung.jpg \
  --page 0 --x 50000 --y 70000 --width 5000 --height 5000 \
  -o 제출본.hwp --json | jq '{output, overflow, binDataId}'
```

워크스루: [../examples/04_insert_image_single.md](../examples/04_insert_image_single.md).

이 action 은 `run` 계획서에 없다. 도장까지 한 계획에 넣지 마라.
1층으로 산출을 만든 뒤 그 산출을 다음 1층 입력으로 쓰거나, 사용자가
원자성을 요구하면 "도장은 계획 밖 1층" 이라고 명시한다.

---

## 7. `edit redact` — 개인정보 마스킹 (#3719 §6-11)

탐지는 읽기 전용 코어 `document_core::queries::pii_scan`.
변경은 검증된 치환 경로 `replace_all_native`. 새 편집 로직이 없다.

### 7.1 사용법

```
rhwp edit redact <파일> [--kind ssn|phone|email|card|all] [--mask <문자>] [--dry-run] [--no-raw] [--verify] [-o <출력>|--in-place] [--json]
```

- `--kind` 기본 `all`. 쉼표 나열.
- `--mask` 한 글자. 영숫자는 거부. 두 글자 이상은 자르지 않고 exit 2.
- `--dry-run` 이 **권장 첫 단계**. `findings[]` 만 보고 파일을 만들지 않는다.
- `--no-raw` 는 `findings[].raw` 를 필드 자체에서 뺀다 (`null` 아님).
  로그·이슈에 봉투를 붙일 거면 처음부터 `--no-raw`.
- `-o` 또는 `--in-place` **필수**. 둘 다 없으면 exit 2. `-o` == 원본도 거부.

### 7.2 탐지 규칙 (보수적)

오탐은 본문을 훼손하고 되돌릴 수 없다. 형태가 맞아도 검증을 통과하지 못하면 탐지하지 않는다.

| 종류 | 형태 | 추가 검증 |
|------|------|-----------|
| `ssn` | `######-#######` | 생년월일 실재(윤년) + 성별/세기 1~8 + mod 11 |
| `card` | `4-4-4-4` / Amex `4-6-5` / 연속 15·16자리 | Luhn |
| `phone` | `01[016789]-3~4-4`, `02-3~4-4` | 하이픈 필수 |
| `email` | `지역부@라벨(.라벨)+` | 라벨 2개 이상 + TLD 영문 2자 이상 |

앞뒤가 숫자면 더 긴 토큰의 일부로 보고 버린다.
**02 외 지역번호, 13·14·19자리 카드, 여권·계좌는 v1 범위 밖.**

### 7.3 봉투 요지

```
findingCount, findings[{kind, raw?, masked, section, paragraph, page, charOffset}],
redactedCount, changedPages, dryRun, inPlace, noRaw, kinds, mask,
output?, outputFormat?, verify?
```

탐지 0건이면 출력 파일을 만들지 않는다.
`findings[].raw` 는 원문 개인정보다. 기본은 포함. 전송하지 마라.

### 7.4 권장 호출

```bash
rhwp edit redact 계약서.hwp --dry-run --json | jq '.findings[] | {kind, page, masked}'
rhwp edit redact 계약서.hwp --dry-run --no-raw --json > 검토용.json
rhwp edit redact 계약서.hwp -o 공개본.hwp --verify --no-raw --json \
  | jq '{redactedCount, changedPages, findingCount}'
```

워크스루: [../examples/05_redact_single.md](../examples/05_redact_single.md).

`run` 계획서에 `redact` action 은 없다. 공개 전 정리는 1층으로 분리한다.

---

## 8. `edit sanitize` — 메타데이터 제거 (#3719 §6-11)

본문 내용은 건드리지 않는다. `export-text` 결과가 전후 동일하다.

### 8.1 사용법

```
rhwp edit sanitize <파일> [--keep-preview] [-o <출력>] [--json]
```

- `--keep-preview` — 미리보기 **이미지**를 남긴다. 미리보기 텍스트는 언제나 대상.
- 기본 산출 이름: `<입력>_sanitized.<확장자>`.

지우는 대상 셋:

1. OLE 요약 정보 (`\x05HwpSummaryInformation`) — title/subject/author/keywords/
   comments/lastSavedBy/revisionNumber/dateString 과 FILETIME 시각들.
   바이트 길이를 바꾸지 않고 비운다.
2. HWPX 저작자 메타 (`Contents/content.hpf` 의 `<opf:metadata>`) — 중립 블록으로 교체.
3. 미리보기 (PrvText·PrvImage).

`removed[]` 는 거짓 보고를 하지 않는다. HWP5 직렬화기는 PrvText 가 비면 본문 앞부분으로
다시 채우므로, 미리보기 텍스트는 **지금 본문과 다를 때만** 지우고 보고한다.

**두 번째 실행은 `removedCount: 0`** 이다. 이것이 첫 실행이 실제로 지웠다는 증거다.

```bash
rhwp edit sanitize 보고서.hwp -o 배포본.hwp --json | jq '.removed[] | "\(.field): \(.before)"'
rhwp edit sanitize 배포본.hwp -o /tmp/재확인.hwp --json | jq .removedCount   # → 0
rhwp export-text 보고서.hwp > /tmp/a.txt
rhwp export-text 배포본.hwp > /tmp/b.txt
# 본문 동일해야 한다
```

워크스루: [../examples/06_sanitize_single.md](../examples/06_sanitize_single.md).

---

## 9. 인접 1층 — `csv-to-table` (#3719 §7)

표 전체를 CSV 로 덮어쓴다. `edit` 우산 아래는 아니지만 같은 안전 장치
(`--dry-run` · `-o` · `--verify` · `invalid[]`)를 쓴다.

```
rhwp csv-to-table <파일> --csv <경로.csv> --table <N> [-o] [--dry-run] [--verify] [--json]
```

치수 계약. 행·열이 다르거나 덮인 칸에 값이 있으면 **한 칸도 쓰지 않고**
`invalid[]` + exit 2. 이 점은 단건 `edit` 와 달리 실패해도 봉투가 나온다.

| `invalid[].reason` | 뜻 |
|--------------------|-----|
| `rowCountMismatch` | CSV 행 수 ≠ 표 행 수 |
| `colCountMismatch` | 해당 행의 열 수 ≠ 표 열 수 |
| `coveredCellNotEmpty` | 병합으로 덮인 칸에 값이 있음 |

처방: 손으로 CSV 를 만들지 말고 `table-to-csv` 가 뽑은 것을 값만 고친다.

워크스루: [../examples/13_csv_to_table_gate.md](../examples/13_csv_to_table_gate.md).

---

## 10. `batch fill` — 서식 1 + 데이터 N (#3238 계열)

```
rhwp batch fill --form <서식> --data <행.jsonl|csv> --out-dir <디렉터리> [--name-field <필드>] [--dry-run] [--json]
```

- 행마다 독립 산출물. 한 행의 `notFound` 가 다른 행을 막지 않는다.
- **행별 `notFound` 가 있어도 최종 exit 0** 일 수 있다.
- 완료 조건은 행 단위: 각 NDJSON 레코드의 `notFound == []` 그리고 `filledCount`.
- 먼저 `--dry-run` 으로 전행을 흘려 `notFound` 가 있는 행을 고친 뒤 실행한다.

원자적 다단계 편집이 아니다. N명분 메일머지다.
워크스루: [../examples/14_batch_fill_row_judgment.md](../examples/14_batch_fill_row_judgment.md).

---

## 11. 1층에서 금지하는 조립

1. `edit fill-fields -o a.hwp` 다음에 `edit replace-text a.hwp -o b.hwp` 를
   "한 작업"으로 묶지 마라. 중간 `a.hwp` 가 반쪽이다. `run` 으로 간다.
2. `--in-place` 를 fill/replace/set-cell/insert-image/sanitize 에 붙이지 마라.
3. HWPX 를 `-o out.hwp` 로 받아 형식을 바꾸지 마라. 바꾸려면 `export-hwpx` 가 아니다 —
   그 방향은 `convert` / 어댑터 경로이고, 편집의 일이 아니다.
4. `fields` 를 건너뛰고 필드 이름을 지어내지 마라.
5. `export-tables` 의 배열 순번을 `--table` 에 넣지 마라. `index` 다.
6. `overflow` 를 무시하고 제출본을 만들지 마라.
7. `filledCount` 만 보고 완료라고 하지 마라.
8. dry-run 의 `changedPages: null` 을 "바뀐 쪽 없음"으로 읽지 마라.
9. `--verify` 없이 저장한 뒤 `verify: null` 을 통과로 읽지 마라.
10. 새 하위명령(`edit ungroup-shape` 등)이 이 스킬 범위에 들어왔다고 가정하지 마라.
    이 스킬이 배선하는 1층은 위 6종 + `csv-to-table` + `batch fill` 이다.

---

## 12. 1층 호출 체크리스트

저장 버튼을 치기 전에 이 목록을 채운다.

- [ ] 발견 명령을 돌렸다 (`fields` / `export-tables` / `search` / `info`).
- [ ] 같은 명령줄에 `--dry-run --json` 을 붙여 봉투를 읽었다.
- [ ] `notFound`·`ambiguous`·`overflow`·`findingCount` 를 완료 조건에 넣었다.
- [ ] `-o` 가 입력과 다른 경로다. `redact` 면 `-o` 또는 명시적 `--in-place`.
- [ ] `--verify` 를 붙일지 결정했고, 안 붙이면 `verify: null` 을 통과로 말하지 않는다.
- [ ] 저장 후 재독 명령이 준비돼 있다 (`fields` / `export-tables` / `search` / 재 sanitize).
- [ ] 눈검증은 `changedPages` 가 배열일 때만, 그 쪽만 `export-svg -p N`.
- [ ] 다음 편집이 남아 있으면 1층을 닫고 [run_plans.md](run_plans.md) 로 간다.

---

## 13. 명령 ↔ 재독 대조표

| 쓰기 | 재독 | 기대 |
|------|------|------|
| `edit fill-fields` | `fields --json` | 채운 이름의 `value` 가 data 와 같다 |
| `edit replace-text` | `search --json` | `find` 의 `matchCount` 가 0 (전건) 또는 1 감소 (occurrence) |
| `edit set-cell` | `export-tables --json` | 해당 `index/row/col` 의 `text` 가 `newText` |
| `edit insert-image` | `info` / 쪽 `export-svg` | `overflow` 가 비었고 해당 쪽에 그림 |
| `edit redact` | `edit redact --dry-run` (산출에) | `findingCount: 0` (같은 kind) |
| `edit sanitize` | `edit sanitize` 재실행 | `removedCount: 0` |
| `csv-to-table` | `table-to-csv` | CSV 왕복이 같음 |
| `batch fill` | 행별 `fields` 또는 레코드 `notFound` | 모든 행 `notFound == []` |

재독이 기대와 다르면 산출물을 사용자에게 넘기지 않고, 원본은 그대로이므로
같은 명령줄을 고쳐 다시 돌린다.

---

## 14. 샘플 파일 (이 저장소)

테스트와 예제가 쓰는 공개 샘플이다. 경로를 지어내지 마라.

| 샘플 | 1층에서 쓰는 이유 |
|------|-------------------|
| `samples/field-01.hwp` | 누름틀 11개. fill-fields·run fill_fields 의 기본 입력 |
| `samples/form-01.hwp` | 서식. visual-regression 과 공유하는 공개 양식 |
| `samples/table-001.hwp` | 표 격자. set-cell / csv-to-table |
| `samples/hwp3-sample.hwp` | 다쪽 치환. `changedPages` 실측 |

새 HWP 픽스처를 이 스킬 폴더에 넣지 않는다. 기존 공개 샘플만 가리킨다.

---

## 15. 다음 문서

- 여러 편집 → [run_plans.md](run_plans.md)
- 저장 후 판정 루프 → [verify_loops.md](verify_loops.md)
- exit 3/4 와 `invalid[]` → [failure_envelopes.md](failure_envelopes.md)
- 단건 워크스루 → [../examples/README.md](../examples/README.md)
