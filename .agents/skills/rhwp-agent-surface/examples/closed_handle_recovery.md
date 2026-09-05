# 레시피 — 닫힌 핸들

층: MCP 세션. 신호: `isError:true`. 픽스처:
[`../fixtures/exceptions/closed_handle.json`](../fixtures/exceptions/closed_handle.json).

## 재현 형태

```
hwp_open     → doc-1
hwp_doc_search {docId:doc-1, query:결재}
hwp_close    {docId:doc-1}
hwp_doc_search {docId:doc-1, query:결재}    # 여기
```

응답:

```json
{"isError":true,
 "content":[{"type":"text","text":
   "{\"error\":\"열려 있지 않은 핸들: doc-1 (hwp_open 먼저)\",
     \"nextCall\":{\"arguments\":{\"path\":\"<열 문서 경로>\"},
                   \"name\":\"hwp_open\",
                   \"why\":\"핸들이 없거나 만료 — hwp_open 으로 docId 를 재발급한 뒤 재시도\"}}"}]}
```

## 다음 수

1. `identical:false` 가 아니다. 런타임 오류다.
2. `nextCall.name` 이 `hwp_open`.
3. 경로를 채워 연다. **새 docId** 를 받는다.
4. 검색을 새 id 로 다시 부른다.
5. 옛 `doc-1` 을 재사용하지 않는다.

서버 프로세스가 재시작돼도 같은 모양이다. 핸들은 영속이 아니다.

## 같은 자리의 다른 층

| 층 | 예 | 재시도 |
|---|---|---|
| JSON-RPC | `tools/unknown` -32601 | 금지. 호스트 버그 |
| isError | 닫힌 핸들, 없는 파일 | 핸들만 nextCall 로 재발급 |
| 봉투 | `notFound`, `identical:false` | 오류가 아님 |
