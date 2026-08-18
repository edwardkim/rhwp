# M-tbl 작업 기록 (#5485)

devel 의 `export-tables` / `table-to-csv` / `csv-to-table` 계약을
치수 · `coveredCellNotEmpty` · `--dry-run` / `--verify` 픽스처로 고정했다.

- 케이스 299 (dimension 90 · covered 81 · export-tables 37 · table-to-csv 36 · verify 31 · dry-run 24)
- 새 CLI 없음. DocumentCore 편집 로직 없음. gym 없음.
- 바이너리 HWP 를 열지 않는다. 모델은 `tests/table_csv_contract.rs` 와 스킬 봉투를 따른다.
- 병합 표 되돌리기는 기존 `edit set-cell` 만 가리킨다.

```
python tools/table_exchange/fatten_catalog.py
python -m unittest discover -s tools/table_exchange/tests -t tools
```
