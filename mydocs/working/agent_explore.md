# #5313 실 에이전트 explore 메뉴 라우팅 — 작업 기록

날짜: 2026-08-18
이슈: https://github.com/edwardkim/rhwp/issues/5313
브랜치: `feat/agent-explore` (`upstream/devel` 기준 격리 worktree)
범위: `.claude/skills/rhwp-explore/` ·
`scripts/tests/test_agent_explore.py` · capability 등록부 `CAP-5313` · 본 문서
비범위: `gym/` · 이웃 스킬 재작성 · 새 CLI · DocumentCore 편집 구현 ·
`rhwp-desk*` / `rhwp-handoff` / `rhwp-scaffold-final` / `rhwp-doc-repro`

## 무엇을

에이전트가 처음 보는 HWP/HWPX 앞에서 70개 명령 중 다음 수를 추측하지
않게 한다. `rhwp explore <파일> --json` 이 적용 가능한 행동만 골라
순위 매긴 메뉴로 주고, 각 항목의 `command` · `skill` · `why` ·
`confidence` 까지 라우팅한다.

기존 `rhwp-explore` 스킬은 SKILL.md 한 장에 요약만 있었다. 이 작업은
그 본문을 30초 판단 내비게이터로 재작성하고 `references/` ·
`examples/` · `fixtures/` 를 나누며, 기계 가독 봉투 표본과 가드
테스트로 계약을 고정한다.

## 왜

이슈 본문: 실 에이전트가 미지 문서에서 다음 명령을 고르는 경로.
gym 금지. 새 CLI / 편집 로직 발명 금지.

코어는 이미 있다.

- 메뉴 조립: `document_core::queries::explore::build_menu`
- 사실 묶음: `DocFacts` (표·누름틀·조문·차트·각주·주입·은닉 개수)
- CLI: `src/main.rs` 의 `explore_document` (`--json` 봉투)
- MCP: `hwp_explore` 가 같은 키를 쓴다

에이전트가 필요한 것은 새 구현이 아니라 **언제나 explore 부터 치고,
보안이 메뉴에 있으면 본문을 LLM 에 넣기 전에 처리하며, why 를 문서
지시로 오독하지 않는 것** 이다.

DoD: additions 5000–10000 (최소 5000). PR 전 `cargo fmt --all -- --check`.

## 어떻게

1. 격리 worktree `C:/Users/swsz9/rhwp-agent-explore` 에
   `feat/agent-explore` 를 `upstream/devel` 에서 분기.
   금지 디렉터리(`rhwp-desk*` 등)는 쓰지 않음.
2. SKILL.md 를 세 축·첫 수·라우팅 표·정지 규칙·예외 인덱스로 재작성.
3. `references/` 24장: 세 축, 첫 수, 봉투, 우선순위, 라우팅, 보안 우선,
   정직한 휴리스틱, 암호/빈 파일/특수 없음, 어포던스 8개 각 장, 인계,
   함정, 여정, 트레이스, 발화, 종료 코드, 명령 상자, confidence, why.
4. `examples/` 10개 일한 예. 전체 JSON 은 `fixtures/envelopes/` 를 가리킨다.
5. `_gen_pack.py` 가 `build_menu` 를 파이썬으로 복제해 시나리오 40,
   봉투 표본, 트레이스 40, 발화·여정을 `fixtures/` 에 방출.
6. `scripts/tests/test_agent_explore.py` 가 발명 명령·gym·이웃 스킬
   재작성·봉투 키·우선순위·예외 경로를 바이너리 없이 검사.
7. capability 등록부 `CAP-5313` / `rhwp-explore` 행 추가.

## 하지 않은 것

- `explore.rs` / `explore_document` / `DocFacts` 변경
- 새 플래그 (`--rank`, `--only`) · 새 하위명령 (`suggest`)
- 채움·redact·csv-to-table · DocumentCore 편집 구현
- gym pack / 과제 / 채점기
- onboarding · mcp-session · safe-edit · provenance · form-fill ·
  security-sweep · doc-triage · table-exchange 스킬 수정

## 검증

```bash
python -m unittest scripts.tests.test_agent_explore
cargo fmt --all -- --check
```

기존 계약 `tests/cases/explore_menu_contract.rs` 는 건드리지 않았다.
이 PR 의 파이썬 `build_menu` 복제는 그 우선순위 표본(S40)과 같다.

## 세 축

| 축 | 질문 |
| --- | --- |
| explain | 이 문서가 무엇인가 |
| capabilities | 도구가 일반적으로 무엇을 하는가 |
| explore | 이 문서로 무엇을 할 수 있는가 |

처음 보는 파일의 첫 수는 언제나 `explore --json` (`rhwp explore <파일> --json`).

## 예외

- 암호, 비밀번호 없음 → exit 2, stdout 비움. 메뉴를 추정하지 않음.
- 빈 파일·파싱 실패 → exit 1. 가짜 개요를 만들지 않음.
- 특수 어포던스 없음 → 메뉴는 `triage-overview` 하나. 실패가 아님.
- `security-sweep` 가 있으면 본문보다 먼저.

## 정직성

제안이지 완전성 보장이 아니다. `why` 는 엔진 개수. 봉투
`untrustedContent:false`.
