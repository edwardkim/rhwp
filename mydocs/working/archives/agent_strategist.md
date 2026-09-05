# #5335 실 에이전트 전략가(근거 대장) 스킬 — 작업 기록

날짜: 2026-08-18
이슈: https://github.com/edwardkim/rhwp/issues/5335
브랜치: `feat/agent-strategist` (`upstream/devel` 기준 격리 worktree)
범위: `.agents/skills/rhwp-strategist/` ·
`tests/cases/agent_strategist_skill_contract.rs` ·
`scripts/tests/test_agent_strategist.py` ·
capability 등록부 `CAP-4903` 스킬 진입점 ·
`.agents/agents/rhwp-strategist.md` 스킬 링크 · 본 문서
비범위: `gym/` · FDE/bug-hunter/chief 스킬 재작성 · 새 CLI ·
DocumentCore · 편집 로직 · 시장 전망 생성

## 무엇을

에이전트가 고객 목표(정부과제 수주, 분기 전략 보고서)를 문서 코퍼스에서
**근거 좌표로 받친 산출물**로 닫으려면, 이미 있는
`tools/strategist/engagement.py` 운영 계약이 스킬로 조립되어야 한다.

엔진은 전략을 만들지 않는다. 스킬이 보장하도록 고정하는 것은 세 가지다.

1. 전수 지도 — 실패한 문서는 `status: failed` 로 남긴다. 조용히 빼지 않는다.
2. 근거 좌표 — `search`/`extract-data` 봉투의 section/paragraph/page/offset 를
   있는 키만 옮긴다. 없는 `page` 를 발명하지 않는다.
3. §5 게이트 — 근거 대장 밖 주장은 `--validate` 가 exit 3 으로 거부한다.

출처 없는 시장 전망·예측은 비범위다.

## 왜

이슈 본문: 실 에이전트 경로. gym 금지. FDE(현장 증상)·Chief(요청 큐)와
층이 다르다. 새 CLI / 편집 로직 발명 금지.

코어는 이미 있다.

- 프로토콜: `engagement.json` `{objective, corpus, questions}`
- 엔진: `tools/strategist/engagement.py` (CAP-4903)
- 정본: `mydocs/manual/strategist_playbook.md`
- 에이전트: `.agents/agents/rhwp-strategist.md`

에이전트가 필요한 것은 새 구현이 아니라 **언제 엔진을 치고, 어느 봉투
키만 옮기며, 어느 정지 규칙으로 멈추는가** 이다. page 를 추정하거나
실패 문서를 지우거나 게이트를 건너뛰면 좌표 재현 계약이 깨진다.

DoD: additions 5000–10000 (최소 5000). PR 전 `cargo fmt --all -- --check`.

## 어떻게

1. 격리 worktree `C:/Users/swsz9/rhwp-agent-strategist` 에
   `feat/agent-strategist` 를 `upstream/devel` 에서 분기.
   `rhwp-desk*` · `rhwp-handoff` · `rhwp-scaffold-final` · `rhwp-doc-repro`
   는 쓰지 않음. 기존 named worktree 를 훔치지 않음.
2. `.agents/skills/rhwp-strategist/SKILL.md` 를 라우터로 신설.
3. `references/` 21장: 권위, engagement 프로토콜, 코퍼스 지도, 근거 대장,
   §5 게이트, 좌표 규칙, search/extract-data 봉투, 종료 코드, 비범위,
   FDE·Chief 층, SWS, 함정, 판단 트리, 레시피 색인, 필드 카탈로그,
   여정, 정지, 인계, 실패 문서, 질문 설계.
4. `examples/` 24개 워크스루. `fixtures/` 에 engagement·ledger·봉투·
   게이트 판정·코퍼스 지도·트레이스.
5. `scripts/tests/test_agent_strategist.py` 가 발명 명령·gym·이웃 스킬
   재작성·page 발명·실패 문서 탈락을 바이너리 없이 검사.
6. `tests/cases/agent_strategist_skill_contract.rs` 가 같은 가드.
7. 에이전트 파일과 capability 등록부 `CAP-4903` 에 스킬 진입점을 연결.

## 하지 않은 것

- `engagement.py` / `sws_audit.py` 동작 변경
- 새 `strategy` / `forecast` / `claim-check` CLI
- gym pack / 과제 / 채점기
- FDE·chief·bug-hunter·다른 스킬 본문 수정
- DocumentCore 편집 구현
- 출처 없는 시장 전망 생성기

## 검증

```bash
python -m unittest scripts.tests.test_agent_strategist
cargo fmt --all -- --check
```

기존 계약과 엔진 본문은 건드리지 않았다. 이 PR 은 그 표면을 스킬이
가리키는지만 고정한다.
