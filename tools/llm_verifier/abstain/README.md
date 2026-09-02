# V-abstain — 필드 모순이면 기권

LLM-as-verifier 축: **ABSTAIN ON CONTRADICTION** (issue #5508).

봉투 필드가 서로 모순이면 `decide()` 는 `abstain` 을 낸다.
통과/실패를 발명하지 않는다. V-proto 분류기를 다시 쓰지 않고,
새 rhwp CLI 도 만들지 않는다.

## 닫힌 집합

`pass` | `fail` | `abstain`

이슈가 든 예:

| 모순 | 결과 |
| --- | --- |
| `identical:true` ∧ `hasSignal:true` | abstain |
| `reproduced:true` ∧ exit 3 | abstain |
| pageCount 일치 ∧ 같은 노드 `STRUCT_MISMATCH` | abstain |
| pageCount 일치 ∧ **다른** 노드 `STRUCT_MISMATCH` | fail (정직한 공존) |

## 코퍼스

`corpus/shard_*.tsv` — 서로 다른 필드 튜플 + `expected`.
생성: `python -m abstain.generate_corpus`
검증: `python -m abstain.verify_corpus`

## 테스트

```text
python -m unittest discover -s tools/llm_verifier/abstain/tests -v
```
