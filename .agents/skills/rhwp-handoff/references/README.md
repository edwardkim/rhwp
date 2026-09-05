# rhwp-handoff 레퍼런스 목록

이 폴더는 세션 간 인수인계 운영 계약의 정본이다. SKILL.md 가 요약이고,
아래 파일이 필드·예외·금지의 상세다. 새 CLI 를 설명하지 않는다.

| 파일 | 닫는 질문 |
|---|---|
| [when-to-handoff.md](when-to-handoff.md) | 컨텍스트 예산·세션 중단·시트 리필 중 언제 넘기는가 |
| [orchestrator-protocol.md](orchestrator-protocol.md) | `tools/handoff/orchestrator.py` 의 task/result/종료 코드 |
| [artifacts.md](artifacts.md) | outgoing 이 닫는 폴더 레이아웃 |
| [result-json.md](result-json.md) | last `result.json` 을 어떻게 읽는가 |
| [journal-chain.md](journal-chain.md) | NDJSON 지문 체인과 `--verify-journal` |
| [incoming-agent.md](incoming-agent.md) | incoming 이 읽는 세 파일 |
| [capsule-parent-chain.md](capsule-parent-chain.md) | `--parent` 로 세션을 잇는 방법 (영수증 재작성 아님) |
| [work-receipt-boundary.md](work-receipt-boundary.md) | 단건 증명 vs 세션 인계 |
| [working-doc-handoff.md](working-doc-handoff.md) | working doc 최소 칸 |
| [isolation-worktree.md](isolation-worktree.md) | 이름 붙은 워킹트리 금지 |
| [staging-named-files.md](staging-named-files.md) | `git add -A` 금지, 이름 있는 파일만 |
| [no-documentcore.md](no-documentcore.md) | DocumentCore 편집 로직 발명 금지 |
| [exception-index.md](exception-index.md) | 예외 네 갈래 색인 |
| [exception-missing-capsule.md](exception-missing-capsule.md) | 캡슐 부재 |
| [exception-parent-hash.md](exception-parent-hash.md) | 부모 해시 불일치 |
| [exception-dirty-worktree.md](exception-dirty-worktree.md) | dirty named worktree |
| [exception-disk-full.md](exception-disk-full.md) | 디스크 가득 |
| [exit-codes.md](exit-codes.md) | 0/1/2/3/4 와 replay 3/1/2 |
| [pitfalls.md](pitfalls.md) | 운영 함정 |
| [decision-tree.md](decision-tree.md) | 요청 → 절차 |
| [recipe-index.md](recipe-index.md) | examples/ 교차표 |
| [envelope-field-catalog.md](envelope-field-catalog.md) | 봉투 필드 사전 |

픽스처 카탈로그: [`../fixtures/catalog.json`](../fixtures/catalog.json).
워크스루: [`../examples/README.md`](../examples/README.md).
작업 기록: [`../../../../mydocs/working/agent_handoff.md`](../../../../mydocs/working/archives/agent_handoff.md).
