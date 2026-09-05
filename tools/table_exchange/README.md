# table_exchange — 표 CSV 왕복 픽스처 (M-tbl / #5485)

devel 의 `export-tables` · `table-to-csv` · `csv-to-table` 계약을 Python 으로
다시 적어, 치수 · `coveredCellNotEmpty` · `--dry-run` / `--verify` 픽스처를
고정한다.

새 CLI 를 만들지 않는다. DocumentCore 편집 로직을 발명하지 않는다.
병합 풀기·표 리사이즈를 구현하지 않는다. 바이너리 HWP 와 `rhwp` 는 부르지
않는다. `gym/` 과 다른 진행 석 파일은 범위 밖이다.

## 한 줄

```bash
python tools/table_exchange/fatten_catalog.py
python -m unittest discover -s tools/table_exchange/tests -t tools
```

## 권위

- `mydocs/manual/cli_commands.md` §export-tables · §table-to-csv · §csv-to-table · §종료 코드 #2707
- `tests/table_csv_contract.rs`
- `.agents/skills/rhwp-table-exchange/`

## 판정

| 신호 | exit | 파일 | 다음 |
|---|---:|---|---|
| `rowCountMismatch` / `colCountMismatch` | 2 | 안 씀 | CSV 치수를 표에 맞춘다 |
| `coveredCellNotEmpty` | 2 | 안 씀 | `edit set-cell` |
| `controlCharacter` / `csvParse` | 2 | 안 씀 | CSV 를 고친다 |
| `--dry-run` | 0/2 | 안 씀 | `changedPages` 는 null |
| `--verify` identical false | 3 | **남김** | `export-tables` 로 재독 |

`invalid[]` 는 전부 모은다. 첫 줄만 고치고 다시 돌리지 않는다.
