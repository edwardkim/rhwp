# 04 — batch info

## 한 줄

메타 스윕. 본문보다 싸다(271건 실측 3.0s vs export-text 67.4s).

## 호출

```bash
rhwp batch info --json [옵션]
```

- 입력: stdin 한 줄 = 경로
- 단건 동형: info --json
- 플래그: `--json`, `--threads`
- 성공 키: `schemaVersion`, `source`, `format`, `pageCount`


## 언제

"폴더에 뭐가 있나", "몇 쪽짜리인가", "HWP3/HWP5/HWPX 섞였나" 가 첫 질문일 때.
본문 추출보다 압도적으로 싸다. 가이드 실측: 271건 info 3.0s, export-text 67.4s.

## 절차

```bash
rhwp batch info --json < 목록.txt > meta.ndjson
jq -r 'select(.error|not) | "\(.source)\t\(.format)\t\(.pageCount)"' meta.ndjson
jq -r 'select(.pageCount >= 10) | .source' meta.ndjson > 큰문서.txt
```

레시피 9 실측 첫 행 취지:

```json
{"format":"hwp5","pageCount":35,"paraCount":630,"schemaVersion":"1.0","source":"samples/2022년 국립국어원 업무계획.hwp","title":"2022년 국립국어원 업무계획"}
```

전 행: `examples/transcripts/T01.ndjson`.

## 선별 패턴

| 목적 | jq |
| --- | --- |
| 10쪽 이상만 본문 | `select(.pageCount>=10) \| .source` |
| HWPX 만 convert | `select(.format=="hwpx") \| .source` |
| 실패만 경로 수정 | `select(.error) \| .source` |
| 총 페이지 | `jq -s '[.[]\|select(.error\|not)\|.pageCount]\|add'` |

## 정지

질문이 규모/형식이면 여기서 끝 (B04). export-text 로 내려가지 않는다.
전건 error 면 작업 디렉터리부터 (B02).

## 스키마

단건 `info --json` 과 같다. `format`, `pageCount`, `title`, `paraCount` 등.
필드 추가는 허용, 삭제·변경은 `cli_json_contract` 가 막는다.

암호 문서는 실패 봉투로 보인다. 그 경로를 빼 단건 `--password` 로 연다.
info 호출 자체에 `--password` 를 붙이면 배치 전체가 exit 2 (B03).


## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `04_axis_info.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
