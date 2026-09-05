# #5312 실 에이전트 시각 회귀(render-diff·ir-diff) — 작업 기록

날짜: 2026-08-18
이슈: https://github.com/edwardkim/rhwp/issues/5312
브랜치: `feat/agent-visual-regression` (`upstream/devel` 기준 격리 worktree)
범위: `.agents/skills/rhwp-visual-regression/` ·
`tests/agent_visual_regression_skill_contract.rs` ·
`scripts/tests/test_agent_visual_regression.py` ·
capability 등록부 `CAP-5312` · 본 문서
비범위: `gym/` · `rhwp-onboarding` · `rhwp-mcp-session` · `rhwp-safe-edit` ·
`rhwp-provenance` · `rhwp-doc-triage` · `rhwp-form-fill` · 새 CLI ·
DocumentCore 편집 구현

## 무엇을

에이전트가 편집/변환 전후 레이아웃 회귀를 **사람 눈이 아니라 px 숫자**로
판정할 때, 자기 라운드트립·두 파일·폴더 배치(`geom_inventory.tsv`) →
`ir-diff --json` → thumbnail/export-png 판단 트리와
`STRUCT_MISMATCH` 를 경로로 읽는 규약을 스킬로 닫는다.

기존 `rhwp-visual-regression` 스킬은 SKILL.md 한 장에 요약만 있었다.
이 작업은 그 본문을 30초 판단 내비게이터로 재작성하고 `references/` ·
`examples/` · `fixtures/` 로 나누며, 기계 가독 픽스처와 가드 테스트로
계약을 고정한다.

## 왜

이슈 본문: 에이전트가 편집/변환 전후 레이아웃 회귀를 숫자로 판정해야
한다. gym 금지. 새 CLI 발명 금지.

코어는 이미 있다.

- 기하: `render-diff` ← `diagnostics::render_geom_diff` (자기 / 두 파일 /
  `--batch` → `geom_inventory.tsv`)
- 상태: PASS / WARN_TEXTRUN / OVER / STRUCT_MISMATCH / PAGE_MISMATCH /
  LOAD_FAIL
- IR: `ir-diff --json` — 차이 = exit 3 (#3274)
- 미리보기: `thumbnail` = 저장 시점 PrvImage (재렌더 아님)
- 재렌더: `export-png` (native-skia)
- 임계: `--max-disp` 기본 1.0px. 구조 불일치는 임계와 무관
- 결정성: A==A 는 항상 PASS

에이전트가 필요한 것은 새 구현이 아니라 **언제 어느 명령을 치고,
STRUCT 빨간불을 경로로 어떻게 읽는가** 이다. 채운 자리의
`STRUCT_MISMATCH` 를 롤백하면 메일머지가 전부 실패로 보인다.

DoD: additions 5000–10000 (최소 5000). PR 전 `cargo fmt --all -- --check`.

## 어떻게

1. 격리 worktree `C:/Users/swsz9/rhwp-agent-visual-regression` 에
   `feat/agent-visual-regression` 을 `upstream/devel` 에서 분기.
   `rhwp-desk*` · `rhwp-handoff` · `rhwp-scaffold-final` ·
   `rhwp-doc-repro` 는 쓰지 않음. 디스크 부족으로 sparse checkout
   (samples/ · mydocs/pr/ 제외).
2. SKILL.md 를 사다리·정지 규칙·인계 인덱스로 재작성.
3. `references/` 25장: 자기/두 파일/배치, STRUCT, 상태, ir-diff,
   thumbnail vs png, 결정성, 임계, 봉투, 함정, 여정, 인계, 신호,
   경로, 트레이스, 발화, TSV, 게이트, 종료 코드, PAGE/LOAD/OVER,
   render-tree.
4. `examples/` 20건: 레시피 06 실측 전사.
5. `_gen_pack.py` 가 `fixtures/` 에 JSON·TSV·트레이스를 방출.
   여정 80+, 발화 120+, 트레이스 50, TSV 카탈로그.
6. `scripts/tests/test_agent_visual_regression.py` 가 발명 명령·gym·
   이웃 스킬 재작성·픽스처 스키마를 바이너리 없이 검사.
7. `tests/agent_visual_regression_skill_contract.rs` 가 같은 가드 +
   표본이 있으면 기존 `render-diff` 자기/A==A PASS, `ir-diff --json`
   identical=0 / 차이=3 만 재현. 렌더 구현을 바꾸지 않음.
8. capability 등록부 `CAP-5312` / `rhwp-visual-regression` 행 추가.

## 하지 않은 것

- `render_geom_diff.rs` · ir-diff · thumbnail · export-png 구현 변경
- 새 CLI 플래그 / `visual-diff` 발명
- gym pack / 과제 / 채점기
- onboarding · mcp-session · safe-edit · provenance · doc-triage ·
  form-fill 스킬 수정
- DocumentCore 편집 경로

## 검증

```bash
python -m unittest scripts.tests.test_agent_visual_regression
# 26 passed (바이너리 없음)
cargo fmt --all -- --check
# sparse 작업 트리: crates/ 는 있음. tests/generated/regression_suite_*.rs
# 는 저장소에 없고 생성 산출이라 cargo fmt --all 이 그 경로를 못 찾는다.
# 추가한 tests/agent_visual_regression_skill_contract.rs 는
# rustfmt --check --config newline_style=Unix 통과. crates/*.rs 도 동일.
```

기존 계약 (`ir_diff_json_contract`, `edit_render_diff_gate`) 은
건드리지 않았다. 이 PR 은 그 표면을 스킬이 인용하는지만 본다.

## 권위

- `mydocs/manual/cli_commands.md` (`render-diff` · `ir-diff` ·
  `thumbnail` · `export-png` · 종료 코드)
- `mydocs/manual/recipes/06_visual_regression_before_after.md`
- `mydocs/manual/ir_diff_command.md`
- `src/diagnostics/render_geom_diff.rs`
