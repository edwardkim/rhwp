# #5324 실 에이전트 버그 헌팅(bug-hunter) — 작업 기록

날짜: 2026-08-18
이슈: https://github.com/edwardkim/rhwp/issues/5324
브랜치: `feat/agent-bug-hunter` (`upstream/devel` 기준 격리 worktree)
범위: `.agents/skills/bug-hunter/` ·
`.claude/skills/rhwp-bug-hunter/` (얇은 포인터) ·
`scripts/tests/test_agent_bug_hunter.py` ·
`tests/agent_bug_hunter_skill_contract.rs` ·
capability 등록부 `CAP-5324` · 본 문서
비범위: `gym/` · 이웃 스킬 재작성 · 새 CLI · DocumentCore ·
실제 접수/로그인/실명인증 · 엔진 버그픽스

## 무엇을

에이전트가 실사례 사용자 여정을 끝까지 돌리고 한컴 공식 출력·법정
서식·제출 요건 정답지와 대조해 재현 가능한 결함을 찾으려면
`bug-hunter` 스킬이 playbook 실행 계약·정답지 확보·재현 이슈 작성·
오인 함정까지 닫혀 있어야 한다.

기존 `.agents/skills/bug-hunter/SKILL.md` 는 6단 요약만 있었다.
이 작업은 그 본문을 30초 판단 내비게이터로 재작성하고
`references/` · `examples/` · `fixtures/` 로 나누며, Claude 쪽
얇은 포인터와 기계 가독 픽스처·가드 테스트로 계약을 고정한다.

**playbook 이 유일한 루브릭이다.** 두 번째 헌팅 점수표를 만들지
않았다.

## 왜

이슈 본문: 실 에이전트 경로. gym 금지. 새 CLI 발명 금지. 코드
수정은 하지 않는다 (헌팅 스킬이지 버그픽스가 아님).

코어는 이미 있다.

- 방법론: `mydocs/manual/bug_hunting_playbook.md`
- 한컴 PDF 대조: `tools/fidelity_compare`
- 기존 CLI: export-svg/png/pdf, render-diff, ir-diff, dump,
  info, fields, export-tables, edit set-cell/fill-fields
- 문자 멀티셋: 기준본 전용=소실, SVG 전용=과잉, 양쪽=치환
- 판정 함정 4종, 예시 1–7, 여정 카탈로그

에이전트가 필요한 것은 새 구현이 아니라 **언제 정답지를 먼저
잡고, 어떤 기존 명령을 끝까지 치며, 관측을 소실/과잉/치환/
재독/계약으로 어떻게 분류하고, 무엇을 이슈 3필수에 담는가** 이다.

DoD: additions 5000–10000 (최소 5000). PR 전 `cargo fmt --all -- --check`.

## 어떻게

1. 격리 worktree `C:/Users/swsz9/rhwp-agent-bug-hunter` 에
   `feat/agent-bug-hunter` 를 `upstream/devel` 에서 분기.
   `rhwp` · `rhwp-desk*` · `rhwp-handoff` · `rhwp-scaffold-final` ·
   `rhwp-doc-repro` 는 쓰지 않음. 이름 있는 worktree 를 훔치지 않음.
2. SKILL.md 를 사다리·정지 규칙·인계 인덱스로 재작성. playbook
   6단을 복제하지 않고 가리킨다.
3. `references/` 25장: 권위, 함정 4종, 여정 선택, 정답지,
   provenance, 자기 일관성 한계, 최종 산출물, 픽셀, 멀티셋,
   재독, 종료 코드, fidelity_compare, 이슈 템플릿, 접수 금지,
   UTF-8, 함정, 카탈로그, 트레이스, 발화, 분류표, 인계, 정지,
   게이트, 기존 CLI.
4. `examples/` 20건: playbook 예시 1–7 + 카탈로그 + 함정/템플릿.
5. `_gen_pack.py` 가 `fixtures/` 에 JSON·TSV·트레이스·이슈
   템플릿을 방출. 여정 80+, 발화 120+, 트레이스 40, 분류 12행.
6. `.claude/skills/rhwp-bug-hunter/SKILL.md` 는 포인터만.
7. `scripts/tests/test_agent_bug_hunter.py` 가 발명 명령·gym·
   이웃 스킬 재작성·playbook 권위·분류 단어·이슈 3필수를
   바이너리 없이 검사.
8. `tests/agent_bug_hunter_skill_contract.rs` 가 같은 가드
   (파일만. 엔진/CLI 를 실행해 고치지 않음).
9. capability 등록부 `CAP-5324` / `rhwp-bug-hunter` 행 추가.
   기존 `CAP-3398` 은 유지하고 Claude 포인터를 연결.

## 하지 않은 것

- DocumentCore · 렌더러 · 직렬화기 변경
- 새 CLI 플래그 / `bug-hunt` 발명
- gym pack / 과제 / 채점기
- 실제 정부 포털 접수·로그인
- onboarding · mcp-session · safe-edit · provenance ·
  doc-triage · form-fill · visual-regression 스킬 본문 수정
- playbook 본문 재작성 (권위를 갈라놓지 않음)
- 열린 다른 PR 파일

## 검증

```bash
python -m unittest scripts.tests.test_agent_bug_hunter
cargo fmt --all -- --check
# crates/ 가 있으므로 이 게이트는 필수.
# 추가한 tests/agent_bug_hunter_skill_contract.rs 는
# rustfmt --check --config newline_style=Unix.
```

엔진 테스트·렌더 골든을 돌리지 않는다. 이 PR 은 스킬이 기존
표면을 인용하는지만 본다.

## 권위

- `mydocs/manual/bug_hunting_playbook.md` — 유일한 루브릭
- `tools/fidelity_compare/README.md`
- `mydocs/manual/cli_commands.md`
- `mydocs/manual/verification/visual_verification_governance.md`
