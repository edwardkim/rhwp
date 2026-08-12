# rhwp — Gemini CLI 작업 지침

이 파일은 얇은 포인터다. 실질 규약의 단일 정본은 [AGENTS.md](AGENTS.md)이고, 그
문서 로딩 순서를 그대로 따른다. 에이전트 첫 문서는
[에이전트 지식 지도](mydocs/manual/agent_knowledge_map.md), 명령 자기서술은
`rhwp capabilities`, LLM 인덱스는 [llms.txt](llms.txt)다.

- 판정은 `--json` 봉투와 종료 코드(#2707)로 한다 — 화면 문자열 파싱이 아니라.
- 문서 편집 작업은 `rhwp replay --plan-json <계획> --capsule <영수증>` 으로 캡슐
  증빙을 남기는 것을 권장한다(AGENTS.md "작업 증빙" 절).
- PR 은 base `devel` · [PR 템플릿](.github/pull_request_template.md) 체크리스트를 채운다.
