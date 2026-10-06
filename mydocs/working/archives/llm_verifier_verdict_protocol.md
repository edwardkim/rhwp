---
kind: working
status: active
issue: 5486
---

# V-proto: 검증기 입력은 종료코드·JSON 봉투만 (#5486)

구현은 `tools/llm_verifier/verdict_protocol/` 만 소유한다.
다른 V-\* (`oracle_vs_self`, `claim_bind`, `best_of_n`, `process_steps`,
`untrusted_sandbox`) 와 파일을 겹치지 않는다.

## 한 줄

LLM-as-verifier 의 합격/불합격은 산문이 아니라 기존 rhwp 종료코드
(0/1/2/3/4) 와 `--json` 봉투 필드(`identical`, `hasSignal`, `reproduced`,
`findingCount`, `verify.identical` …) 만으로 읽는다. 새 rhwp CLI 없음.

## 기계 계약

- 입력: `Observation` = `{command, exitClass, envelope?, sourceTag}`
- 추출: 지식지도 §2-2 판정 필드만 (`extract_judgment`)
- 출력: `ProtocolDecision.machineVerdict` ∈
  `{pass, io_fail, usage_fail, judgment_fail, page_verify_fail, inconsistent}`
- 유일키: `(command, exitClass, judgmentFingerprint, sourceTag)`
- 코퍼스: `corpus/shards/*.json` (`generate_corpus.py` 재생성)

분류 규칙은 `src/classify.rs` 가 정본이다. 이 문서는 포인터다.
