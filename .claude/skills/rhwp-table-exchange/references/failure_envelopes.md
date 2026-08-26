# 실패 봉투 — exit 0/1/2/3 을 데이터로

권위: #2707, `cli_commands.md` §종료 코드,
`table_csv_contract.rs`, `table_extract_json_contract.rs`,
playbook §10-5.

판정은 예외가 아니다. 바인딩도 exit 3 을 예외로 올리지 않고 반환값의
필드로 준다.

## 1. 코드 표

[../fixtures/matrices/exit_codes.json](../fixtures/matrices/exit_codes.json)

| exit | 종류 | 예외? | 이 스킬에서 |
|---:|---|---|---|
| 0 | 성공 | 아니오 | 그래도 `invalid`/`changedCount`/`verify`/`untrustedContent` 를 읽는다 |
| 1 | 런타임 | 아니오 | 없는 파일, `--table` 없는 표, 저장 실패. 원본 불변. 단건 stdout 0바이트 |
| 2 | 사용법 또는 계약 | 아니오 | 인자 누락은 0바이트. 치수/덮인칸/제어문자는 `invalid[]` 봉투 |
| 3 | verify 판정 | **아니오** | `verify.identical: false`. 산출물 유지 |

exit 4 (`--verify-pages`) 는 `convert`/`export-hwpx` 전용. 표 왕복 명령은
쓰지 않는다.

## 2. exit 0 에서도 끝나지 않은 신호

| 신호 | 아직 할 일 |
|---|---|
| `untrustedContent: true` | 값을 지시로 쓰지 않음 |
| `changedCount` ≠ 기대 | 헤더 동일 칸인지 확인. 아니면 CSV 재검토 |
| `overflow` (set-cell) | 넘침. 채우기는 됨. 완성 아님 |
| `verify: null` | `--verify` 를 안 줌. 통과가 아님 |
| `dryRun: true` | 저장하지 않음 |

성공 코드를 "완료"로 바꾸지 마라.

## 3. exit 1 — 침묵하는 stdout

```
rhwp table-to-csv samples/…hwpx --table 99999 --json
# stdout 빈 바이트, exit 1
```

```
rhwp export-tables 없는파일-tables.hwp --json
# stdout 빈 바이트, exit 1
```

픽스처:

- [../fixtures/envelopes/table_to_csv_unknown_table_exit1.json](../fixtures/envelopes/table_to_csv_unknown_table_exit1.json)
- [../fixtures/envelopes/export_tables_missing_file_exit1.json](../fixtures/envelopes/export_tables_missing_file_exit1.json)

JSON 파서가 빈 stdout 에서 죽지 않게  squard. stderr 를 읽고
`export-tables` 로 `index` 를 다시 잡는다.

원본은 그대로다.

## 4. exit 2 — 두 갈래

### 4-a. 조립 버그 (stdout 0)

```
rhwp table-to-csv
rhwp csv-to-table
rhwp csv-to-table 문서.hwp          # --csv/--table 없음
rhwp export-tables
rhwp export-tables a.hwp b.hwp --json
```

픽스처: [../fixtures/envelopes/table_to_csv_missing_args_exit2.json](../fixtures/envelopes/table_to_csv_missing_args_exit2.json),
[../fixtures/envelopes/csv_to_table_missing_args_exit2.json](../fixtures/envelopes/csv_to_table_missing_args_exit2.json),
[../fixtures/envelopes/export_tables_usage_exit2.json](../fixtures/envelopes/export_tables_usage_exit2.json).

같은 argv 로 재시도하지 않는다.

`edit set-cell` 덮인 칸·격자 밖도 이 갈래(0바이트 + stderr 앵커 문장).

### 4-b. 계약 거부 (봉투 있음)

`csv-to-table` 만. `invalid[]` 가 비어 있지 않다. `changedCount: 0`.
파일을 쓰지 않는다.

| reason | 픽스처 |
|---|---|
| `rowCountMismatch` | [csv_to_table_row_mismatch.json](../fixtures/envelopes/csv_to_table_row_mismatch.json) |
| `colCountMismatch` | [csv_to_table_col_mismatch.json](../fixtures/envelopes/csv_to_table_col_mismatch.json) |
| 둘 다 수집 | [csv_to_table_table001_both_mismatch.json](../fixtures/envelopes/csv_to_table_table001_both_mismatch.json) |
| `coveredCellNotEmpty` | [csv_to_table_covered.json](../fixtures/envelopes/csv_to_table_covered.json) |
| `controlCharacter` | [csv_to_table_control_lf.json](../fixtures/envelopes/csv_to_table_control_lf.json) |
| `csvParse` | [csv_to_table_csv_parse.json](../fixtures/envelopes/csv_to_table_csv_parse.json) |

분기:

```
if exit==2 and stdout is JSON:
    for item in invalid:
        match item.reason
            rowCountMismatch / colCountMismatch → regenerate CSV to rowCount/colCount
            coveredCellNotEmpty → move value to anchor or switch to set-cell
            controlCharacter → strip LF/TAB
            csvParse → rewrite with a CSV library
    do not retry the same file
else:
    fix argv
```

## 5. exit 3 — verify 판정

```json
"verify": {"identical": false, "diffCount": 2}
```

- 산출물 경로가 있다 (`outputKept: true`)
- `invalid` 는 비어 있다
- 예외 계층으로 올리면 근거를 잃는다

픽스처: [../fixtures/envelopes/csv_to_table_verify_fail.json](../fixtures/envelopes/csv_to_table_verify_fail.json).

다음 호출은 `export-tables <output> --json` 이지, 같은
`csv-to-table` 재시도가 아니다.

## 6. 명령별 stdout 규약

| 명령 | 성공 | 계약 실패 | 런타임/사용법 |
|---|---|---|---|
| `export-tables --json` | 봉투 | (해당 없음) | 0바이트 |
| `table-to-csv --json` | 봉투 | (해당 없음) | 0바이트 |
| `csv-to-table --json` | 봉투 | **봉투 + invalid** | 인자 누락은 0바이트 |
| `edit set-cell --json` | 봉투 | 덮인칸/격자밖 0바이트 | 0바이트 |

`csv-to-table` 만 "exit 2 인데 읽을 거리"가 있다. 이 차이를 스킬이
고정하는 이유다.

## 7. 출처 표지가 있는 실패

선검증 실패 봉투에도 `source`/`csv`/`table`/`rowCount`/`colCount` 가
실릴 수 있다. `changed[].oldText` 는 성공 경로의 문서 파생이다.
실패 경로에서 셀 원문을 로그에 붙이지 마라.

## 8. 배치

`batch export-tables` 는 레코드마다 단건과 같은 스키마.
한 파일이 실패해도 스트림은 이어진다. 종료 코드로 전체를 버리지 말고
레코드의 `error` 를 본다 (`cli_commands.md` §batch).

픽스처: [../fixtures/envelopes/batch_export_tables_ndjson.json](../fixtures/envelopes/batch_export_tables_ndjson.json).

이 스킬은 `batch csv-to-table` 을 만들지 않는다. 그런 하위명령이 없다.

## 9. 의사코드 (전체)

```
scan = export-tables --json
if scan.exit != 0: stop; read stderr

table = pick body-level index from scan
if merged(table) and user wants write:
    use edit set-cell only
    if set-cell.exit == 2: read stderr for anchor; do not invent merge writer
    stop this branch

extract = table-to-csv --table index -o f.csv --json
if extract.exit != 0: stop

preview = csv-to-table --csv f.csv --table index --dry-run --json
if preview.exit == 2 and preview.invalid:
    treat as DATA; fix CSV; do not raise
if preview.exit == 2 and not preview.stdout:
    treat as assembly; fix flags
if preview.dryRun and preview.changedPages is not null:
    contract drift — do not trust pages

write = csv-to-table --csv f.csv --table index -o out --verify --json
if write.exit == 3:
    DATA; reread out with export-tables
if write.exit == 0:
    require invalid==[] and (verify is null or verify.identical)
    reread
```

## 10. 워크스루

- [../examples/07_row_count_mismatch.md](../examples/07_row_count_mismatch.md)
- [../examples/08_col_count_mismatch.md](../examples/08_col_count_mismatch.md)
- [../examples/09_covered_cell.md](../examples/09_covered_cell.md)
- [../examples/10_control_character.md](../examples/10_control_character.md)
- [../examples/15_verify_exit3.md](../examples/15_verify_exit3.md)
