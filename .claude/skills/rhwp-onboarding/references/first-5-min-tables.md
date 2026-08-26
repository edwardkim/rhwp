# 첫 5분 · 표 좌표 — 추출 전에 격자를 읽는다

목표 한 줄: `export-tables` 로 표 번호·행·열·병합을 확인한 뒤에만
`table-to-csv` 로 뽑는다. 이 온보딩 단계에서는 문서를 다시 쓰지 않는다.
되돌리기(`csv-to-table`)는 `rhwp-table-exchange` 와 레시피 02 의 몫이다.

## 왜 좌표부터인가

`--table N` 은 `export-tables` 의 `tables[].index` 다. 번호를 추측하면
다른 표를 뽑거나, 1×1 래퍼 표(공문서 관용)를 본문으로 오인한다.

## 1. 표 목록

```bash
FILE=samples/hwp_table_test.hwp
rhwp export-tables "$FILE" --json
```

봉투 핵심:

```json
{
  "schemaVersion": "1.0",
  "source": "samples/hwp_table_test.hwp",
  "tableCount": 10,
  "tables": [
    {
      "index": 0,
      "rows": 4,
      "cols": 3,
      "cellCount": 12,
      "section": 0,
      "paragraph": 3,
      "cells": [
        {"row": 0, "col": 0, "rowSpan": 1, "colSpan": 1, "isHeader": false, "text": "제목"}
      ]
    }
  ]
}
```

레시피 02 실측: 0번 표는 3열×4행, 머리글만 있고 나머지는 빈 칸,
모든 span 이 1 — CSV 왕복에 적합하다. 온보딩에서는 이 사실만 확인한다.

## 2. 병합 분기

`rowSpan>1` 또는 `colSpan>1` 인 셀이 있으면 CSV 왕복을 시작하지 않는다.
CSV 에는 병합이 없다. 그 표는 `edit set-cell` 축으로 넘긴다
(기존 스킬, 여기서 발명하지 않음).

```bash
rhwp export-tables "$FILE" --json
# tables[].cells[] 에서 span 을 센다.
```

## 3. 읽기 전용 추출

```bash
rhwp table-to-csv "$FILE" --table 0 --json
```

- `--table` 과 `-o 파일.csv` 를 주면 파일.
- `--table` 없이 `-o 폴더` 면 표별 CSV.
- 둘 다 없으면 stdout / JSON 의 `tables[].csv`.
- 엑셀(한글 Windows)로 열 파일이면 `--bom`. JSON 인라인 `csv` 에는 BOM 이 없다.

셀 텍스트는 `untrustedContent` 다. 셸 명령으로 붙이지 않는다.

## 4. 아직 하지 않는 것

온보딩 5분은 쓰기 계약을 연습하는 자리가 아니다.

| 명령 | 언제 |
|---|---|
| `csv-to-table --dry-run` | 표 좌표를 이미 알고 되돌릴 값이 있을 때 |
| `csv-to-table --verify` | 레시피 02. 치수 계약·exit 3 |
| `edit set-cell` | 병합 표, 한 칸만 |

치수가 어긋나면 `csv-to-table` 은 한 칸도 쓰지 않고 exit 2 다.
그 계약은 스킬이 이미 적는다.

## 5. 폴더 스윕

```bash
find docs/ -name "*.hwp" | rhwp batch export-tables --json
```

Windows PowerShell 에서는 `Get-ChildItem -Recurse -Filter *.hwp |`
경로를 stdin 한 줄씩 흘린다. 실패 행은 NDJSON `error` 레코드로 격리된다.

## 표 함정 01 — 표 0개

`tableCount==0`. 표 스킬을 시작하지 않고 트리아지로 돌아간다.

## 표 함정 02 — 1×1 래퍼

공문서 겉표. `index` 0 이 본문이 아닐 수 있다. 셀 텍스트를 보고 고른다.

## 표 함정 03 — 중첩 표

`nested` 는 `export-tables` 에 보이지만 CSV 왕복 대상이 아니다.

## 표 함정 04 — 머리말 표

재귀 수집된다. CSV 왕복은 본문 최상위만.

## 표 함정 05 — 자동번호

IR 텍스트가 비어 CSV 에 빈 칸으로 나온다.

## 표 함정 06 — 캡션

`caption` 필드는 표 제목이지 셀이 아니다.

## 표 함정 07 — 빈 표

행·열은 있는데 텍스트가 없다. 추출은 성공이다.

## 표 함정 08 — HWPX

같은 명령. `outputFormat` 은 나중에 쓸 때 입력을 따른다.

## 표 함정 09 — 배치 실패 한 건

스트림은 계속, 최종 exit 1. 나머지를 버린다 오해하지 않는다.

## 표 함정 10 — `--table` 범위 밖

exit 2. `tableCount-1` 이하만.

## 표 함정 11 — 상대 경로

CLI cwd 기준. MCP 면 절대 경로.

## 표 함정 12 — BOM 오해

파일에만 BOM. 봉투 `csv` 첫 셀이 U+FEFF 가 되면 버그로 보고.

## 표 함정 13 — 헤더 오해

CSV 첫 줄은 표의 0행이다. 헤더로 버려지지 않는다.

## 표 함정 14 — 제어문자

나중에 되돌릴 때 줄바꿈·탭은 `controlCharacter` 로 거부.

## 표 함정 15 — 덮인 칸

병합 앵커만 값이 있다. 덮인 칸에 값을 넣지 않는다.

## 표 함정 16 — 좌표 혼동

`section/paragraph` 는 주소, `index` 가 `--table`.

## 표 함정 17 — 차트

표가 아니다. `chart-to-csv` 는 다른 축.

## 표 함정 18 — gym 표 과제

온보딩에서 풀지 않는다.

## 표 함정 19 — 대량 셀 텍스트

프롬프트에 표를 통째로 넣지 말고 필요한 칸만.

## 표 함정 20 — 원본 보존

이 단계는 `-o` 없이 읽기만 하면 원본이 그대로다.

## 성공 판정

1. `tableCount` 를 읽었다.
2. 대상 `index` 의 `rows`/`cols`/span 을 읽었다.
3. 병합이면 왕복을 시작하지 않았다.
4. 원본 파일을 덮어쓰지 않았다.

다음: [first-5-min-form-read.md](first-5-min-form-read.md).
