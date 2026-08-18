# 13 — jq 로 성공/실패를 가르고 실패만 재시도

실패 행은 삭제하는 대상이 아니라 **재시도 입력**이다.

## 분리

```bash
jq -r 'select(.error) | .source' 결과.ndjson > 실패.txt
jq -c 'select(.error|not)' 결과.ndjson > 성공.ndjson
```

`select(.error)` 는 키가 있는 행만. `tableCount:0` 같은 빈 성공은 여기 안 온다.

## 부류를 가른 뒤에 재시도

```bash
# 경로 오타 — 재시도 금지. 목록을 고친다.
jq -r 'select(.error|test("os error 2")) | .source' 결과.ndjson

# 일시적 IO 만 재시도
jq -r 'select(.error|test("sharing|temporarily"; "i")) | .source' 결과.ndjson > 재시도.txt
cat 재시도.txt | rhwp batch export-text --json > 재시도.ndjson
```

부류 표는 `28_retry_classes.md`. 암호 신호는 단건 `--password` 로 빼고
batch 재시도에 넣지 않는다.

## 원본 스트림에 병합

재시도가 성공하면 원본 NDJSON 의 그 `source` 줄을 새 줄로 바꾼다.
성공 줄을 다시 돌리지 않는다.

```bash
# source 를 키로 재시도 결과를 덮어쓴다
jq -s -c '
  (.[1] | map({key:.source, value:.}) | from_entries) as $u
  | .[0][] | ($u[.source] // .)
' 결과.ndjson 재시도.ndjson > 병합.ndjson
```

병합 후에도 게이트: 목록 줄 수 = 병합 줄 수, 실패는 남은 `error` 만.

## 사용법 오류는 재시도 입력이 없다

exit 2 이면 stdout 이 비어 `실패.txt` 도 비다. 플래그를 고친다.

## 금지

- 성공 4건을 포함해 5건을 다시 돌리기
- `grep error` 로 본문에 "error" 가 있는 성공 행을 실패로 오탐
- 재시도 stdout 을 원본 뒤에 이어 붙여 줄 수를 6으로 만들기

## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `13_jq_split_retry.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
