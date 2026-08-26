# 계약 테스트 지도 인덱스

이슈 #5342. 스킬 `rhwp-knowledge-map`. gym 아님. 새 CLI 없음.
이 장은 지도 행을 다시 쓰지 않는다. 앵커와 다음 점프만 적는다.

표면을 고칠 때 어느 테스트가 red 여야 하는지는 지도 §8 이다.
테스트 본문을 여기 옮기지 않는다.

| 축 | 절 |
| --- | --- |
| 봉투·계약 | §8-1 |
| 조회 | §8-2 |
| 편집 | §8-3 |
| 변환·렌더 | §8-4 |
| MCP | §8-5 |
| 보안 | §8-6 |

이 스킬 자신의 계약은 `scripts/tests/test_agent_knowledge_map.py`
와 `tests/cases/agent_knowledge_map_skill_contract.rs` 다.
바이너리를 부르지 않는다.
