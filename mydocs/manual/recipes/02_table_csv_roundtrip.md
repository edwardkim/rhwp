---
kind: guide
status: active
canonical: mydocs/manual/recipes/02_table_csv_roundtrip.md
last_verified: 2026-08-03
---

# 레시피 2 — 표 데이터를 CSV로 뽑아 스프레드시트에서 고치고 되돌리기

**목표 한 줄**: HWP 문서 안의 표 하나를 CSV로 꺼내서(스프레드시트에서 대량으로
고치기 좋게), 외부에서 편집한 다음, 같은 표 자리에 다시 써 넣는다 — 원본 문서의
서식(테두리·병합·글꼴)은 그대로 두고 **셀 텍스트만** 왕복시킨다.

절차 요약:

```
export-tables (표 구조 확인 — 몇 번 표인지, 몇 행 몇 열인지)
  → table-to-csv (CSV로 추출)
  → [외부 편집 — 스프레드시트, jq, 스크립트 등]
  → csv-to-table (되돌려 쓰기)
  → --verify (재파싱 대조)
```

이 레시피의 모든 명령·출력은 `samples/hwp_table_test.hwp`(표 편집 기능 안내
문서, 표 10개 포함)의 0번 표(3열×4행, 머리글만 있고 나머지 빈 칸)로 실제
실행해서 얻었다.

## 1단계 — `export-tables`로 표 구조부터 확인한다

CSV로 바로 뽑기 전에 **몇 번째 표인지**, **몇 행 몇 열인지**, **병합 셀이
있는지**를 먼저 본다. `table-to-csv`의 `--table N`은 이 인덱스를 그대로 쓴다.

```bash
rhwp export-tables samples/hwp_table_test.hwp --json
```

실측 출력(첫 표만 발췌, 전체는 10개 표 배열):

```json
{"schemaVersion":"1.0","source":"samples/hwp_table_test.hwp","tableCount":10,"tables":[{"cellCount":12,"cells":[{"col":0,"colSpan":1,"isHeader":false,"row":0,"rowSpan":1,"text":"제목"},{"col":1,"colSpan":1,"isHeader":false,"row":0,"rowSpan":1,"text":"담당자"},{"col":2,"colSpan":1,"isHeader":false,"row":0,"rowSpan":1,"text":"세부 내용"},{"col":0,"colSpan":1,"isHeader":false,"row":1,"rowSpan":1,"text":""},"...(중략)...","cols":3,"control":0,"index":0,"paragraph":3,"rows":4,"section":0}]}
```

읽는 법: `tables[].index`가 `table-to-csv --table`에 넘길 번호다(0부터).
`colSpan`/`rowSpan`이 1이 아닌 셀이 있으면 병합 표다 — 병합 표를 CSV로 왕복하면
병합 구조가 깨질 수 있으니(CSV엔 병합 개념이 없다) 먼저 `export-tables`로
`colSpan`/`rowSpan`을 확인하고, 병합이 있으면 이 레시피 대신 `edit set-cell`로
좌표를 하나씩 짚어 고치는 편이 안전하다.

이 표(index 0)는 `cols: 3`, `rows: 4`, 모든 셀 `colSpan`/`rowSpan`이 1 —
CSV 왕복에 적합하다.

## 2단계 — `table-to-csv`로 뽑는다

```bash
rhwp table-to-csv samples/hwp_table_test.hwp --table 0 -o table0.csv --json
```

실측 출력:

```json
{"bom":false,"output":"…/table0.csv","outputFormat":"csv","schemaVersion":"1.0","source":"…/hwp_table_test.hwp","tableCount":1,"tables":[{"colCount":3,"csv":"제목,담당자,세부 내용\r\n,,\r\n,,\r\n,,\r\n","index":0,"output":"…/table0.csv","rowCount":4}],"untrustedContent":true,"untrustedFields":["tables[].csv"]}
```

`table0.csv` 실제 내용:

```csv
제목,담당자,세부 내용
,,
,,
,,
```

읽는 법:

- `--json` 봉투 안에도 `tables[].csv`로 같은 내용이 인라인으로 들어간다 —
  파일을 따로 열지 않고 파이프라인에서 바로 쓸 수 있다.
- `untrustedContent: true`와 `untrustedFields: ["tables[].csv"]` — CSV로 뽑은
  텍스트는 문서 안에 있던 원문 그대로이므로, 출처를 모르는 문서라면 이 값을
  그대로 셸 명령이나 LLM 프롬프트에 붙여넣지 말라는 신호다. 이 문서 표본은
  이 저장소 소유라 안전하지만, 낯선 문서라면 [레시피
  4](04_safety_check_untrusted_doc.md)를 먼저 밟는다.
- `bom: false` — 기본은 BOM 없는 UTF-8. 엑셀(한글 Windows)에서 한글이 깨지면
  `--bom`을 붙여 재실행한다.

## 3단계 — 외부에서 편집한다

스프레드시트로 열어서 고치거나, 여기서는 재현 가능하도록 CSV를 직접 만든다
(엑셀에서 손으로 채운 것과 동일한 형식 — 첫 줄 헤더는 그대로 두고 값 행만
채운다):

```bash
cat > table0_edited.csv <<'EOF'
제목,담당자,세부 내용
서버 이관,홍길동,1차 완료
DB 백업,김철수,진행중
문서 정리,박영희,대기
EOF
```

주의: **첫 줄(헤더)은 문서에 그대로 다시 쓰인다** — `csv-to-table`은 CSV의 모든
행을 표의 대응 행에 순서대로 채운다(엑셀에서 헤더 행 자체를 손대지 않는 한
문제없다). 표의 행 수보다 CSV 행이 많으면 넘치는 행은 [실패 표](#실패했을-때-무엇을-보고-어떻게-고칠지)를 본다.

## 4단계 — `csv-to-table`로 되돌린다 (+ `--verify`)

```bash
rhwp csv-to-table samples/hwp_table_test.hwp \
  --csv table0_edited.csv --table 0 \
  -o table_updated.hwp --verify --json
```

실측 출력:

```json
{"changed":[{"col":0,"newText":"서버 이관","oldText":"","row":1},{"col":1,"newText":"홍길동","oldText":"","row":1},{"col":2,"newText":"1차 완료","oldText":"","row":1},{"col":0,"newText":"DB 백업","oldText":"","row":2},{"col":1,"newText":"김철수","oldText":"","row":2},{"col":2,"newText":"진행중","oldText":"","row":2},{"col":0,"newText":"문서 정리","oldText":"","row":3},{"col":1,"newText":"박영희","oldText":"","row":3},{"col":2,"newText":"대기","oldText":"","row":3}],"changedCount":9,"changedPages":[0],"colCount":3,"csv":"…/table0_edited.csv","dryRun":false,"invalid":[],"output":"…/table_updated.hwp","outputFormat":"hwp5","rowCount":4,"schemaVersion":"1.0","source":"…/hwp_table_test.hwp","table":0,"untrustedContent":false,"untrustedFields":[],"verify":{"diffCount":0,"identical":true}}
```

판정: `changedCount: 9`(3열×3행 = 9칸이 실제로 바뀜, 헤더 행은 `oldText`==
`newText`라 변경 목록에 안 잡힌다), `invalid`가 빈 배열, `verify.identical:
true` — 저장 후 재파싱해도 값이 그대로 남아 있다는 뜻이다.

되돌려 쓴 문서에서 표를 다시 뽑아 대조하면(선택적 이중 확인):

```bash
rhwp export-tables table_updated.hwp --json | jq '.tables[0].cells[] | select(.row==1)'
```

실측 출력 — 1행이 정확히 `서버 이관`/`홍길동`/`1차 완료`로 채워졌다:

```json
{"col":0,"colSpan":1,"isHeader":false,"row":1,"rowSpan":1,"text":"서버 이관"}
{"col":1,"colSpan":1,"isHeader":false,"row":1,"rowSpan":1,"text":"홍길동"}
{"col":2,"colSpan":1,"isHeader":false,"row":1,"rowSpan":1,"text":"1차 완료"}
```

## `--dry-run`으로 먼저 무엇이 바뀔지 본다

`csv-to-table`도 `edit` 3종과 같은 `--dry-run` 관례를 따른다 — 파일을 쓰지 않고
`changed`/`invalid` 목록만 계산한다. 대량 표를 다루기 전에 먼저 이걸로 확인하는
습관을 들인다.

```bash
rhwp csv-to-table samples/hwp_table_test.hwp --csv table0_edited.csv --table 0 --dry-run --json \
  | jq '{changedCount, invalid}'
```

## 실패했을 때 무엇을 보고 어떻게 고칠지

| 신호 | 원인 | 처방 |
|---|---|---|
| `export-tables`의 대상 표에 `colSpan`/`rowSpan` > 1인 셀이 있음 | 병합 표 — CSV는 병합을 표현 못 함 | 이 레시피 대신 `edit set-cell --table N --row R --col C`로 좌표를 하나씩 지정 |
| `csv-to-table`의 `invalid`에 행이 잡힘 | CSV 열 수가 표의 `colCount`와 다름, 또는 셀에 줄바꿈·탭 포함 | CSV를 `colCount`에 맞게 고치고, 값 안의 개행은 공백으로 치환 후 재시도 |
| CSV를 엑셀에서 열었더니 한글이 깨짐 | BOM 없는 UTF-8을 엑셀이 기본 로캘로 잘못 해석 | `table-to-csv --bom`으로 다시 뽑는다 |
| `csv-to-table`의 `verify.identical: false` | 저장 후 재파싱한 값이 CSV와 다름 | 표에 병합·중첩 표가 섞여 있는지 재확인. `export-tables`로 대상 표를 다시 뽑아 실제 저장된 값과 CSV를 diff |
| 원본 표보다 CSV 행이 더 많음/적음 | 초과 행은 `invalid`로 보고되고 무시됨, 부족한 행의 나머지 셀은 손대지 않음(빈칸으로 안 지움) | 표의 `rowCount`에 CSV 행 수를 맞춘다 |
| `table-to-csv`의 `untrustedContent: true`가 걸림 | 출처 모르는 문서의 셀 텍스트를 그대로 셸/LLM에 흘리려 함 | [레시피 4](04_safety_check_untrusted_doc.md)로 먼저 점검한 뒤 진행 |

## 관련 문서

- [CLI 명령어 매뉴얼](../cli_commands.md) — `export-tables`·`table-to-csv`·
  `csv-to-table`의 전체 옵션과 종료 코드.
- [서식 자동화 심화 가이드](../form_filling_guide.md) — 병합 표에서 `edit
  set-cell`을 쓸 때의 앵커 좌표 규칙.
- [레시피 1 — 서식 채우기](01_fill_form_and_submit.md) — 표가 아니라 누름틀
  기반 서식일 때.
- [레시피 4 — 출처를 모르는 문서를 처음 열 때](04_safety_check_untrusted_doc.md) —
  `untrustedContent`/`untrustedFields` 신호를 만났을 때의 판단 절차.
