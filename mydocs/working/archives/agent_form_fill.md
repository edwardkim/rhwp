# #5300 실 에이전트 서식 채우기(누름틀·메일머지) — 작업 기록

날짜: 2026-08-18
이슈: https://github.com/edwardkim/rhwp/issues/5300
브랜치: `feat/agent-form-fill` (`upstream/devel` 기준 격리 worktree)
범위: `.claude/skills/rhwp-form-fill/` ·
`tests/cases/agent_form_fill_skill_contract.rs` · `scripts/tests/test_agent_form_fill.py` ·
capability 등록부 `CAP-5300` · 본 문서
비범위: `gym/` · `rhwp-onboarding` · `rhwp-mcp-session` · `rhwp-safe-edit` ·
`rhwp-provenance` · `rhwp-doc-triage` · 새 edit 로직 · 새 CLI 플래그

## 무엇을

에이전트가 공공·사내 HWP/HWPX 서식의 누름틀을 채우고, 명단 N행으로
메일머지 산출물을 만들 때 **조사 → 선검증 → 실행 → 재파싱 → 제출 정리**
순서를 빠뜨리거나, 같은 이름 필드의 첫 칸만 채운 채 제출하거나,
`batch fill` 에 stdin 파일 목록을 넣는 실수를 줄인다.

기존 `rhwp-form-fill` 스킬은 SKILL.md 한 장에 요약만 있었다. 이 작업은
그 본문을 30초 판단 내비게이터로 재작성하고 `references/` 에 장을 나누며,
기계 가독 픽스처와 가드 테스트로 계약을 고정한다.

## 왜

이슈 본문: 에이전트가 서식 누름틀을 채우고 메일머지하는 **실사용 경로**.
gym 금지. 새 edit 로직 발명 금지.

코어는 이미 있다.

- 조회: `fields` ← `collect_all_fields()` (#3281)
- 채움: `edit fill-fields` ← `set_field_value_by_name` (#3329)
- 순번: `이름[N]` (#3476)
- 메일머지: `batch fill` 이 행마다 그 경로를 다시 부름 (#3719 §6-6)
- 선검증/자기검증: `--dry-run` / `--verify` (#3702)
- 제출 정리: `edit sanitize` (#3719 §6-11)

에이전트가 필요한 것은 새 구현이 아니라 **언제 어느 명령을 치고,
어느 봉투 필드로 멈추는가** 이다. `ambiguous` 를 침묵 성공으로 읽으면
14칸 중 1칸만 채워진 규제영향분석서가 제출된다.

DoD: additions 5000–10000 (최소 5000). PR 전 `cargo fmt --all -- --check`.

## 어떻게

1. 격리 worktree `C:/Users/swsz9/rhwp-agent-form-fill` 에
   `feat/agent-form-fill` 을 `upstream/devel` 에서 분기.
   `rhwp-desk*` 는 쓰지 않음.
2. SKILL.md 를 사다리·정지 규칙·인계 인덱스로 재작성.
3. `references/` 21장: fields 조사, fill-fields, `이름[순번]`, batch fill,
   dry-run/verify, sanitize, 봉투, 함정, 여정, 인계, 신호, 데이터 형식,
   name-field, insert-image, 축 선택, 트레이스, 발화 행렬, 필드 카탈로그,
   jq 게이트, 종료 코드.
4. `_gen_pack.py` 가 `references/fixtures/` 에 JSON 을 방출.
   여정 100+, 발화 120+, 메일머지 80행, 순번 사례, 트레이스 40.
5. `scripts/tests/test_agent_form_fill.py` 가 발명 명령·gym·이웃 스킬
   재작성·픽스처 스키마를 바이너리 없이 검사.
6. `tests/cases/agent_form_fill_skill_contract.rs` 가 같은 가드 + 표본이 있으면
   기존 `fields` / `fill-fields --dry-run` / 빈 CSV exit 2 만 재현.
   채움 구현을 바꾸지 않음.
7. capability 등록부 `CAP-5300` / `rhwp-form-fill` 행 추가.

## 하지 않은 것

- `set_field_value_by_name` · batch fill 루프 · sanitize 바이트 처리 변경
- 머리말/각주 필드 재귀 확장 (사각지대로 문서화만)
- fill-fields overflow (#3480) 구현
- gym pack / 과제 / 채점기
- onboarding · mcp-session · safe-edit · provenance · doc-triage 스킬 수정

## 검증

```bash
python -m unittest scripts.tests.test_agent_form_fill
cargo fmt --all -- --check
cargo test --test agent_form_fill_skill_contract
```

기존 계약 (`fields_json_contract`, `edit_fill_fields_contract`,
`edit_field_occurrence_contract`, `batch_fill_contract`,
`edit_verify_contract`) 은 건드리지 않았다. 이 PR 은 그 표면을 스킬이
인용하는지만 본다.

## 권위

- `mydocs/manual/cli_commands.md` (`fields` · `edit fill-fields` ·
  `batch fill` · `edit insert-image` · `edit sanitize` · 종료 코드)
- `mydocs/manual/recipes/01_fill_form_and_submit.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `mydocs/manual/form_filling_guide.md`
