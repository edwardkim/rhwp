---
kind: guide
status: active
canonical: gym/docs/coverage.md
last_verified: 2026-08-18
---

# gym 커버리지 측정 — pack×명령 격자와 미사용 연산자

이 문서는 `gym/tools/coverage.py` 가 내는 `gymCoverage` 1.0 봉투의 정본이다.
구현은 [`gym/tools/coverage.py`](../tools/coverage.py), 계약 시험은
[`scripts/tests/test_gym_coverage.py`](../../scripts/tests/test_gym_coverage.py)
다. 작업 기록은 [`mydocs/working/gym_coverage.md`](../../mydocs/working/archives/gym_coverage.md)
를 본다.

## 왜 이 도구인가

새 gym 과제를 만들기 전 "이 능력이 이미 커버돼 있나"를 재는 장치가 없어서,
이미 `fill-fields` 를 커버한 core-cli T07 위에 중복 pack(#4781)이 만들어졌다
자진 철회된 사고가 있었다. 이 도구는 그 재발을 막는다 — 만들기 전에
**진짜 빈 곳**만 잰다.

명령 합계만으로는 부족하다. `export-text` 가 어떤 pack 에 있는지는 합계가
말하지 않는다. `value_in` 이 REGISTRY 에 등록돼 있는데 어떤 과제도 안 쓰는지도
합계가 말하지 않는다. 그래서 같은 스캔이 **pack×명령 격자**(`packs`)와
**미사용 검사 연산자**(`unusedOperators`)를 같은 봉투에 낸다.

기존 키(`agentFacingTotal`·`covered`·`uncovered`·`coveragePercent`·
`uncoveredByCategory`·`coveredCommands`·`excludedNonAgent`)의 의미는
그대로다. 새 키는 덧붙일 뿐 분모를 바꾸지 않는다.

## 무엇을 재나 (정직한 분모)

capabilities 의 `category` 로 **에이전트-대면 명령**만 분모로 삼는다.

| 포함 | 제외 |
|---|---|
| `batch` · `edit` · `export` · `query` | `diagnostic` · `internal` · `serve` |

`diagnostic`(hwp5-*·dump-* 개발 probe)·`internal`·`serve`(인프라)를 분모에
넣으면 커버리지가 실제보다 낮게 나와, 진단 도구를 '빈 곳'으로 오인하게 만든다.
잴 게 없으면(분모 0) 빈 곳도 없다 — `coveragePercent` 는 100 이고 0 나누기를
하지 않는다.

한 명령은 gym 과제·기준풀이의 `checks[].cmd[0]` 또는 `steps[].run[0]` /
`steps[].answer.*.cmd[0]` 에 나타나면 '노출'로 친다. 두 번째 토큰
(`fill`·`--kind`)은 세지 않는다. 격자 칸은 명령 이름이지 플래그가 아니다.

## 사용

```bash
python gym/tools/coverage.py --bin target/debug/rhwp
python gym/tools/coverage.py --bin target/debug/rhwp --json
python gym/tools/coverage.py --capabilities cap.json --json
```

`--bin` 과 `--capabilities` 둘 다 없으면 종료 코드 2 다. 바이너리 없이
순수 함수(`measure`·`report`·`used_commands_by_pack`·`unused_operators`)를
시험할 수 있다. 가드는 그 경로만 탄다.

## 봉투 계약 (`gymCoverage` 1.0)

```json
{
  "kind": "gymCoverage",
  "schemaVersion": "1.0",
  "agentFacingTotal": 40,
  "covered": 18,
  "uncovered": 22,
  "coveragePercent": 45,
  "uncoveredByCategory": {"edit": ["csv-to-table"], "query": ["explain"]},
  "coveredCommands": ["batch", "export-text", "info"],
  "excludedNonAgent": ["hwp5-inventory", "mcp-serve"],
  "packs": {
    "extraction": ["chart-to-csv", "export-text", "extract-data"],
    "table-csv": ["csv-to-table", "export-tables", "table-to-csv"],
    "batch-ops": ["batch", "search"]
  },
  "unusedOperators": ["deep_contains", "value_in"]
}
```

### 기존 키

- `agentFacingTotal` — 분모. 에이전트-대면 카테고리의 **고유 명령 이름** 수.
- `covered` / `uncovered` — 분모 ∩ gym 호출 / 분모 − gym 호출.
- `coveragePercent` — `100 * covered // total`. 정수 나눗셈. 분모 0 이면 100.
- `uncoveredByCategory` — 미노출 명령을 카테고리별로 모아 정렬.
- `coveredCommands` — 노출된 명령 이름, 정렬.
- `excludedNonAgent` — 분모 밖 명령 이름, 정렬.

### 격자 키 `packs`

`packId → [명령…]`. 행은 `packs/<id>/pack.json` 이 있는 폴더다. 과제가
없는 pack 도 **빈 목록**으로 남긴다. 누락된 행과 빈 행을 구분하는 것이
격자의 존재 이유다.

한 pack 의 과제 `cmd` 와 기준풀이 `run` / `answer.*.cmd` 를 합친다.
명령은 정렬되고 중복은 제거된다. pack 사이를 넘나들지 않는다 —
extraction 의 `export-text` 가 table-csv 행에 새지 않는다.

### 미사용 연산자 `unusedOperators`

`gym.core.checks.REGISTRY` 키 가운데, 어떤 과제의 `checks[].op` 에도
안 나온 이름을 정렬해 낸다. **기준풀이는 세지 않는다.** 기준풀이에는
`checks` 가 없고, 있어도 채점 연산자가 아니기 때문이다.

라이브 오라클의 기본 연산자(`answer_eq`·`file_exists`)는 실제 gym 에서
반드시 쓰인다. 가드가 그걸 고정한다. `deep_contains` 는 편집 축에서
전역 훑기로 막혀 있어, 조회 pack 이 안 쓰면 미사용으로 남는 것이
정상이다.

## 사람용 출력

`--json` 이 없으면 다음 블록을 낸다.

```
에이전트-대면 gym 커버리지: 18/40 (45%)
미노출 (진짜 빈 곳 — 여기부터 새 과제):
  [edit] csv-to-table
  [query] explain
제외(비-에이전트 N개): diagnostic·internal·serve 는 분모 밖
pack×명령 격자 (18 pack):
  [extraction] chart-to-csv, export-text, extract-data
  [render-tree] (없음)
미사용 연산자 (2): deep_contains, value_in
```

격자가 비면 `(pack 스캔 없음)`, 한 pack 의 명령이 없으면 `(없음)`,
미사용 연산자가 없으면 `REGISTRY 전부가 과제에 노출됨` 을 찍는다.
JSON 키를 사람용에서 빼지 않는다 — 리뷰어가 `--json` 없이 돌려도
격자·미사용을 본다.

## 스캔 규칙 (구현이 지키는 것)

1. `list_pack_ids` 는 `pack.json` 이 있는 폴더만 행으로 삼는다.
   매니페스트 없는 디렉터리·파일은 무시한다.
2. `iter_pack_docs` 는 `tasks/` · `reference/` 아래 `*.json` 만 읽는다.
   `notes.txt` 같은 비 JSON 은 건너뛴다.
3. `commands_in_doc` 은 빈 `cmd`/`run` 과 비-객체 `answer` 를 무시한다.
4. `operators_in_doc` 은 `op` 키가 있는 check 만 모은다.
5. `used_commands_by_pack` 은 빈 pack 을 `[]` 로 남긴다.
6. `unused_operators(registry=…)` 는 시험을 위해 등록부를 주입할 수 있다.
   생략하면 라이브 `REGISTRY` 를 읽는다.
7. `measure` 는 파일·바이너리에 접근하지 않는다. 가드가 픽스처로 시험한다.
8. `report` 는 capabilities 배열 + gym 루트 스캔을 한 봉투로 합친다.

## 새 과제를 만들기 전에

1. `python gym/tools/coverage.py --bin target/debug/rhwp --json` 을 돌린다.
2. `uncoveredByCategory` 가 진짜 빈 곳이다. 여기 없는 명령을 과제로 만들면
   T07/`fill-fields` 중복이 다시 생긴다.
3. `packs[<id>]` 가 이미 그 명령을 갖고 있으면, **같은 pack 안에서** 표본·
   필드·플래그만 갈라 과제를 늘린다. 새 pack 을 만들지 않는다.
4. `unusedOperators` 에 있는 연산자를 쓰려면, 그 연산자가 편집 축
   `GLOBAL_SCAN_OPS`(`deep_contains`·`not_contains`)인지 먼저 본다.
   편집 pack 은 좌표 지목(`csv_cell_eq`·`cell_text_eq`)을 쓴다.
5. 과제를 넣은 뒤 격자를 다시 잰다. 새 명령이 그 pack 행에 나타났는지,
   쓰인 연산자가 `unusedOperators` 에서 빠졌는지 확인한다.

## 이 PR 이 격자에 채운 칸

얇은 pack 세 곳을 **기존 명령만** 으로 두껍게 했다. 새 CLI·새 pack·
T07 복제는 없다.

### extraction — 조회 (읽기)

| 명령 | 과제 | 지목 |
|---|---|---|
| `chart-to-csv` | EX01 세로막대, EX15 가로막대, EX16 꺽은선, EX17 원형, EX18 HWPX 쌍 | `rowCount` |
| `export-text` | EX02·EX11·EX12·EX14·EX22 쪽수, EX04·EX13 글자수 | `pageCount`/`pages[].text` |
| `extract-data` | EX03·EX05·EX06 종류, EX07 `--kind all`, EX08–EX10 시험지, EX19·EX20 0건, EX21 `--limit` | `itemCount` |

같은 명령이라도 `--kind date|amount|number` 와 `--limit` 과 표본이 다른
계약이다. 값을 JSON 에 박제하지 않는다 — `answer_eq`/`len_answer_eq` 가
채점 시점에 rhwp 를 다시 돌린다.

### table-csv — 편집 (표 CSV 왕복)

| 명령 | 과제 | 지목 |
|---|---|---|
| `table-to-csv` | TC02·TC04·TC14 좌표, TC05·TC06 BOM, TC10–TC13·TC15·TC17 치수 | `csv_cell_eq`/`utf8_bom`/`rowCount`/`colCount` |
| `csv-to-table` | TC01·TC03·TC09·TC16 되쓰기, TC07·TC08 `--dry-run` | `cell_text_eq` · `changedCount` |
| `export-tables` | 되쓰기 재검증 | `cell_text_eq` |

편집 축이므로 `deep_contains` 를 쓰지 않는다. 격자는
`samples/hwpx/basic-table-01.hwpx` 의 1..12, `(1,2)=7` 이다.

### batch-ops — 자동화 (메일머지)

| 명령 | 과제 | 지목 |
|---|---|---|
| `batch fill` | BO01–BO12 | 산출 파일 존재 + 원본과 다름 |
| `search` | 각 부의 병합 토큰 | `matchCount >= 1` |

`edit fill-fields`(T07) 가 아니다. 서식 1 + 데이터 N → 문서 N 부다.
HWPX/HWP5 × CSV/JSONL × 순번/`--name-field` 를 갈라 놓았다.

## 하지 않는 것

- 새 pack 디렉터리를 만들지 않는다.
- core-cli T07 을 복제하지 않는다. `fill-fields` 는 그 과제의 소유다.
- `pack.json` 의 `runner` 신원을 바꾸지 않는다. 요구 명령만 이미 선언된
  것을 쓴다.
- 진단 명령을 분모에 넣지 않는다.
- 기준풀이 `steps` 의 연산자를 `unusedOperators` 분모에서 빼지 않는다.
- 골든 숫자(쪽수·항목 수)를 과제 JSON 에 박제하지 않는다.

## 가드가 고정하는 불변식

`scripts/tests/test_gym_coverage.py` 와 `scripts/tests/test_gym_packs.py`
가 다음을 매 CI 마다 확인한다.

- 분모는 에이전트-대면만. diagnostic/serve/internal 은 빈 곳이 아니다.
- `measure` 가 `packs`·`unusedOperators` 를 항상 싣는다. 스캔을 안 넘기면
  빈 값이다.
- 격자 행·명령은 정렬되고 pack 사이를 넘나들지 않는다.
- 실제 gym 의 extraction/table-csv/batch-ops 행에 위 표의 명령이 있다.
- REGISTRY − 과제 op = `unusedOperators`. `answer_eq`·`file_exists` 는
  미사용이 아니다.
- `--json` 사람용 모두 격자·미사용을 언급한다. 소스 없으면 exit 2.
- 세 pack 의 과제 ID 는 EX01–EX22 · TC01–TC17 · BO01–BO12 이고 각 짝
  기준풀이가 있다.
- `python gym/tools/audit.py` 가 위반 0 이다.

## 실패 모드 — 격자를 잘못 읽는 법

1. **합계만 보고 새 pack 을 만든다.** `covered` 가 낮다고 빈 곳이 아니다.
   `uncoveredByCategory` 에 없는 명령을 과제로 만들면 T07 사고가 재발한다.
2. **한 pack 행이 비었다고 그 명령을 전 gym 이 안 쓰는 줄 안다.** 격자는
   pack 축이다. `export-text` 는 extraction 에 있고 core-cli 에도 있다.
   빈 행은 "이 pack 이 안 쓴다"이지 "저장소가 안 쓴다"가 아니다.
3. **미사용 연산자를 버그로 본다.** `deep_contains` 가 남는 것은 편집 축이
   전역 훑기를 막아서다. 조회 pack 이 쓰기 전에는 남는 것이 맞다.
4. **기준풀이 `run` 의 두 번째 토큰을 명령으로 센다.** `batch fill` 의
   명령은 `batch` 다. `fill` 은 하위명령이다. 격자에 `fill` 칸은 없다.
5. **`--kind date` 를 새 명령으로 센다.** 플래그는 명령이 아니다. 같은
   `extract-data` 칸 안에서 표본·종류·필드를 갈라 과제를 늘린다.
6. **골든 숫자를 과제에 박제한다.** `pageCount: 2` 를 JSON 에 적으면
   픽스처가 진화할 때 과제가 거짓말을 한다. 라이브 오라클만 쓴다.
7. **편집 과제에 `deep_contains` 를 넣는다.** 스키마가 막는다. 좌표를
   지목하거나 `allowGlobalScan` 으로 사유를 밝혀야 한다.
8. **`runner` 신원을 과제 추가와 함께 고친다.** 신원은 기준 실행의
   지문이다. 과제를 늘리는 일과 무관하다.

## 순수 함수 표

바이너리·네트워크 없이 시험 가능한 진입점이다.

| 함수 | 입력 | 출력 |
|---|---|---|
| `commands_in_doc` | 과제/기준 JSON | 명령 이름 집합 |
| `operators_in_doc` | 과제 JSON | 연산자 이름 집합 |
| `list_pack_ids` | gym 루트 | pack id 목록(정렬) |
| `iter_pack_docs` | gym 루트, `tasks`/`reference` | `(packId, path, doc)` |
| `used_commands` | gym 루트 | 전 pack 명령 집합 |
| `used_commands_by_pack` | gym 루트 | pack → 명령 목록 |
| `used_operators` | gym 루트 | 과제 op 집합 |
| `registered_operators` | (없음, REGISTRY) | 등록 연산자 |
| `unused_operators` | gym 루트, 선택 registry | 미사용 목록 |
| `measure` | capabilities + used + 선택 격자/미사용 | `gymCoverage` 봉투 |
| `report` | capabilities + gym 루트 | 스캔을 합친 봉투 |
| `format_human` | 봉투 | 사람용 문자열(개행으로 끝) |

`measure` 에 `packs`/`unused_operators` 를 안 넘기면 빈 격자·빈 목록이다.
가드 `test_measure_keeps_schema_and_adds_grid_keys` 가 이 기본값을 고정한다.

## 라이브 gym 에서 기대하는 격자 칸

가드가 실제 `gym/packs` 를 스캔해 확인하는 최소 집합이다.

```
extraction : chart-to-csv, export-text, extract-data
table-csv  : csv-to-table, export-tables, table-to-csv
batch-ops  : batch, search
```

이 세 칸이 빠지면 이번 PR 의 확장이 사라진 것이다. 다른 pack 행은
이 도구가 읽기만 하고 과제 ID 를 바꾸지 않는다.

## 관련 문서

- 도구 모듈 독스트링: [`gym/tools/coverage.py`](../tools/coverage.py)
- 전 pack 정합 감사: [`gym/tools/audit.py`](../tools/audit.py)
- pack 스키마: [`gym/core/schema.py`](../core/schema.py)
- 검사 연산자 등록부: [`gym/core/checks.py`](../core/checks.py)
- extraction 과제: [`gym/packs/extraction/`](../packs/extraction/)
- table-csv 과제: [`gym/packs/table-csv/`](../packs/table-csv/)
- batch-ops 과제: [`gym/packs/batch-ops/`](../packs/batch-ops/)
- 이슈: [#5208](https://github.com/edwardkim/rhwp/issues/5208)
- PR: [#5212](https://github.com/edwardkim/rhwp/pull/5212)
