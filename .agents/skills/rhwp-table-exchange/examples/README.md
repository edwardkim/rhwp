# 표↔CSV 왕복 워크스루

실사용 에이전트가 기존 CLI 만으로 표를 뽑고 되돌리는 장면이다.
gym 과제가 아니다. 새 명령을 만들지 않는다.

각 편은 명령 → 기대 봉투 → 읽어야 할 키 → 실패 시 다음 수 순이다.
숫자는 [references/sample_transcripts.md](../references/sample_transcripts.md) 와
`fixtures/` 가 정본이다.

| 편 | 장면 | 명령 |
|---:|---|---|
| [01](01_coordinate_scan.md) | 좌표·병합 스캔 | `export-tables` |
| [02](02_single_table_extract.md) | 표 하나 CSV | `table-to-csv --table` |
| [03](03_all_tables_folder.md) | 전량 폴더 | `table-to-csv -o DIR` |
| [04](04_bom_excel.md) | 엑셀 한글 | `--bom` |
| [05](05_dry_run_preview.md) | 선확인 | `csv-to-table --dry-run` |
| [06](06_verify_success.md) | 저장+자기검증 | `--verify` identical |
| [07](07_row_count_mismatch.md) | 행 수 거부 | `rowCountMismatch` exit 2 |
| [08](08_col_count_mismatch.md) | 열 수 거부 | `colCountMismatch` exit 2 |
| [09](09_covered_cell.md) | 덮인 칸 | `coveredCellNotEmpty` |
| [10](10_control_character.md) | 줄바꿈·탭 | `controlCharacter` |
| [11](11_header_row_pitfall.md) | 헤더=0행 | 치수/덮어쓰기 |
| [12](12_merged_fallback_set_cell.md) | 병합 폴백 | `edit set-cell` 만 |
| [13](13_nested_out_of_v1.md) | 중첩 표 | v1 밖 |
| [14](14_roundtrip_hwp_table_test.md) | 레시피 02 전체 | 왕복 닫힘 |
| [15](15_verify_exit3.md) | IR 차이 | exit 3 데이터 |
| [16](16_batch_harvest.md) | 여러 문서 | `batch export-tables` |
| [17](17_rfc4180_quoting.md) | 쉼표·따옴표 | RFC 4180 |
| [18](18_index_not_zero.md) | index≠0 | 머리말 표 |

카탈로그: [../fixtures/catalog.json](../fixtures/catalog.json).
