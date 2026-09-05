# 07 — batch export-tables

## 한 줄

격자 JSON. 병합 rowSpan/colSpan 보존.

## 호출

```bash
rhwp batch export-tables --json [옵션]
```

- 입력: stdin 한 줄 = 경로
- 단건 동형: export-tables --json
- 플래그: `--json`, `--threads`
- 성공 키: `schemaVersion`, `source`, `tableCount`, `tables`


## 언제

폴더의 표를 격자 JSON 으로 수확. 병합을 유지해야 할 때.
Markdown 표는 병합을 잃는다 (`| 5월 |  |  |`). 이 축은 `rowSpan`/`colSpan`
앵커만 낸다.

## 호출

```bash
rhwp batch export-tables --json < 목록.txt | jq -c '{source, tableCount}'
```

`tests/batch_axes_contract.rs` 는 `samples/table-001.hwp` 에서 병합이
배치 경로에도 남는지 고정한다. 이 스킬은 그 계약을 인용만 한다.

## 0건

`tableCount: 0` 은 성공이다 (B07). 재시도하지 않는다.
표가 머리말/글상자 안에만 있으면 `info` 의 표 열거는 놓치고
`export-tables` 는 재귀 수집한다. 실측: `samples/basic/treatise sample.hwp`
는 info 1개, export-tables 3개 (cli_commands 명문).

## 한계 (단건과 동일)

- 셀 안 자동번호는 IR 에 값이 없어 빈 자리
- 1×1 래퍼 표(공문서 관용)도 표 하나로 잡힘 — 소비자가 필터
- CSV 가 필요하면 선별 후 단건 `table-to-csv` (이 스킬에 `batch table-to-csv` 없음)

## 인계

표를 고쳐 되돌리려면 `rhwp-table-exchange`. 배치는 읽기만.


## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `07_axis_export_tables.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
