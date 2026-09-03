# V-oracle — 한컴 정답지 vs 자기일관성 (#5487)

LLM-as-Verifier 축 2. 한컴 공식 PDF가 **있을 때만** 독립 오라클을 열고,
없으면 `render-diff A==A` / `dump-pages` 자기일관성만 정직하게 주장한다.

이 디렉터리만 소유한다. `tools/fidelity_compare`, `tools/oracle_public`,
`scripts/visual_sweep.py` 는 **재작성하지 않는다**. 그 도구들이 이미 내는
필드·판정 문자열을 데이터로 읽는다.

## 판단 트리

입력 5열:

| 열 | 의미 | 계약 출처 |
| --- | --- | --- |
| `has_hangul_pdf` | `oracle_resolver` pairs[] 에 공식 PDF가 있는가 | `oracle_public/oracle_resolver.py` |
| `versions` | 한컴 연도 토큰. `+` 일치, `!` 쪽수 불일치 | resolver 2018/2020/2022/2024, multiver 는 2010 포함 |
| `page_count_match` | `dump-pages` 쪽수 == 한컴 PDF 쪽수 | `page_smoke` MATCH / fidelity `page-count-ledger.tsv` |
| `render_self_pass` | 같은 문서를 두 번 그려 A==A | `rhwp render-diff` |
| `cheap_ok` | 값싼 전처리(ERROR/LFS/run-state 누락 없음) | `page_smoke` ERROR, fidelity `run-state.tsv` |

출력 `expected_verdict_class`:

```
has_hangul_pdf = false
    versions ≠ none     → NO_ORACLE_VERSION_TOKEN_WITHOUT_PDF
    cheap_ok = false    → NO_ORACLE_SELF_CHEAP_FAIL
    render_self_pass=0  → NO_ORACLE_SELF_RENDER_FAIL
    else                → NO_ORACLE_SELF_CONSISTENT

has_hangul_pdf = true
    versions none/unknown → ORACLE_UNVERSIONED
    versions invalid      → ORACLE_YEAR_OUT_OF_CONTRACT
    versions A!B          → ORACLE_MULTIVER_DISAGREE
    page_count_match = 0  → ORACLE_PAGECOUNT_MISMATCH   (값싼 독립 오라클)
    cheap_ok = false      → ORACLE_CHEAP_FAIL
    render_self_pass = 0  → ORACLE_BLOCKED_BY_SELF
    else                  → ORACLE_TRUSTED              (fidelity_compare / visual_sweep 후보 열림)
```

`ORACLE_TRUSTED` 에서도 픽셀·문자 보고는 fidelity_compare 계약대로
**candidate** 이다. 최종 결함 확정이 아니다.

한컴 PDF가 없는데 fidelity_compare 나 visual_sweep 을 돌리고
“한컴과 같다”고 쓰는 것은 부정직한 주장이다. 그 경우 허용 도구는
`rhwp render-diff A==A` 와 `rhwp dump-pages --json` 뿐이다.

## 명령

```text
python tools/llm_verifier/oracle_vs_self/cli.py \
  --has-hangul-pdf 0 --versions none \
  --page-count-match 1 --render-self-pass 1 --cheap-ok 1

python -m tools.llm_verifier.oracle_vs_self.generate_corpus
python -m tools.llm_verifier.oracle_vs_self.verify_corpus
python -m unittest discover -s tools/llm_verifier/oracle_vs_self/tests -v
```

## 코퍼스

`corpus/shard_*.tsv` 각 행은 서로 다른 결정 사례다. 주석 패딩이 아니다.
최소 100000행. `verify_corpus.py` 가 모든 행을 `decide()` 와 대조한다.

열: `has_hangul_pdf, versions, page_count_match, render_self_pass, cheap_ok,
expected_verdict_class` + 샘플 정체(문서 family·포맷·oracle_root·쪽수).
