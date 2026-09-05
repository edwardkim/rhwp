# 08 — batch fields

## 한 줄

서식 템플릿 일괄 조사. fieldCount 0 은 오류가 아님.

## 호출

```bash
rhwp batch fields --json [옵션]
```

- 입력: stdin 한 줄 = 경로
- 단건 동형: fields --json
- 플래그: `--json`, `--threads`
- 성공 키: `schemaVersion`, `source`, `fieldCount`, `fields`


## 언제

폴더에 서식 후보가 여러 개. 누름틀 있는 파일만 고를 때.

```bash
rhwp batch fields --json < 서식목록.txt \
  | jq -c 'select(.fieldCount>0) | {source, fieldCount}'
```

`fieldCount: 0` 은 오류가 아니다 (B08). 그 파일은 누름틀 서식이 아니다.
표 칸 서식이면 `rhwp-table-exchange` 로 인계.

## 스키마

단건 `fields --json` 과 같다. `fields[].name` / `guide` / `memo`.
머리말·각주 안 필드를 다 잡는다고 단정하지 않는다 (form-fill 스킬의
사각지대 문서와 같다). 이 스킬은 조회만 하고 채우지 않는다.

## 다음에 채울 때

한 서식을 고르면 `rhwp-form-fill` 또는 이 스킬의 `batch fill`(명단 N행).
폴더의 서식 N개에 명단 1개를 한 번에 붓는 명령은 없다.


## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `08_axis_fields.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
