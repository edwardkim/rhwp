---
kind: working
status: active
issue: 5507
---

# V-repeat 반복 평가 · 분산 축소 (#5507)

작업 브랜치: `feat/v-repeat-eval`
대상: `tools/llm_verifier/repeat_eval/` · `mydocs/working/llm_verifier_repeat_eval.md`

## 한 줄

같은 산출을 K번 기계 검사한다. 종료코드와 기존 `--json` 봉투 필드만 읽고, 범주는 다수결·수치는 평균으로 줄여 분산을 낮춘다.

## 이슈가 요구한 것 / 하지 말라는 것

요구:

- LLM-as-Verifier CLAIM V-repeat
- 같은 산출을 K번 기계 검사 (repeated evaluation)
- 다수결 / 평균으로 exit+JSON 필드 축소
- 코퍼스 행 = `(artifact, k, check, votes, variance, final)`
- 파일 소유권은 `tools/llm_verifier/repeat_eval/` 과 이 문서뿐
- 추가 100000줄 이상. 미만이면 PR 금지
- `cargo fmt --all -- --check`, base `devel`, 한국어 PR, `closes #5507`

하지 말라는 것:

- V-bon 후보 순위 (`best_of_n`, `expectedRank`)
- V-decomp 기준 분해 (`atomPass`, `holisticScore`)
- 새 rhwp CLI
- `gym/` 변경
- `git add -A`
- 금지 워크트리(`rhwp`, `rhwp-desk*`, `rhwp-handoff`, `rhwp-scaffold-final`, `rhwp-doc-repro`)
- 주석·공백 패딩으로 줄 수 채우기

## 축소 규칙

범주 (`exitClass`, bool, text, `passFail`):

1. K개 관측 값을 센다
2. 최다 득표가 과반이면 그 값
3. 동률이면 fail-closed (더 나쁜 exit, `false`, `fail`)
4. `disagreement = 1 - majorityFrac`

수치 (`filledCount`, `diffCount` 등):

1. K개 관측의 산술평균
2. 표본분산 `(n-1)`
3. 의도값과 평균 차가 0.5 미만이면 pass

같은 `(artifact, check)` 에 K 사다리를 둔다. 시드 0..k-1 은 prefix-stable 이라 K가 커져도 앞 관측은 그대로다. 단일 뒤집기는 K가 커질수록 다수결 분산이 줄어든다.

## 만진 경로 / 만지지 않은 경로

만짐:

- `tools/llm_verifier/repeat_eval/` (축소기, 표·분산, 코퍼스 생성, 스키마, 시험)
- `mydocs/working/llm_verifier_repeat_eval.md`

만지지 않음:

- `Cargo.toml` 워크스페이스 (소유권 밖). 크레이트는 자체 `[workspace]`
- `gym/`
- V-bon / V-decomp / V-step 축
- rhwp CLI 표면

## 코퍼스

- 생성: `python tools/llm_verifier/repeat_eval/generate_corpus.py`
- 한 레코드 = 산출 1개 + 검사 1개 + 시드 K개 봉투 + 표 + 분산 + 최종
- 유일키 `(artifactId, k, check)`. 주석 패딩 없음
- 명령 가족: info, verify, ir-diff, layout-anomaly, replay, fill-fields, render-diff, convert, replace-text, redact, set-cell, csv-to-table, sanitize
- 검사: 기존 봉투 필드와 `exitClass` / `passFail` 만. `holisticScore`·`bestOfN` 거부

## 시험 명령

```text
python tools/llm_verifier/repeat_eval/generate_corpus.py
cargo test --manifest-path tools/llm_verifier/repeat_eval/Cargo.toml
cargo fmt --all -- --check
```

실측: 코퍼스 1101 레코드 · 46 샤드 · 230700 줄. 크레이트 시험 37 passed.

## fmt 게이트

```text
cargo fmt --all -- --check
```

루트 워크스페이스 멤버만 `--all` 이 본다. 이 축 크레이트는

```text
cargo fmt --manifest-path tools/llm_verifier/repeat_eval/Cargo.toml --all -- --check
```

도 통과해야 한다.
