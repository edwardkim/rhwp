# V-nonce — 문서 텍스트가 검증 기준이 되지 못하게 (#5491)

LLM-as-Verifier 축 6. `untrustedContent` / `untrustedFields` 로 표지된
문서 파생 텍스트를 nonce 경계 안의 데이터로만 다루고, 검증 기준·지시
자리로 새면 차단한다.

이 디렉터리만 소유한다. `.claude/skills/rhwp-provenance` 는 다시 쓰지
않는다. 새 rhwp CLI 는 없다.

## 열

`(excerpt, nonce, slot, leaked_into_criteria, expected_block)`

각 행은 서로 다른 배치다. 주석 패딩이 아니다.

## 허용 자리

- `user_display`
- `llm_data_block` (nonce 경계가 온전할 때)

그 외 자리와 `leaked_into_criteria=yes` 는 차단.

## 검증

```text
python tools/llm_verifier/untrusted_sandbox/generate_corpus.py
python tools/llm_verifier/untrusted_sandbox/verify_corpus.py
python -m unittest discover -s tools/llm_verifier/untrusted_sandbox/tests -v
cargo fmt --all -- --check
```
