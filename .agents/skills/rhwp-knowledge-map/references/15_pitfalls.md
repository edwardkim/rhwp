# 함정

이슈 #5342. 스킬 `rhwp-knowledge-map`. gym 아님. 새 CLI 없음.
이 장은 지도 행을 다시 쓰지 않는다. 앵커와 다음 점프만 적는다.

1. 지도를 교본처럼 통독한다 → R10. 한 절만.
2. §0 개수를 답으로 암기한다 → R02. 재측정.
3. 필드 이름을 영어 관용으로 만든다 → R07.
4. 대전에 필드 뜻을 다시 적는다 → R15.
5. 지도와 CLI 매뉴얼이 다를 때 지도를 고친다 → R06, 상세를 따른다.
6. extract-pages 쪽 번호를 search.page 와 혼동 → 지도 §3-3 앵커 후 CLI 매뉴얼.
7. isError 만 보고 identical:false 를 실패로 처리 → 지도 §4 앵커.
8. 세션 도구를 capabilities --mcp 에서 찾는다 → tools/list.
9. gym 과제로 진입점을 시험한다 → R12.
10. 지식지도 CLI 를 제안한다 → R11.

유지 규약(지도 말미): 새 표면은 행만 추가. 수치는 실행해서 갱신.
링크 검사: `py scripts/check_markdown_links.py mydocs/manual/agent_knowledge_map.md`.
