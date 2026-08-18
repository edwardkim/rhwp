# 13 — CSV 되돌리기 게이트 (`csv-to-table`)

층: 1 인접. 표 전체를 덮어쓰되 치수가 다르면 **한 칸도 쓰지 않는다**.

권위: [single_edit.md](../references/single_edit.md) §9, playbook §13.

표 왕복의 전체 스킬은 rhwp-table-exchange 다. 이 편은 안전 편집 관점의
`invalid[]` + dry-run 만 고정한다.

## 1. 올바른 순서

```bash
rhwp table-to-csv 양식.hwpx --table 12 -o t12.csv --json
# 값만 수정. 행·열·병합 빈 칸을 유지
rhwp csv-to-table 양식.hwpx --csv t12.csv --table 12 --dry-run --json
rhwp csv-to-table 양식.hwpx --csv t12.csv --table 12 -o 완성.hwpx --verify --json
rhwp table-to-csv 완성.hwpx --table 12
```

손으로 CSV 를 만들지 않는다. 병합 자리를 사람이 비우기는 어렵다.

## 2. 거부 표본

픽스처 [../fixtures/envelopes/csv_to_table_invalid.json](../fixtures/envelopes/csv_to_table_invalid.json).

```
$ rhwp csv-to-table samples/table-001.hwp --csv out/bad.csv --table 0 --dry-run --json
{"changed":[],"changedCount":0,"colCount":9,"rowCount":19,
 "invalid":[
   {"actual":2,"expected":19,"reason":"rowCountMismatch",
    "message":"CSV 행 수 2 가 표 0 의 행 수 19 와 다릅니다 — 표 크기는 바꾸지 않습니다."},
   {"actual":2,"expected":9,"reason":"colCountMismatch","row":0},
   {"actual":2,"expected":9,"reason":"colCountMismatch","row":1}
 ]}
exit=2
```

`coveredCellNotEmpty` 는 덮인 칸에 값이 있을 때. 앵커에 두고 그 칸은 비운다.

exit 2 여도 봉투가 있다. 단건 `edit` 와 다른 점이다.

## 3. 체크리스트

- [ ] `table-to-csv` 산출을 고쳤다
- [ ] dry-run 의 `invalid[]` 가 비었다
- [ ] `--table` 이 `index` 다
