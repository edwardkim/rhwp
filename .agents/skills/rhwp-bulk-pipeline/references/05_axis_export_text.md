# 05 — batch export-text

## 한 줄

본문 일괄. 페이지 단위가 필요하면 선별 후 단건.

## 호출

```bash
rhwp batch export-text --json [옵션]
```

- 입력: stdin 한 줄 = 경로
- 단건 동형: export-text --json 의 문서 단위 축약(pages[] 대신 text)
- 플래그: `--json`, `--threads`
- 성공 키: `schemaVersion`, `source`, `pageCount`, `text`


## 언제

코퍼스 본문, RAG 원문, "폴더 전체를 텍스트로".
페이지 단위 청킹이 필요하면 배치 `text`(문서 전체) 로 1차 수확한 뒤
필요한 문서만 단건 `export-text --json` 의 `pages[]` 를 쓴다.
가이드 시나리오 2 의 실측 조합이다.

## 호출

```bash
rhwp batch export-text --json --threads 4 < 목록.txt > 결과.ndjson
```

성공 레코드: `{"schemaVersion","source","pageCount","text"}`.
단건의 `pages[]` 가 아니라 **문서 단위 `text`** 다. 혼동하지 말 것.

레시피 9 실측: 성공 4 + 실패 1 (`samples/없는파일.hwp`), exit 1.
전사는 `examples/transcripts/T02.ndjson`.

## 재시도

```bash
jq -r 'select(.error) | .source' 결과.ndjson > 재시도.txt
# 부류를 가른 뒤에만
cat 재시도.txt | rhwp batch export-text --json > 재시도.ndjson
```

`os error 2` 는 경로를 고친다. 같은 목록을 다시 넣지 않는다.

## 게이트

입력 5 = 성공 4 + 실패 1. 공식은 `14_gate_n_equals.md`.

## `--max-chars` 는 단건

단건 `export-text --json --max-chars N` 은 컨텍스트 상한이다.
배치 축에 같은 플래그를 발명해 붙이지 않는다. 거대 문서는 info 로
걸러 단건으로 내린다.

## 성능

271건 67.4s (가이드, 32코어 Windows release). 선별 75건은 6.2s.
먼저 info, 그다음 필요한 것만 본문.


## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `05_axis_export_text.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
