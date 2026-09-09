---
kind: working
status: active
issue: 5487
claim: V-oracle
---

# V-oracle: 한컴 정답지 vs 자기일관성 선택 트리 (#5487)

작업 브랜치: `feat/v-oracle-vs-self`
대상: `tools/llm_verifier/oracle_vs_self/`
이슈: [V-oracle: 한컴 정답지 vs 자기일관성 선택 트리](https://github.com/edwardkim/rhwp/issues/5487)

## 1. 한 줄

한컴 공식 PDF가 있으면 독립 오라클, 없으면 render-diff A==A / 쪽수 자기일관성만
정직하게 주장하는 판단 트리와 결정 사례 코퍼스를 넣는다.
`fidelity_compare` · `oracle_public` · `visual_sweep.py` 는 재작성하지 않는다.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- LLM-as-Verifier 축 2. WHEN to trust Hangul-official PDF vs self-consistency
- 판단 트리 + 픽스처
- 파일 소유: `tools/llm_verifier/oracle_vs_self/` + 이 문서
- 결정 사례 코퍼스 열:
  `(has_hangul_pdf, versions, page_count_match, render_self_pass, cheap_ok,
  expected_verdict_class)`
- 서로 다른 행. 주석 패딩 금지
- `git diff --shortstat upstream/devel` ≥ 100000 insertions
- HARD GATE: `cargo fmt --all -- --check`
- isolation worktree, 브랜치 `feat/v-oracle-vs-self` from `upstream/devel`
- origin `kevin9327`, 한국어 PR, base `devel`, `closes #5487`

금지:

- `tools/fidelity_compare` 재작성
- `tools/oracle_public` 재작성
- `scripts/visual_sweep.py` 재작성
- `verdict_protocol`, `claim_bind`, `best_of_n`, `process_steps`, `untrusted_sandbox`
- `gym/`
- `git add -A`
- 금지 워크트리: `rhwp`, `rhwp-desk*`, `rhwp-handoff`, `rhwp-scaffold-final`, `rhwp-doc-repro`

## 3. 왜 이 축인가

한컴 PDF가 없는 문서에 대해 fidelity_compare 시트를 열어 “한컴과 같다”고
쓰면 검증기가 정답지를 발명한 것이 된다. 반대로 공식 PDF가 있는데도
A==A 만 보고 “오라클 통과”라고 쓰면 독립 대조를 건너뛴 것이다.

기존 도구는 이미 신호를 낸다.

- `oracle_resolver`: pairs[] / unmatched[]
- `page_smoke`: MATCH / MISMATCH / ERROR
- `multiver_index`: 같은 stem 의 연도별 쪽수 불일치
- `fidelity_compare`: `page-count-ledger.tsv` (candidate)
- `visual_sweep.py`: hwp+pdf TARGET, schema_version 1
- `rhwp render-diff`: A==A 자기일관성

이 파동은 그 신호를 묶는 **선택 트리**만 소유한다.

## 4. 트리 요약

`has_hangul_pdf=false` 이면 독립 오라클 도구
(`tools/fidelity_compare`, `scripts/visual_sweep.py`) 를 막는다.
버전 토큰이 같이 오면 `NO_ORACLE_VERSION_TOKEN_WITHOUT_PDF`.

`has_hangul_pdf=true` 이어도 연도가 계약 밖이거나 다중 버전 쪽수가 갈리면
한 연도를 정답지로 고르지 않는다. 쪽수 불일치는 page_smoke MISMATCH 로서
값싼 독립 주장이다. A==A 실패면 공식 PDF가 있어도 시각 오라클을 열지 않는다
(`ORACLE_BLOCKED_BY_SELF`).

다섯 입력이 모두 통과할 때만 `ORACLE_TRUSTED`. 그 다음 픽셀·문자 보고는
여전히 candidate.

## 5. 검증

```text
python -m unittest discover -s tools/llm_verifier/oracle_vs_self/tests -v
# 42 tests, OK
python tools/llm_verifier/oracle_vs_self/verify_corpus.py
# 122400 rows, 11 verdict classes, decide() 일치
cargo fmt --all -- --check
# 종료 코드 0
```

코퍼스 모든 행의 `expected_verdict_class` 는 `decide()` 재계산과 같다.
closed-set 축 표는 11개 판정 계급을 빠짐없이 덮는다.

## 6. 소유 경로

- `tools/llm_verifier/oracle_vs_self/`
- `mydocs/working/llm_verifier_oracle_vs_self.md`
