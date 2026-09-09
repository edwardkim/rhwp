---
kind: working
status: active
issue: 5308
---

# 에이전트 작업 영수증·감사·계보 스킬 고도화 (#5308)

작업 브랜치: `feat/agent-work-receipt`
대상 스킬: `.claude/skills/rhwp-work-receipt/`
이슈: [agent: 작업 영수증·감사·계보 스킬 고도화](https://github.com/edwardkim/rhwp/issues/5308)

## 1. 한 줄

실사용 에이전트가 한 일을 재실행으로 증명하도록, 이미 devel 에 있는
`replay` / `--capsule` / `--parent` / `audit` / `lineage` 를 스킬·픽스처·시험으로
배선한다. 새 CLI 없음. gym 없음. 귀속·서명 주장 없음.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- replay attest (입력·계획·산출 SHA-256 3종) 과 verify (`--expect-output-sha256`)
- `--capsule` / `--parent` 해시 체인, 캡슐 불변, 부모 경로는 캡슐 파일 기준
- audit 폴더 `reproducedRate` 회계 (비재귀 `*.capsule.json`)
- lineage / lineage `--deep` (`parentOk`, `lineageOk`, `reproduced`, `brokenAt`)
- exit 3 = 판정 데이터, exit 1 = IO, exit 2 = 사용법
- `toolVersion` 불일치 함정, attribution/signature claim 없음
- `.claude/skills/rhwp-work-receipt/` SKILL.md + references/ + examples/ + fixtures
- `mydocs/working/agent_work_receipt.md` (이 파일)
- 계약 시험 (순수, 픽스처 기반)
- additions 5000–10000, 최소 5000
- PR 전 `cargo fmt --all -- --check`
- isolation worktree, 브랜치 `feat/agent-work-receipt` from `upstream/devel`
- 한국어 PR, base `devel`, `closes #5308`, `--body-file`
- `git add -A` 금지

금지:

- gym/
- 다른 스킬
- 열린 PR 파일
- DocumentCore 편집 구현
- 새 rhwp CLI 명령

## 3. 왜 스킬만 키우나

K 트랙의 작업 영수증 스킬은 이미 SKILL.md 한 장으로 사다리 3단과 함정을
적고 있다. 구멍은 레시피·예외 봉투·픽스처·시험이 없어서 에이전트가
플래그를 발명하거나 exit 3 을 크래시로 읽거나 캡슐을 포맷터로 저장하는
경로가 표본으로 막혀 있지 않다는 점이다.

구현 (`cmd_replay`, `cmd_audit`, `cmd_lineage`) 은 그대로다.
`tests/audit_contract.rs` · `tests/lineage_contract.rs` 가 엔진을 이미 지킨다.
이 파동은 그 계약을 스킬이 같은 단어로 인용하는지만 본다.

## 4. 범위

만진 것:

| 경로 | 역할 |
|------|------|
| `.claude/skills/rhwp-work-receipt/SKILL.md` | 라우터. 자식 표, 하지 않는 것 |
| `references/replay-attest.md` | 3해시 · attest/verify |
| `references/capsule-chain.md` | 불변 · 상대 경로 · 같은 파일 거부 |
| `references/audit-accounting.md` | 비재귀 · reproducedRate |
| `references/lineage-chronicle.md` | 3축 · --deep · brokenAt |
| `references/exit-codes.md` | 3/1/2 |
| `references/pitfalls.md` | toolVersion · 귀속 금지 |
| `references/decision-tree.md` | 요청 라우팅 |
| `references/envelope-field-catalog.md` | 키 사전 |
| `references/recipe-index.md` | 교차표 |
| `examples/01`–`20` | 워크스루 |
| `fixtures/` | 캡슐·봉투·계획·레이아웃·시나리오·해시 벡터 |
| `scripts/tests/test_agent_work_receipt.py` | 파일 계약 (바이너리 없음) |
| `tests/cases/agent_work_receipt_skill_contract.rs` | 같은 계약을 Rust 로 |
| `mydocs/working/agent_work_receipt.md` | 이 기록 |

만지지 않은 것:

- `src/` CLI 구현, DocumentCore
- `gym/` 전부
- `.claude/skills/rhwp-onboarding/`
- `.claude/skills/rhwp-mcp-session/`
- `.claude/skills/rhwp-provenance/`
- `.claude/skills/rhwp-doc-triage/`
- `.claude/skills/rhwp-safe-edit/`
- 공개 샘플 HWP 바이너리

## 5. 기존 계약의 지도

| 계약 | 출처 |
|------|------|
| replay 3해시 · attest/verify | #4391, `cmd_replay` |
| `--capsule` 자기완결 | #4393 |
| `--parent` 해시 체인 | #4401 |
| audit 재현율 | #4393, `audit_contract.rs` |
| lineage 3축 · brokenAt | #4401, `lineage_contract.rs` |
| exit 3 = 판정 | #2707 |
| 서명 사이드카 (비범위) | #4509 |
| 앵커 (비범위) | #4543 |
| 스킬 실재 명령 가드 | #4508, `skills_contract.rs` |

capability 카탈로그에 `rhwp-work-receipt` 행이 원래 없었다. 이 파동은
스킬을 새로 만들지 않고 확장만 하므로 카탈로그 행을 추가하지 않았다.
등록이 필요하면 별도 이슈에서 `CAP-5308` 을 논의한다.

## 6. 디렉터리 규약

```
.claude/skills/rhwp-work-receipt/
  SKILL.md
  references/          라우터 자식
  examples/            워크스루 20 + README
  fixtures/
    catalog.json       목록의 단일 출처
    capsules/          workCapsule + 변조 표본
    envelopes/         단별 봉투 + _skillMeta.exit
    plans/             유효/무효 계획
    audit-layouts/     비재귀 회계 폴더
    lineage-layouts/   상대 경로·깨진 연대
    transcripts/       argv 표본
    hash-vectors/      SHA-256 벡터
    scenario_catalog.json
```

`catalog.json` 이 목록의 단일 출처다. stray JSON 이 생기면 시험이 실패한다.

## 7. 시험

```
python -m unittest scripts.tests.test_agent_work_receipt
```

확인하는 것:

- 레이아웃, frontmatter, 자식 문서
- 레퍼런스별 계약 토큰 (3해시, --expect-output-sha256, 비재귀, parentOk…)
- catalog ↔ 디스크 파일
- 캡슐 kind/planText/planSha256 정합
- 자식 parent 경로가 절대 경로가 아님
- 자식 inputSha256 == 부모 outputSha256
- audit 레이아웃 비재귀 카운트
- reproducedRate = reproduced/total
- 봉투 _skillMeta.exit ∈ {0,1,2,3}
- scenario_catalog 가 replay/audit/lineage 만 명령으로 씀
- gym/ 를 편집하라고 하지 않음
- attributionClaim == false
- 형제 스킬 경로가 사라지지 않음

Rust `tests/cases/agent_work_receipt_skill_contract.rs` 가 같은 불변식을
픽스처만 읽고 고정한다. 바이너리를 부르지 않는다.

## 8. fmt 게이트

```
cargo fmt --all -- --check
```

`newline_style = Unix`. crates/ 가 이 워크트리에 있다.

## 9. PR 메모

- base: `devel`
- head: `kevin9327:feat/agent-work-receipt`
- 제목 한국어, 본문 `--body-file` UTF-8 without BOM
- `closes #5308`
- `git add -A` 를 쓰지 않고 경로를 지정해 add
