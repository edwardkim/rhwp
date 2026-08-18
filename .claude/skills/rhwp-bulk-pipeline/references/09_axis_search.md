# 09 — batch search

## 한 줄

아카이브 전역 검색. --query 필수, 파일당 1000건 상한.

## 호출

```bash
rhwp batch search --json [옵션]
```

- 입력: stdin 한 줄 = 경로
- 단건 동형: search --json
- 플래그: `--json`, `--threads`, `--query`
- 성공 키: `schemaVersion`, `source`, `query`, `matchCount`, `matches`


## 언제

아카이브에서 "위임전결이 어느 문서 어느 쪽".

```bash
rhwp batch search --query "위임전결" --json < 목록.txt \
  | jq -c 'select(.matchCount > 0) | {source, pages:[.matches[].page]}'
```

`--query` 는 **필수**. 없으면 exit 2, stdin 미소비 (B09).
`tests/batch_axes_contract.rs` 가 단건 `search --json` 과 동형임을 고정.

## 상한

파일당 매치 1,000건. 스트림이 부푸는 것을 막는다.
대소문자는 구분한다. `Hwp` 와 `hwp` 는 다른 질의 — 두 번 친다.

## 0건

`matchCount: 0` 은 성공. 그 문서에 검색어가 없을 뿐이다.
실패 목록에 넣지 않는다.

## 페이지 주소

`matches[].page` 는 0 기준. 사람에게 보여 줄 때만 +1.
검색 히트 문서만 본문이 필요하면 그 `source` 를 모아 `export-text`.


## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `09_axis_search.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
