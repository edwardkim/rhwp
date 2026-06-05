# PR #1159 처리 보고서 — render: add glyph payload resource proof diagnostics

- **작성일**: 2026-05-31
- **PR**: #1159 → **MERGED** (devel, 머지커밋 `8f3d2e91`)
- **컨트리뷰터**: @seo-rii (Seohyun Lee) — PageLayerTree / glyph payload replay 시리즈 (P19→P20)
- **연결 이슈**: `Refs #536` (Closes 아님 — 자동 클로즈 없음, z-order 후속은 별도 sweep)
- **판단**: **머지** ✅ (작업지시자 승인)

## 결정 사유

P19(color layer / bitmap glyph / sanitized SVG glyph payload vocabulary + COLRv1 일부 CanvasKit
replay gate)의 후속. P20 은 실제 replay 를 더 열기 전 **proof/diagnostics 단계**로,
payload resource identity 분리와 native font construction 실패 이유 명시를 담당. 전부
**additive opt-in**, 기본 renderer 경로 불변, glyph-id replay 는 여전히 비활성. 위험 낮음, 검증 통과.

## 변경 요약 (10 파일, +1039 / −76)

| 파일 | 변경 |
|------|------|
| `src/paint/paint_op.rs` (+211) | `payload_resource_key()`/`has_payload_resource_key()` — family prefix(`glyphPayload:bitmapGlyph:imageRef:` vs `:svgGlyph:svgRef:`)로 numeric ref 충돌 방지, strict contract 게이트 |
| `src/paint/resources.rs` (+54) | `font_blob_ref_lookup` (intern 시점 구축) + `font_blob_bytes_for_ref()` 재해시 없는 조회, blake3 fingerprint 통합 |
| `src/paint/schema.rs` (±2) | layer 14→15 / resource 3→4 (additive minor) |
| `src/paint/json.rs` (+201) | `payloadResourceKey` 직렬화(Some 일 때만 emit) + feature flag + 회귀 2건 |
| `src/renderer/skia/renderer.rs` (+287) | `NativeGlyphRunReplayProof`(22 사유) — contract(portable) vs construction(typeface) 분리, BTreeSet 일괄·정렬 보고. `can_replay`=`typeface_constructible` → `TypefaceConstructionNotImplemented` 항상 추가로 **replay 여전히 비활성**. 회귀 4건 |
| `rhwp-studio/{core/types.ts, view/glyph-outline-payload-status.ts(+151 신규)}` | Rust 와 동일 family prefix key contract 미러(echo+derive, fabricate 안 함) |
| `rhwp-studio/tests/render-backend.test.ts` (+108) | family disjoint / palette 변경 시 key 분리 / incomplete 억제 |
| `README.md` / `docs/text-ir-v2.md` | P20 항목 + schema bump + #536 후속 메모 |

## 검증 결과

| 단계 | 명령 | 결과 |
|------|------|------|
| merge | `git merge --no-ff` | ✅ CLEAN (충돌 0) |
| fmt | `cargo fmt --all --check` | ✅ clean |
| clippy(lib) | `cargo clippy --lib` | ✅ 0 warning/error |
| build | `cargo build` | ✅ Finished |
| 전체 테스트 | `cargo test --tests` | ✅ **1879 passed, 0 failed** (99 스위트) |
| native-skia proof | `cargo test --features native-skia --lib native_skia_glyph_run` | ✅ 4 passed |
| studio | `npm --prefix rhwp-studio test -- render-backend` / `run build` | ✅ pass / built |
| CI(PR) | Build&Test / Canvas diff / CodeQL / Analyze ×3 | ✅ 전부 SUCCESS |

## 처리 절차

1. PR 정보 확인 — MERGEABLE / **BEHIND** (head 정합, base 앞섬). 연결 `Refs #536`, CI green.
2. 컨트리뷰터 사이클 점검 — @seo-rii PageLayerTree/glyph 시리즈. 관련 열린 PR 무충돌.
3. 4개 소스 diff(paint/json/skia/studio) + docs/helper/test 전수 검토. `_` placeholder/생략 없음 확인.
4. 로컬 `pr1159-verify` 머지 시뮬레이션 전체 검증 통과 → `pr_1159_review.md` 작성 → 승인.
5. 메인테이너 로컬 `--no-ff` 머지(`a2a5593c..8f3d2e91`) + 머지 후 재검증 + push. PR head 조상 확인.
6. PR close(머지 명시) + 보고서 + orders 갱신.

## 비고

- BEHIND 이므로 GitHub UI 머지 불가 → 메인테이너 로컬 통합. cross-repo `--no-ff` 라
  GitHub 자동 'Merged' 표시 안 됨(#1178/#1180/#1182 동일 패턴) — 실제 머지 완료.
- `Refs #536` 이므로 자동 클로즈 없음. z-order 후속(normalized PageLayerTree replay iterator)은
  별도 renderer sweep 범위.
- native Skia glyph-id replay 미활성(proof 단계)이라 시각 회귀 위험 거의 없음.
- @seo-rii 의 PageLayerTree/CanvasKit replay 누적 기여(P15~P20)의 일부.
