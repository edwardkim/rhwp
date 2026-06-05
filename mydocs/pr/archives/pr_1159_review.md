# PR #1159 검토 — render: add glyph payload resource proof diagnostics

- **작성일**: 2026-05-31
- **PR**: #1159 (OPEN)
- **컨트리뷰터**: @seo-rii (Seohyun Lee) — PageLayerTree / glyph payload replay 시리즈 (P19→P20)
- **연결 이슈**: `Refs #536` (Closes 아님 — 자동 클로즈 없음, #536 은 z-order 후속 메모만 README 반영)
- **base/head**: `devel` ← `render-p20` (`e4574eab`)
- **mergeable**: MERGEABLE / **BEHIND** (head 는 정합, base 가 앞서감 → 로컬 머지 필요)
- **규모**: 10 파일, +1039 / −76
- **CI**: 전부 SUCCESS (Build&Test / Canvas visual diff / CodeQL / Analyze ×3). WASM 만 skip.
- **마일스톤**: v1.0.0 / 라벨: enhancement

## 1. PR 정보 확인

P19(color layer / bitmap glyph / sanitized SVG glyph payload vocabulary + COLRv1 일부 CanvasKit
replay gate)의 후속. P20 은 **실제 replay 를 더 열기 전 단계**로, (a) payload resource identity 분리,
(b) native font construction 실패 이유 명시를 담당하는 **proof/diagnostics 단계**다.

핵심 목표 3가지:
1. payload family 가 다른데 numeric ref 만 같아서 cache key 가 충돌하는 상황 방지
2. font blob ref 는 있으나 bytes 가 없는 상황을 조용히 통과시키지 않음
3. TTC/OTC face index, variation axis 때문에 native Skia exact typeface 구성이 막히는 이유를 분리 보고

## 2. 변경 내용 검토

### (1) `src/paint/paint_op.rs` (+211) — payload_resource_key
- `payload_resource_key()` / `has_payload_resource_key()` 추가.
- **strict contract 게이트**: monochrome 는 항상 None, color/bitmap/svg 는 각각
  `has_colrv0/v1_*_contract` / `has_strict_visual_contract` / `has_static_sanitized_contract`
  를 만족할 때만 key 생성 → **incomplete payload 는 key 없음** (review follow-up: `-` placeholder 충돌 회피).
- payload family 별 prefix(`glyphPayload:colorLayers:` / `:bitmapGlyph:imageRef:` / `:svgGlyph:svgRef:`)
  로 **숫자 ref 충돌 방지** — PR 목적 정확히 달성.
- 부동소수 `{:.6}` 고정 포맷으로 결정성 확보.
- [x] family prefix 로 ref 충돌 차단
- [x] strict contract 미달 시 key 미생성

### (2) `src/paint/resources.rs` (+54) — font blob ref lookup
- `font_blob_ref_lookup: HashMap<String, FontBlobResourceId>` 를 **intern 시점에 구축**,
  `font_blob_bytes_for_ref()` 로 lookup 시 재해시 없이 조회 (review follow-up 반영).
- `kind != FontBlob` 이면 거부. fingerprint 를 blake3 로 통합(이전 `resource_fingerprint` 대체).
- 회귀 테스트 1건(versioned ref resolve + kind/len mismatch 거부).

### (3) `src/paint/schema.rs` (+2/−2) — additive minor bump
- layer 14→15, resource table 3→4. **additive minor** (PR 본문 일치).

### (4) `src/paint/json.rs` (+201) — payloadResourceKey 직렬화
- feature flag(`has_glyph_outline_payload_resource_keys`) 를 `has_payload_resource_key()` 로 집계,
  optional/known features 에 `text.glyphOutline.payloadResourceKey` 추가.
- `PaintOp::GlyphOutline` 직렬화에서 key 가 `Some` 일 때만 emit (**additive**, 기존 필드 보존).
- 회귀 테스트 2건(colorLayers + bitmap/svg, incomplete payload key 없음 검증 포함).

### (5) `src/renderer/skia/renderer.rs` (+287) — replay proof 정교화 (핵심)
- `NativeGlyphRunReplayProofReason`(22 사유) + `NativeGlyphRunReplayProof { contract_replayable, typeface_constructible, reasons }`.
- 기존 early-return 누적식 → `BTreeSet` 누적으로 변경 → **모든 실패 사유를 한 번에, 전역 정렬로** 보고.
- **계약(contract) vs 구성(construction) 분리**: `ReplayEligibilityNotPortable` ↔ `FontBlobNotPortable`
  분리, `FontBlobBytesMissing` 를 `font_blob_bytes_for_ref` 로 검출, face_index≠0 / variation 은
  construction_reasons 로 격리.
- **호환성 핵심**: `native_skia_can_replay_glyph_run` 은 `typeface_constructible` 반환 →
  contract 가 replayable 해도 `TypefaceConstructionNotImplemented` 가 항상 추가되어
  **실제 replay 는 여전히 비활성**. PR 본문 "blocker 분리 proof 단계, replay enable 안 함" 과 정확히 일치.
- 회귀 테스트 4건(missing bytes / eligibility vs portability 분리 / face_index+variation / missing face).

### (6) rhwp-studio (+160) + docs (+24)
- `core/types.ts`: `payloadResourceKey?: string` 옵셔널 필드 추가(주석 포함).
- `view/glyph-outline-payload-status.ts`(151, 신규): Rust contract 미러. exporter 가 emit 한 key 를
  echo + 테스트 검증용 derive helper(family prefix 동일). **fabricate 안 함**.
- `tests/render-backend.test.ts`(+108): echo / derive+family distinct / **refs collide 시 family 분리** 3 시나리오.
- README / docs/text-ir-v2.md: P20 항목 + schema bump + #536 후속 메모(코드 범위 아님).

## 3. 위험 평가

- **낮음.** 전부 **additive opt-in diagnostics**. 기본 renderer 경로 불변, unsupported payload 는
  `TextRun` fallback 유지, native Skia glyph-id replay 는 여전히 비활성(proof 단계).
- schema 는 additive minor bump 으로 하위 호환.
- `_` placeholder / 생략 코드 없음 확인 (json 201줄 / skia 287줄 / helper 151줄 실제 추가).
- feedback_image_renderer_paths_separate: 이 PR 은 paint IR + skia + studio 경로만 건드리며
  svg.rs/web_canvas.rs 의 이미지 함수와 무관(glyph payload key 는 export metadata 경로).

## 4. 검증 결과 (로컬, 머지 시뮬레이션 `pr1159-verify`)

| 단계 | 명령 | 결과 |
|------|------|------|
| merge | `git merge --no-ff` | ✅ CLEAN (충돌 0) |
| fmt | `cargo fmt --all --check` | ✅ clean |
| clippy(lib) | `cargo clippy --lib` | ✅ 0 warning/error |
| 전체 테스트 | `cargo test --tests` | ✅ **1856 passed, 0 failed** |
| native-skia proof | `cargo test --features native-skia --lib native_skia_glyph_run` | ✅ 6 passed, 0 failed |
| studio | `npm --prefix rhwp-studio test -- render-backend` | ✅ 1 file passed |
| studio build | `npm --prefix rhwp-studio run build` | ✅ built |
| CI(PR) | Build&Test / Canvas diff / CodeQL / Analyze ×3 | ✅ 전부 SUCCESS |

## 5. 판단 (예정)

전체 검증 통과 → **머지** 권고. BEHIND 이므로 메인테이너 로컬 `--no-ff` 머지 + push.
`Refs #536` 이므로 자동 클로즈 없음(#536 은 별도 후속). 결과는 `pr_1159_report.md` 에 기록.
