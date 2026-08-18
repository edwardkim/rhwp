# rhwp-mcp-session references

실 에이전트가 `rhwp mcp-serve` 를 붙이고 세션/무상태를 고르기 위한 레퍼런스다.
gym 트레이스·벤치와 무관하다.

| 문서 | 내용 |
|---|---|
| [session_lifecycle.md](session_lifecycle.md) | hwp_open → doc_* → close |
| [session_tools.md](session_tools.md) | 세션 도구 카드 |
| [stateless_when.md](stateless_when.md) | 무상태를 고르는 때 |
| [capabilities_ssot.md](capabilities_ssot.md) | capabilities --mcp 단일 출처 |
| [error_recovery.md](error_recovery.md) | 판정 3층과 복구 |
| [pairing.md](pairing.md) | 세션↔무상태 짝 |
| [host_attach.md](host_attach.md) | 호스트 부착 |
| [decision_tree.md](decision_tree.md) | 판단 트리 |
| [fixtures/](fixtures/) | 기계 검증용 JSON |

픽스처는 `scripts/tests/test_agent_mcp_session.py` 가 소스 allowlist 와 대조한다.
