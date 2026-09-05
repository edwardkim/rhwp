# 명령·필드 카탈로그

이 스킬이 입 밖으로 내는 명령과, 봉투·체크리스트가 고정하는 키.
새 명령을 여기 추가하지 않는다.

## 허용 명령 가족

| 가족 | 허용 형태 | 금지 형태 |
|------|-----------|-----------|
| git | `fetch`, `worktree add/list`, `add -- <path>`, `diff`, `push`, `config` | `git add -A`, `git add .`, 기존 named worktree checkout |
| gh | `issue list/view/create`, `pr list/create/view/checks` | `--base main`, 한글 here-string 파이프 |
| cargo | `fmt --all -- --check`, `fmt --all`, `clippy -- -D warnings`, `test --test <이름>` | `cargo fmt --check` 를 게이트로 |
| python | `python -m unittest scripts.tests.test_agent_contributor` | gym 채점기 |
| node | `rust-test-suite-manifest.mjs --generate/--check` | generated 수기 수정 |
| rhwp | `replay --capsule`, `audit`, `lineage` (포인터) | `rhwp contribute`, `rhwp pr-gate`, `rhwp receipt` |

## HARD GATE 필드

| 키 | 값 |
|----|----|
| `hardGate` | `cargo fmt --all -- --check` |
| `staleFmt` | `cargo fmt --check` |
| `staleFmtRejected` | `true` |
| `newlineStyle` | `Unix` |
| `cratesPresentImpliesFmtMustPass` | `true` |
| `firstPrCheckbox` | HARD GATE 와 동일 문자열 |

## PR 필드

| 키 | 값 |
|----|----|
| `base` | `devel` |
| `bodyFile` | `true` (UTF-8 without BOM) |
| `closes` | 이슈 번호 (이 파동은 `5322`) |
| `titleKo` | `true` |
| `head` | `kevin9327:feat/agent-contributor` |

## 예외 분류

| `classification` | 뜻 | `isFailure` |
|------------------|-----|-------------|
| `noci` | 검사가 안 뜸 (paths-ignore, docs-only) | `false` |
| `FAILURE` | 뜬 검사가 빨강 | `true` |
| `duplicate` | 같은 주제 열린 PR | 새 PR 안 만듦 |
| `sparse` | `crates/` 없음 | sparse-checkout add |
| `crlf` | autocrlf vs Unix | 로컬 autocrlf=false |
| `stolen` | named worktree 사용 시도 | 거절 |

## 봉투 `_skillMeta`

| 키 | 제약 |
|----|------|
| `skill` | `rhwp-contributor` |
| `issue` | `5322` |
| `command` | `git` / `gh` / `cargo` / `python` / `node` / `rhwp` / `replay` / `audit` / `lineage` / `read` |
| `exit` | `0` 성공, `1` IO/환경, `2` 사용법(거절), `3` 판정(중복·FAILURE) |
| `branch` | `ok` / `stale` / `crlf` / `noci` / `failure` / `duplicate` / `sparse` / `stolen` / `usage` / `layout` |
| `hardGate` | 항상 HARD GATE 문자열 |
| `staleFmtRejected` | 항상 `true` |

exit 3 은 도구 크래시가 아니다. 중복 PR 또는 CI FAILURE 같은 **판정**이다.

## 금지 토큰

에이전트가 아래를 실행 예로 쓰면 계약 실패다. 금지 안내로 등장하는 것은 허용.

- `git add -A`
- `git add .` (스테이징 레시피)
- `cargo fmt --check` (게이트로)
- `rhwp contribute`
- `rhwp pr-gate`
- `gh pr create --base main`
