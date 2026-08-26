# rhwp — Replit Agent 작업 지침 (얇은 포인터)

실질 규약의 단일 정본은 AGENTS.md — 그 문서 로딩 순서를 그대로 따른다.
에이전트 첫 문서는 mydocs/manual/agent_knowledge_map.md, 명령 자기서술은 `rhwp capabilities`.

- 판정은 `--json` 봉투와 종료 코드(0/1/2/3, #2707)로 한다.
- 문서 편집 작업은 `rhwp replay --plan-json <계획> --capsule <영수증>` 캡슐 증빙 권장
  (연속 작업은 `--parent` 계보, 폴더 재검증은 `rhwp audit` — AGENTS.md "작업 증빙" 절).
- PR 은 base devel · 브랜치는 최신 upstream/devel 기준 · PR 템플릿 체크리스트를 채운다.
