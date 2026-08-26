# 06 — batch export-structure

## 한 줄

개요/조문 일괄. --mode auto|outline|clause.

## 호출

```bash
rhwp batch export-structure --json [옵션]
```

- 입력: stdin 한 줄 = 경로
- 단건 동형: export-structure --json
- 플래그: `--json`, `--threads`, `--mode`
- 성공 키: `schemaVersion`, `source`, `mode`


## 언제

편람·법령·보고서의 개요/조문만 일괄. 본문 전체보다 작고 목차 검색에 맞다.

## `--mode`

| 값 | 의미 |
| --- | --- |
| `auto` | 기본. 문서에 맞춰 고른다 |
| `outline` | 개요 |
| `clause` | 조문 |

오타는 exit 2 (B06). `chapters`, `heading`, `toc` 는 없다.

```bash
rhwp batch export-structure --json --mode outline < 목록.txt > 구조.ndjson
```

성공 레코드는 단건 `export-structure --json` 과 같다.

## 정지

질문이 "목차만" 이면 여기서 끝. 본문 축으로 내려가지 않는다.
구조가 비어 있는 짧은 문서는 실패가 아니다. `error` 키가 있는지만 본다.

## 단건으로 내릴 때

배치로 후보를 고른 뒤 한 문서의 조문을 자세히 보려면
`rhwp export-structure 파일.hwp --json --mode clause`.
배치와 같은 스키마라 소비 코드를 재사용한다.


## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `06_axis_export_structure.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
