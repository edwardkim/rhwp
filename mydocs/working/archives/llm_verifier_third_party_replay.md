---
kind: working
status: active
issue: 5502
claim: V-replay
---

# V-replay: 제3자 재실행으로만 노동을 인정 (#5502)

작업 브랜치: `feat/v-replay-third-party`
대상: `tools/llm_verifier/third_party_replay/`
이슈: [V-replay: 제3자 재실행으로만 노동을 인정](https://github.com/edwardkim/rhwp/issues/5502)

## 1. 한 줄

구현자의 말은 증거가 아니다. `rhwp replay --expect-output-sha256` 가 쓴
`reproduced` 와 workCapsule `receipt.reproduced` 만으로 노동을 인정하거나
기각한다. work-receipt 스킬·M-rcpt 픽스처는 재작성하지 않고 필드 계약만 감싼다.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- LLM-as-Verifier 축 THIRD-PARTY REPLAY
- 구현자 산문 불채택. 제3자 재실행 `reproduced` 만
- 파일 소유: `tools/llm_verifier/third_party_replay/` + 이 문서
- 코퍼스 열: `(plan, expect_sha, reproduced, toolVersion, verdict)`
- 서로 다른 행. 주석 패딩 금지
- `git diff --shortstat upstream/devel` ≥ 100000 insertions
- HARD GATE: `cargo fmt --all -- --check`
- isolation worktree, 브랜치 `feat/v-replay-third-party` from `upstream/devel`
- origin `kevin9327`, 한국어 PR, base `devel`, `closes #5502`

금지:

- `.agents/skills/rhwp-work-receipt` 재작성
- M-rcpt 도구·픽스처 재작성
- `verdict_protocol`, `oracle_vs_self`, `claim_bind`, `best_of_n`,
  `process_steps`, `untrusted_sandbox`
- `gym/`
- `git add -A`
- 금지 워크트리: `rhwp`, `rhwp-desk*`, `rhwp-handoff`, `rhwp-scaffold-final`,
  `rhwp-doc-repro`

## 3. 왜 이 축인가

`replay` 의 attest 모드는 3해시를 **발급**할 뿐 `reproduced=null` 이다.
구현자가 "재실행했다"고 쓰면 검증기가 제3자가 된 것이 아니다.
`--expect-output-sha256` 로 같은 계획 원문을 다시 돌려 `reproduced` 가
true 일 때만 노동을 인정한다. false 면 exit 3 — 도구 고장이 아니라 판정
데이터다.

기존 계약이 이미 신호를 낸다.

- `rhwp replay --json`: `mode`, `expectedOutputSha256`, `reproduced`,
  `toolVersion`, `planSha256`
- workCapsule: `planText` + `receipt.reproduced`
- 64 hex 가 아니면 봉투 없이 exit 2
- `toolVersion` 선대조 (함정 레퍼런스)

이 파동은 그 신호를 묶는 **검증기 래퍼**만 소유한다.

## 4. 트리 요약

`source=prose` 또는 `mode=absent` 이면 `PROSE_NOT_EVIDENCE`.
계획 원문이 없으면 `NO_PLAN`. `toolVersion` 이 비면
`TOOL_VERSION_MISSING`. 주장 버전과 영수증 버전이 다르면 해시 대조 전에
`TOOL_VERSION_MISMATCH`.

`mode=attest` 또는 `reproduced=null` 이고 expect sha 가 없으면
`ATTEST_NOT_THIRD_PARTY` — 발급이지 제3자 검증이 아니다.

verify 인데 expect sha 가 없으면 `NO_EXPECT_SHA`. 64 hex 가 아니면
`INVALID_EXPECT_SHA` (CLI 정규화: 대문자는 lowercase).

그 다음 `reproduced=true` → `LABOR_ACCEPTED`, `false` → `LABOR_REJECTED`.
구현자 산문 열(`implementer_claim`)은 `decide()` 입력이 아니다.

## 5. 검증

```text
python -m unittest discover -s tools/llm_verifier/third_party_replay/tests -v
python tools/llm_verifier/third_party_replay/verify_corpus.py
cargo fmt --all -- --check
```

## 6. 소유 범위

- `tools/llm_verifier/third_party_replay/`
- `mydocs/working/llm_verifier_third_party_replay.md`
