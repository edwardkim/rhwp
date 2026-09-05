# V-fresh — toolVersion이 다르면 재현 합격을 인정하지 않는다

Issue #5518. 영수증 `toolVersion` 이 검증기 바이너리 버전과 다르면
`reproduced:true` 라도 합격이 아니다 (낡은 도구).

이 디렉터리는 기존 `receipt.toolVersion` 계약을 **감싸는 검증기**다.
새 rhwp CLI 를 만들지 않는다.
`.agents/skills/rhwp-work-receipt` 와 V-replay 를 재작성하지 않는다.

## 축

`(attest_version, verify_version, reproduced, accepted)`

| 이유 | 뜻 | accepted |
|---|---|---|
| `FRESH_REPRODUCED` | 버전 일치 + `reproduced=true` | true |
| `FRESH_NOT_REPRODUCED` | 버전 일치 + `reproduced=false` | false |
| `FRESH_ABSENT` | 버전 일치 + 재현 주장 없음 | false |
| `STALE_TOOL` | 버전 불일치 + `reproduced=true` | **false** |
| `STALE_AND_NOT_REPRODUCED` | 버전 불일치 + `reproduced=false` | false |
| `STALE_AND_ABSENT` | 버전 불일치 + 재현 주장 없음 | false |
| `ATTEST_VERSION_MISSING` | 영수증 버전 공란 | false |
| `VERIFY_VERSION_MISSING` | 검증기 버전 공란 | false |

V-replay 는 같은 버전으로 계획을 다시 돌리는 축이다. 이 게이트는
**버전이 다른 도구** 가 `reproduced:true` 를 들고 와도 인정하지 않는다.

대조는 trim 후 문자열 식별이다. `0.8.4` 와 `0.8.4+git.abc`,
`v0.8.4`, `rhwp 0.8.4` 는 서로 다른 바이너리다.

## 검증

```text
python tools/llm_verifier/tool_version_gate/generate_corpus.py
cargo test --manifest-path tools/llm_verifier/tool_version_gate/Cargo.toml
cargo fmt --all -- --check --manifest-path tools/llm_verifier/tool_version_gate/Cargo.toml
```

루트 워크스페이스에 이 크레이트를 넣지 않는다. 소유 범위 밖이다.
