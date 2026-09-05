# 04 — batch fill (메일머지)

`batch fill` (#3719 §6-6) 은 **입력 축이 다른 batch** 다.

다른 `batch` 하위명령(`info`·`fields`·`search`·…)은 stdin 에 **파일 경로
목록**을 받는다. `fill` 만은 서식 1개(`--form`) + 데이터 파일 1개
(`--data`) 이고, 산출은 데이터 행 수만큼이다.

stdin 에 파일 목록을 파이프하면 **아무 일도 안 일어난다**. fill 축은
stdin 을 읽지 않는다. 계약: `tests/batch_fill_contract.rs`.

```bash
rhwp fields 신청서.hwp --json | jq -r '.fields[].name'
rhwp batch fill --form 신청서.hwp --data 명단.csv \
  --out-dir output/filled --name-field 성명 --json > filled.ndjson
```

## 인자

| 인자 | 필수 | 뜻 |
| --- | --- | --- |
| `--form` | 예 | 서식 `.hwp`/`.hwpx` |
| `--data` | 예 | `.jsonl` 또는 `.csv` |
| `--out-dir` | 예 | 산출 폴더. **dry-run 에도 필수** |
| `--name-field` | 아니오 | 파일명으로 쓸 컬럼 |
| `--dry-run` | 아니오 | 파일을 쓰지 않고 행별 판정 |
| `--verify` | 아니오 | 행마다 저장 직후 재파싱 |
| `--threads N` | 아니오 | 기본 CPU 코어. 출력 순서는 입력 순 |
| `--json` | 권장 | stdout = NDJSON |

`--out-dir` 값이 `-` 로 시작하면 `./-결과` 처럼 쓴다.

## 데이터

확장자로 판별한다.

- `.jsonl` — 한 줄 = JSON 객체 1개. 키 = 누름틀 이름 또는 `이름[N]`
- `.csv` — 첫 줄 헤더 = 누름틀 이름. BOM·따옴표 허용

둘 다 **UTF-8**. 행 0개(헤더만)는 exit 2:
`오류: --data 에 데이터 행이 없습니다`.

레시피 05 실측 JSONL:

```
{"myMsg01":"김철수 귀하"}
{"myMsg01":"이영희 귀하"}
```

실측 NDJSON (요약):

```json
{"row":0,"filledCount":1,"output":"batch_out\\0001.hwp","outputFormat":"hwp5",…}
{"row":1,"filledCount":1,"output":"batch_out\\0002.hwp","outputFormat":"hwp5",…}
```

## 파일명

`--name-field` 생략: `0001.hwp` 식 1 기준 순번. 자릿수는 행 수에 맞추고
최소 4자리.

`--name-field 성명`: 그 컬럼 값. 금지 문자는 `_` 치환. 동명은 `_2` 접미.

산출 경로는 **한 행도 쓰기 전에** 전부 정해, 병렬에서도 이름이 실행
순서에 좌우되지 않는다. 명령이 중복 값의 "오류" 를 내지 않는다. 데이터
쪽에서 유일키를 만드는 것은 호출자 몫이다.

## name-field 와 notFound

파일명 용도 컬럼도 채울 필드 후보로 검사된다. 서식에 `성명` 누름틀이
없으면 매 행 `notFound` 에 `"성명"` 이 뜬다. **실패가 아니다.**

게이트:

```bash
jq -c 'select((.notFound - ["성명"] | length>0) or (.ambiguous|length>0))' filled.ndjson
```

## 봉투

성공 레코드 = 단건 fill-fields 봉투 + `row`(0 기준).

실패 레코드 = 공통 실패 스키마 + `row`. **실패한 행도 스트림에 남는다.**
사라지면 처리 누락을 셀 수 없다.

stdout 은 NDJSON 뿐. 요약 `batch fill: N행 중 …` 은 stderr.

종료:

- 전부 성공 0
- 한 행이라도 실패 1
- 인자 오류·빈 데이터 2
- verify 불일치가 최종 코드에 반영 (채움·저장 자체는 성공, exit 3 계약)

서식을 못 열면 시작 전에 한 번만 판정하고 N번 반복 보고하지 않는다.

## 단건과의 관계

결과는 `edit fill-fields` 를 행마다 호출한 것과 같다. 값 하나 절차는
02장. 이 장은 "여러 행" 만 다룬다.

N명분을 도구 N번 부르는 것도 동작하지만, 메일머지 요청에는 `batch fill`
한 번이 맞다. 새 루프 스크립트를 이 스킬이 발명하지 않는다.

## 하지 말 것

- `find … \| rhwp batch fill` (stdin 축이 아님)
- dry-run 에서 `--out-dir` 생략
- 요약 줄만 보고 성공 판정
- 동명 덮어쓰기를 명령 버그로 재구현해서 "고치기"
- gym pack 으로 행 수 채우기
