# 12 — batch fill

## 한 줄

서식 1 + 데이터 N. stdin 목록이 아니다.

## 호출

```bash
rhwp batch fill --json [옵션]
```

- 입력: stdin 목록 아님. `--form` + `--data`
- 단건 동형: edit fill-fields --json + row
- 플래그: `--json`, `--threads`, `--form`, `--data`, `--out-dir`, `--name-field`, `--verify`, `--dry-run`
- 성공 키: `schemaVersion`, `source`, `row`, `dryRun`, `filledCount`, `filled`, `notFound`, `ambiguous`


## 언제

서식 1개 + 명단 N행 → 산출 N개. 진짜 메일머지.
`edit fill-fields` 는 1→1 이라 N명분을 만들려면 N번 호출해야 한다.

```bash
rhwp fields 신청서.hwp --json | jq -r '.fields[].name'
rhwp batch fill --form 신청서.hwp --data 신청자목록.csv \
  --out-dir output/filled --name-field 성명 --json > filled.ndjson
```

## 입력이 다르다

stdin 파일 목록이 **아니다**. `--form` + `--data` + `--out-dir`.
`--data` 는 `.jsonl` 또는 `.csv`(UTF-8). 빈 헤더만 있으면 exit 2.
`--dry-run` 에도 `--out-dir` 는 필수.

상세 금지 사항은 `17_fill_not_stdin.md`.
채움 규칙·순번·sanitize 는 `rhwp-form-fill` 스킬이 정본이다.
이 장은 배치 입력 축과 게이트만 닫는다.

## 레코드

단건 `edit fill-fields --json` + `row`(0 기준).
실패 행도 스트림에 남는다. `notFound` / `ambiguous` 는 성공 레코드의
필드이지 `error` 키가 아니다. 게이트는 그 배열 길이로 가른다
(`27_gate_recipes.md` Q10).

## 파일명

`--name-field` 생략 시 1 기준 순번, 최소 4자리.
금지 문자는 `_`, 겹치면 `_2`. 경로는 쓰기 전에 전부 정해 병렬에서도
실행 순서에 좌우되지 않는다.

## 서식 손상

못 여는 서식은 시작 전 한 번만 판정. N번 반복 보고하지 않는다.


## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `12_axis_fill.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
