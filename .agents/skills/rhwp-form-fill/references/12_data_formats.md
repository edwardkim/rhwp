# 12 — 데이터 형식

에이전트가 값을 어디에 담느냐만 정리한다. 파서를 새로 만들지 않는다.
`--data` 가 이미 받는 네 가지다.

## 1. 인라인 JSON

```bash
rhwp edit fill-fields 서식.hwp --data '{"myMsg01":"홍길동 귀하"}' -o out.hwp --json
```

짧을 때. 셸 인용이 깨지기 쉽다. PowerShell 은 작은따옴표 JSON 을
다르게 해석한다. 한국어 Windows 에서는 파일 쪽을 권장.

## 2. @파일 (단건)

```bash
# row.json — UTF-8, 객체 하나
{"myMsg01":"파일에서 읽은 값"}
rhwp edit fill-fields 서식.hwp --data @row.json -o out.hwp --json
```

- `@` 다음이 경로다
- 내용은 JSON 객체 하나. 배열·JSONL 금지
- UTF-8. BOM 없는 쪽을 권장
- 키가 `fields[].name` 또는 `이름[N]`

픽스처: `fixtures/data/row_form01.json`, `row_field01.json`,
`row_repeat_14.json`.

## 3. JSONL (batch)

한 줄 = 객체 하나. 줄마다 단건 `--data` 와 같은 모양.

```
{"myMsg01":"김철수 귀하"}
{"myMsg01":"이영희 귀하"}
```

확장자가 `.jsonl` 이어야 판별된다. `.txt` 로 저장하면 형식 오류.

픽스처: `fixtures/data/mailmerge_12.jsonl`.

## 4. CSV (batch)

첫 줄 헤더 = 누름틀 이름. BOM·따옴표 허용.

```
성명,myMsg01,이메일
홍길동,홍길동 귀하,hong@example.go.kr
```

- 헤더 철자는 `fields[].name` 과 같아야 한다
- 빈 헤더/행 0개 → exit 2
- `--name-field` 컬럼이 서식에 없어도 된다 (F11)

픽스처: `fixtures/data/mailmerge_12.csv`, `empty_header_only.csv`.

## 인코딩

| 저장 | 결과 |
| --- | --- |
| UTF-8 | 정상 |
| UTF-8 BOM | CSV 는 허용. JSON 은 구현에 따라 거절될 수 있어 BOM 없이 |
| CP949 / EUC-KR | exit 1, valid UTF-8 아님 |

Python: `open(path, "w", encoding="utf-8", newline="\n")`.
PowerShell: `Set-Content -Encoding utf8` 는 BOM 을 붙일 수 있다.
`utf8NoBOM` 또는 .NET `UTF8Encoding($false)` 를 쓴다.

## 키 설계

```
고유 이름     → "회사명"
반복 이름     → "목차1[0]" … "목차1[4]"
파일명 전용   → "성명" (서식에 없어도 됨, 게이트에서 제외)
쓰지 말 것    → "성명_1", "성명  ", "Name"
```

값을 비우려면 빈 문자열 `""`. 키를 빼면 그 칸은 그대로 둔다.

## 숫자·날짜

`--data` 값이 문자열이 아니면 JSON 표현으로 들어간다. 서식이
`"2026. 8. 18."` 을 원하면 문자열로 넣는다. 이 스킬이 날짜 포맷터를
발명하지 않는다. `guide`/`memo` 를 읽는다.
