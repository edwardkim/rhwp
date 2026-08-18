---
kind: working
status: active
canonical: gym/docs/checks.md
last_verified: 2026-08-18
---

# gym 코어 지목 채점 연산자 — 작업 기록 (#5205)

정본 목록은 [`gym/docs/checks.md`](../../gym/docs/checks.md) 다. 이 문서는
왜 그 어휘가 생겼는지, 어떤 예외를 실패로 고정했는지, 단위시험이 무엇을
재현하는지를 남긴다. pack 과제 JSON 은 여기서 바꾸지 않는다.

## 한 줄 결론

에이전트 산출(JSON / CSV / NDJSON / 텍스트)을 `deep_contains` 로 메우면
"값이 어딘가 있으면 통과"가 되어 엉뚱한 칸을 채운 제출이 만점을 받는다.
#5205 는 좌표를 지목하는 파일 연산자를 `REGISTRY` 에 추가한다. CLI 는
부르지 않는다. 기존 키와 `GLOBAL_SCAN_OPS` 는 그대로다.

## 배경

gym 4부(#4653)가 판정 어휘를 `gym/core/checks.py` 한곳으로 모았다. pack 이
늘어도 과제 파일은 연산자를 고르기만 한다. 그런데 파일 산출을 채점하는
어휘가 `file_exists`·`json_value_eq`·`csv_cell_eq` 에 머물러 있으면, 길이와
키 집합과 줄 단위 관측을 과제마다 손 검사로 때우게 된다. 손 검사는 전역
훑기로  degenerates 한다.

#5205 1차는 여섯 개를 등록했다.

| op | 지목 |
|---|---|
| `json_len_eq` | JSON 배열/객체 길이 |
| `csv_row_count_eq` | CSV 논리 행 수 (`utf-8-sig`) |
| `ndjson_count_eq` | 비어 있지 않은 NDJSON 줄 수 |
| `ndjson_field_eq` | NDJSON `row` + `path` |
| `json_keys_contain` | 객체 키 포함 |
| `text_line_eq` | 텍스트 `line` 완전 일치 |

1차만으로는 타입·하한·헤더 전체·한 줄 포함을 표현하지 못한다. 과제가
`json_value_eq` 로 타입을 우회하거나, `text_line_eq` 전체 문자열을 박제하게
된다. 후속 지목 연산자는 같은 결(좌표, CLI 없음, 예외는 실패)로 구멍을
메운다.

| op | 지목 |
|---|---|
| `json_type_eq` | JSON 값 타입 이름 |
| `json_len_ge` | 길이 하한 |
| `json_array_item_eq` | 배열 `index` 항목 |
| `csv_col_count_eq` | 지목 행 열 수 |
| `csv_header_eq` | 0번 행 == `values` |
| `csv_row_eq` | 지목 행 == `values` |
| `ndjson_keys_contain` | NDJSON 행 키 포함 |
| `ndjson_len_eq` | NDJSON 행 배열/객체 길이 |
| `text_line_count_eq` | 텍스트 물리 줄 수 |
| `text_line_contains` | 지목 줄 부분 문자열 |

`text_line_contains` 는 파일 전체가 아니라 **한 줄**을 본다. 그래서
`GLOBAL_SCAN_OPS` 에 넣지 않는다.

## REGISTRY 를 깨지 않는 규칙

`REGISTRY[op] = (fn, needs_cli)` 다. 스키마(`schema.validate_task`)와 러너
(`eval_check`)와 단위시험이 이 맵을 공유한다.

지켜야 할 것:

1. **기존 키를 삭제하지 않는다.** `same_hash` 부터 `cell_text_eq` 까지
   1차 이전 키는 그대로 남아 있어야 한다.
2. **`needs_cli` 를 뒤집지 않는다.** 파일 연산자에 `True` 를 주면 모든
   기존 과제가 `cmd` 를 요구하게 되고, 봉투 연산자에 `False` 를 주면 CLI
   없이 빈 봉투를 본다.
3. **`GLOBAL_SCAN_OPS` 는 `{deep_contains, not_contains}` 만.** 지목
   연산자를 여기 넣으면 편집 과제가 스키마에서 막힌다.
4. **추가만 한다.** 새 키는 파일 연산자면 `(op_*, False)` 로 붙인다.

단위시험 `RegistryContractTests.test_registry_keeps_existing_keys` 가 1을
고정하고, `test_global_scan_ops_unchanged` 가 3을 고정한다.

## 경로 문법 (`dig`)

`a.b[2].c` 형태. 빈 문자열은 문서 전체. 구현은 `gym/core/checks.py` 의
`dig()`.

관측한 실패 모드:

| 입력 | 결과 |
|---|---|
| `""` | 루트 값 |
| `items` | 객체 키 |
| `items[0]` | 배열 0번 |
| `items[0].tags` | 중첩 |
| `items[0].tags[1]` | 중첩 배열 |
| `gone` | `KeyError` → 연산자 실패 |
| `items[9]` | `IndexError` → 실패 |
| `meta[0]` | 객체에 정수 키 → `KeyError` → 실패 |
| `note[0]` | 문자열 인덱싱은 Python 이 한 글자를 돌려주므로, 길이/키 연산자는
  타입 검사에서 거절해야 한다. `json_len_eq` 는 스칼라를 거절한다. |

`json_array_item_eq` 는 `path` 가 가리키는 **배열**에서 `index` 를 읽는다.
`path="items[0].tags"`, `index=1` 이 `dig` 경로 `items[0].tags[1]` 과 같다.
`index` 를 분리한 이유: 과제 JSON 에서 좌표를 숫자로 남기고, `bool` 거절을
한곳에서 하기 위함이다.

## `norm` 계약

`norm` 은 숫자와 숫자 문자열을 같게 본다. `bool` 은 숫자로 바꾸지 않는다.

| 왼쪽 | 오른쪽 | 같은가 |
|---|---|---|
| `3` | `"3"` | 예 |
| `3` | `"3.0"` | 예 |
| `3` | `3.0` | 예 |
| `True` | `1` | 아니오 (`bool` 우선) |
| `False` | `0` | 아니오 |
| `"갑"` | `"갑"` | 예 |
| `"01"` | `"1"` | 예 (`float`) |
| `None` | `None` | 예 (`return v`) |

CSV 헤더·행 전체 비교(`csv_header_eq`, `csv_row_eq`)는 `norm` 을 쓰지
않는다. 헤더 `"01"` 과 `"1"` 을 같다고 보면 열 이름이 섞인다. 셀 하나
비교(`csv_cell_eq`, `json_value_eq`, `json_array_item_eq`,
`ndjson_field_eq`)만 `norm` 을 쓴다.

## 예외 행렬

모든 신규 파일 연산자는 예외를 삼켜 `{"ok": False, "actual": "...실패..."}`
를 돌린다. 채점기가 스택을 흘리면 과제 하나가 런너를 죽인다.

### 파일 계층

| 상황 | json_* | csv_* | ndjson_* | text_* |
|---|---|---|---|---|
| 파일 없음 | 실패 | 실패 | 실패 | 실패 |
| 빈 파일 | JSON 파싱 실패 | 행수 0 (count 만 통과 가능) | 줄수 0 (count 만 통과 가능) | 줄수 0. 좌표 지목은 `줄 0 없음` |
| 잘못된 UTF-8 | 실패 | 실패 | 실패 | 실패 |
| UTF-8 BOM | JSON 은 BOM 을 거부할 수 있음. CSV 는 `utf-8-sig` 로 제거 | BOM 이 행을 늘리지 않음 | JSON 줄 파싱에 BOM 이 끼면 실패 | 첫 줄에 U+FEFF 가 남으면 `text_line_eq` 불일치 |
| CRLF | JSON 무관 | 논리 행은 LF 와 같음 | 빈 줄 규칙 동일 | `rstrip("\\r\\n")` 으로 벗김 |

### 좌표 계층

| 상황 | 적용 | 결과 |
|---|---|---|
| `row=-1` / `line=-1` / `index=-1` | 모든 좌표 연산자 | `음수` 실패 |
| `row=True` / `line=False` / `index=True` | 동일 | `정수` 실패 (`bool` 은 `int` 하위형) |
| 범위 밖 | 동일 | `없음` 또는 인덱스 실패 |
| `path` 부재 | JSON/NDJSON | `실패` |
| 스칼라에 `len` | `json_len_eq`, `json_len_ge`, `ndjson_len_eq` | `배열/객체가 아님` |
| 배열에 `keys` | `json_keys_contain`, `ndjson_keys_contain` | `객체가 아님` |
| `keys="id"` (문자열) | 키 연산자 | `문자열 목록` 실패 |
| `values="name"` | `csv_header_eq`, `csv_row_eq` | 동일 |
| `value=12` (숫자) | `text_line_contains`, `json_type_eq` | 타입 실패 |

### JSON 깨짐

다음 바디는 `json_len_eq`/`json_type_eq`/`json_keys_contain` 모두 실패다.

- `""` (빈 파일)
- `"   \\n"` (공백만)
- `'{"a":'` (잘림)
- `'{"a":1,}'` (trailing comma — 표준 json 모듈)
- `"{'a': 1}"` (작은따옴표)
- `"items=3"`
- `"[1,2] junk"`
- `"NaN"`, `"undefined"`

NDJSON 은 줄 단위라 한 줄만 깨져도 그 `row` 의 `ndjson_field_eq` 만 실패한다.
앞·뒤 줄은 살아 있다. `NDJSON_BAD` 픽스처(`{"id":1}\\n{not-json\\n{"id":3}`)에서
`row=2` 의 `id=3` 은 통과한다. 빈 줄을 세지 않으므로 `row` 는 0,1,2 다.

### CSV 따옴표 안 개행

```
name,note
"갑","여러
줄"
을,단줄
```

물리 줄은 4 줄이지만 `csv.reader` 논리 행은 3 이다. `csv_row_count_eq`
value=3, `csv_row_eq` row=1 values=`["갑", "여러\\n줄"]`. 물리 줄 수로 세는
과제는 오검출한다.

### 텍스트 줄 수

| 바디 | `text_line_count_eq` |
|---|---|
| `""` | 0 |
| `"한줄"` (개행 없음) | 1 |
| `"한줄\\n"` | 1 |
| `"한줄\\n\\n"` | 2 |
| `"갑\\n\\n을\\n"` | 3 |
| CRLF 3줄 | 3 |
| 40줄 픽스처 | 40 |

`text_line_eq` 는 마지막 개행 없는 줄도 읽는다. 빈 파일의 `line=0` 은
실패다 — 빈 줄을 통과로 위장하지 않는다.

## 픽스처 카탈로그 (단위시험)

`scripts/tests/test_gym_check_pinpoint.py` 가 아래를 코드 상수로 둔다.
별도 자산 파일은 만들지 않는다 — 파일 연산자 시험이 저장소 픽스처에
의존하면 pack 과 섞인다.

### `NESTED` JSON

```json
{
  "meta": {"schema": "v1", "ok": true, "count": 3},
  "items": [
    {"id": 1, "name": "갑", "tags": ["초안", "표"], "qty": 2},
    {"id": 2, "name": "을", "tags": ["확정"], "qty": 5},
    {"id": 3, "name": "병", "tags": [], "qty": 0}
  ],
  "empty": [],
  "blank": {},
  "note": "한 줄 메모",
  "flag": false,
  "nil": null,
  "pi": 3.14
}
```

관측:

- 루트 키 8개 → `json_len_eq` path="" value=8
- `items` 길이 3, `items[0].tags` 길이 2, `items[2].tags` 길이 0
- `meta.ok` 타입 `boolean`, `nil` 타입 `null`, `pi` 타입 `number`
- `flag` 에 `json_type_eq` value=`number` 는 실패

### CSV

- `CSV_SIMPLE`: `name,qty,note` + 갑/을/병 3 데이터. 논리 행 4.
- `CSV_QUOTED`: 따옴표 안 개행. 논리 행 3.
- `CSV_HANGUL_HEADER`: `이름,수량,비고`.
- `CSV_WIDE`: 8열.
- 0..30 행 생성 행렬: 헤더만 있으면 1, 빈 파일은 0.

### NDJSON

- `NDJSON_SIMPLE`: 레코드 3, 사이에 빈 줄·공백 줄.
- `NDJSON_TYPES`: null/bool/num/str/arr/obj 각 1줄.
- `NDJSON_BAD`: 가운데 줄 비 JSON.
- 0..25 레코드 생성 행렬(3의 배수마다 빈 줄 삽입). 빈 줄은 세지 않으므로
  기대 줄 수는 레코드 수와 같다.

### 텍스트

- `TEXT_SIMPLE`: `첫째\\n둘째\\n셋째` (마지막 개행 없음).
- `TEXT_CRLF`: CRLF 3줄.
- `TEXT_BLANK`: 가운데 빈 줄.
- `TEXT_HANGUL`: `이름: 홍길동` / `수량: 12` / `비고: 없음`.
- `TEXT_LONG`: `줄00` … `줄39` 40줄. `text_line_eq` 가 각 줄을 맞고, 이웃
  줄 값과는 불일치.

## 단위시험 지도

| 모듈 | 역할 |
|---|---|
| `scripts/tests/test_gym_checks.py` | 1차 6연산자 대표 경로 + REGISTRY/스키마 계약 + pack 스키마 불변 |
| `scripts/tests/test_gym_check_pinpoint.py` | 1차+후속 좌표 행렬, 유니코드, 예외 카탈로그, CLI 미호출 |
| `scripts/tests/test_gym_packs.py` | 기존 pack 구조. 연산자 추가만으로는 깨지지 않아야 한다 |
| `gym/tools/audit.py` | pack 정합. 연산자 PR 은 pack 을 건드리지 않으므로 통과해야 한다 |

`test_gym_checks.SchemaAcceptanceTests` 는 편집 축 pack 에서도 지목
연산자가 `cmd` 없이 통과하고, `cmd` 를 붙이면 `CLI` 오류가 나는지 본다.

`ExceptionPathCatalogTests.test_missing_file_fails_every_pinpoint_op` 는
16개 지목 연산자 전부에 대해 빈 제출 폴더를 돌린다. 하나라도 `ok=True` 면
부재를 통과로 위장한 것이다.

`test_no_cli_on_extra_ops` 는 `runner.run_cli` 를 폭발 스텁으로 갈아끼운다.
후속 10개 연산자가 스텁을 건드리지 않아야 한다.

## 스키마와의 경계

`validate_task` 는 연산자 필드의 세부를 검사하지 않는다. `op` 가
`REGISTRY` 에 있고, `needs_cli` 와 `cmd` 의 유무가 맞으면 통과다.
`row` 가 빠진 `ndjson_field_eq` 는 스키마가 아니라 실행 시
`KeyError` → 실패로 떨어진다. 과제 작성 실수는 채점에서 드러난다.

이 느슨함은 의도다. 필드 스키마를 과제 JSON 마다 강제하면 연산자 추가가
스키마 마이그레이션이 된다. 단위시험이 실행 시 예외를 고정한다.

편집 축(`axis` 가 `편집`/`보안`으로 시작)에서 `deep_contains` 는
`allowGlobalScan` 없이 거절된다. 지목 연산자는 이 축에서도 그대로 통과한다.

## pack 을 건드리지 않는 이유

연산자 PR 에서 기존 과제를 새 연산자로 갈아끼우면 두 가지가  entangle 된다.

1. 판정 어휘의 회귀와 과제 내용의 회귀를 한 커밋에서 구분할 수 없다.
2. `audit.py` 의 기준 풀이 왕복·ID 고유성 검사가 과제 변경에 민감해진다.

그래서 #5205 계열은 **어휘 + 단위시험 + 문서** 만 넣는다. pack 예시는
아주 작지 않으면 넣지 않는다. 과제가 새 연산자를 고르는 일은 별 PR 이다.

## CLI 를 부르지 않는 이유

파일 연산자의 존재 이유는 "바이너리 없이 산출물 좌표를 잰다" 다. 추출
결과를 rhwp 로 다시 뽑아 `value_eq` 하는 길이 이미 있다. JSON/CSV/NDJSON
산출은 에이전트가 쓴 파일 그 자체이므로, 그 파일을 여는 쪽이 맞다.

`needs_cli=False` 가 아니면 `schema` 가 `cmd` 를 요구하고, CI 의
unittest 잡이 바이너리를 찾게 된다. 이 checkout 은 crates 없는 작업
트리에서도 같은 시험을 돌려야 한다.

## 구현 메모

헬퍼는 후속 연산자만 쓴다. 1차 6개는 동작을 고정한 채 그대로 둔다.

- `_require_nonneg_int(name, value)` — `bool` 거절, 음수 거절
- `_require_str_list(name, value)` — `keys`/`values`
- `json_type_name(value)` — `bool` 을 `number` 로 부르지 않음
- `_csv_row_at(path, row)` — `utf-8-sig` + `csv.reader`
- `_ndjson_record_at(path, row)` — 빈 줄 제외 후 `json.loads`
- `_text_line_at(path, line)` — `rstrip("\\r\\n")`

`json_len_ge` 의 비교는 `float(actual) >= float(expected)` 다. 숫자
문자열 하한(`"3"`)을 받는다. `bool` 하한은 `float(True)==1.0` 이 되므로
권장하지 않는다. 과제는 정수 리터럴을 쓴다.

`csv_header_eq` 는 항상 0번 행을 본다. `row` 필드를 받지 않는다. 헤더가
두 번째 줄인 변칙 CSV 는 `csv_row_eq` row=1 로 지목한다.

`ndjson_count_eq` 는 JSON 유효성을 보지 않는다. 깨진 줄도 "비어 있지 않은
줄"이면 센다. 형식까지 보려면 같은 파일에 `ndjson_field_eq` 를 한 줄
이상 붙인다.

`text_line_contains` 의 빈 needle 은 Python `"" in s` 와 같이 참이다.
과제가 빈 문자열로 만점을 주지 않게 과제 쪽에서 `value` 를 비우지 않는다.

## 연산자별 필드 치트시트

| op | 필수 | 선택 | 비교 |
|---|---|---|---|
| `json_len_eq` | `file`, `value` | `path` | `norm` |
| `json_len_ge` | `file`, `value` | `path` | `float` 하한 |
| `json_type_eq` | `file`, `value` | `path` | 문자열 동등 |
| `json_array_item_eq` | `file`, `index`, `value` | `path` | `norm` |
| `json_keys_contain` | `file`, `keys` | `path` | 포함 (여분 허용) |
| `csv_row_count_eq` | `file`, `value` | | `norm` |
| `csv_col_count_eq` | `file`, `row`, `value` | | `norm` |
| `csv_header_eq` | `file`, `values` | | 리스트 동등 |
| `csv_row_eq` | `file`, `row`, `values` | | 리스트 동등 |
| `ndjson_count_eq` | `file`, `value` | | `norm` |
| `ndjson_field_eq` | `file`, `row`, `value` | `path` | `norm` |
| `ndjson_keys_contain` | `file`, `row`, `keys` | `path` | 포함 |
| `ndjson_len_eq` | `file`, `row`, `value` | `path` | `norm` |
| `text_line_eq` | `file`, `line`, `value` | | 문자열 동등 |
| `text_line_contains` | `file`, `line`, `value` | | `in` |
| `text_line_count_eq` | `file`, `value` | | `norm` |

모든 행에 `name` 을 붙이는 것은 pack 과제 계약이다. 단위시험 픽스처도
같은 습관을 따른다.

## 잘못된 사용

- **파일 전체를 `deep_contains`.** 추출 JSON 어딘가에 `"3"` 이 있으면
  통과한다. `json_len_eq` path=`items` value=3 을 쓴다.
- **`text_line_eq` 로 파일 전체 덤프를 한 줄에 박제.** 개행이 있으면
  첫 줄만 비교된다. 여러 줄이면 `text_line_eq` 를 줄마다 두거나
  `text_line_count_eq` 와 짝을 짓는다.
- **`csv_row_count_eq` 로 따옴표 안 개행을 물리 줄로 셈.** 논리 행을 쓴다.
- **`ndjson_field_eq` 의 `row` 를 물리 줄 번호로 셈.** 빈 줄은 건너뛴다.
- **`json_keys_contain` 로 키 부재를 검사.** 이 연산자는 포함만 본다.
  없어야 하는 키는 별 연산자가 없다. 전역 `not_contains` 로 우회하지
  말고, 필요한 키만 지목하거나 pack 을 기다린다.
- **파일 연산자에 `cmd`.** 스키마가 거절한다.
- **`json_type_eq` value=`int`.** 이름은 `number` 다. JSON 은 int/float
  구분이 없다.

## 로컬 게이트 (이 PR)

```
python -m unittest scripts/tests/test_gym_checks.py scripts/tests/test_gym_packs.py
python -m unittest scripts/tests/test_gym_check_pinpoint.py
python gym/tools/audit.py
```

`cargo fmt --all` 은 이 변경이 Python gym 코어·단위시험·문서뿐이라
돌리지 않는다. crates 가 없는 작업 트리에서도 위 세 명령이면 충분하다.

## 후속 (이 문서의 밖)

- pack 과제가 새 연산자를 고르는 일 — 별 PR, 기준 풀이 왕복 필수
- `json_keys_eq`(정확 집합), `ndjson_count_ge`, `csv_col_count_ge` 같은
  대칭 어휘 — 필요가 과제에서 나온 뒤에 추가
- 스키마가 `row`/`keys` 필수 키를 검증하게 하는 일 — 연산자 추가 비용을
  올리는 트레이드오프. 지금은 실행 시 실패로 둔다

## 변경 파일

- `gym/core/checks.py` — 후속 10개 연산자 + 헬퍼. 기존 REGISTRY 키 유지
- `gym/docs/checks.md` — 사람용 목록
- `mydocs/working/gym_core_checks.md` — 이 기록
- `scripts/tests/test_gym_checks.py` — 등록 계약 확장
- `scripts/tests/test_gym_check_pinpoint.py` — 좌표·예외 행렬

pack·profile·PARK.md·다른 gym/tools 는 바꾸지 않았다.
