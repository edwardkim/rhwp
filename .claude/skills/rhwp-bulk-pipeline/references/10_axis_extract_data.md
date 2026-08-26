# 10 — batch extract-data

## 한 줄

날짜·금액·수량. --limit 은 문서마다.

## 호출

```bash
rhwp batch extract-data --json [옵션]
```

- 입력: stdin 한 줄 = 경로
- 단건 동형: extract-data --json
- 플래그: `--json`, `--threads`, `--kind`, `--limit`
- 성공 키: `schemaVersion`, `source`, `kind`, `itemCount`, `totalItemCount`, `truncated`, `counts`, `items`


## 언제

폴더에서 날짜·금액·수량을 **주소와 함께** 수확. 평문 정규식은 주소를 잃는다.

```bash
rhwp batch extract-data --json --limit 3 < 목록.txt
```

레시피 9 실측 `counts` (limit 3 이어도 counts 는 절단 전):

| source | amount | date | number | total |
| --- | --- | --- | --- | --- |
| 2022 국립국어원 업무계획 | 65 | 29 | 203 | 297 |
| 2024-05 수출입 현황 | 0 | 22 | 124 | 146 |
| field-01 | 0 | 0 | 0 | 0 |
| hwp3-sample | 0 | 0 | 11 | 11 |

첫 행은 297건 중 3건만 `items` 에 실리고 `truncated:true`.

## `--limit` 는 문서마다

배치 전체 상한이 아니다. 앞 문서가 한도를 다 써 뒤 문서가 0건이 되면
소비자는 "값이 없다"와 "한도를 썼다"를 구별하지 못한다.
`tests/batch_extract_data_contract.rs` 가 같은 문서를 stdin 에 두 번 넣어
두 레코드가 독립 절단인지 고정한다.

## `--kind`

`date` | `amount` | `number` | `all`(기본).
단건과 같다. 정규화 규약(`normalized` null, 부분 날짜 `2026-01`)도 단건 명문.

## 표면 목록

`cli_commands.md` §batch 헤더 줄에 extract-data 가 빠져 있을 수 있다.
존재 근거는 `rhwp capabilities` 의 batch.subcommands 와 레시피 9,
`src/main.rs` 디스패치, `batch_extract_data_contract.rs`.
없는 명령으로 취급해 새 명령을 발명하지 않는다.

## 0건

`itemCount: 0` 은 성공 (field-01 실측).


## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `10_axis_extract_data.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
