# V-replay — 제3자 재실행으로만 노동을 인정

Issue #5502. 구현자 산문은 증거가 아니다.
`rhwp replay --expect-output-sha256` 봉투의 `reproduced` 와
workCapsule `receipt.reproduced` 만 본다.

이 디렉터리는 기존 replay 계약을 **감싸는 검증기**다.
`.agents/skills/rhwp-work-receipt` 와 M-rcpt 픽스처를 재작성하지 않는다.

## 축

`(plan, expect_sha, reproduced, toolVersion) -> verdict`

| verdict | 뜻 |
|---|---|
| `LABOR_ACCEPTED` | verify + 64hex expect + `reproduced=true` + toolVersion |
| `LABOR_REJECTED` | verify + `reproduced=false` (exit 3) |
| `ATTEST_NOT_THIRD_PARTY` | attest / `reproduced=null` — 발급이지 제3자 검증이 아님 |
| `PROSE_NOT_EVIDENCE` | replay/capsule 봉투 없음 |
| `NO_EXPECT_SHA` | `--expect-output-sha256` 없음 |
| `INVALID_EXPECT_SHA` | 64 hex 계약 위반 |
| `TOOL_VERSION_MISSING` | `toolVersion` 공란 |
| `TOOL_VERSION_MISMATCH` | 주장 버전 ≠ 영수증 버전 |
| `NO_PLAN` | 계획 원문 없음 |

## 검증

```text
python -m unittest discover -s tools/llm_verifier/third_party_replay/tests -v
python tools/llm_verifier/third_party_replay/verify_corpus.py
```
