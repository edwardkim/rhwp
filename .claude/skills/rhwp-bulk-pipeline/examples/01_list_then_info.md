# 예제 01 — 목록 후 info 선점검

목표가 "폴더에 뭐가 있나" 이면 본문을 뽑지 않는다.

```bash
find samples/ \( -name '*.hwp' -o -name '*.hwpx' \) | head -20 > /tmp/bulk-list.txt
rhwp batch info --json < /tmp/bulk-list.txt > /tmp/meta.ndjson
jq -r 'select(.error|not) | "\(.format)\t\(.pageCount)\t\(.source)"' /tmp/meta.ndjson
```

레시피 9 재현 목록은 `lists/recipe9.txt`. 전사 `transcripts/T01.ndjson`.
게이트: 5 = 4 + 1, exit 1 이 정상.

PowerShell 은 `12_windows_listing.md`.

이슈 #5311. gym 아님. 새 CLI 아님.
