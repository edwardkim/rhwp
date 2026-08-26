# V-shadow — 서로 다른 검사 두 개가 합의해야 합격 (#5510)

LLM-as-Verifier 축. **이미 있는** 기계 명령 두 개가 동시에 합격해야
합격이다. 한 명령만 합격하는 것은 합의가 아니다.

한 봉투 안 필드 모순은 V-abstain 이다. 같은 명령을 K 번 돌리는 것은
V-repeat 이다. 이 디렉터리는 그 둘을 구현하지 않는다.

새 `rhwp` CLI 를 만들지 않는다. `ir-diff` · `verify` · `layout-anomaly`
등이 이미 내는 필드만 데이터로 읽는다.

## 판단 트리

입력 4열:

| 열 | 의미 | 예 |
| --- | --- | --- |
| `check_a` | 기존 기계 검사 A | `ir-diff` (`identical`) |
| `check_b` | 기존 기계 검사 B | `verify-pages` (`failCount=0`) / `layout-anomaly` (`hasSignal=false`) |
| `a_pass` | A 가 자기 합격 조건을 만족하는가 | 0/1 |
| `b_pass` | B 가 자기 합격 조건을 만족하는가 | 0/1 |

`expected_joint` 는 **서로 다른 명령** 이고 `a_pass=1` 이고 `b_pass=1`
일 때만 1 이다.

```
same command_key?     → SAME_CHECK_NOT_SHADOW   expected_joint=0
a_pass ∧ b_pass       → JOINT_PASS              expected_joint=1
a_pass ∧ ¬b_pass      → SHADOW_A_ONLY           expected_joint=0
¬a_pass ∧ b_pass      → SHADOW_B_ONLY           expected_joint=0
else                  → JOINT_BOTH_FAIL         expected_joint=0
```

표본 쌍:

- `ir-diff` identical 과 `dump-pages`/`verify-pages` 쪽수 일치
- `fill-verify` `verify.identical` 과 `layout-anomaly` `hasSignal=false`

## 명령

```text
python tools/llm_verifier/shadow_agree/cli.py \
  --check-a ir-diff --check-b layout-anomaly --a-pass 1 --b-pass 1

python tools/llm_verifier/shadow_agree/generate_corpus.py
python tools/llm_verifier/shadow_agree/verify_corpus.py
python -m unittest discover -s tools/llm_verifier/shadow_agree/tests -v
```

## 코퍼스

`corpus/shard_*.tsv` 각 행은 서로 다른
`(check_a, check_b, a_pass, b_pass, expected_joint)` 사례다.
주석 패딩이 아니다. 최소 100000행. `verify_corpus.py` 가 모든 행을
`decide()` 와 대조한다.
