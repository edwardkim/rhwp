---
kind: investigation
status: active
canonical: gym/docs/coverage.md
last_verified: 2026-08-18
---

# gym 커버리지 격자·추출/표CSV/배치 확장 작업 노트

이 문서는 PR #5212 (`feat/gym-coverage-and-extract`) 를 키운 작업 기록이다.
규범 문서는 [gym/docs/coverage.md](../../../gym/docs/coverage.md) 다.

## 무엇을 했는가

`coverage.py` 가 명령 합계만 내지 않고 pack×명령 격자와 REGISTRY 미사용
연산자를 같은 `gymCoverage` 1.0 봉투에 낸다. 기존 키의 의미는 그대로다.

얇은 pack 세 곳에 과제+기준풀이를 더했다. 첫 커밋은 EX03·EX04 / TC02·TC03 /
BO02·BO03 여섯 건이었다. 이 노트는 그 위에 EX05–EX22 / TC04–TC17 /
BO04–BO12 를 같은 계약으로 붙인 기록이다.

건드리지 않은 것:

- 새 CLI, 새 pack, core-cli T07 을 베끼는 일, `fill-fields` 과제
- `profiles/` · `gym/README.md` · `gym/PARK.md` · `gym/core/checks.py`
- 다른 pack 의 과제 ID
- `cargo fmt --all` (JSON·문서·테스트만 바꿨다)
- 세 pack 의 `runner` 신원. 요구 명령은 이미 선언된 것만 쓴다.

## 왜 이 두께인가

커버리지 도구는 "한 숫자"로 닫히지 않는다. 같은 `extract-data` 라도

- `--kind date` 인가 `amount` 인가 `number` 인가 필터 없는 전종인가
- 읽는 필드가 `itemCount` 인가 `totalItemCount` 인가
- `--limit` 이 잘라도 총량이 남는가
- 표본이 홍보 문서인가 1쪽/2쪽 시험지인가

가 다른 계약이다. `export-text` 는 쪽수와 첫 쪽 글자수와 `-p 1` 둘째 쪽과
`--max-chars` 절단이 갈라지고, `chart-to-csv` 는 행/열/개수와 차트 종류가
갈라진다. 과제를 합치면 에이전트가 힌트 한 줄을 외워 모든 추출을 통과한다.

표 CSV 왕복도 마찬가지다. 추출만 보면 `(0,0)=1` 한 칸으로 닫히고, 되쓰기는
첫 칸만 바꾸면 다른 칸을 밀어도 모른다. 그래서 좌표를 여러 칸 찍고, 바꾼
칸과 유지 칸을 한 과제에 같이 둔다. `deep_contains` 는 쓰지 않는다.

메일머지는 형식(CSV/JSONL) × 컨테이너(HWPX/HWP5) × 이름(순번/`--name-field`)
이 서로 다른 실패를 만든다. 한 조합만 있으면 나머지 세 축을 추측한다.

## 커버리지 도구가 더한 것

`measure` 가 받는 인자가 늘었다.

- `packs`: packId → 그 pack 이 부르는 명령. 생략하면 `{}`.
- `unused_operators`: 등록됐지만 과제가 안 쓰는 연산자. 생략하면 `[]`.

반환 봉투에 `packs`·`unusedOperators` 가 항상 있다. 가드
`test_measure_keeps_schema_and_adds_grid_keys` 가 기본값을 고정하고,
`test_measure_embeds_pack_grid_and_unused_operators` 가 정렬을 고정한다.

사람용 `format_human` 도 같은 두 블록을 빠뜨리지 않는다. `--json` 을 안
줘도 리뷰어가 격자를 본다.

CLI 는 그대로 `--bin` / `--capabilities` / `--json` 이다. 소스 없으면
exit 2. 새 플래그는 없다.

## 과제 계보 — extraction

### 기존 (devel)

| ID | 명령 | 요지 |
|---|---|---|
| EX01 | `chart-to-csv` | 세로막대 차트 1 `rowCount` |
| EX02 | `export-text` | 홍보 문서 `pageCount` |

### 첫 확장 (EX03–EX04)

| ID | 명령 | 요지 |
|---|---|---|
| EX03 | `extract-data --kind date` | 홍보 문서 `itemCount` |
| EX04 | `export-text -p 0` | 1쪽 시험지 글자 수 |

### 이번 확장 (EX05–EX22)

| ID | 명령 | 표본 | 지목 |
|---|---|---|---|
| EX05 | `extract-data --kind amount` | 홍보 | `itemCount` |
| EX06 | `extract-data --kind number` | 홍보 | `itemCount` |
| EX07 | `extract-data --kind all` | 홍보 | `itemCount` |
| EX08 | `extract-data --kind date` | exam-kor-1p | `itemCount` |
| EX09 | `extract-data --kind date` | exam-kor-1p.hwpx | `itemCount` |
| EX10 | `extract-data --kind amount` | exam-kor-1p | `itemCount` |
| EX11 | `export-text` | exam-kor-2p | `pageCount` |
| EX12 | `export-text` | exam-kor-2p.hwpx | `pageCount` |
| EX13 | `export-text -p 0` | exam-kor-2p | 글자 수 |
| EX14 | `export-text` | exam-kor-3p | `pageCount` |
| EX15 | `chart-to-csv` | 가로막대 | `rowCount` |
| EX16 | `chart-to-csv` | 꺽은선 | `rowCount` |
| EX17 | `chart-to-csv` | 원형 | `rowCount` |
| EX18 | `chart-to-csv` | 세로막대 HWPX | `rowCount` |
| EX19 | `extract-data --kind date` | 빈 HWPX | `itemCount` (0건) |
| EX20 | `extract-data --kind amount` | 빈 HWPX | `itemCount` (0건) |
| EX21 | `extract-data --kind date --limit 1` | 홍보 | `itemCount` |
| EX22 | `export-text` | table-001 | `pageCount` |

EX07 은 core-cli T04 와 같은 전종 수확이지만 표본이 다르고 `--kind all` 을
명시한다. T04 는 월간 수출입 현황이고 `len_answer_eq` 로 `items` 길이를
잰다. T07(`fill-fields`) 과는 축이 다르다.

EX21 은 `--limit 1` 로 items 를 자른다. 보고할 값은 잘린 `itemCount` 다.

EX19·EX20 은 빈 HWPX 에서 0건을 보고한다. 0건은 오류가 아니다.

EX09·EX12·EX18 은 같은 문서의 HWPX 쌍이다. 컨테이너가 달라도 봉투 필드
이름은 같다.

## 과제 계보 — table-csv

`samples/hwpx/basic-table-01.hwpx` 표 0 은 3×4, 값이 1..12 다.

```
(0,0)=1   (0,1)=2   (0,2)=3   (0,3)=4
(1,0)=5   (1,1)=6   (1,2)=7   (1,3)=8
(2,0)=9   (2,1)=10  (2,2)=11  (2,3)=12
```

TC01 만 다른 표본(`samples/143E433F503322BD33.hwp` 재정표)을 쓴다.

### 기존 (devel)

| ID | 명령 | 요지 |
|---|---|---|
| TC01 | `csv-to-table` | 재정표 적립금 328→999, dry-run `changedCount=0` |

### 첫 확장 (TC02–TC03)

| ID | 명령 | 요지 |
|---|---|---|
| TC02 | `table-to-csv` | `(0,0)=1` `(1,2)=7` |
| TC03 | `csv-to-table` | `(0,0) 1→100`, `(1,2)` 유지 7 |

### 이번 확장 (TC04–TC17)

| ID | 명령 | 요지 |
|---|---|---|
| TC04 | `table-to-csv` | table-001 첫 칸 `구 분` |
| TC05 | `table-to-csv --bom` | basic-table-01 BOM + `(0,0)=1` |
| TC06 | `table-to-csv --bom` | table-001 BOM + `구 분` |
| TC07 | `csv-to-table --dry-run` | basic-table-01 `changedCount` |
| TC08 | `csv-to-table --dry-run` | 재정표 `changedCount` |
| TC09 | `csv-to-table` | `(1,2) 7→77`, `(0,0)` 유지 |
| TC10 | `table-to-csv --json` | basic-table-01 행·열 |
| TC11 | `table-to-csv --json` | table-001 행·열 |
| TC12 | `table-to-csv --json` | 재정표 HWPX 행 수 |
| TC13 | `table-to-csv --json` | multi-table-001 표 0 행 수 |
| TC14 | `table-to-csv` | 재정표 머리 칸 `구분` |
| TC15 | `table-to-csv --json` | table-004 행 수 |
| TC16 | `csv-to-table` | `(0,1) 2→200`, `(0,0)`·`(1,2)` 유지 |
| TC17 | `table-to-csv --json` | table-text.hwpx 행 수 |

TC05 제목은 추출이다. 표기 실수가 아니라 "다른 좌표 쌍"이다.

TC11 은 table-editing TB07 과 같은 `--bom` 이지만 표본이 다르다. TB07 은
`table-001.hwp` 의 "구 분" 첫 셀이고, TC11 은 1..12 격자다.

TC12–TC15 는 산출 파일이 아니라 봉투 숫자를 보고한다. 왕복의 "크기 계약"
을 파일 없이 잰다. `csv-to-table` 은 크기·라벨이 다르면 exit 2 로 거부한다.

편집 축이므로 `GLOBAL_SCAN_OPS`(`deep_contains`·`not_contains`)를 쓰지
않는다. 좌표 지목은 `csv_cell_eq`(제출 CSV)와 `cell_text_eq`(되쓴 문서).

## 과제 계보 — batch-ops

서식은 `samples/hwpx/form-01.hwpx` 와 `samples/form-01.hwp` 다. 누름틀
이름은 `myMsg01` 하나다. `edit fill-fields` 가 아니라 `batch fill` 이다.

### 기존 (devel)

| ID | 형식 | 요지 |
|---|---|---|
| BO01 | HWPX + CSV 3행 | 순번 0001–0003, 토큰 계약서/신규 |

### 첫 확장 (BO02–BO03)

| ID | 형식 | 요지 |
|---|---|---|
| BO02 | HWPX + CSV 2행 | `--name-field myMsg01` → AlphaMerge·BetaMerge |
| BO03 | HWP5 + JSONL 2행 | 순번 0001–0002, JsonlAlpha·JsonlBeta |

### 이번 확장 (BO04–BO12)

| ID | 형식 | 이름 | 요지 |
|---|---|---|---|
| BO04 | HWPX + CSV | `--dry-run` | 예정 행 수 `planned=2` |
| BO05 | HWPX + CSV 3행 | 순번 + `--verify` | BO01 데이터에 자기검증 |
| BO06 | HWPX + CSV | `--name-field outname` | gamma·delta 파일명 |
| BO07 | form-02 HWPX + CSV | 순번 | FormTwoAlpha·Beta |
| BO08 | form-02 HWP5 + JSONL | 순번 | FormTwoJsonlA·B |
| BO09 | HWP5 + CSV | 순번 | HwpCsvAlpha·Beta |
| BO10 | HWPX + JSONL | 순번 | HwpxJsonlOne·Two |
| BO11 | HWP5 + JSONL | `--dry-run` | 예정 행 수 `planned=2` |
| BO12 | HWP5 + CSV | `--name-field myMsg01` | NameHwpAlpha·Beta |

채점은 산출 파일 존재 + `differs_from_input` + `search` 로 그 행의 토큰이
그 부에 들어갔는지를 본다. NDJSON 스트림은 단일 봉투가 아니라서 값을
길로 캐지 않는다.

토큰은 과제 사이에 겹치지 않게 골랐다. 한 문자열이 두 부에 들어가면
어느 행이 어느 파일인지 판별이 안 된다.

## 기준풀이 규약

모든 새 과제는 `tasks/X.json` ↔ `reference/X.json` 짝이다. `id` 가 같고
파일이름이 같다. `audit.py` 가 고아·짝 없음을 막는다.

- 추출 답안: `steps[0].answer.<key> = {cmd, path, len?}`. `cmd`/`path` 는
  과제의 check 와 같다. `len_answer_eq` 만 `"len": true`.
- 표 추출: `table-to-csv {input} --table 0 -o {sub:table.csv} --json`.
  BOM 과제는 `--bom` 을 더한다.
- 표 되쓰기: `csv-to-table {input} --table 0 --csv gym/packs/table-csv/assets/<id>-edit.csv -o {sub:out.hwpx} --json`.
- 메일머지: `batch fill --form {input} --data <asset> --out-dir {sub:out} --json`.
  이름 필드 과제는 `--name-field myMsg01` 을 더한다.

자리표 `{input}`·`{sub:…}`·`{file:…}` 는 러너가 치환한다. 기준풀이를
손으로 돌릴 때 리터럴로 남기면 다음 세대가 입력을 잃는다.

## 가드

```bash
python -m unittest scripts/tests/test_gym_coverage.py scripts/tests/test_gym_packs.py
python gym/tools/audit.py
```

커버리지 가드가 고정하는 것:

- 분모는 에이전트-대면만. 진단·serve 는 빈 곳이 아니다.
- 분모 0 이면 100. 0 나누기 없음.
- `packs`·`unusedOperators` 키가 항상 있다.
- 격자 행·명령 정렬, pack 사이 누수 없음.
- 실제 gym 의 세 얇은 pack 행에 알려진 명령이 있다.
- REGISTRY − 과제 op = 미사용 목록. `answer_eq`·`file_exists` 는 남아 있지
  않다.
- CLI `--json` / 사람용 / 소스 없음 exit 2.

pack 가드가 고정하는 것:

- EX01–EX22 · TC01–TC17 · BO01–BO12 와 짝 기준풀이.
- extraction 은 라이브 오라클 답안만. `fill-fields` 없음.
- table-csv 는 `GLOBAL_SCAN_OPS` 없음. 되쓰기는 유지 칸을 본다.
- batch-ops 는 `batch fill` 만. 순번/`--name-field`·CSV/JSONL·HWP5/HWPX
  가 모두 있다.
- 새 pack 디렉터리 없음. T07 이 여전히 `fill-fields` 를 소유한다.
- `audit.py` 위반 0.

`cargo fmt --all` 은 돌리지 않는다. Rust 변경이 없고 sparse/worktree
환경에서 포맷 드리프트를 키운다.

## 라이브 채점을 안 한 이유

로컬에 `rhwp` 바이너리가 없다. 추출·배치 과제는 라이브 오라클이라 숫자를
박제하지 않아도 채점 시점에 기댓값이 생긴다. 표 칸 값(1..12, `(1,2)=7`)은
SH04·TC02·TC03 이 이미 쓰는 같은 격자다. BOM 과 `--name-field` 는 CLI
매뉴얼과 TB07·BO02 가 검증한 동작이다.

바이너리가 생기면 다음으로 닫는다.

```bash
python gym/tools/build_baseline.py --agent local --pack extraction
python gym/score.py --agent local --pack extraction
python gym/tools/build_baseline.py --agent local --pack table-csv
python gym/score.py --agent local --pack table-csv
python gym/tools/build_baseline.py --agent local --pack batch-ops
python gym/score.py --agent local --pack batch-ops
```

실패하면 그 과제의 힌트·연산자·자리표가 틀린 것이다. 숫자를 고치지 말고
명령을 고친다.

## 하지 말 것 (재발 방지)

- `gym/packs/<새이름>/` 을 만들지 않는다. 빈 곳은 기존 pack 의 표본·필드다.
- T07 을 복사해 extraction 에 `fill-fields` 과제를 넣지 않는다. #4781.
- 편집 과제에 `deep_contains` 를 넣지 않는다. #4600.
- `pack.json` 의 `rhwpCommit`·`capabilitiesSha256` 을 과제 추가로 갱신하지
  않는다. 신원이 바뀌면 점수의 기준 실행이 바뀐 것이다.
- `profiles/maintainer.json` 을 손대지 않는다. 새 pack 이 없으므로
  전 표면 목록은 그대로다.
- 다른 열린 gym PR 의 pack(text-editing·serialization·security 등)에
  과제를 넣지 않는다. 이 브랜치의 범위는 커버리지 도구와 세 얇은 pack 이다.

## 문서 역할

| 경로 | 역할 |
|---|---|
| `gym/docs/coverage.md` | 봉투·격자·미사용 연산자 규범 |
| `mydocs/working/gym_coverage.md` | 이 작업 기록 (본 문서) |
| `gym/tools/coverage.py` | 구현 + 모듈 독스트링 |
| `scripts/tests/test_gym_coverage.py` | 도구 계약 |
| `scripts/tests/test_gym_packs.py` | pack 구조 + 세 얇은 pack 확장 |
| `gym/tools/audit.py` | 전 pack 정합 |

`mydocs/README.md` 매니페스트에는 올리지 않는다. working 노트는 단계
기록이고, 규범은 `gym/docs/coverage.md` 다.

## 크기 게이트

`git diff --shortstat upstream/devel` 의 insertions 가 3000 이상이어야
한다. 합계를 맞추려고 난수 JSON 이나 T07 클론을 넣지 않았다. 늘어난 줄은
격자 가드, 세 pack 의 여정 과제, 규범 문서다.

## 관련

- 이슈 #5208
- PR #5212
- 철회된 중복 pack #4781
- 편집 전역 훑기 금지 #4600
- extract-data 종류 `date|amount|number|all` — `mydocs/manual/cli_commands.md`
