# rhwp-q-kit — 조회 CLI 50개

선점 목록: `C:/Users/swsz9/.rhwp-cli-registry.json` 의 `claimed_pack50`.
이미 있는 `rhwp-agent` 80명령, `rhwp-q-*` 15명령, 진행 중 단건 PR 4개(para-shape, object-cycle, page-def, section-starts)와 이름이 겹치지 않는다.

이 바이너리는 DocumentCore 공개 조회만 부른다. 편집 API·더미 JSON 코퍼스·src `#[cfg(test)]` 는 없다. 회귀는 `tests/cases/agent_q_kit_contract.rs`.
