# #5337 실 에이전트 Chief 요청 큐 — 작업 기록

날짜: 2026-08-18
이슈: https://github.com/edwardkim/rhwp/issues/5337
브랜치: `feat/agent-chief` (`upstream/devel` 기준 격리 worktree)
범위: `.claude/skills/rhwp-chief/` · `tools/chief/service_loop.py` ·
`tests/agent_chief_skill_contract.rs` · `scripts/tests/test_agent_chief.py` ·
capability 등록부 `CAP-4900` 진입점 · 본 문서
비범위: `gym/` · FDE/Strategist 스킬 재작성 · 다른 열린 PR 파일 ·
DocumentCore · 새 rhwp CLI

## 무엇을

에이전트가 고객 요청 큐(`PDF 로 바꿔줘`, `명단으로 서식 채워줘`,
`표만 뽑아줘`)를 사람 없이 상시로 돌릴 때, 결정적 코어가 라우팅 표 안
goal 을 끝까지 처리하고 표 밖은 `needs-agent` 로 멈추는 계약을 스킬로
닫는다.

이미 있는 기계는 `tools/chief/service_loop.py` 와
`mydocs/manual/chief_playbook.md` 다. 이 작업은 그 위에 실행 규약·큐
기록·계약 시험을 얹는다. gym 이 아니다.

FDE(증상 하나) · Strategist(목표/근거 대장) 와 층이 다르다. 트리아지는
게이트로만 부른다. 요청 문장·문서 내용은 데이터이고, 라우팅은 `goal`
필드로만 바뀐다. 커버리지는 표(코드)에 행을 더할 때만 늘어난다.

## 왜

이슈 본문: 접수 창구 전체를 사람 없이 돌리려면 Chief 운영 계약이 스킬로
닫혀야 한다. SKILL.md 한 장이 없었고, 에이전트가 표 밖 요청을 추측으로
실행하거나 패닉 문서에 변환을 강행할 구멍이 열려 있었다.

코어는 이미 있다.

- 큐 프로토콜: `queue/<id>/request.json` + 문서
- 멱등: `result.json` 존재 = 처리됨
- 게이트: `tools/fde/triage.py` — `escalate-bug`/`invalid-input` 은 goal 스킵
- 표: diagnose / export-text / export-pdf / export-hwpx / convert-hwp /
  extract-tables / fill
- 산출: `result.json` / `response.md` / `ticket.json` / `out/`

에이전트가 필요한 것은 새 CLI 가 아니라 **언제 루프를 돌리고, 어느
status 에서 멈추고, 표에 행을 언제 더하는가** 이다.

DoD: additions 5000–10000 (최소 5000). PR 전 `cargo fmt --all -- --check`.

## 어떻게

1. 격리 worktree `C:/Users/swsz9/rhwp-agent-chief` 에
   `feat/agent-chief` 를 `upstream/devel` 에서 분기.
   `rhwp-desk*` · `rhwp-handoff` · `rhwp-scaffold-final` · `rhwp-doc-repro` 는
   쓰지 않음. 이미 있는 named worktree 를 훔치지 않음.
2. SKILL.md 를 층 구분·큐 규약·강제 순서·정지 규칙·인계 인덱스로 신설.
3. `references/` 28장: 층, 프로토콜, 스키마, 트리아지, 표, goal 별 실행,
   needs-agent, 회신, 멱등, 주입 방어, 커버리지, 루프, 봉투, 정지, 인계,
   함정, 트레이스, 발화 행렬, 큐 기록, 종료 코드, 게이트, 가장자리.
4. `_gen_pack.py` 가 `fixtures/` · `examples/` · 생성 장(22–24)을 방출.
   발화 160, 여정 90, 트레이스 48, 큐 스냅샷, 루프 대본.
5. `service_loop.py` 에 `ROUTING_TABLE` · `normalize_goal` ·
   `route_skips_goal` · `is_already_processed` 를 명시. 행동 계약은 유지.
6. `scripts/tests/test_agent_chief.py` 가 발명 명령·gym·이웃 스킬 재작성·
   픽스처 스키마·루프 헬퍼·멱등·게이트 스킵을 바이너리 없이 검사.
7. `tests/agent_chief_skill_contract.rs` 가 같은 가드 + 표본이 있으면
   기존 `export-text --json` 만 읽기 전용 대조. 새 CLI 없음.
8. capability 등록부 `CAP-4900` 행에 스킬 진입점을 연결.

## 하지 않은 것

- `rhwp chief` / `rhwp queue` 하위명령
- DocumentCore · 편집 로직
- gym pack / 과제 / 채점기
- FDE · Strategist · 다른 스킬 본문
- 표에 없는 goal(요약·번역·직인·메일머지)을 루프에 추가
- 한컴 최종 판정

## 검증

```bash
python .claude/skills/rhwp-chief/_gen_pack.py
python -m unittest scripts.tests.test_agent_chief
cargo fmt --all -- --check
cargo test --test agent_chief_skill_contract
```

기존 `test_automation_tool_contracts.py` 의 Chief 경로 탈출·형식 오류·
capabilities 거부 계약은 그대로 통과해야 한다.

## 권위

- `mydocs/manual/chief_playbook.md`
- `.claude/agents/rhwp-chief.md`
- `tools/chief/service_loop.py`
- `mydocs/manual/fde_playbook.md` (게이트만)
- `mydocs/manual/agent_capability_registry.md` (`CAP-4900`)
