---
kind: working
status: active
issue: 5489
---

# V-bon Best-of-N 기계 순위 (#5489)

작업 브랜치: `feat/v-bon-best-of-n`
대상: `tools/llm_verifier/best_of_n/` · `mydocs/working/llm_verifier_best_of_n.md`

## 한 줄

후보 N개의 **최종 산출**을 기존 dry-run / `--verify` / `ir-diff` 봉투 필드만으로 순위 매긴다. 산문 점수와 V-step(`process_steps`, #5490)은 없다.

## 이슈가 요구한 것 / 하지 말라는 것

요구:

- LLM-as-Verifier CLAIM V-bon
- 기계 필드: `changedCount`, `invalid`, `verify.identical`, `exitClass`
- 각 후보 집합에 `expectedRank`
- 파일 소유권은 `tools/llm_verifier/best_of_n/` 과 이 문서뿐
- `cargo fmt --all -- --check`, base `devel`, 한국어 PR, `closes #5489`

하지 말라는 것:

- 산문 점수
- 과정 단계 보상(`process_steps` — V-step / #5490)
- `gym/` 변경
- 새 rhwp CLI
- `git add -A`
- 금지 워크트리(`rhwp`, `rhwp-desk*`, `rhwp-handoff`, `rhwp-scaffold-final`, `rhwp-doc-repro`)

## 순위 키 (작을수록 좋음)

1. `invalid` 비어 있음/거짓 > 채워짐/참
2. `exitClass` 0 > 3 > 4 > 1 > 2 (성공 > 판정실패 > 쪽검증 > IO > 사용법)
3. `verify.identical` true > 없음(dry-run/`null`) > false
4. `|changedCount - intendedChangedCount|`, 그다음 `changedCount`
5. 동률은 `candidateId` 로 안정 정렬. 같은 비교키는 경쟁 순위(1,1,3)

`ir-diff` 의 최상위 `identical`/`diffCount` 는 기존 봉투에서 `verify.identical`/`changedCount` 로 끌어올린다.

## 만진 경로 / 만지지 않은 경로

만짐:

- `tools/llm_verifier/best_of_n/` (순위기, 봉투 lift, 코퍼스 생성, 픽스처, 스키마, 테스트)
- `mydocs/working/llm_verifier_best_of_n.md`

만지지 않음:

- `Cargo.toml` 워크스페이스 (소유권 밖)
- `gym/`
- V-step / `process_steps`
- 다른 `tools/llm_verifier/*` 축

## 코퍼스

- 생성: `python tools/llm_verifier/best_of_n/generate_corpus.py`
- 한 레코드 = 후보 집합 N개 + 네 필드 + `expectedRank`
- 주석 패딩 없음. 집합 식별키 `(command, mode, sample, intended, n, candidate fingerprints)` 유일
- 명령 가족: fill-fields, csv-to-table, ir-diff, convert, replace-text, redact, set-cell, csv-to-chart, sanitize, run
- 모드: dry-run / verify / ir-diff

## 시험 명령

```text
python tools/llm_verifier/best_of_n/test_rank.py
python tools/llm_verifier/best_of_n/test_envelopes.py
python tools/llm_verifier/best_of_n/test_corpus.py
```

실측: 16 + 3 + 7 = 26 tests OK.

## fmt 게이트

```text
cargo fmt --all -- --check
```

워크스페이스에 Rust 를 추가하지 않았다. 기존 멤버만 검사한다.

## PR 메모

- base `devel`
- `closes #5489`
- `--body-file` UTF-8 without BOM
- origin `kevin9327`
- 제목·본문 한국어
