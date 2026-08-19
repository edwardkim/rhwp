# skill_router

사용자 요청을 `request → intent → requiredCapabilities → skillSelection → executionGraph`
한 장의 JSON 봉투로 만든다. 의존성 0 (Python 3 표준 라이브러리).

```bash
python tools/skill_router/route.py "이 서식 채워줘" --json
python tools/skill_router/route.py "PR 올려" --json
```

봉투 키는 고정이다: `schemaVersion`, `request`, `intent`, `requiredCapabilities`,
`skillSelection`, `executionGraph`, `untrustedContent`, `untrustedFields`.

`executionGraph` 는 `{nodes, edges}` 다. 노드는 `id, skill, action, command`.
가장자리가 `from → to`.
