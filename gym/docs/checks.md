---
kind: guide
status: active
canonical: gym/docs/checks.md
last_verified: 2026-08-18
---

# gym 채점 연산자 목록

판정 어휘의 정본은 `gym/core/checks.py` 의 `REGISTRY` 다. 이 문서는 그 등록부를
사람이 고를 수 있게 풀어 쓴 목록이다. 과제 JSON 은 연산자를 **고르기만** 하고
정의하지 않는다 — 과제마다 판정 논리가 흩어지면 #4600 같은 오검출이 pack 수만큼
늘어난다.

CLI 를 부르지 않는 파일 연산자(`needs_cli=False`)는 제출 폴더의 산출물만 본다.
바이너리 없이 단위시험이 돈다. 봉투 연산자(`needs_cli=True`)는 `cmd` 로 rhwp 를
호출해 나온 JSON 봉투의 좌표를 본다.

## 연산자를 고르는 규칙

1. **대상을 지목하라.** `deep_contains` 는 "값이 어딘가 있으면 통과"라 엉뚱한 곳을
   고친 제출을 걸러내지 못한다. 편집·산출 과제는 `value_eq`·`cell_text_eq`·
   `json_len_eq`·`csv_row_eq` 처럼 좌표를 지목하는 연산자를 쓴다.
2. **기대값을 박제하지 마라.** 정답은 채점 시점에 rhwp 로 재계산하거나
   (`answer_eq` 계열), rhwp 자신에게 판정을 시킨다(`{sha256:}` + replay).
   파일 연산자의 `value` 는 "이 자리의 관측"이지 골든 스냅샷이 아니다.
3. **부재를 통과로 위장하지 마라.** 좌표가 없으면 `None` 이 아니라 실패 문자열을
   남기고 `ok=False` 다. 빈 파일·깨진 UTF-8·음수 좌표·bool 좌표도 실패다.
4. **전역 훑기는 편집 과제에서 금지.** `GLOBAL_SCAN_OPS = {deep_contains,
   not_contains}`. 편집·보안 축에서 쓰려면 `allowGlobalScan` 으로 사유를 밝혀야
   한다(#4600).

`schema.validate_task` 는 미등록 연산자를 거절하고, `needs_cli` 가 거짓인데
`cmd` 가 있으면 거절한다. 등록만 하고 스키마를 우회하는 길은 없다.

## 파일 연산자 — CLI 를 부르지 않는다

아래 연산자는 제출물 자체를 연다. `file` 은 제출 폴더 기준 상대 경로다.

### 해시·존재

| op | 필드 | 판정 |
|---|---|---|
| `same_hash` | `files` (2개) | 두 파일 SHA-256 이 같다 |
| `differs_from_input` | `file` | 산출 SHA-256 ≠ 과제 `input` SHA-256 |
| `file_exists` | `file`, `minBytes?` | 파일이 있고 크기가 하한 이상(기본 1) |
| `files_differ` | `files` | 두 파일 해시가 서로 다르다 |
| `utf8_bom` | `file`, `value?` | 선두 3바이트가 UTF-8 BOM 인지 |

`differs_from_input` 은 무편집 복사본을 거부한다. `file_exists` 만 쓰면 빈
껍데기나 쓰레기 바이트가 통과할 수 있으므로, 형식 연산자(`xml_root_eq`,
`json_type_eq`, `csv_header_eq`)와 짝을 짓는다.

### XML

| op | 필드 | 판정 |
|---|---|---|
| `xml_root_eq` | `file`, `value` | root local-name 이 `value` |

파싱 실패(깨진 마크업, 파일 없음)는 `ok=False` 이고 `actual` 에
`XML 파싱 실패:` 접두가 붙는다.

### JSON 지목 — #5205 와 후속

에이전트가 남긴 `answer.json`·구조 덤프·추출 결과를 **전역 훑기 없이** 좌표로
잰다. 경로 문법은 `dig()` 의 점·대괄호: `items[0].tags`, `meta.ok`, 빈 경로는
문서 전체.

| op | 필드 | 판정 |
|---|---|---|
| `json_value_eq` | `file`, `path?`, `value` | 좌표 값이 `value` (숫자 문자열은 `norm` 으로 같게) |
| `json_len_eq` | `file`, `path?`, `value` | 배열/객체 길이가 `value` |
| `json_len_ge` | `file`, `path?`, `value` | 배열/객체 길이가 `value` 이상 |
| `json_type_eq` | `file`, `path?`, `value` | 타입이 `array`/`object`/`string`/`number`/`boolean`/`null` |
| `json_array_item_eq` | `file`, `path?`, `index`, `value` | 배열의 0부터 세는 항목이 `value` |
| `json_keys_contain` | `file`, `path?`, `keys` | 객체가 `keys` 문자열을 모두 가짐 |

공통 예외 (`ok=False`, `actual` 에 `실패` 포함):

- 파일 없음, 빈 파일, 깨진 JSON, 잘못된 UTF-8
- `path` 가 없거나 인덱스 범위 밖 (`KeyError`/`IndexError`)
- `json_len_eq`/`json_len_ge` 가 스칼라·null 을 만남 (`TypeError`)
- `json_keys_contain` 이 배열/스칼라를 만남, 또는 `keys` 가 문자열 목록이 아님
- `json_array_item_eq` 의 `index` 가 음수이거나 `bool` (`True` 는 `int` 의
  하위형이라 명시 거절)
- `json_type_eq` 의 `value` 가 문자열이 아님

`norm` 은 `bool` 을 숫자로 바꾸지 않는다. `json_len_eq` 의 `value=True` 는
길이 1 과 같지 않다. 숫자와 숫자 문자열(`"3"`, `"3.0"`)만 같다.

#### 예: 추출 결과 길이

```json
{
  "name": "항목 수",
  "op": "json_len_eq",
  "file": "answer.json",
  "path": "items",
  "value": 3
}
```

`{"items":[1,2,3],"extra":[]}` 는 통과한다. `deep_contains` 로 `"items"` 문자열만
찾으면 `extra` 만 채운 제출도 통과한다 — 그것이 지목 연산자를 쓰는 이유다.

#### 예: 필수 키

```json
{
  "name": "식별 키",
  "op": "json_keys_contain",
  "file": "answer.json",
  "path": "row",
  "keys": ["id", "name"]
}
```

여분 키는 허용한다. 키 집합이 정확히 같아야 하면 `json_len_eq` 로 키 개수를
한 번 더 잰다.

#### 예: 타입 고정

```json
{
  "name": "tags 는 배열",
  "op": "json_type_eq",
  "file": "out.json",
  "path": "items[0].tags",
  "value": "array"
}
```

`true`/`false` 는 `boolean` 이지 `number` 가 아니다. `null` 은 `object` 가 아니다.

### CSV 지목

CSV 는 `utf-8-sig` 로 연다. 선두 BOM 은 헤더 첫 칸을 더럽히지 않는다.
`csv.reader` 가 따옴표 안 개행을 한 논리 행으로 묶는다.

| op | 필드 | 판정 |
|---|---|---|
| `csv_cell_eq` | `file`, `row`, `col`, `value` | 좌표 셀이 `value` (`norm`) |
| `csv_row_count_eq` | `file`, `value` | 논리 행 수(헤더 포함)가 `value` |
| `csv_col_count_eq` | `file`, `row`, `value` | 지목 행의 열 수가 `value` |
| `csv_header_eq` | `file`, `values` | 0번 행이 문자열 목록과 정확히 같음 |
| `csv_row_eq` | `file`, `row`, `values` | 지목 행 전체가 `values` 와 같음 |

`csv_header_eq`/`csv_row_eq` 는 `norm` 을 쓰지 않는다. 헤더 `"01"` 과 `"1"` 은
다른 칸이다. 셀 값 비교가 필요하면 `csv_cell_eq` 를 쓴다.

예외:

- 파일 없음, 깨진 UTF-8, `csv.Error`
- `row`/`col` 이 음수·`bool`·범위 밖
- 빈 파일에 `csv_header_eq` — 헤더 행 없음
- `values` 가 문자열 목록이 아님

#### 예: 표 추출 행 수

```json
{
  "name": "헤더+데이터 3행",
  "op": "csv_row_count_eq",
  "file": "table.csv",
  "value": 3
}
```

```json
{
  "name": "헤더",
  "op": "csv_header_eq",
  "file": "table.csv",
  "values": ["이름", "수량", "비고"]
}
```

```json
{
  "name": "첫 데이터 행",
  "op": "csv_row_eq",
  "file": "table.csv",
  "row": 1,
  "values": ["갑", "1", "초안"]
}
```

### NDJSON 지목

한 줄이 한 레코드다. `iter_ndjson_lines` 는 앞뒤 공백을 버리고 **빈 줄은 세지
않는다**. 줄 번호(`row`)는 비어 있지 않은 줄만 0부터 센다.

| op | 필드 | 판정 |
|---|---|---|
| `ndjson_count_eq` | `file`, `value` | 비어 있지 않은 줄 수 |
| `ndjson_field_eq` | `file`, `row`, `path?`, `value` | 그 줄 JSON 의 좌표가 `value` |
| `ndjson_keys_contain` | `file`, `row`, `path?`, `keys` | 그 줄(또는 `path` 객체)이 키를 모두 가짐 |
| `ndjson_len_eq` | `file`, `row`, `path?`, `value` | 그 줄의 배열/객체 길이 |

예외:

- 파일 없음, 깨진 UTF-8
- 대상 줄이 JSON 이 아님 (`ndjson_field_eq`/`keys`/`len`)
- `row` 음수·`bool`·범위 밖
- 길이 연산자가 스칼라를 만남
- `keys` 가 문자열 목록이 아님

빈 줄만 있는 파일의 `ndjson_count_eq` 는 0 이다. `ndjson_field_eq` 의 `row=0` 은
실패(`행 0 없음`)다.

#### 예: 배치 스트림 한 건

```json
{
  "name": "둘째 레코드 id",
  "op": "ndjson_field_eq",
  "file": "batch.ndjson",
  "row": 1,
  "path": "id",
  "value": 2
}
```

```text
{"id":1,"name":"갑"}

{"id":2,"name":"을"}
{"id":3}
```

위 픽스처에서 비어 있지 않은 줄은 3개이고, `row=1` 은 `id=2` 다. 물리 줄 번호로
세면 틀린다.

### 텍스트 지목

텍스트는 UTF-8 로 연다. 줄 끝 `\n`/`\r\n` 은 비교 전에 벗긴다. 마지막 줄에
개행이 없어도 한 줄로 센다.

| op | 필드 | 판정 |
|---|---|---|
| `text_line_eq` | `file`, `line`, `value` | 0부터 세는 한 줄이 `value` 와 같음 (부분 일치 아님) |
| `text_line_contains` | `file`, `line`, `value` | 그 줄이 `value` 부분 문자열을 가짐 |
| `text_line_count_eq` | `file`, `value` | 물리 줄 수. 빈 줄도 센다. 빈 파일은 0 |

`text_line_contains` 는 **한 줄**을 지목한다. 파일 전체를 훑지 않으므로
`GLOBAL_SCAN_OPS` 가 아니다. 빈 문자열 needle 은 모든 줄에 포함된다.

예외:

- 파일 없음, 깨진 UTF-8
- `line` 음수·`bool`·범위 밖
- 빈 파일에서 `line=0` — `줄 0 없음`
- `text_line_contains` 의 `value` 가 문자열이 아님

#### 예: 작업 영수증 한 줄

```json
{
  "name": "둘째 줄",
  "op": "text_line_eq",
  "file": "receipt.txt",
  "line": 1,
  "value": "수량: 12"
}
```

```json
{
  "name": "이름 포함",
  "op": "text_line_contains",
  "file": "receipt.txt",
  "line": 0,
  "value": "홍길동"
}
```

## 봉투 연산자 — CLI 를 부른다

`cmd` 가 필수다. 파일 연산자에 `cmd` 를 붙이면 스키마가 거절한다.

| op | 필드 | 판정 |
|---|---|---|
| `answer_eq` | `cmd`, `path?`, `answer` | 봉투 좌표 == `answer.json` 의 키 |
| `len_answer_eq` | `cmd`, `path?`, `answer` | 봉투 좌표 길이 == answer 키 |
| `len_ge` | `cmd`, `path?`, `value` | 길이 ≥ value |
| `value_eq` | `cmd`, `path?`, `value` | 봉투 좌표 == value (`norm`) |
| `value_ge` | `cmd`, `path?`, `value` | 숫자 ≥ value |
| `value_in` | `cmd`, `path?`, `values` | 값이 목록 중 하나 (`norm`) |
| `deep_contains` | `cmd`, `path?`, `value` | 부분 문자열 전역 존재 (전역 훑기) |
| `not_contains` | `cmd`, `path?`, `value` | 부분 문자열 전역 부재 (전역 훑기) |
| `cell_text_eq` | `cmd`, `path?`, `table`, `row`, `col`, `value` | 표 좌표 텍스트 |

편집 과제에서 `deep_contains`/`not_contains` 를 쓰려면 `allowGlobalScan` 이
있어야 한다. 표는 `cells[0]` 순서 가정이 아니라 `(row, col)` 로 찾는다(#4600).

## REGISTRY 계약

`REGISTRY` 는 `(callable, needs_cli)` 맵이다. 기존 키를 지우거나 `needs_cli`
플래그를 뒤집으면 스키마·채점·단위시험이 한꺼번에 깨진다.

현재 파일 연산자(`needs_cli=False`):

- `same_hash`, `differs_from_input`, `file_exists`, `files_differ`
- `xml_root_eq`, `json_value_eq`, `csv_cell_eq`, `utf8_bom`
- `json_len_eq`, `csv_row_count_eq`, `ndjson_count_eq`, `ndjson_field_eq`
- `json_keys_contain`, `text_line_eq`
- `json_type_eq`, `json_len_ge`, `json_array_item_eq`
- `csv_col_count_eq`, `csv_header_eq`, `csv_row_eq`
- `ndjson_keys_contain`, `ndjson_len_eq`
- `text_line_count_eq`, `text_line_contains`

현재 봉투 연산자(`needs_cli=True`):

- `answer_eq`, `len_answer_eq`, `len_ge`
- `value_eq`, `value_ge`, `value_in`
- `deep_contains`, `not_contains`, `cell_text_eq`

`GLOBAL_SCAN_OPS` 는 `{deep_contains, not_contains}` 만 가진다. 지목 연산자를
여기에 넣지 않는다.

## 예외 경로 요약

모든 파일 연산자는 다음을 **통과로 위장하지 않는다**.

| 상황 | 결과 |
|---|---|
| 제출 파일 없음 | `ok=False`, `actual` 에 실패/없음 |
| 빈 JSON 파일 | 파싱 실패 |
| 빈 CSV / 빈 NDJSON / 빈 텍스트 | 행·줄 수 0 은 통과 가능. 좌표 지목은 실패 |
| 잘못된 UTF-8 | `UnicodeError` → 실패 |
| 음수 `row`/`col`/`line`/`index` | `IndexError` → 실패 (`음수`) |
| `bool` 좌표 (`True`/`False`) | `TypeError` → 실패 (`정수`) |
| 경로 없음 | `KeyError`/`IndexError` → 실패 |
| 배열이 필요한데 객체/스칼라 | `TypeError` → 실패 |
| `keys`/`values` 가 문자열 목록이 아님 | `TypeError` → 실패 |

단위시험은 `scripts/tests/test_gym_checks.py` 와
`scripts/tests/test_gym_check_pinpoint.py` 가 위 행렬을 파일 픽스처만으로 재현한다.
CLI 스텁을 심어 파일 연산자가 `run_cli` 를 부르지 않는지도 본다.

## 새 연산자를 넣을 때

1. `gym/core/checks.py` 에 `op_*` 를 추가한다. 예외는 삼켜서 `ok=False` 로 돌려
   채점기가 죽지 않게 한다.
2. `REGISTRY` 에 **추가만** 한다. 기존 키·플래그는 그대로 둔다.
3. 좌표 연산자면 `GLOBAL_SCAN_OPS` 에 넣지 않는다. `needs_cli=False` 면 `cmd` 를
   받지 않는다.
4. `gym/docs/checks.md`(이 문서)와 `mydocs/working/gym_core_checks.md` 에 필드·
   예외·예시를 적는다.
5. `scripts/tests/test_gym_checks.py` 에 등록 계약과 대표 예외를,
   `test_gym_check_pinpoint.py` 에 좌표 행렬을 넣는다.
6. pack 과제는 연산자가 안정된 뒤에만 고른다. 연산자 PR 에서 pack 을 대량으로
   바꾸지 않는다.
7. `python -m unittest scripts/tests/test_gym_checks.py scripts/tests/test_gym_packs.py`
   와 `python gym/tools/audit.py` 를 돌린다.

## 관련

- 구현: `gym/core/checks.py`, `gym/core/schema.py`, `gym/core/runner.py`
- 작업 기록: `mydocs/working/gym_core_checks.md`
- 이슈: #5205 (JSON/CSV/NDJSON 지목 채점), #4653 (판정 단일 출처), #4600 (좌표 지목)
