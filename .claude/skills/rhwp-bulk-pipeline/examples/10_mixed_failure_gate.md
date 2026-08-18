# 예제 10 — 혼합 실패 게이트

레시피 9 숫자: 입력 5, 성공 4, 실패 1, exit 1.

```bash
입력=$(wc -l < examples/lists/recipe9.txt)
성공=$(jq -c 'select(.error|not)' examples/transcripts/T02.ndjson | wc -l)
실패=$(jq -c 'select(.error)' examples/transcripts/T02.ndjson | wc -l)
echo "입력 $입력 = 성공 $성공 + 실패 $실패"
test "$입력" -eq $((성공 + 실패))
```

`head -1` 으로 T17 처럼 자르면 5 ≠ 1 + 0. 그때는 결과 파일이 아니라
파이프를 고친다.

이슈 #5311. gym 아님. 새 CLI 아님.
