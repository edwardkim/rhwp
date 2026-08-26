# M-tbl 표 CSV 왕복 픽스처 요약

- issue: #5485
- cases: **299**
- generator: `tools/table_exchange/fatten_catalog.py`

## 가족

| family | count |
|---|---:|
| covered | 81 |
| dimension | 90 |
| export-tables | 37 |
| dry-run | 24 |
| table-to-csv | 36 |
| verify | 31 |

## 종료 코드

| exit | count |
|---|---:|
| 0 | 115 |
| 1 | 2 |
| 2 | 176 |
| 3 | 6 |

## invalid reason

| reason | cases |
|---|---:|
| coveredCellNotEmpty | 73 |
| controlCharacter | 9 |
| rowCountMismatch | 61 |
| colCountMismatch | 38 |
| csvParse | 3 |

## 하지 않은 것

- 새 CLI 없음
- DocumentCore 편집 로직 없음
- 병합 풀기·표 리사이즈 없음
- gym/ 미수정
- 다른 진행 석 파일 미수정
