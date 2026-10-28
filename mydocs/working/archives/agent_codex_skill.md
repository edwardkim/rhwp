---
kind: working
status: active
issue: 5318
---

# 에이전트 대전(Codex) 항해 스킬 고도화 (#5318)

작업 브랜치: `feat/agent-codex`
대상 스킬: `.claude/skills/rhwp-codex/`
이슈: [agent: 대전(Codex) 항해 스킬 고도화](https://github.com/edwardkim/rhwp/issues/5318)

## 1. 한 줄

실사용 에이전트가 71개 명령 표면을 대전 교본으로 항해하도록, 이미 devel 에
있는 `mydocs/manual/agent_codex/` · `tools/gen_agent_codex.py` ·
`rhwp capabilities --search` 를 스킬·픽스처·시험으로 배선한다.
새 CLI 없음. gym 없음. 생성 장 수기 수정 없음.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- 철학 4규약 (판정=데이터 · 결정론 · 출처 표지 · 원본 무훼손)
- 요청→명령 트리: 파악/수확/편집/변환/검증/보안/대량 → 장 번호
- 생성 장 vs 손글 읽는 법. `generated:` frontmatter 수기 금지
- 재생성/신선도: `python tools/gen_agent_codex.py` 와 `--check` (exit 3 = stale, DATA)
- `capabilities --search` 폴백
- 봉투 필드 사전은 지식지도 §2-2, 스킬이 아님
- 85장은 개발자 전용
- `.claude/skills/rhwp-codex/` SKILL.md + references/ + examples/ + fixtures
- `mydocs/working/agent_codex_skill.md` (이 파일)
- 계약 시험 (형제 에이전트 PR 과 같은 꼴)
- additions 5000–10000, 최소 5000
- PR 전 `cargo fmt --all -- --check`
- isolation worktree, 브랜치 `feat/agent-codex` from `upstream/devel`
- 한국어 PR, base `devel`, `closes #5318`, `--body-file`
- `git add -A` 금지

금지:

- gym/
- 다른 스킬
- 열린 PR 파일
- DocumentCore 편집 구현
- 새 rhwp CLI 명령
- 생성 장 수기 수정
- 새 live-oracle gym pack

## 3. 왜 스킬만 키우나

기존 `rhwp-codex` 는 입장 순서 30초만 있었다. 에이전트가 장 번호를 얻거나
생성 장을 손으로 고치거나, exit 3 을 크래시로 읽거나, 필드 사전을 스킬에
베끼거나, 85장 dump 로 일상 작업을 하는 경로가 표본으로 막혀 있지 않았다.

생성기(`tools/gen_agent_codex.py`)와 커버리지 가드(`tests/agent_codex_contract.rs`)는
이미 있다. 이 파동은 그 계약을 스킬이 같은 단어로 인용하는지만 본다.

## 4. 범위

만진 것:

| 경로 | 역할 |
|------|------|
| `.claude/skills/rhwp-codex/SKILL.md` | 30초 인덱스 · 7갈래 · 정지 표 |
| `references/` | 4규약 · 트리 · 읽는 법 · 신선도 · 검색 · 경계 · 85 · 가족 장 항해 |
| `examples/` | 실측 표본을 흉내 내는 레시피 28편 |
| `fixtures/` | 카탈로그 · 발화 · 여정 · 검색 폴백 · 생성 장에서 추출한 봉투 전사 |
| `tools/gen_skill_pack.py` | 생성 장을 **읽어** 전사를 방출. 생성 장은 고치지 않음 |
| `tests/agent_codex_skill_contract.rs` | Rust 계약 |
| `scripts/tests/test_agent_codex_skill.py` | 파일 계약 (바이너리 없음) |
| `mydocs/working/agent_codex_skill.md` | 이 기록 |

만지지 않은 것:

- `mydocs/manual/agent_codex/` 생성 장 본문
- `tools/gen_agent_codex.py` 생성기 로직
- `gym/`
- 이웃 스킬 SKILL.md
- DocumentCore / 새 CLI

## 5. 추출 전사

`fixtures/envelopes/*.json` 은 생성 장의 실측 JSON 을 옮긴 것이다.
대형 자기서술 봉투(매니페스트·스키마)는 키와 표지만 남겼다.
정본은 여전히 생성 장이다. 전사가 어긋나면 생성 장을 고치지 말고
추출기를 다시 돌린다.

## 6. 검증

```bash
python -m unittest scripts.tests.test_agent_codex_skill
cargo fmt --all -- --check
cargo test --test agent_codex_skill_contract
```

기존 `tests/agent_codex_contract.rs` (전 명령 장 보유) 와
`python tools/gen_agent_codex.py --check` (신선도) 는 건드리지 않았다.

## 7. 하지 않은 것 (재확인)

- 새 `rhwp` 하위명령
- 생성 장 JSON 손수정
- 지식지도 §2-2 사전 복제
- gym pack
- 이웃 스킬 본문 수정
