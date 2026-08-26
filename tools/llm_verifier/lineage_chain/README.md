# V-lineage — 부모 산출=자식 입력 해시 사슬만 인정

Issue #5516. 구현자 산문은 증거가 아니다.
`rhwp lineage --json` 봉투의 `parentOk` · `lineageOk` · `brokenAt` 과
부모 `outputSha256` / 자식 `inputSha256` 만 본다.

이 디렉터리는 기존 lineage 계약을 **감싸는 검증기**다.
`.claude/skills/rhwp-work-receipt` 를 재작성하지 않는다.
단건 `rhwp replay --expect-output-sha256` / `reproduced` 는 V-replay 축이며
여기서 재구현하지 않는다.

## 축

`(parent_out, child_in, parentOk, lineageOk, brokenAt) -> verdict`

| verdict | 뜻 |
|---|---|
| `CHAIN_ACCEPTED` | `parent_out == child_in` + `parentOk=true` + `lineageOk=true` + `brokenAt` 없음 |
| `LINEAGE_BROKEN` | 부모 산출 ≠ 자식 입력 / `lineageOk=false` (`brokenAt` 명세) |
| `PARENT_TAMPERED` | `parentOk=false` — 부모 파일 바이트 변조 |
| `ROOT_ONLY` | 뿌리. 두 축이 null. 사슬 주장 없음 |
| `PROSE_NOT_EVIDENCE` | lineage 봉투 없음 |
| `HEAD_MISSING` | 머리 캡슐 IO, exit 1 |
| `USAGE` | 사용법, exit 2 |
| `PARENT_SHA_MISSING` | `parent.sha256` 누락·비hex |
| `PARENT_FIELD_MISSING` | `parent` 키 자체 없음 (합법 뿌리와 다름) |
| `KIND_NOT_CAPSULE` | `kind != workCapsule` |
| `HASH_DEFECT` | 64 hex 가 아니거나 사슬 주장에 해시 없음 |
| `ENVELOPE_CONTRADICTS` | 봉투 필드가 해시 등식과 모순 |

## 검증

```text
python -m unittest discover -s tools/llm_verifier/lineage_chain/tests -v
python tools/llm_verifier/lineage_chain/verify_corpus.py
```
