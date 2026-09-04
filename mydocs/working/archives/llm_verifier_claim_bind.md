---
kind: working
status: active
issue: 5488
---

# V-bind: 주장마다 문서 좌표 강제 (#5488)

작업 브랜치: `feat/v-bind-claim-coords`
대상: `tools/llm_verifier/claim_bind/` · 본 문서
워크트리: `C:\Users\swsz9\rhwp-v-bind-claim-coords` (`upstream/devel` 격리)

## 한 줄

LLM 주장 한 줄마다 기존 `search`/`extract-data` 봉투의
`section`·`paragraph`·`page`·`charOffset` 가 있어야 통과하고,
좌표가 없거나 불완전하거나 봉투 밖 키를 지으면 실패한다.

## 이슈가 요구한 것

- LLM-as-verifier 축 3: 자연어 CLAIM 을 문서 좌표에 결속
- 좌표 출처는 이미 있는 search/extract-data 봉투뿐
- 미결속 주장은 FAIL
- 전략가·출처 스킬 재작성 금지
- 새 CLI 발명 금지
- 파일 소유: `tools/llm_verifier/claim_bind/` + 본 문서
- `git diff --shortstat upstream/devel` 100000 insertions 이상
- `(claim_text, coords_present, field_set, pass/fail)` 실픽스처 코퍼스
- 한국어 문서 주장의 서로 다른 행. 주석 패딩·로렘 금지
- `cargo fmt --all -- --check`, base `devel`, 한국어 PR, `closes #5488`

## 하지 말라는 것

- `rhwp-strategist` / `rhwp-provenance` 스킬 재작성
- `verdict_protocol` · `oracle_vs_self` · `best_of_n` · `process_steps` · `untrusted_sandbox`
- 새 rhwp CLI
- `gym/`
- `git add -A`
- 금지 워크트리 (`rhwp`, `rhwp-desk*`, `rhwp-handoff`, `rhwp-scaffold-final`, `rhwp-doc-repro`)

## 만진 경로

- `tools/llm_verifier/claim_bind/` (라이브러리, 스키마, 봉투 표본, 120000행 코퍼스)
- `mydocs/working/llm_verifier_claim_bind.md`
- 워크스페이스 등록: `Cargo.toml` members 에 `tools/llm_verifier/claim_bind` (fmt `--all` 대상)

## 만지지 않은 경로

- `.claude/skills/rhwp-strategist/`, `.claude/skills/rhwp-provenance/`
- `tools/strategist/engagement.py`
- `tools/llm_verifier/verdict_protocol/` 및 다른 verifier 축
- `gym/`, DocumentCore, rhwp CLI 명령

## 어떻게

1. 최신 `upstream/devel` 에서 `feat/v-bind-claim-coords` 격리 worktree.
2. `llm-verifier-claim-bind` 라이브러리: 봉투 파서, 필수 4키 검사, 발명 키 거부, 봉투 매치 대조.
3. 실패 종류: `unbound` · `incomplete_coords` · `invented_key` · `empty_claim` · `envelope_mismatch` · `unknown_envelope_kind`.
4. `scripts/gen_claim_corpus.py` 가 과업지시서·판결문·예산·입찰·기안 등 실무 문장 120000행을 NDJSON 샤드로 생성. 각 행은 고유 `rowId`와 (빈 문장 실패행 제외) 고유 `claimText`.
5. 통합 시험이 전 행을 다시 판정해 라벨과 일치하는지, 한국어인지, 로렘/패딩이 없는지 검사.

## 판정 계약

| 조건 | verdict |
| --- | --- |
| 필수 4키 모두 있고 발명 키 없음 | pass |
| 좌표 없음 | fail / unbound |
| 4키 중 일부만 | fail / incomplete_coords |
| `pdfPage`·`humanPage`·`line` 등 | fail / invented_key |
| 주장 문장 공백 | fail / empty_claim |
| 좌표가 봉투 매치에 없음 | fail / envelope_mismatch |

`page` 는 봉투 그대로 0 기준. 없는 `page` 를 1-based 로 만들지 않는다.

## 시험 명령

```bash
cargo test -p llm-verifier-claim-bind
cargo fmt --all -- --check
```

## PR 메모

- base `devel`
- `closes #5488`
- `--body-file` UTF-8 without BOM
- 제목·본문 한국어
- 첫 체크박스: `cargo fmt --all -- --check`
