# #5311 실 에이전트 폴더 일괄 파이프라인(batch) — 작업 기록

날짜: 2026-08-18
이슈: https://github.com/edwardkim/rhwp/issues/5311
브랜치: `feat/agent-bulk-pipeline` (`upstream/devel` 기준 격리 worktree)
범위: `.claude/skills/rhwp-bulk-pipeline/` ·
`tests/agent_bulk_pipeline_skill_contract.rs` ·
`scripts/tests/test_agent_bulk_pipeline.py` · 본 문서
비범위: `gym/` · 다른 스킬 본문 · 새 CLI 명령 · DocumentCore 편집 구현 ·
열린 PR 파일

## 무엇을

에이전트가 폴더의 HWP/HWPX 수백 건을 `rhwp batch` 로 한 번에 처리할 때
**stdin 한 줄 = 경로 → stdout 순수 NDJSON → stderr 사람 요약**,
실패 행을 봉투로 남기고 jq 로 갈라 재시도하며, 입력 N = 성공 + 실패
게이트를 닫는 실사용 경로를 스킬로 고정한다.

기존 `rhwp-bulk-pipeline` 은 SKILL.md 한 장에 요약만 있었다. 이 작업은
그 본문을 30초 판단 내비게이터로 재작성하고 `references/` · `examples/` ·
`fixtures/` 로 축·게이트·전사를 나눈다.

## 왜

이슈 본문: 에이전트가 폴더의 HWP/HWPX 수백 건을 batch 로 처리해야 한다.
gym 금지. 새 CLI 발명 금지.

코어는 이미 있다.

- 읽기 축: `batch info` · `export-text` · `export-structure` · `export-tables` ·
  `fields` · `search` · `extract-data`
- 쓰기 축: `batch convert` (이름 예약, CLI 전용) · `batch fill` (서식 1 + 데이터 N)
- 계약: stdin 목록, NDJSON, 실패 봉투 `exitClass:runtime`, `--threads` 입력 순서
  보존, 종료 집계 0/1/2/3/4
- 실측 원형: 레시피 9 (5 = 4 + 1, exit 1), 레시피 5, `cli_json_pipeline_guide.md`

에이전트가 필요한 것은 새 구현이 아니라 **어느 축을 치고, 실패 행을 어디로
보내며, 숫자가 맞을 때 멈추는가** 이다. 실패 행을 지우면 게이트가 깨지고,
`--password` 를 batch 에 붙이면 평문 271건이 exit 2 로 죽는다.

DoD: additions 5000–10000 (최소 5000). PR 전 `cargo fmt --all -- --check`.

## 어떻게

1. 격리 worktree `C:/Users/swsz9/rhwp-agent-bulk-pipeline` 에
   `feat/agent-bulk-pipeline` 을 `upstream/devel` 에서 분기.
   `rhwp-desk*` · `rhwp-handoff` · `rhwp-scaffold-final` · `rhwp-doc-repro` 는
   쓰지 않음. 디스크 부족으로 **sparse checkout** (crates/ · 스킬 · 매뉴얼 ·
   기존 batch 계약 테스트).
2. SKILL.md 를 사다리·정지 규칙 B01–B18·인계 인덱스로 재작성.
3. `references/` 33장: 트리, stdin/NDJSON, 실패 봉투, 스레드 순서, 9축,
   jq 재시도, N 게이트, 암호 금지, convert 이름 예약, fill 입력 축,
   종료 집계, 함정, 인계, 여정, 발화, 봉투, stderr, 목록, 트레이스,
   jq 레시피, 재시도 부류, PowerShell, 표본, 폴더 메뉴.
4. `_gen_pack.py` 가 `fixtures/` 와 `examples/transcripts/` 를 방출.
   여정 80+, 발화 100+, 레시피 9 실측 5=4+1, 트레이스 T01–T20.
5. `examples/` 12개 레시피 + 목록/명단 픽스처.
6. `scripts/tests/test_agent_bulk_pipeline.py` 가 발명 명령·gym·픽스처
   스키마를 바이너리 없이 검사.
7. `tests/agent_bulk_pipeline_skill_contract.rs` 가 같은 가드를 순수
   픽스처로 검사. `CARGO_BIN_EXE_rhwp` 를 부르지 않는다.

## 하지 않은 것

- 새 `batch` 서브커맨드·플래그
- `batch` 전역 `--password` 구현 (명문 그대로 거부)
- convert 이름 예약 로직 변경
- fill 을 stdin 목록으로 바꾸는 일
- gym pack / 과제 / 채점기
- onboarding · mcp-session · safe-edit · provenance · doc-triage ·
  form-fill 스킬 수정
- DocumentCore 편집 구현

## 검증

```bash
python -m unittest scripts.tests.test_agent_bulk_pipeline
cargo fmt --all -- --check
cargo test --test agent_bulk_pipeline_skill_contract
```

기존 계약 (`batch_axes_contract`, `batch_extract_data_contract`,
`batch_fill_contract`, `batch_parallel_determinism_contract`) 은
건드리지 않았다. 이 PR 은 그 표면을 스킬이 인용하는지만 본다.

sparse checkout 에서 `src/` 가 없으면 `cargo test --test …` 는 링크되지
않을 수 있다. 파이썬 시험과 rustfmt 는 crates/ 가 있으면 돌아야 한다.

## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md` (PR #4182)
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
