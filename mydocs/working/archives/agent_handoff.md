---
kind: working
status: active
issue: 5339
handoffTrigger: context_budget
taskId: t-ord
---

# 에이전트 세션 핸드오프 오케스트레이터 스킬 신설 (#5339)

작업 브랜치: `feat/agent-handoff`
정본 스킬: `.agents/skills/rhwp-handoff/`
이슈: [agent: 세션 핸드오프 오케스트레이터 스킬 신설](https://github.com/edwardkim/rhwp/issues/5339)

## 1. 한 줄

실사용 에이전트가 긴 작업을 세션 사이에 넘기도록, 이미 devel 에 있는
`tools/handoff/orchestrator.py` 와 `replay --capsule` / `--parent` 를
운영 계약으로 닫는다. work-receipt 는 단건 증명(`replay` / `audit` / `lineage`). 이 스킬은 세션 인수인계.
새 CLI 없음. gym 없음.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- `.agents/skills/rhwp-handoff/` SKILL.md + references/ + examples/ + fixtures
- `mydocs/working/agent_handoff.md` (이 파일)
- 계약 시험 (파이썬 + 픽스처 기반 Rust)
- 언제 넘기는가: 컨텍스트 예산, 세션 중단, 시트 리필
- incoming 은 last result.json / capsule / working doc
- `--parent` 로 체인
- 예외: 캡슐 부재, 부모 해시 불일치, dirty named worktree, disk full
- 금지: DocumentCore 발명, `git add -A`, named worktree checkout
- additions 5000–10000, 최소 5000 (목표 7420)
- PR 전 `cargo fmt --all -- --check`
- isolation worktree, 브랜치 `feat/agent-handoff` from `upstream/devel`
- 한국어 PR, base `devel`, `closes #5339`, `--body-file`

금지:

- gym/
- 다른 스킬 재작성 (특히 rhwp-work-receipt)
- 열린 PR 파일
- DocumentCore 편집 구현
- 새 rhwp CLI 명령

## 3. 왜 스킬로 닫나

오케스트레이터와 replay 는 이미 있다. 구멍은 운영이다 — 에이전트가
컨텍스트가 바닥인데도 대화를 인계로 착각하거나, 이름 붙은 트리를 비우거나,
영수증 스킬을 다시 쓴다. 이 스킬은 그 경로를 같은 단어로 고정한다.

## 4. 범위

만진 것:

| 경로 | 역할 |
|------|------|
| `.agents/skills/rhwp-handoff/SKILL.md` | 신설. 요약 규약 |
| `references/` | 트리거·프로토콜·예외·금지 |
| `examples/` | 워크스루 28 |
| `fixtures/` | 캡슐 체인·봉투·저널·시나리오 |
| `scripts/tests/test_agent_handoff_skill.py` | 파일 계약 (기존 test_agent_handoff.py 는 그대로) |
| `tests/cases/agent_handoff_skill_contract.rs` | 같은 계약의 Rust 면 |
| `mydocs/working/agent_handoff.md` | 이 기록 |

만지지 않은 것:

- `src/` CLI, DocumentCore
- `gym/` 전부
- `.agents/skills/rhwp-work-receipt/` 본문
- `.agents/skills/rhwp-onboarding/` 외 형제 스킬
- `scripts/tests/test_agent_handoff.py` (오케스트레이터 기존 시험)

## 5. 인계 머리 (이 작업 자체)

- result: (이 PR 은 문서·픽스처. 오케스트레이터 실구동 산출은 fixtures/results/)
- capsule: `.agents/skills/rhwp-handoff/fixtures/capsules/s24.capsule.json` (체인 머리 표본)
- parent: `s23.capsule.json` (상대)
- journal: `fixtures/journals/ok.ndjson`

## 6. 남은 목표

후임이 이 스킬을 열어 세션을 넘길 때 세 파일만 읽고, 네 예외에서 멈추며,
work-receipt 를 재작성하지 않게 하는 것. 구현 코드는 이미 있다.

## 7. 다음 명령

`python -m unittest scripts.tests.test_agent_handoff_skill`

## 8. 하지 말 것

- DocumentCore 편집 로직 발명 금지
- git add -A 금지
- 이름 붙은 워킹트리 checkout 금지
- 새 CLI / gym / work-receipt 재작성

## 9. 검증

- 계약 시험이 catalog·캡슐 lineageOk·예외 exit·금지 경로를 파일만으로 고정
- `cargo fmt --all -- --check` (crates/ 존재, Unix newline)
- 기존 `scripts/tests/test_agent_handoff.py` 는 이 PR 이 수정하지 않음
