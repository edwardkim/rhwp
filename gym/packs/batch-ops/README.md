---
kind: guide
status: active
canonical: gym/packs/batch-ops/README.md
last_verified: 2026-08-18
---

# batch-ops — 다문서 대량 처리 (batch fill)

이 pack 은 서식 1개와 데이터 N행으로 문서 N부를 한 번에 만드는
진짜 메일머지(`batch fill`)를 채점한다. `batch` 의 다른 축
(`batch info` · `batch export-text` …)은 stdin 파일 목록을 읽는다.
`batch fill` 만 `--form` + `--data` 이고, gym 러너가 그대로
구동할 수 있는 유일한 batch 축이다.

채점은 산출물 축이다. NDJSON 스트림을 단일 봉투로 길을 수 없어
`answer_eq` 로 행 레코드를 대조하지 않는다. 기준 풀이가 `batch fill`
로 `out/` 을 채우고, 채점은 **파일 존재**와 **각 문서에 그 행의
값이 들어갔는지**를 단건 `search` 로 재검증한다.

선확인(`--dry-run`) 과제는 산출이 없다. 데이터 파일의 행 수를
`answer.json` 의 `planned` 로 제출하고 `json_value_eq` 로 자산
계약을 대조한다. NDJSON 을 라이브 오라클로 파싱하지 않는다.

새 CLI 는 없다. 표본은 `samples/form-01` · `samples/form-02` 의
HWP/HWPX 쌍만 쓴다. 누름틀 이름은 전부 `myMsg01` 이다.

## 왜 이 pack 인가

에이전트가 "명단 N명에게 같은 서식을 채워 달라"고 할 때 가장 흔한
실패는 일곱 가지다.

1. **단건을 N번 부른다.** `edit fill-fields` 는 서식 1 → 산출 1이다.
   N부를 만들려면 도구를 N번 불러야 한다. `batch fill` 은 한 번이다.
2. **stdin 목록을 넣는다.** 다른 `batch` 축의 습관이다. `batch fill`
   은 stdin 을 읽지 않는다. `--form` 과 `--data` 다.
3. **이름을 순번으로 고정한다.** `--name-field` 를 생략하면 4자리
   순번(`0001.hwpx`). 필드 값으로 이름 붙이려면 `--name-field` 가
   필수다.
4. **이름 열과 본문 열을 섞는다.** `--name-field outname` 이면
   파일 이름은 `outname` 이고 본문은 `myMsg01` 이다. 파일 이름에
   본문 값이 있기를 기대하면 실패한다.
5. **형식을 섞는다.** 서식이 `.hwp` 면 산출도 `.hwp`. `.hwpx` 면
   `.hwpx`. 데이터를 `.jsonl` 로 넣든 `.csv` 로 넣든 산출 형식은
   서식을 따른다.
6. **dry-run 인데 파일을 남긴다.** `--dry-run` 이어도 `--out-dir`
   는 필수다. 선검증이 실행과 같은 명령줄에서 `--dry-run` 하나만
   빼면 되게 하려는 것이다. 파일은 쓰지 않는다.
7. **검증을 생략한다.** `--verify` 는 행마다 저장 직후 IR 을 본다.
   채움은 성공해도 검증 실패가 최종 종료 코드에 남는다.

이 pack 의 과제는 위 구멍을 여정으로 나눈다.

## 하지 않는 것

1. **T07 을 복제하지 않는다.** T07 은 단건 `fill-fields` 다. 이
   pack 은 N부다. 과제 힌트에 `fill-fields` 를 넣지 않는다.
2. **stdin batch 축을 끌어오지 않는다.** `batch info` /
   `batch export-text` 는 다른 이야기이고 gym 러너가 stdin 을
   물리지 않는다.
3. **새 누름틀 이름을 만들지 않는다.** 표본의 필드는 `myMsg01`
   하나다. 데이터 헤더/키는 그 이름이다. 이름 열을 추가할 때만
   `outname` 을 덧붙인다.
4. **새 CLI 를 만들지 않는다.** `batch fill` 과 채점용 `search`.
5. **profiles / gym/README / PARK / checks.py 를 고치지 않는다.**
6. **deep_contains 를 쓰지 않는다.** 본문 확인은 `search` 의
   `matchCount >= 1` 이다. 전역 훑기 연산자가 아니다.
7. **빈 CSV(헤더만)를 과제로 두지 않는다.** 0부 메일머지는 레시피에
   적혀 있으나, 이 pack 은 1부 이상을 채점한다.

## 요구 capability

`pack.json` 의 `requires.commands` 는 `batch` 와 `search` 다.
없으면 `unavailable`. 기준 실행 신원은 이 확장에서 바꾸지 않는다.

## 명령 계약

### batch fill

```
rhwp batch fill --form <서식.hwp|서식.hwpx> --data <행.jsonl|행.csv> \
  --out-dir <폴더> --json [--name-field <필드>] [--verify] [--dry-run] [--threads N]
```

| 플래그 | 의미 | 대표 과제 |
|---|---|---|
| `--form` | 서식 1개. 행마다 다시 연다 | 전 과제 |
| `--data` | `.csv`(첫 줄 헤더=누름틀) 또는 `.jsonl`(줄마다 객체) | CSV: BO01, BO02, BO05–BO07, BO09, BO12–BO17 / JSONL: BO03, BO08, BO10, BO18, BO20 |
| `--out-dir` | 필수. dry-run 에도 필수 | 전 과제 |
| (이름 생략) | 1 기준 순번, 최소 4자리 | BO01, BO03, BO05, BO07–BO10, BO13, BO14, BO18, BO20 |
| `--name-field myMsg01` | 본문 필드가 곧 파일 이름 | BO02, BO12, BO15, BO17 |
| `--name-field outname` | 별도 이름 열. 본문은 myMsg01 | BO06, BO16 |
| `--verify` | 행마다 저장 직후 자기검증 | BO05, BO15, BO20 |
| `--dry-run` | 파일을 쓰지 않음 | BO04, BO11, BO19 |
| `--json` | 행마다 NDJSON 레코드 | 전 실행 |

성공 레코드는 `edit fill-fields --json` 과 같은 봉투에 `row`(0 기준)
가 붙는다: `schemaVersion`, `source`, `row`, `dryRun`, `filledCount`,
`filled`, `notFound`, `ambiguous`, `output?`, `outputFormat?`,
`verify?`.

이름 규칙:

- 파일명 금지 문자는 `_` 로 치환.
- 이름이 겹치면 `_2` 를 붙인다.
- 산출 경로는 **한 행도 쓰기 전에** 전부 정해, 병렬에서도 순서가
  이름을 바꾸지 않는다.
- `--name-field outname` 이면 `outname` 은 누름틀이 아니므로
  `notFound: ["outname"]` 이 실릴 수 있다. 본문 채움은 성공이다.

`--data` CSV 는 BOM·따옴표를 허용한다. 이 pack 의 자산은 BOM 없는
UTF-8, 따옴표 없는 ASCII 센티넬이다. 검색 바늘이 안정적이다.

### search (채점 전용)

```
rhwp search <산출> --json -- <바늘>
```

`matchCount >= 1` 이면 그 행의 값이 그 문서에 들어갔다고 본다.
바늘은 자산에 손수한 센티넬이다. 다른 행의 센티넬과 겹치지 않게
골랐다.

dry-run 과제는 `search` 를 부르지 않는다. 파일이 없다.

## 표본과 자산

| 표본 | 형식 | 필드 | 과제 |
|---|---|---|---|
| `samples/hwpx/form-01.hwpx` | HWPX | myMsg01 | BO01, BO02, BO04–BO06, BO10, BO13–BO15, BO18 |
| `samples/form-01.hwp` | HWP5 | myMsg01 | BO03, BO09, BO11, BO12, BO16, BO20 |
| `samples/hwpx/form-02.hwpx` | HWPX | myMsg01 | BO07, BO17, BO19 |
| `samples/form-02.hwp` | HWP5 | myMsg01 | BO08 |

form-01 과 form-02 는 둘 다 누름틀 `myMsg01` 하나다. 서식 본문이
조금 다르다. 같은 데이터로 두 서식을 채점하는 것이 이 확장의
형식 격자다.

| 자산 | 행 | 이름 전략 | 과제 |
|---|---|---|---|
| `BO01-data.csv` | 3 | 순번 | BO01, BO05 |
| `BO02-data.csv` | 2 | myMsg01=AlphaMerge/BetaMerge | BO02, BO04 |
| `BO03-data.jsonl` | 2 | 순번, JsonlAlpha/JsonlBeta | BO03, BO11 |
| `BO06-data.csv` | 2 | outname=gamma/delta | BO06 |
| `BO07-data.csv` | 2 | 순번, FormTwo* | BO07, BO19 |
| `BO08-data.jsonl` | 2 | 순번, FormTwoJsonl* | BO08 |
| `BO09-data.csv` | 2 | 순번, HwpCsv* | BO09 |
| `BO10-data.jsonl` | 2 | 순번, HwpxJsonl* | BO10 |
| `BO12-data.csv` | 2 | myMsg01=NameHwp* | BO12 |
| `BO13-data.csv` | 1 | 순번, SingleRowOnly | BO13 |
| `BO14-data.csv` | 4 | 순번, QuadMerge* | BO14 |
| `BO15-data.csv` | 2 | myMsg01=VerifyName* | BO15 |
| `BO16-data.csv` | 2 | outname=hwpouta/hwpoutb | BO16 |
| `BO17-data.csv` | 2 | myMsg01=FormTwoNamed* | BO17 |
| `BO18-data.jsonl` | 3 | 순번, TripleJsonl* | BO18 |
| `BO20-data.jsonl` | 2 | 순번, VerifyJsonl* | BO20 |

센티넬은 ASCII 식별자라 `search` 가 공백·쉼표에 흔들리지 않는다.
레시피의 `"김철수, 대표"` 인용 행은 이 pack 에 넣지 않았다.

## 여정 지도

### J1. 순번 이름 메일머지

`--name-field` 없음. `0001` 부터.

| ID | 서식 | 데이터 | 부 수 |
|---|---|---|---|
| BO01 | form-01.hwpx | CSV 3행 | 3 |
| BO03 | form-01.hwp | JSONL 2행 | 2 |
| BO07 | form-02.hwpx | CSV 2행 | 2 |
| BO08 | form-02.hwp | JSONL 2행 | 2 |
| BO09 | form-01.hwp | CSV 2행 | 2 |
| BO10 | form-01.hwpx | JSONL 2행 | 2 |
| BO13 | form-01.hwpx | CSV 1행 | 1 |
| BO14 | form-01.hwpx | CSV 4행 | 4 |
| BO18 | form-01.hwpx | JSONL 3행 | 3 |

실패 모드:

- 1부만 만들고 3부를 제출했다고 한다. `file_exists` 가 마지막 부를
  못 찾는다.
- 같은 문서를 세 번 복사한다. `search` 가 다른 행의 바늘을 못 찾는다.
- 산출 확장자를 반대로 붙인다(`.hwp` 서식에 `.hwpx`).
- `0001` 대신 `1` 또는 `001`. 자릿수는 최소 4.

### J2. --name-field 본문 필드

파일 이름 = `myMsg01` 값.

| ID | 서식 | 산출 이름 |
|---|---|---|
| BO02 | form-01.hwpx | AlphaMerge.hwpx, BetaMerge.hwpx |
| BO12 | form-01.hwp | NameHwpAlpha.hwp, NameHwpBeta.hwp |
| BO17 | form-02.hwpx | FormTwoNamedA.hwpx, FormTwoNamedB.hwpx |

실패 모드:

- 순번으로 저장하고 이름을 바꾼다. 본문 값이 틀리면 `search` 실패.
  이름만 맞추고 본문이 비면 역시 실패.
- 대소문자를 바꾼다. Windows 는 열리지만 제출 경로가 리터럴이다.

### J3. --name-field 별도 열

파일 이름 = `outname`, 본문 = `myMsg01`.

| ID | 서식 | 산출 이름 | 본문 바늘 |
|---|---|---|---|
| BO06 | form-01.hwpx | gamma.hwpx, delta.hwpx | GammaMerge, DeltaMerge |
| BO16 | form-01.hwp | hwpouta.hwp, hwpoutb.hwp | HwpOutAlpha, HwpOutBeta |

실패 모드:

- `--name-field myMsg01` 로 돌려 `GammaMerge.hwpx` 를 낸다.
  제출 경로는 `gamma.hwpx`.
- `outname` 값을 본문에 넣는다. 바늘은 `GammaMerge` 다.

### J4. --verify

값이 맞고 IR 도 맞아야 한다. 채점기는 IR 을 직접 보지 않고,
기준 풀이가 `--verify` 를 켠 채 산출을 만든다. 값이 틀리면
`search` 가 거절한다.

| ID | 조합 |
|---|---|
| BO05 | 순번 3부 + verify |
| BO15 | name-field myMsg01 + verify |
| BO20 | HWP5 JSONL + verify |

### J5. --dry-run

산출 없음. `planned` = 데이터 행 수(헤더 제외 / JSONL 줄 수).

| ID | 자산 | planned |
|---|---|---|
| BO04 | BO02-data.csv | 2 |
| BO11 | BO03-data.jsonl | 2 |
| BO19 | BO07-data.csv | 2 |

이 숫자는 자산 계약이다. 문서에서 온 값이 아니다. 골든 쪽수와
다르다. CSV 를 고치면 과제와 테스트를 같이 고쳐야 한다.

실패 모드:

- 파일을 실제로 쓴다. 과제는 `answer.json` 만 본다. 원본을
  더럽히면 저장소 문제다.
- `planned` 에 헤더를 포함해 3을 낸다. 계약은 데이터 행이다.

## 과제 목록 (BO01–BO20)

### BO01 — 서식 1 + 데이터 3행

기존. 이 pack 의 원형. `계약서` / `신규` 바늘. 중간 행(`월간`)은
채점하지 않는다. 1부와 3부만 보면 단건 복사를 거를 수 있다.

### BO02 — --name-field myMsg01

기존. AlphaMerge / BetaMerge. 이름과 본문이 같다.

### BO03 — HWP5 + JSONL

기존. 형식 격자의 한 모서리. 산출 `.hwp`.

### BO04 — dry-run HWPX CSV

BO02 자산의 행 수 2. 파일을 만들지 마라.

### BO05 — --verify 3부

BO01 과 같은 자산, 자기검증 켜짐. 바늘도 같다.

### BO06 — --name-field outname

별도 이름 열의 첫 과제. `gamma` / `delta`.

### BO07 / BO08 — form-02

같은 필드, 다른 서식. HWPX+CSV 와 HWP5+JSONL.

### BO09 / BO10 — 형식 교차

BO03 의 반대와 보강. HWP5+CSV, HWPX+JSONL. 네 모서리가 닫힌다.

```
           CSV              JSONL
HWPX       BO01/BO07/BO14   BO10/BO18
HWP5       BO09             BO03/BO08/BO20
```

### BO11 — dry-run HWP5 JSONL

BO03 자산, planned=2.

### BO12 — HWP5 + name-field myMsg01

BO02 의 HWP5 판. 산출 `.hwp`.

### BO13 — 1행

최소 대량. `0001` 만 제출. `0002` 를 만들어도 채점이 보지 않는다.
데이터 2행을 넣으면 본문 바늘 `SingleRowOnly` 가 1부에 없을 수 있다.

### BO14 — 4행

BO01 보다 한 부 더. 1부와 4부 바늘이 다르다.

### BO15 — verify + name-field

두 플래그를 같이. 이름과 검증을 동시에 놓치기 쉽다.

### BO16 — HWP5 + outname

BO06 의 HWP5 판.

### BO17 — form-02 + name-field

다른 서식, 같은 이름 전략.

### BO18 — JSONL 3행

BO01 의 JSONL 판. 순번 3부.

### BO19 — form-02 dry-run

BO07 자산, planned=2.

### BO20 — HWP5 JSONL + verify

BO03 에 검증을 켠 것.

## 채점 연산자

| 연산자 | CLI | 쓰임 |
|---|---|---|
| `file_exists` | 아니오 | 산출 각 부 |
| `differs_from_input` | 아니오 | 서식 복사 거부 |
| `value_ge` | `search` | 바늘 ≥ 1 |
| `json_value_eq` | 아니오 | dry-run planned |

금지: `fill-fields` 힌트, `deep_contains`, 새 CLI.

`search` 는 조회 명령이다. 편집 전역 훑기가 아니다. 이 pack 의
axis 는 `자동화` 라 `GLOBAL_SCAN_OPS` 가 적용되지 않는다. 그래도
전역 훑기 연산자는 쓰지 않는다.

## 명령 레시피

```bash
# 누름틀 이름 확인 (이 pack 밖, 사람이 할 일)
rhwp fields samples/hwpx/form-01.hwpx --json | jq -r '.fields[].name'

# 순번 3부
rhwp batch fill --form samples/hwpx/form-01.hwpx \
  --data gym/packs/batch-ops/assets/BO01-data.csv \
  --out-dir out --json

# 본문 필드로 이름
rhwp batch fill --form samples/hwpx/form-01.hwpx \
  --data gym/packs/batch-ops/assets/BO02-data.csv \
  --out-dir out --name-field myMsg01 --json

# 별도 이름 열
rhwp batch fill --form samples/hwpx/form-01.hwpx \
  --data gym/packs/batch-ops/assets/BO06-data.csv \
  --out-dir out --name-field outname --json

# HWP5 + JSONL
rhwp batch fill --form samples/form-01.hwp \
  --data gym/packs/batch-ops/assets/BO03-data.jsonl \
  --out-dir out --json

# 선확인 (파일 없음)
rhwp batch fill --form samples/hwpx/form-01.hwpx \
  --data gym/packs/batch-ops/assets/BO02-data.csv \
  --out-dir out --dry-run --json

# 자기검증
rhwp batch fill --form samples/hwpx/form-01.hwpx \
  --data gym/packs/batch-ops/assets/BO01-data.csv \
  --out-dir out --verify --json

# 채점과 같은 재검색
rhwp search out/0001.hwpx --json -- 계약서 | jq '.matchCount'
```

채점 왕복:

```bash
python gym/tools/build_baseline.py --agent baseline --pack batch-ops --bin target/debug/rhwp
python gym/score.py --agent baseline --pack batch-ops --bin target/debug/rhwp
```

dry-run 과제는 기준 풀이가 `const` 로 `planned` 를 쓴다. 바이너리를
부르지 않는다. 자산 행 수가 바뀌면 과제·기준·테스트를 같이 고쳐라.

## 실패 모드 상세

### stdin 습관

```
printf 'samples/form-01.hwp\n' | rhwp batch fill --data rows.csv --out-dir out
```

이건 동작하지 않는다. `--form` 이 없다. exit 2.

### 단건 N번

```
rhwp edit fill-fields form.hwpx --data '{"myMsg01":"A"}' -o out/0001.hwpx
rhwp edit fill-fields form.hwpx --data '{"myMsg01":"B"}' -o out/0002.hwpx
```

값이 맞으면 채점은 통과할 수 있다. 이 pack 은 **한 번의 batch fill**
을 측정한다. 힌트와 기준 풀이가 그 축이다. T07 을 복제한 풀이다.

### 이름 충돌

두 행의 `--name-field` 값이 같으면 두 번째가 `_2` 를 붙는다.
이 pack 의 자산은 이름이 겹치지 않는다.

### notFound outname

BO06·BO16 의 봉투에 `notFound: ["outname"]` 이 실리는 것은
정상이다. `outname` 은 누름틀이 아니다. `filledCount` 는 1
(`myMsg01`)이어야 한다. 채점기는 봉투를 보지 않고 본문을 검색한다.

### dry-run 과 out-dir

`--out-dir` 를 빼면 사용법 오류다. 빈 폴더를 만들어 두고
`--dry-run` 을 켜면 폴더는 비어 있어야 한다. 과제는 폴더를
제출받지 않는다.

### 검색 바늘 공백

`계약서 초안 검토 요청` 전체를 바늘로 쓰면 줄바꿈에 흔들릴 수
있다. BO01 은 `계약서` / `신규` 로 짧게 잡는다. 신규 자산은
한 토큰 센티넬이다.

### 형식과 확장자

서식 `.hwp` + 데이터 `.csv` → 산출 `.hwp`.
서식 `.hwpx` + 데이터 `.jsonl` → 산출 `.hwpx`.
데이터 확장자는 산출 형식을 바꾸지 않는다.

### threads

`--threads` 는 이름 예약 뒤에 돈다. 이 pack 은 켜지 않는다.
병렬이 이름을 바꾸지 않는다는 계약은 코어 테스트의 몫이다.

## 커버리지와의 관계

```
[batch-ops] batch, search
```

두 명령은 BO01 로 이미 노출돼 있었다. 이번 확장은
`--name-field` · `--dry-run` · `--verify` · HWP/HWPX · CSV/JSONL ·
1/2/3/4행 · form-02 를 격자화한다. 분모는 그대로다.

남는 빈 곳:

- `batch info` / `batch export-text` / `batch convert` (stdin 축).
  gym 러너가 stdin 을 물리지 않아 과제로 넣지 않았다.
- 빈 데이터(0행).
- 이름 충돌 `_2`.
- 따옴표·쉼표가 있는 CSV 값.
- `--threads`.

stdin 축을 넣으려면 러너가 목록을 물리는 별도 장치가 필요하다.
이 확장은 그 장치를 만들지 않는다. 새 CLI 도 아니다.

## 스키마 불변식

`scripts/tests/test_gym_batch_ops_pack.py` 가 CI 에서 다시 본다.

- 과제 id 는 `BOnn`, 기준 풀이와 1:1.
- 산출 과제는 `file_exists` + `differs_from_input` + `search`.
- dry-run 과제는 `json_value_eq` planned, 산출 파일 없음.
- 명령 화이트리스트: `batch`, `search`.
- 첫 토큰 `batch` 의 둘째는 `fill` 뿐.
- 자산이 실재하고, `--data` 경로가 그 자산을 가리킨다.
- `fill-fields` · 새 CLI 이름 부재.
- `runner` 신원 고정.

## 재현

```bash
python gym/tools/build_baseline.py --agent baseline --pack batch-ops --bin target/debug/rhwp
python gym/score.py               --agent baseline --pack batch-ops --bin target/debug/rhwp
```

BO01–BO03 은 기존 축이다. BO04–BO20 은 같은 명령·같은 서식·같은
필드로 플래그·형식·행 수 격자를 늘린 확장이다. 새 pack 도, 새 CLI
도, T07 복제도 없다.

## 관련

- `core-cli` T07 — 단건 fill-fields. 이 pack 과 축이 다르다.
- `extraction` — 읽기. 서식을 채우지 않는다.
- `table-csv` — 표 칸. 누름틀이 아니다.
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md` — 실측 레시피.
- `mydocs/working/gym_coverage_and_extract.md`.
