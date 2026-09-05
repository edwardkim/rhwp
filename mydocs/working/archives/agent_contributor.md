---
kind: working
status: active
issue: 5322
---

# 에이전트 기여 절차(contributor) 스킬 고도화 (#5322)

작업 브랜치: `feat/agent-contributor`
대상 스킬: `.agents/skills/rhwp-contributor/`
이슈: [agent: 기여 절차(contributor) 스킬 고도화](https://github.com/edwardkim/rhwp/issues/5322)

## 1. 한 줄

실사용 에이전트가 rhwp 기여 1건을 공식 8단(이슈→분석→`upstream/devel`
브랜치→구현→로컬 게이트→영수증 포인터→처리 결과→한국어 PR)으로 완주하도록
스킬·픽스처·시험으로 배선한다. HARD GATE 는 `cargo fmt --all -- --check`.
새 CLI 없음. gym 없음. DocumentCore 편집 로직 발명 없음.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- 필수 순서: 이슈 → 분석 → 브랜치 from `upstream/devel` → 구현 → 로컬 게이트 →
  작업 영수증 → 처리 결과 문서 → 한국어 PR
- HARD GATE: `cargo fmt --all -- --check` (`cargo fmt --check` 아님).
  SKILL.md 의 낡은 표기를 정본 명령으로 맞춘다
- clippy `-D warnings`, 관련 `cargo test`, 렌더/레이아웃이면 시각 근거
- PR 템플릿 첫 체크박스 = fmt 게이트
- DocumentCore 편집 로직 발명 금지, `git add -A` 금지, named worktree 훔침 금지
- 예외: 스파스에 `crates/` 없음, Windows autocrlf vs rustfmt Unix,
  중복 열린 PR, CI noci vs FAILURE
- 작업 영수증은 `replay --capsule` / `audit` / `lineage` 포인터만
  (그 스킬을 다시 쓰지 않음)
- `.agents/skills/rhwp-contributor/` SKILL.md + references/ + examples/ + fixtures
- `mydocs/working/agent_contributor.md` (이 파일)
- 계약 시험 (순수, 픽스처 기반)
- additions 5000–10000, 최소 5000
- PR 전 `cargo fmt --all -- --check`
- isolation worktree, 브랜치 `feat/agent-contributor` from `upstream/devel`
- 한국어 PR, base `devel`, `closes #5322`, `--body-file`
- `git add -A` 금지

금지:

- gym/
- 다른 스킬 (영수증 스킬 본문 포함)
- 열린 PR 파일
- DocumentCore 편집 구현
- 새 rhwp CLI 명령

## 3. 왜 스킬만 키우나

기존 `rhwp-contributor` 는 체크리스트 한 장이었다. 에이전트가
`cargo fmt --check` 만 돌리거나, 본진 워크트리에서 구현하거나,
`git add -A` 로 범위를 섞거나, 영수증 스킬을 재작성하거나, noci 와
FAILURE 를 섞는 구멍이 표본으로 막혀 있지 않았다.

구현은 그대로다. 이 파동은 정본(`AGENTS.md`, `CONTRIBUTING.md`,
`local_validation.md` §4.3, PR 템플릿)을 같은 단어로 인용하는지만 본다.

## 4. 범위

만진 것:

| 경로 | 역할 |
|------|------|
| `.agents/skills/rhwp-contributor/SKILL.md` | 라우터. 8단, HARD GATE, 하지 않는 것 |
| `references/` | 단계·예외·함정·필드 카탈로그 |
| `examples/01`–`24` | 워크스루 |
| `fixtures/` | 체크리스트·봉투·시나리오 카드·금지 워크트리 |
| `scripts/tests/test_agent_contributor.py` | 파일 계약 (바이너리 없음) |
| `tests/cases/agent_contributor_skill_contract.rs` | 같은 계약을 Rust 로 |
| `mydocs/working/agent_contributor.md` | 이 기록 |

만지지 않은 것:

- `src/` CLI 구현, DocumentCore
- `gym/` 전부
- `.agents/skills/rhwp-work-receipt/` 본문
- `.agents/skills/rhwp-onboarding/` 외 형제 스킬
- `.github/pull_request_template.md` (열린 fmt-gate PR 과 겹침)
- 공개 샘플 HWP 바이너리

## 5. 기존 계약의 지도

| 계약 | 출처 |
|------|------|
| `cargo fmt --all -- --check` | CONTRIBUTING.md 포맷 정책, CI Lint |
| `newline_style = Unix` | rustfmt.toml |
| 범위별 검증 | local_validation.md §4.3 |
| 작업 증빙 포인터 | AGENTS.md 작업 증빙 절, #4528 / rhwp-work-receipt |
| 한글 `--body-file` | AGENTS.md, pr_review_workflow.md 3.4.1 |
| 문서 전용 CI 예외 | feedback_docs_only_ci_exempt.md |
| 스킬 실재 명령 가드 | #4508, skills_contract.rs |

capability 카탈로그에 `rhwp-contributor` 행은 이미 있다 (`CAP-4561`).
이 파동은 스킬을 새로 만들지 않고 확장만 하므로 카탈로그 행을 바꾸지 않았다.

## 6. 디렉터리 규약

```
.agents/skills/rhwp-contributor/
  SKILL.md
  references/          라우터 자식
  examples/            워크스루 24 + README
  fixtures/
    catalog.json       목록의 단일 출처
    checklists/        8단
    envelopes/         게이트·예외 + _skillMeta.exit
    transcripts/       argv 표본
    layouts/           금지 워크트리·스파스 crates
    scenario-cards/    시나리오 1장 1파일
    pr-bodies/         한국어 본문 표본
    scenario_catalog.json
    gate-matrix.json
```

`catalog.json` 이 목록의 단일 출처다. stray JSON 이 생기면 시험이 실패한다.

## 7. 시험

```
python -m unittest scripts.tests.test_agent_contributor
```

확인하는 것:

- 레이아웃, frontmatter, 자식 문서
- HARD GATE 철자, 낡은 `cargo fmt --check` 거절
- catalog ↔ 디스크 파일
- 금지 워크트리 레지스트리
- 봉투 `_skillMeta.exit` ∈ {0,1,2,3}
- noci ≠ FAILURE
- `git add -A` 거절 봉투
- scenario_catalog ≥ 80, 발명 명령 없음
- gym/ 를 편집하라고 하지 않음
- 형제 스킬 경로가 사라지지 않음
- PR 본문 첫 체크박스 = fmt 게이트
- rustfmt `newline_style=Unix`

Rust `tests/cases/agent_contributor_skill_contract.rs` 가 같은 불변식을
픽스처만 읽고 고정한다. 바이너리를 부르지 않는다.

## 8. fmt 게이트

```
cargo fmt --all -- --check
```

`newline_style = Unix`. `crates/` 가 이 워크트리에 있으면 반드시 통과한다.

## 9. PR 메모

- base: `devel`
- head: `kevin9327:feat/agent-contributor`
- 제목 한국어, 본문 `--body-file` UTF-8 without BOM
- `closes #5322`
- `git add -A` 를 쓰지 않고 경로를 지정해 add
