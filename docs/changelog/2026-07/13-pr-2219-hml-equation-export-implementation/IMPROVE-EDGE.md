# IMPROVE EDGE CASES - PR 2219 HML equation and export

## Outcome

Edge pass는 명세로 확정된 손실 진단과 RPC atomicity의 두 빈틈을 보강했다.

1. 미지원 `SCRIPT` attribute 진단이 이제 XML entity를 decode한 `name=value`를 path/code와 함께 남긴다. 이전 구현은 value를 버려 REQ-HML-EQ-001의 durable path/name/value 계약을 충족하지 못했다.
2. `exportHml` handler 실패가 partial `result`/bytes 없이 `RPC_ERROR` envelope만 반환하는 회귀 테스트를 추가했다. 구현은 이미 NFR-03 atomicity 계약을 만족해 source 변경이 필요하지 않았다.

No commit or push was performed.

## Edge audit

- Empty/missing/duplicate/nested `SCRIPT`: duplicate/nested는 first-direct-only 및 exact blocker가 고정돼 있다. Empty와 missing은 현재 같은 empty script IR로 canonicalize된다. 차단 여부는 명세가 정하지 않아 변경하지 않았다.
- Unknown attrs/children: equation attr, SCRIPT attr, unknown child가 durable `UnsupportedEquationSemantics` warning으로 남고 edit 후 blocker로 유지된다. 이번 수정으로 SCRIPT attr value도 보존된다.
- XML: Text, CDATA, general/entity reference, export escaping, illegal XML 1.0 characters, malformed nesting이 parser/serializer tests로 고정돼 있다.
- Offsets/layout/hit: 8-unit slot, interleaved text offsets, intrinsic non-zero bbox, following text non-overlap, hit target, stale offset aggregation이 고정돼 있다.
- Attributes: evidenced negative baseline, BaseUnit 1200, asymmetric color, Font/Version entity decoding 및 export/reparse가 고정돼 있다.
- Edit/state: public equation edit recomputes intrinsic size; unknown import metadata survives equation edit and blocks export. Studio equation property apply already records undo snapshots; metadata is owned outside Document snapshots.
- Non-HML/protocol: non-HML save-state/export is typed-blocked; v1 remains additive; old transferable-only peer still connects; transfer copies preserve handler-owned bytes; malformed/version/session/error envelopes and all transferred-port cleanup are covered.

## RED to GREEN

- RED: `cargo test --test hml_parser equation_accepts_only_the_first_direct_script_and_warns_for_duplicates_and_nested_scripts` failed because the warning message was only `FutureMode`.
- GREEN: the same test passed with decoded `FutureMode=matrix&inline` retained.
- GREEN: `cargo test --test hml_parser --test hml_serializer` - parser 32/32, serializer 29/29.
- GREEN: `cd rhwp-studio && npm test` - 269/269, including error-only `exportHml` response.
- GREEN: `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `git diff --check`.

## Decision gates / residual risk

1. Missing `SCRIPT`: P0 says a single direct SCRIPT is supported but does not say whether an absent SCRIPT is invalid, blocked, or equivalent to empty. Current import accepts it and canonical export emits an empty SCRIPT. Corpus/schema evidence or a product decision is required before changing that compatibility behavior.
2. Numeric domains: `BaseUnit=0`, `BaseUnit > i32::MAX`, colors above `0x00FF_FFFF`, and property-edit baseline values outside `i16` lack an HWPML validity/clamp/block rule. Renderer paths contain signed conversions, so arbitrary extremes should not be declared supported without schema/corpus evidence. No guessed clamp or blocker was added.
3. Capability visibility: low-level parents can inspect `rhwp-connected.capabilities` as specified. `@rhwp/editor` does not expose negotiated capabilities publicly; adding such an API is a separate product-contract choice, not required by the frozen API.
4. Fresh-context delegation was attempted but the parent run had reached its agent-thread limit; this isolated executor completed the audit directly.
