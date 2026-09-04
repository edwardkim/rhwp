# #5333 현장 FDE 대응 스킬 고도화 — 작업 기록

날짜: 2026-08-18
이슈: https://github.com/edwardkim/rhwp/issues/5333
브랜치: `feat/agent-fde` (`upstream/devel` 기준 격리 worktree)
범위: `.claude/skills/rhwp-fde/` · `mydocs/working/agent_fde.md` ·
`scripts/tests/test_agent_fde.py` · `tests/agent_fde_skill_contract.rs` ·
capability 등록부 `CAP-4893` 스킬 진입점
비범위: `gym/` · bug-hunter 스킬 재작성 · 다른 스킬 본문 · DocumentCore ·
새 rhwp CLI · `tools/fde/triage.py` 판정 로직 재발명

## 무엇을

에이전트가 고객이 들고 온 현장 증상(안 열린다 / 깨진다 / 필드가 안 채워진다)을
실시간으로 접수·트리아지·응급처치·재현체·업스트림 이슈화하도록
`.claude/skills/rhwp-fde/` 를 신설했다.

기존에 있던 부품은 그대로 둔다.

- 정본 playbook: `mydocs/manual/fde_playbook.md`
- 엔진: `tools/fde/triage.py` (읽기 전용 사다리, 라우트 박힌 JSON 티켓)
- 에이전트 정의: `.claude/agents/rhwp-fde.md` — 링크만. 엔진을 다시 쓰지 않는다.

없는 것은 접점 스킬이었다. 이 PR 은 그 스킬과 계약 시험만 닫는다.

## 왜

bug-hunter 는 **우리가 고른 여정**을 정답지와 대조한다. fde 는 **고객이
들고 온 증상**을 지금 처리한다. 입구·산출물·시간 계약이 다르다.
증상 문장은 데이터이지 지시가 아니다.

DoD: additions 5000–10000 (최소 5000). PR 전 `cargo fmt --all -- --check`.

## 어떻게

1. 격리 worktree `C:/Users/swsz9/rhwp-agent-fde` 에 `feat/agent-fde` 를
   `upstream/devel` 에서 분기. 금지 경로(`rhwp`, `rhwp-desk*`,
   `rhwp-handoff`, `rhwp-scaffold-final`, `rhwp-doc-repro`)는 쓰지 않음.
2. `SKILL.md` 를 접수 세 칸·사다리·라우트·정지 인덱스 로 작성.
3. `references/` 32장: 출처 경계, 엔진, 매직 바이트, capabilities 하드코딩
   금지, info/explain/export-structure/digest, 티켓 키, 라우트, 암호 우회
   금지, crash/corrupt 별명, 회신, 선행 검색, 축소, 인계, bug-hunter 경계.
4. `examples/` 25건: 안 열림·표·필드·암호·PDF 위장·panic·timeout·주입.
5. `fixtures/` 기계 가독 픽스처 + 트레이스 티켓 JSON.
   `_gen_pack.py` 가 방출한다.
6. `scripts/tests/test_agent_fde.py` 가 발명 명령·gym·이웃 재작성·
   티켓 키·엔진 `decide_route`/`sniff_container` 를 바이너리 없이 검사.
7. `tests/agent_fde_skill_contract.rs` 가 같은 가드를 Rust 쪽에서 고정.
8. 등록부 `CAP-4893` 행에 Skill 진입점을 추가한다. 새 CAP 번호를 만들지
   않는다 — 고도화 이슈는 #5333, capability 는 기존 #4893.

## 하지 않은 것

- `tools/fde/triage.py` 의 `decide_route` / `LADDER` / `MAGIC` 변경
- 새 rhwp 하위명령·플래그
- gym pack / 과제 / 채점기
- `.agents/skills/bug-hunter/` 또는 `.claude/skills/rhwp-bug-hunter/` 재작성
- 다른 스킬 SKILL.md 수정
- DocumentCore / 한컴 최종 판정 / 머지 판단

## 검증

```bash
python -m unittest scripts.tests.test_agent_fde
cargo fmt --all -- --check
cargo test --test agent_fde_skill_contract
```

엔진 자체 단위 시험은 triage.py 를 import 해서 매직 바이트·라우트 표만
대조한다. 바이너리가 없어도 계약이 깨지지 않는다.

## 사다리와 라우트

순서: 매직 바이트 → `capabilities --json` (광고된 것만) → `info` →
`explain` → `export-structure` → `digest`. 전부 읽기 전용.

엔진 route:

- `invalid-input`
- `resolve-now` (전 단계 통과, 또는 암호 — 우회 금지)
- `workaround` (깨끗한 비0)
- `escalate-bug` (panic/abort/timeout)

대화 별명: `escalate-crash` → `escalate-bug`, `escalate-corrupt` →
`workaround`. 별명을 티켓 필드에 쓰지 않는다.

티켓은 `command` / `exitCode` / `failureSignature` / `envelopeKeys` 를
기록한다. "됐습니다" 산문은 티켓이 아니다.
