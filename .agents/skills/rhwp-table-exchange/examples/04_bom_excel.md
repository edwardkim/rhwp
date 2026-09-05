# 04 — 엑셀에서 한글이 깨질 때 (`--bom`)

권위: `table_csv_contract.rs` `bom_flag_only_affects_the_file_not_the_envelope`.

## 증상

Windows 엑셀이 `제목` 을 깨진 문자로 보여 준다.

## 명령

```bash
rhwp table-to-csv samples/hwp_table_test.hwp --table 0 -o table0.csv --bom --json
```

## 기대

- 봉투 `bom: true`
- `tables[0].csv` 첫 글자는 `제` (U+FEFF 아님)
- 파일 앞 3바이트 `EF BB BF`

```bash
python -c "p=open('table0.csv','rb').read(3); print(p.hex())"
# efbbbf
```

JSON 봉투를 `.csv` 로 저장하지 마라. `-o` 가 낸 파일을 엑셀에 넣는다.

픽스처: [../fixtures/envelopes/table_to_csv_bom_file.json](../fixtures/envelopes/table_to_csv_bom_file.json),
[../fixtures/loops/bom_excel.json](../fixtures/loops/bom_excel.json).
