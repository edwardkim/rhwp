# #5342 실 에이전트 지식 지도 진입점 스킬 — 작업 기록

날짜: 2026-08-18
이슈: https://github.com/edwardkim/rhwp/issues/5342
브랜치: `feat/agent-knowledge-map` (`upstream/devel` 기준 격리 worktree)
범위: `.claude/skills/rhwp-knowledge-map/` ·
`scripts/tests/test_agent_knowledge_map.py` ·
`tests/cases/agent_knowledge_map_skill_contract.rs` ·
본 문서
비범위: `gym/` · 이웃 스킬 재작성 · 새 CLI · DocumentCore 편집 구현 ·
`rhwp-desk*` / `rhwp-handoff` / `rhwp-scaffold-final` / `rhwp-doc-repro` ·
`rhwp-codex` / `rhwp-agent-surface` 본문

## 무엇을

에이전트가 rhwp 참조 문서에 들어갈 때
`llms.txt` → `mydocs/manual/agent_knowledge_map.md` → 요청이 고른
canonical **하나** 순서를 닫는다. 이 스킬은 문서 진입점 라우터다.
대전 장 항해(`rhwp-codex`)와 3층 계약(`rhwp-agent-surface`)을
다시 쓰지 않는다.

지도는 요약·앵커만 담는다. 기존 행을 재서술하지 않는다.
지도와 상세가 다르면 상세를 따른다. 봉투 필드 이름은 지도 §2 에서만
가져온다.

## 왜

이슈 본문: 에이전트가 지식 지도를 읽고 나머지 canonical 으로
라우팅하려면 스킬이 닫혀야 한다. gym 금지. 새 CLI / 편집 로직
발명 금지. Codex·표면 스킬 재작성 금지.

정본은 이미 `llms.txt` 와 `agent_knowledge_map.md` 에 있다.
에이전트에게 필요한 것은 새 교본이 아니라 **첫 읽기 · 재측정 ·
§2 조회 · 예외 · 점프** 다.

DoD: additions 5000–10000 (최소 5000). PR 전 `cargo fmt --all -- --check`.

## 어떻게

1. 격리 worktree `C:/Users/swsz9/rhwp-agent-knowledge-map` 에
   `feat/agent-knowledge-map` 를 `upstream/devel` 에서 분기.
   금지 디렉터리와 이름 있는 worktree 는 쓰지 않음.
2. SKILL.md 를 첫 읽기·재측정·§2·정지·점프 인덱스로 작성.
3. `references/` 26장: 순서, 재측정, 나무, 대조, 경계, 사전,
   권위, 절 인덱스, 점프, 예외, stale, 버전, 상세 우선, 정지,
   인계, 함정, 여정, 발화, 조회, 3문, 표본, 계약, MCP, 발췌,
   결정표, 이웃.
4. `_gen_pack.py` 가 지도에서 절·필드 **이름**을 추출해
   픽스처를 만든다. 정의 문장을 복제하지 않는다.
   `fixtures/transcripts.json` 은 llms.txt·지도 문단 발췌.
   살아 있는 CLI 를 돌리지 않음.
5. `scripts/tests/test_agent_knowledge_map.py` 와
   `tests/cases/agent_knowledge_map_skill_contract.rs` 가
   발명 명령·gym·§2 이름·예외 네 갈래·발췌 계약을
   바이너리 없이 검사.
6. 신규 rust 시험은 `tests/cases/` 에 둔다 (suite-policy:
   PR base 에 없는 integration source).

## 하지 않은 것

- `rhwp-codex` / `rhwp-agent-surface` 본문 수정
- 지도 행 재서술
- 새 knowledge-map 하위명령
- DocumentCore / gym pack
- 살아 있는 봉투를 다시 실행해 지어내기
- capability 등록부 수정 (형제 PR 과 겹칠 수 있음)

## 검증

```bash
python -m unittest scripts.tests.test_agent_knowledge_map
cargo fmt --all -- --check
```

정본 지도와 `llms.txt` 는 읽기만 했다.

## 예외

- last_verified 30일 초과 → R04 (2026-08-18 기준 지도는 신선,
  `2026-08-11`)
- 바이너리 버전 불일치 → R05 (지도 `v0.8.3` vs package `0.8.4`.
  바이너리가 이긴다)
- 지도 ≠ canonical → R06
- §2 에 없는 필드 이름 → R07
