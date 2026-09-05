# 예제 10 - 혼합 성공과 실패 게이트

```bash
rhwp batch export-text --json < examples/lists/mixed_inputs.txt \
  | jq -c '{source, ok, error, output}'
```

입력마다 한 줄의 NDJSON 봉투를 보존한다. 성공 레코드와 실패 레코드를 섞어도
실패 문서를 조용히 빼지 않으며, 마지막 종료 코드는 전체 실패 상태를 요약한다.
재시도 전에 `error`와 `source`를 묶어 기록하고, 성공한 문서의 산출물 이름은 다시
예약하지 않는다. 전사 `T10.ndjson`.

이슈 #5311. gym 아님. 새 CLI 아님.
