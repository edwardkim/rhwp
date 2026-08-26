# rhwp — Copilot 작업 지침

rhwp 는 Rust+WASM 한글(HWP/HWPX/HWP3/HML) 문서 엔진이고, CLI `--json` 봉투와 MCP 서버를
갖춰 **AI 에이전트가 1급 소비자**다. 이 파일은 얇은 포인터다 — 실질 규약의 단일 정본은
[AGENTS.md](../AGENTS.md)이며, 거기의 문서 로딩 순서를 그대로 따른다.

핵심만 요약하면:

1. 첫 문서는 [에이전트 지식 지도](../mydocs/manual/agent_knowledge_map.md) — 작업별 명령
   결정 표·봉투 필드 사전·실패 사전이 한 곳에 있다. 명령 자기서술은 `rhwp capabilities`.
2. 판정은 화면 출력이 아니라 **`--json` 봉투와 종료 코드**(0 성공/1 런타임/2 사용법/3 판정
   데이터, #2707)로 한다.
3. 문서 편집 작업은 영수증과 함께 남기는 것을 권장한다 —
   `rhwp replay --plan-json <계획> --capsule work.capsule.json` (연속 작업은 `--parent`
   로 계보 연결, 폴더 재검증은 `rhwp audit`). 상세: AGENTS.md 의 "작업 증빙" 절.
4. PR 은 base `devel`, 브랜치는 최신 `upstream/devel` 기준. PR 본문은
   [템플릿](pull_request_template.md)의 체크리스트를 채운다.
5. 렌더링·레이아웃 변경은 시각 검증 근거(PDF/SVG 비교)를 남긴다.
