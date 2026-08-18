# 10 — structure-outline

켤 때: `structure_node_count > 0`. 우선순위 70. skill `rhwp-doc-triage`.

```
rhwp export-structure <file> --json
```

confidence: 노드 ≥ 3 이면 high, 아니면 medium. 항목은 1개여도 있다.

## why

`제목·조문 구조 N개 노드 — 조문 단위 인용·RAG 청킹`

## 다음

조문 단위로 읽고 인용하는 것은 doc-triage 의 `export-structure` 장.
이 스킬은 메뉴에 올려 줄 뿐이다. 장문(≥10쪽)이면 `long-doc-digest` 도
같이 켜지고, 구조가 위(70>40)다. 긴 법령은 조문부터 보는 편이 맞다.
노드 1개는 confidence medium 이지만 항목은 남는다.
