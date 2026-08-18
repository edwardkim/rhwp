---
kind: guide
status: active
canonical: gym/packs/extraction/README.md
last_verified: 2026-08-18
---

# extraction — 데이터 추출 (읽기)

이 브랜치에 등재된 과제는 **EX01–EX28** 다. EX23–EX28 은 날짜 표본·
누적 차트·3/4쪽 글자·시험지 전종류·수량 격자다.

이 pack 은 문서를 **고치지 않고** 값을 뽑는 읽기 축이다. 차트 격자
(`chart-to-csv`), 본문 페이지(`export-text`), 날짜·금액·수량
(`extract-data`) 세 명령만 쓴다. 새 CLI 는 없다. 표본은 이미
저장소 `samples/` 에 있는 파일만 쓴다.

채점은 라이브 오라클이다. 행 수·쪽 수·항목 수를 JSON 에 박제하지
않는다. `answer_eq` / `len_answer_eq` 가 채점 시점에 rhwp 를 다시
돌려 기댓값을 계산한다. 에이전트가 제출하는 숫자는 "내가 센 값"이고,
정답은 "지금 이 바이너리가 같은 명령으로 센 값"이다.

이 문서는 pack 내부 안내서다. `gym/README.md` · `gym/PARK.md` ·
`gym/profiles/` 는 이 확장에서 건드리지 않는다. pack 의 과제 수가
늘어도 운동장 지도의 존 배치는 그대로다 — 조회존의 추출 어트랙션이
길어질 뿐이다.

## 왜 이 pack 인가

에이전트가 문서에서 숫자를 뽑을 때 가장 흔한 실패는 네 가지다.

1. **평문을 밖에서 정규식으로 센다.** `export-text` 로 본문을 쏟고
   파이썬으로 날짜를 세면 값은 나와도 주소가 소멸한다. 근거를 댈 수
   없다. `extract-data` 는 값마다 구역·문단·페이지·문자 오프셋을 붙인다.
2. **종류를 섞는다.** 날짜·금액·수량은 한 봉투에 같이 실릴 수 있다.
   `--kind date` 를 빼면 `all` 이 되고, 금액 과제에 날짜를 섞어 세면
   라이브 오라클이 거절한다.
3. **형식 쌍을 같다고 가정하지 않는다.** 같은 시험지의 `.hwp` 와
   `.hwpx` 는 본문이 같아 보여도 파서가 다르다. 이 pack 은 양쪽을
   따로 채점한다(EX08/EX09, EX11/EX12, EX01/EX18).
4. **0건을 실패로 본다.** 빈 문서에서 날짜가 0건인 것은 오류가 아니다.
   종료 코드 0, `itemCount: 0`. EX19·EX20 이 그 계약을 고정한다.

이 pack 의 과제는 위 구멍을 여정으로 나눈다.

## 하지 않는 것

경계가 과제 수보다 중요하다.

1. **표를 고치지 않는다.** 표 CSV 왕복은 `table-csv` pack 이다.
   이 pack 에 `csv-to-table` 을 끌어오지 않는다.
2. **누름틀을 채우지 않는다.** `fill-fields` 는 core-cli T07 이다.
   추출 pack 에 T07 을 복제하지 않는다.
3. **배치 메일머지를 하지 않는다.** `batch fill` 은 `batch-ops` 다.
4. **새 CLI 를 만들지 않는다.** `chart-to-csv` · `export-text` ·
   `extract-data` 만 쓴다. `runner.*` 신원은 그대로 둔다.
5. **골든 숫자를 박제하지 않는다.** 행 수·쪽 수·항목 수는 전부
   `answer_eq` 경로로 재계산한다. 지시문에 예시 숫자를 넣지 않는다
   (EX21 의 `--limit 1` 은 상한이지 정답이 아니다).
6. **profiles / gym/README / PARK / checks.py 를 고치지 않는다.**
7. **합성 픽스처를 추가하지 않는다.** 차트·홍보문·시험지·빈 HWPX 는
   이미 저장소에 있다.

## 요구 capability

`pack.json` 의 `requires.commands` 는 `chart-to-csv`, `export-text`,
`extract-data` 다. 바이너리에 이 명령이 없으면 점수는 0 이 아니라
`unavailable` 이다. 부재를 실패로 위장하지 않는 것이 이 저장소의
결이다.

기준 실행 신원(`runner.rhwpVersion` · `rhwpCommit` ·
`capabilitiesSha256`)은 이 확장에서 바꾸지 않는다. 점수의 신원을
조용히 갈아끼우지 않기 위해서다.

## 명령 계약

### extract-data

```
rhwp extract-data <파일> [--kind date|amount|number|all] [--limit N] [--json]
```

| 플래그 | 의미 | 대표 과제 |
|---|---|---|
| `--kind date` | 날짜만. `2026년 8월 2일`, `2026. 8. 2.`, `2026-08-02` | EX03, EX08, EX09, EX19, EX21, EX23 |
| `--kind amount` | 금액만. `1,234원`, `₩1,234`, `3,180백만원` | EX05, EX10, EX20 |
| `--kind number` | 수량만. `12개`, `3.5%`, `1,000명`. 단위 없는 맨 숫자는 항목이 아니다 | EX06, EX28 |
| `--kind all` | 세 종류 전부. 생략과 같다 | EX07, EX26 |
| `--limit N` | 이번 응답 상한. `totalItemCount` 는 절단 전 총량 | EX21 |
| `--json` | 기계 봉투 | 전 과제 |

봉투 키: `schemaVersion`, `source`, `kind`, `itemCount`,
`totalItemCount`, `truncated`, `counts`, `items[]`.

항목: `{kind, raw, normalized, currency?, unit?, section, paragraph,
page?, charOffset, length, cell?, textbox?}`.

정규화 규약 — 모르는 것은 모른다고 한다.

- 날짜 `normalized` 는 ISO-8601. 일이 없으면 `2026-01` 로 둔다.
  없는 날을 1일로 채우지 않는다.
- 두 자리 연도(`'26.8.2`)는 세기를 추정하지 않고 `normalized: null`.
- 한글 수사 금액(`일금 백이십삼만원`)은 v1 범위 밖.
- 금액 배수는 정수 연산만 (`1.5억원` → `150000000`). 정수로 떨어지지
  않으면 `null`.
- **0건은 오류가 아니다.** `itemCount: 0`, 종료 코드 0.

이 pack 은 `itemCount` 만 묻는다. `items[0].normalized` 를 박제하지
않는다. 표기가 진화하면 오라클이 따라간다.

### export-text

```
rhwp export-text <파일> [-p <쪽>] [--json]
```

| 플래그 | 의미 | 대표 과제 |
|---|---|---|
| (없음) | 전 쪽. `pageCount` | EX02, EX11, EX12, EX14, EX22, EX27 |
| `-p 0` | 첫 쪽만. `pages[0].text` 길이 | EX04, EX13, EX25 |
| `--json` | 기계 봉투 | 전 과제 |

쪽 번호는 **0 기준**이다. `-p 1` 은 둘째 쪽이다. 이 pack 의 글자 수
과제는 전부 첫 쪽(`-p 0`)이다. `info` 의 `pageCount` 와
`export-text` 의 `pageCount` 는 같은 개수여야 하지만, 이 pack 은
추출 봉투만 본다 — 렌더 쪽수와 추출 쪽수를 섞지 않는다.

글자 수는 `len_answer_eq` 다. 본문 문자열을 답에 넣지 않는다. 길이만
대조한다. 공백·개행이 바뀌면 길이가 바뀌고 과제가 실패한다. 그것이
의도다.

### chart-to-csv

```
rhwp chart-to-csv <파일> [--chart <번호>] [-o <경로>] [--bom] [--json]
```

| 플래그 | 의미 | 대표 과제 |
|---|---|---|
| `--chart 1` | 문서 순서 **1부터**. 표 `--table` 과 달리 0 이 아니다 | EX01, EX15–EX18, EX24 |
| `--json` | `charts[0].rowCount` | 전 차트 과제 |

행 = 카테고리, 열 = 계열. 원본 데이터 시트와 같은 모양이다.
이 pack 은 행 수만 묻는다. CSV 본문을 제출하지 않는다. 차트 편집
왕복은 `studio-e2e` ST01 (`csv-to-chart`) 이다.

차트 번호 0 을 넣으면 사용법 오류다. EX01 의 힌트가 `--chart 1` 인
이유다.

## 표본

과제는 이미 저장소에 있는 표본만 재사용한다.

| 표본 | 쓰임 | 비고 |
|---|---|---|
| `samples/chart/세로막대형/묶은세로막대형.hwp` | EX01 | 기존. 차트 1 행 수. |
| `samples/chart/세로막대형/묶은세로막대형.hwpx` | EX18 | EX01 의 HWPX 쌍. |
| `samples/chart/세로막대형/누적세로막대형.hwp` | EX24 | 같은 가족, 묶음이 아니라 누적. |
| `samples/chart/가로막대형/묶은가로막대형.hwp` | EX15 | 가로 막대. |
| `samples/chart/라인/꺽은선형.hwp` | EX16 | 꺾은선. |
| `samples/chart/원형/2차원원형.hwp` | EX17 | 원형. |
| `samples/20250130-hongbo.hwp` | EX02, EX03, EX05–EX07, EX21 | 실문서. 날짜·금액·수량·본문. |
| `samples/exam-kor-1p.hwp` | EX04, EX08, EX10, EX28 | 1쪽 시험지 HWP. |
| `samples/hwpx/exam-kor-1p.hwpx` | EX09, EX26 | 같은 시험지 HWPX. |
| `samples/exam-kor-2p.hwp` | EX11, EX13 | 2쪽. |
| `samples/hwpx/exam-kor-2p.hwpx` | EX12 | 2쪽 HWPX. |
| `samples/exam-kor-3p.hwp` | EX14, EX25 | 3쪽. |
| `samples/exam-kor-4p.hwp` | EX27 | 4쪽. |
| `samples/table-001.hwp` | EX22 | 표 표본의 본문 쪽수. |
| `samples/2010-01-06.hwp` | EX23 | 날짜가 이름에 있는 실문서. |
| `samples/hwpx/blank_hwpx.hwpx` | EX19, EX20 | 빈 문서. 0건 계약. |

새 픽스처를 추가하지 않는다. 한글 경로 차트 표본은 이미
`samples/chart/` 아래에 있다.

## 여정 지도

과제는 명령 가족이 아니라 **사람이 하는 일**로 묶는다. 한 여정이
여러 과제를 가질 수 있다. 표본이 바뀌면 같은 명령도 다른 계약이 된다.

### J1. 차트 격자 행 수 (`chart-to-csv --chart 1`)

차트가 있는 문서에서 데이터 행(카테고리) 수를 센다. 차트 번호는
1부터다. 0 을 넣으면 실패다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| EX01 | 묶은세로막대형 행 수 | 세로막대형 HWP | `charts[0].rowCount` |
| EX15 | 묶은가로막대형 행 수 | 가로막대형 HWP | 같은 경로 |
| EX16 | 꺽은선형 행 수 | 라인 HWP | 같은 경로 |
| EX17 | 2차원원형 행 수 | 원형 HWP | 같은 경로 |
| EX18 | 묶은세로막대형 HWPX | EX01 의 쌍 | 같은 경로 |
| EX24 | 누적세로막대형 행 수 | 같은 가족, 누적 | 같은 경로 |

실패 모드:

- `--chart 0` 을 넣는다. 표 좌표계와 섞은 것이다.
- 차트 없이 `export-text` 로 숫자를 센다. 카테고리 개수가 아니다.
- HWP 과제를 HWPX 산출로 바꾸거나 그 반대. 입력 경로가 과제에 박혀 있다.
- CSV 파일을 제출한다. 이 여정은 `answer.json` 의 숫자만 받는다.

### J2. 본문 쪽수 (`export-text` pageCount)

문서가 몇 쪽에 걸치는지 추출 봉투로 센다. 렌더 PNG 쪽수와 섞지 않는다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| EX02 | 홍보문 쪽수 | 20250130-hongbo.hwp | `pageCount` |
| EX11 | 2쪽 시험지 HWP | exam-kor-2p.hwp | 같은 경로 |
| EX12 | 2쪽 시험지 HWPX | exam-kor-2p.hwpx | 같은 경로 |
| EX14 | 3쪽 시험지 | exam-kor-3p.hwp | 같은 경로 |
| EX22 | 표 표본 쪽수 | table-001.hwp | 같은 경로 |
| EX27 | 4쪽 시험지 | exam-kor-4p.hwp | 같은 경로 |

실패 모드:

- `info` 의 `pageCount` 를 가져온다. 같은 숫자일 수는 있으나 이 과제의
  오라클은 `export-text` 다. 명령이 다르면 측정 축이 다르다.
- `-p 0` 만 뽑고 1이라고 쓴다. 전 쪽 과제가 아니다.
- 파일 이름을 쪽수로 믿는다(`exam-kor-2p` → 2). 라이브 오라클이 다시 센다.

### J3. 첫 쪽 글자 수 (`export-text -p 0` + len)

첫 쪽 본문 문자열의 길이다. 본문을 답에 넣지 않는다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| EX04 | 1쪽 시험지 글자 수 | exam-kor-1p.hwp | `pages[0].text` 길이 |
| EX13 | 2쪽 시험지 첫 쪽 | exam-kor-2p.hwp | 같은 경로 |
| EX25 | 3쪽 시험지 첫 쪽 | exam-kor-3p.hwp | 같은 경로 |

실패 모드:

- 전 쪽 텍스트 길이를 센다. `-p 0` 이 빠지면 `pages` 전체가 나온다.
- `pages[0].text` 를 답에 넣고 길이를 빠뜨린다. `len_answer_eq` 는
  숫자와 길이를 대조한다.
- 쪽 번호 1 기준을 쓴다. `-p 1` 은 둘째 쪽이다.

### J4. 날짜 수확 (`extract-data --kind date`)

날짜만 센다. 금액·수량을 섞으면 실패다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| EX03 | 홍보문 날짜 | hongbo.hwp | `itemCount` |
| EX08 | 1쪽 시험지 HWP 날짜 | exam-kor-1p.hwp | 같은 경로 |
| EX09 | 같은 시험지 HWPX | exam-kor-1p.hwpx | 같은 경로 |
| EX19 | 빈 HWPX 날짜 0건 | blank_hwpx.hwpx | 같은 경로 |
| EX21 | `--limit 1` 절단 | hongbo.hwp | 이번 응답 `itemCount` |
| EX23 | 2010-01-06 표본 | 2010-01-06.hwp | 같은 경로 |

실패 모드:

- `--kind` 를 빼 `--kind all` 과 같은 전량을 센다.
- `totalItemCount` 를 `itemCount` 자리에 넣는다. EX21 은 특히 다르다.
  `--limit 1` 이면 이번 응답은 최대 1, 총량은 `totalItemCount` 다.
- 0건을 실패로 보고 명령을 바꿔 무언가를 찾는다. EX19 는 0이 정답일
  수 있다.
- 파일 이름의 날짜(`2010-01-06`)를 1건으로 센다. 본문에 그 표기가
  있는지는 오라클이 안다.

### J5. 금액·수량·전종류

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| EX05 | 홍보문 금액 | hongbo.hwp | `--kind amount` `itemCount` |
| EX06 | 홍보문 수량 | hongbo.hwp | `--kind number` |
| EX07 | 홍보문 전종류 | hongbo.hwp | `--kind all` |
| EX10 | 시험지 금액 | exam-kor-1p.hwp | amount |
| EX20 | 빈 HWPX 금액 0건 | blank_hwpx.hwpx | amount |
| EX26 | 시험지 HWPX 전종류 | exam-kor-1p.hwpx | all |
| EX28 | 시험지 수량 | exam-kor-1p.hwp | number |

실패 모드:

- 단위 없는 표 숫자를 수량으로 센다. 코어는 단위가 있는 것만 인정한다.
- `counts.amount` 와 `itemCount` 를 섞는다. `--kind amount` 일 때
  둘은 같아야 하지만, 오라클 경로는 `itemCount` 다.
- `all` 과 `date` 를 같은 숫자로 제출한다. 문서에 금액이 하나라도
  있으면 달라진다.

## 과제 목록 (EX01–EX28)

### EX01 — 차트 데이터 추출 — 행 수

기존. `samples/chart/세로막대형/묶은세로막대형.hwp`, `--chart 1`,
`charts[0].rowCount`. 이 pack 의 첫 차트 과제다. 이후 차트 과제는
같은 연산·다른 표본이다.

### EX02 — 본문 텍스트 추출 — 페이지 수

기존. 홍보문 `export-text` `pageCount`. 실문서라 쪽수가 1이 아닐 수
있다. 파일 이름으로 추측하지 마라.

### EX03 — 날짜 수확 — extract-data --kind date

기존. 홍보문 날짜 `itemCount`. EX05·EX06·EX07 의 기준점이다.
`--kind` 를 바꾸면 숫자가 달라질 수 있다.

### EX04 — 첫 쪽 본문 추출 — 글자 수

기존. `exam-kor-1p.hwp`, `-p 0`, `len_answer_eq`. 1쪽 문서라도
`-p 0` 을 빼면 경로가 `pages[0].text` 가 아닐 수 있다. 빼지 마라.

### EX05 — 금액 수확

홍보문 `--kind amount`. EX03 과 같은 문서, 다른 종류. 두 답을
같은 숫자로 내면 보통 틀린다.

### EX06 — 수량 수확

홍보문 `--kind number`. 단위 없는 숫자는 항목이 아니다. 표가 커도
수량이 적을 수 있다.

### EX07 — 전종류 수확

홍보문 `--kind all`. EX03+EX05+EX06 의 합과 같아야 하는 것이
정상이지만, 겹치는 구간은 코어가 한 번만 먹는다(왼쪽에서 오른쪽,
되추적 없음). 합을 손으로 더하지 말고 `all` 을 돌려라.

### EX08 / EX09 — 시험지 날짜, HWP 와 HWPX

같은 시험지의 두 형식. 각각 따로 채점한다. 한 형식의 답을 다른
형식에 복사하는 것은 허용되지만, 오라클은 각 입력을 다시 돌린다.
파서가 다르면 숫자가 갈라질 수 있다. 그때는 복사한 쪽이 실패한다.

### EX10 — 시험지 금액

1쪽 시험지에 금액 표기가 있을 수도, 없을 수도 있다. 0건이면 0을
제출하면 된다.

### EX11 / EX12 — 2쪽 시험지 쪽수, HWP 와 HWPX

파일 이름의 `2p` 를 믿지 마라. `export-text --json` 의 `pageCount` 다.

### EX13 — 2쪽 시험지 첫 쪽 글자 수

EX04 와 같은 연산, 다른 표본. 둘째 쪽을 섞으면 길이가 달라진다.

### EX14 — 3쪽 시험지 쪽수

`exam-kor-3p.hwp`. EX11·EX27 과 함께 쪽수 격자를 만든다.

### EX15 — 묶은가로막대형 행 수

세로가 아니라 가로. 차트 1. 행 = 카테고리 계약은 같다.

### EX16 — 꺽은선형 행 수

라인 차트. 표식이 없는 기본 꺽은선. `표식이있는꺽은선형` 과 섞지 마라.

### EX17 — 2차원원형 행 수

원형. 3차원원형·원형대원형과 섞지 마라. 경로는 과제 `input` 에 있다.

### EX18 — 묶은세로막대형 HWPX

EX01 의 쌍. 차트 데이터가 같으면 행 수도 같다. 달라도 각각이 정답이다.

### EX19 / EX20 — 빈 문서 0건

`blank_hwpx.hwpx`. 날짜 0, 금액 0. 종료 코드 0. 빈 결과를 오류로
승격하는 래퍼를 쓰면 실패다. 파이프라인은 0건을 삼켜야 한다.

### EX21 — --limit 1

홍보문 날짜를 최대 1건만. 묻는 것은 `itemCount`(이번 응답)다.
문서에 날짜가 10건이어도 답은 0 또는 1 이다. `totalItemCount` 를
내면 실패다.

### EX22 — table-001 쪽수

표 pack 이 즐겨 쓰는 표본의 **본문** 쪽수. 표 행 수가 아니다.
`table-to-csv` 의 `rowCount` 를 가져오면 축이 틀린다.

### EX23 — 2010-01-06 날짜

파일 이름에 날짜가 있다. 본문 추출 건수와 이름이 같으리라는 보장은
없다. 오라클만 믿는다.

### EX24 — 누적세로막대형 행 수

EX01 과 같은 폴더, 다른 파일. `묶은` 과 `누적` 을 바꾸면 다른
차트다. 행 수가 같을 수는 있어도 입력이 다르다.

### EX25 — 3쪽 시험지 첫 쪽 글자 수

EX13 의 3쪽 판. 첫 쪽만.

### EX26 — 시험지 HWPX 전종류

EX09(날짜) + EX10(금액 HWP) + EX28(수량 HWP) 의 HWPX 전종류.
형식을 섞어 제출하지 마라.

### EX27 — 4쪽 시험지 쪽수

쪽수 격자의 맨 끝. `exam-kor-4p.hwp`.

### EX28 — 시험지 수량

`--kind number`. 문제 번호(`제3조` 식)는 수량이 아니다. 코어가
서수를 걸러 준다.

## 채점

모든 신규 과제는 `submit.kind = answer` 다. 제출물은
`answer.json` 한 장이다.

| 연산자 | 쓰임 | 과제 |
|---|---|---|
| `answer_eq` | 라이브 봉투 경로와 제출 키 대조 | EX01–EX03, EX05–EX12, EX14–EX24, EX26–EX28 |
| `len_answer_eq` | 문자열 길이와 제출 숫자 대조 | EX04, EX13, EX25 |

`file_exists` 가 없다. 산출 파일을 내면 채점이 읽지 않는다.
`deep_contains` 가 없다. 전역 훑기는 조회 과제에서도 쓰지 않는다 —
경로를 지목하는 편이 정직하다.

기준 풀이(`reference/EXnn.json`)는 같은 명령을 다시 돌려 답을 채운다.
`const` 박제는 이 pack 에 없다.

## 명령 레시피

```bash
# 차트 1의 행 수
rhwp chart-to-csv samples/chart/세로막대형/묶은세로막대형.hwp --chart 1 --json \
  | jq '.charts[0].rowCount'

# 본문 쪽수
rhwp export-text samples/20250130-hongbo.hwp --json | jq '.pageCount'

# 첫 쪽 글자 수
rhwp export-text samples/exam-kor-1p.hwp -p 0 --json \
  | jq '.pages[0].text | length'

# 날짜 / 금액 / 수량 / 전종류
rhwp extract-data samples/20250130-hongbo.hwp --kind date --json | jq '.itemCount'
rhwp extract-data samples/20250130-hongbo.hwp --kind amount --json | jq '.itemCount'
rhwp extract-data samples/20250130-hongbo.hwp --kind number --json | jq '.itemCount'
rhwp extract-data samples/20250130-hongbo.hwp --kind all --json | jq '.itemCount'

# 절단 — 이번 응답 vs 총량
rhwp extract-data samples/20250130-hongbo.hwp --kind date --limit 1 --json \
  | jq '{itemCount, totalItemCount, truncated}'

# 빈 문서 — 0건, exit 0
rhwp extract-data samples/hwpx/blank_hwpx.hwpx --kind date --json \
  | jq '{itemCount, kind}'
```

채점 왕복:

```bash
python gym/tools/build_baseline.py --agent baseline --pack extraction --bin target/debug/rhwp
python gym/score.py --agent baseline --pack extraction --bin target/debug/rhwp
```

## 실패 모드 상세

### 종류 혼동

`--kind date` 과제에 `--kind all` 의 숫자를 내면 `answer_eq` 가
거절한다. 홍보문처럼 날짜와 금액이 같이 있는 문서에서 특히 잘 걸린다.
EX03·EX05·EX06·EX07 네 숫자를 한 번에 뽑아 비교하는 것이 안전한
습관이다.

### 형식 혼동

`.hwp` 와 `.hwpx` 는 같은 본문처럼 보여도 입력 경로가 다르다.
EX08 의 답을 EX09 에 복사하는 것은 기준 풀이가 하는 일이 아니다.
기준 풀이는 각 입력을 다시 돌린다.

### 차트 번호 0 기준

`table-to-csv --table 0` 과 `chart-to-csv --chart 1` 은 다른
전통이다. 차트에는 `export-tables` 같은 발견 명령이 없어 번호가
문서 순서 그 자체다. 0 을 넣으면 사용법 오류(exit 2)이고 봉투가
없다. `answer_eq` 는 봉투 파싱 실패로 떨어진다.

### 쪽 번호 1 기준

`export-text -p 1` 은 둘째 쪽이다. EX04·EX13·EX25 는 `-p 0`.
렌더 도구의 `--from 1` 과 섞지 마라. 이 pack 은 `extract-pages` 를
쓰지 않는다.

### 0건을 오류로 승격

EX19·EX20 은 빈 문서다. 래퍼가 `itemCount==0` 을 exit 1 로 바꾸면
과제가 아니라 도구가 틀린 것이다. 코어 계약은 0건 = 성공이다.

### --limit 과 totalItemCount

EX21 은 `itemCount` 만 묻는다. `totalItemCount` 를 내면 홍보문
날짜가 2건 이상일 때 실패한다. 절단 총량을 보고 싶으면 다른 과제를
만들어야 한다. 이 pack 은 이번 응답만 잰다.

### 본문 길이를 정규식으로

`export-text` 없이 바이너리를 열어 글자 수를 세면 `len_answer_eq` 가
거절한다. 오라클은 rhwp 가 뽑은 `pages[0].text` 의 길이다. 숨은
문자·필드 코드가 본문에 포함되는지는 명령이 정한다.

### 새 명령으로 우회

`digest` · `search` · `info` · `export-structure` 는 이 pack 의
오라클이 아니다. 같은 숫자가 나와도 측정 축이 다르다. 기준 풀이는
세 명령만 부른다.

## 커버리지와의 관계

`gym/tools/coverage.py` 는 이 pack 의 `checks[].cmd[0]` 과
`steps[].answer.*.cmd[0]` 을 스캔한다. 확장 후 격자 행은 대략:

```
[extraction] chart-to-csv, export-text, extract-data
```

세 명령은 이미 EX01–EX04 로 노출돼 있었다. 이번 확장은 **빈 명령을
메우는 것이 아니라** 같은 명령의 종류·형식·표본 격자를 촘촘히 하는
것이다. 커버리지 퍼센트는 거의 그대로다. 분모(에이전트-대면 명령)도
그대로다. 정직한 분모는 `batch`·`edit`·`export`·`query` 만이고,
`diagnostic`/`internal`/`serve` 는 빈 곳이 아니다.

남는 빈 곳(이 pack 밖):

- `extract-data` 의 항목 주소(`section`/`paragraph`/`page`)를 묻는
  과제. 지금은 `itemCount` 만.
- `chart-to-csv --bom` · `--chart` 생략(전량).
- `export-text` 의 쪽 범위 여러 쪽(`-p 1` 등).
- `csv-to-chart` 왕복 — `studio-e2e` 의 축.

그 빈 곳은 이 README 의 "하지 않는 것"과 같다. 억지로 메우면 축이
흐려진다.

## 스키마 불변식

`scripts/tests/test_gym_extraction_pack.py` 가 CI 에서 다시 본다.

- 과제 id 는 `EXnn`, 기준 풀이와 1:1.
- 모든 과제는 `submit.kind = answer`.
- 연산자는 `answer_eq` 또는 `len_answer_eq`.
- 명령 화이트리스트: `extract-data`, `export-text`, `chart-to-csv`.
- 표본은 위 표의 경로만.
- `fill-fields` · `csv-to-table` · `batch` · `deep_contains` 문자열
  부재.
- `runner` 신원을 갈아끼우지 않는다.
- pack README 와 working 문서가 있다.

## 재현 (기준 풀이 왕복 — 이 pack 의 admission)

```bash
python gym/tools/build_baseline.py --agent baseline --pack extraction --bin target/debug/rhwp
python gym/score.py               --agent baseline --pack extraction --bin target/debug/rhwp
```

모든 과제가 라이브 오라클이므로 골든 파일을 고칠 일은 없다. 바이너리가
추출 규칙을 바꾸면 기준 풀이와 채점이 같이 움직인다. 그것이 이
저장소의 결이다.

## 관련

- `table-csv` — 표를 뽑아 고쳐 되넣기. 읽기가 아니라 편집.
- `batch-ops` — 서식 1 + 데이터 N. 추출이 아니라 대량 쓰기.
- `studio-e2e` ST01 — 차트 데이터 편집 왕복.
- `gym/tools/coverage.py` — 분모 정직성.
- `mydocs/working/gym_coverage_and_extract.md` — 이번 확장의 작업 기록.
- `mydocs/manual/cli_commands.md` — `extract-data` · `export-text` ·
  `chart-to-csv` 정본.

EX01–EX04 는 기존 축이다. EX05–EX28 은 같은 연산자·같은 CLI 로
종류·형식·표본 지목을 더 촘촘히 늘린 확장이다. 새 pack 도, 새 CLI 도,
T07/fill-fields 복제도 없다.
