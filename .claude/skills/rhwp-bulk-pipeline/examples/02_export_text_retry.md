# 예제 02 — 본문 추출과 실패 재시도

```bash
rhwp batch export-text --json --threads 4 < examples/lists/recipe9.txt > /tmp/text.ndjson
jq -r 'select(.error) | .source' /tmp/text.ndjson
# samples/없는파일.hwp  — R-PATH. 재시도하지 말고 목록에서 뺀다.
```

전사는 `transcripts/T02.ndjson`. `--threads 8` 순서 보존은 `T16.ndjson`
(같은 5줄 순서).

성공 행의 `text` 를 다시 돌리지 않는다. 경로를 고친 실패만
`lists/missing_only.txt` 가 아니라 **고친 경로** 로 한 줄짜리 목록을 만든다.

이슈 #5311. gym 아님. 새 CLI 아님.
