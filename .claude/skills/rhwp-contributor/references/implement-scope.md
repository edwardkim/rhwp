# 4단 — 구현 범위

기존 결(명명·주석 밀도·모듈 경계)을 따른다. 이 스킬이 다루는 기여는
"공식 절차대로 완주"이지, 렌더러나 DocumentCore 를 새로 짜는 일이 아니다.

## 만져도 되는 것 (이 스킬 고도화 파동)

- `.claude/skills/rhwp-contributor/` (`SKILL.md`, `references/`, `examples/`, `fixtures/`)
- `mydocs/working/agent_contributor.md`
- 이 스킬의 계약 시험 (`tests/cases/agent_contributor_skill_contract.rs`,
  `scripts/tests/test_agent_contributor.py`)

다른 주제의 기여는 그 이슈가 정한 경로만 만진다.

## 만지지 않는 것

| 금지 | 이유 |
|------|------|
| `src/document_core/` 편집 로직 발명 | 기존 CLI 계약 밖 |
| `gym/` | 이 스킬은 실기여. gym 아님 |
| 다른 `.claude/skills/*/SKILL.md` | 스킬 본문 재작성 금지. 포인터만 |
| 열린 PR 이 이미 고치는 파일 | 가로채기 금지 |
| 새 `[[bin]]` / 새 rhwp 하위명령 | 새 CLI 금지 |

DocumentCore 를 **읽기**는 분석 단계에서 할 수 있다. **새 편집 연산·새
필드 쓰기 경로를 이 스킬이 설계하지 않는다.**

## 새 표면

새 CLI/MCP 표면이 정말 필요하면
[에이전트 표면 플레이북](../../../mydocs/manual/agent_surface_playbook.md)
등재 절차를 **별도 이슈**로 따른다. 이 스킬 파동에서 몰래 추가하지 않는다.

## 닫는 증거

`git diff --name-only upstream/devel` 가 이슈 범위 안이다.

예제: [07_implement_without_documentcore.md](../examples/07_implement_without_documentcore.md),
[22_no_new_cli.md](../examples/22_no_new_cli.md).
