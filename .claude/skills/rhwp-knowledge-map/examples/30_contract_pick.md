# 계약 테스트 고르기

이슈 #5342. 스킬 `rhwp-knowledge-map`. gym 아님. 새 CLI 없음.
이 장은 지도 행을 다시 쓰지 않는다. 앵커와 다음 점프만 적는다.

요청: search 봉투를 바꾸면 어느 테스트가 red.
지도 §8-2 앵커. 테스트 파일을 열어 확인.
정지 R01.

## 첫 읽기 점검

1. `llms.txt` 가 지식 지도를 가리키는지 확인한다.
2. 지도 frontmatter `last_verified` 를 오늘과 비교한다 (30일).
3. 지도 §0 바이너리 표기와 `rhwp capabilities` 의 version 을 비교한다.
4. 요청에 필요한 절 하나만 연다. §2 전수 사전을 통독하지 않는다.
5. 권위 열의 canonical **하나**를 연다. 지도와 다르면 그쪽.
6. 실무 작업이면 이웃 스킬로 점프하고 이 스킬을 닫는다.

## 하지 말 것

- 지도 표를 이 예제에 다시 적기
- `schema_version` / `page_count` 같은 철자 변형
- `rhwp knowledge-map` 발명
- gym pack 으로 이 경로를 재현
- `rhwp-codex` 또는 `rhwp-agent-surface` 본문 수정

## 재측정 (숫자가 필요하면)

```
rhwp capabilities
rhwp capabilities --mcp
rhwp mcp-serve   # initialize → tools/list
```

정본: `llms.txt`, `mydocs/manual/agent_knowledge_map.md`.
지도 행 재서술 금지. 필드 이름 발명 금지. gym 아님.
