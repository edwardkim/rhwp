# 02 — 단건 fill-fields

`edit fill-fields <파일> --data <JSON|@파일> [옵션]` (#3329).

검증된 코어 `set_field_value_by_name` 을 재사용한다. **새 편집 로직이
없고**, 필드 값만 바꾸므로 레이아웃·구조는 불변이다. 이 스킬이 다른
쓰기 경로를 열어 우회하지 않는다.

```bash
rhwp fields 신청서.hwp --json | jq -r '.fields[].name'
rhwp edit fill-fields 신청서.hwp --data @row.json -o output/작성본.hwp --verify --json
rhwp fields output/작성본.hwp --json | jq -c '[.fields[]|select(.value!="")|{name,value}]'
```

## --data

- 인라인: `--data '{"필드이름":"값"}'`
- 파일: `--data @row.json` (대량·셸 인용 회피)
- 키 = `fields[].name` 또는 `이름[N]`
- 값이 문자열이 아니면 JSON 표현으로 넣는다
- **UTF-8**. CP949 는 `stream did not contain valid UTF-8` 로 exit 1

`@파일` 의 내용은 레시피 01 이 실측한 객체 하나다. 배열이나 JSONL 을
단건 `--data` 에 넣지 않는다. N행은 [04_batch_fill.md](04_batch_fill.md).

## 산출 경로

| 지정 | 결과 |
| --- | --- |
| `-o out.hwp` | 그 경로 |
| `-o` 생략 | `<입력명>_filled.<입력과 같은 확장자>` (입력 옆) |
| `--dry-run` | 파일을 만들지 않음. `output` 키 없음 |

형식 보존 (#3383):

- HWPX 입력 → HWPX 산출, `outputFormat: "hwpx"`
- HWP5/HWP3 입력 → HWP5 산출, `outputFormat: "hwp5"`
- HWPX 에 `-o ….hwp` 를 명시하면 HWP5 로 저장하되 stderr 경고
- HWP 에 `-o ….hwpx` 를 줘도 형식은 바뀌지 않음. 변환은 `export-hwpx`

이 스킬은 형식 변환을 겸하지 않는다.

## 통과 판정

셋 다 만족해야 단건 완료:

1. `notFound: []`
2. `ambiguous: []`
3. `--verify` 를 붙였으면 `verify.identical: true`
4. `filledCount` 가 의도한 개수

`filledCount` 만 보면 오타 필드가 빈 칸으로 제출된다.

레시피 01 실측 (성공):

```json
{"ambiguous":[],"changedPages":[0],"confusable":[],"dryRun":false,"filled":[{"name":"myMsg01","occurrence":0,"value":"홍길동 귀하"}],"filledCount":1,"notFound":[],"output":"…/form-01_filled.hwp","outputFormat":"hwp5","schemaVersion":"1.0","source":"…/form-01.hwp","verify":null}
```

`--verify` 실측:

```json
{"verify":{"diffCount":0,"identical":true}, … }
```

## 실패와 원본

- 필드 설정이 하나라도 실패하면 출력 파일을 쓰지 않고 exit 1
- 없는 파일: exit 1, stdout 비움
- 깨진 JSON / `--data` 없음: exit 2
- `--verify` 차이: exit 3, **산출물은 남는다**

원본은 어떤 실패에서도 불변. `-o` 를 원본과 같게 주지 않는다.

## 재독

보고를 믿지 않는다. 산출물을 `fields --json` 으로 다시 읽어
`fields[name].value` 가 요청과 같은지 대조한다. `--verify` 는 그 재파싱을
명령 안에 넣은 것이다 (#3702). 둘 중 하나는 필수.

```bash
rhwp fields out.hwp --json \
  | jq -e '[.fields[] | select(.name=="myMsg01")][0].value == "홍길동 귀하"'
```

## 바꾸지 않는 것

- 문단 모양, 표 격자, 쪽 정의
- 누름틀 자체(이름·형식)
- 머리말/각주 사각지대 필드 (01장)
- 로고 그림

값이 칸을 넘치는지는 fill-fields 축의 overflow 보고가 #3480 진행 중이다.
지금은 긴 값이면 `export-svg` 로 해당 쪽만 본다. set-cell 축의 overflow 를
이 명령에 이식하지 않는다.

## 최소 명령줄 습관

```bash
# 1) 조사
rhwp fields 서식.hwp --json > /tmp/fields.json
# 2) 키를 그대로 복사해 row.json 작성 (UTF-8)
# 3) 선검증
rhwp edit fill-fields 서식.hwp --data @row.json -o output/작성본.hwp --dry-run --json
# 4) 실행+검증
rhwp edit fill-fields 서식.hwp --data @row.json -o output/작성본.hwp --verify --json
```

3 과 4 는 `--dry-run` 한 토큰만 다르다. 인자를 다시 조립하지 않는다.
