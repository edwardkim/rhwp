# PR #1117 검토 문서

- PR: <https://github.com/edwardkim/rhwp/pull/1117>
- 제목: `render: gate advanced glyph outline payloads`
- 관련 이슈: <https://github.com/edwardkim/rhwp/issues/536>
- 작성일: 2026-05-26
- 작성자: Codex

## 1. PR 상태

| 항목 | 값 |
|---|---|
| 상태 | open |
| base | `devel` |
| head | `render-p19` |
| head sha | `b2134a312d88a461173cfbe45b67fa675ae8810a` |
| mergeable | true |
| 작성자 | `seo-rii` |
| 커밋 수 | 6 |
| 변경 파일 | 14개 |
| 변경량 | +2642 / -48 |

CI 확인:

| workflow | conclusion |
|---|---|
| Build & Test | pass |
| Canvas visual diff | pass |
| CodeQL | pass |
| Analyze (rust) | pass |
| Analyze (javascript-typescript) | pass |
| Analyze (python) | pass |
| WASM Build | skipping |

변경 파일:

```text
README.md
docs/text-ir-v2.md
rhwp-studio/src/core/types.ts
rhwp-studio/src/view/canvaskit-renderer.ts
rhwp-studio/src/view/glyph-outline-payload-status.ts
rhwp-studio/tests/render-backend.test.ts
src/paint/json.rs
src/paint/mod.rs
src/paint/paint_op.rs
src/paint/schema.rs
src/paint/text_v2.rs
src/paint/text_variants.rs
src/renderer/canvaskit_policy.rs
src/renderer/layer_renderer.rs
```

## 2. PR 주장

PR #1117은 Text IR v2 P19 단계로, `GlyphOutline`에 고급 glyph payload vocabulary를 추가한다.

핵심 주장:

```text
1. GlyphOutline payload family 추가
   - colorLayers
   - bitmapGlyph
   - svgGlyph

2. payload family exclusivity와 contract validation 추가

3. COLRv1 graph 중 제한된 subset만 CanvasKit에서 직접 replay
   - solid path
   - linear gradient path
   - radial gradient path
   - full-circle sweep gradient path
   - transform chain ending in one supported leaf

4. unsupported graph/payload는 TextRun fallback 유지

5. CanvasKit/Studio 진단 helper가 같은 reject reason을 쓰도록 정렬
```

## 3. Copilot 지적 처리 상태

Copilot은 총 10개 코멘트를 남겼고, 컨트리뷰터는 모두 `b2134a31`에서 후속 수정했다고 답변했다.

확인한 주요 처리:

```text
1. mixed payload와 empty monochrome paths reject reason 분리
2. sweep gradient는 full-circle only contract로 명시
3. has_colrv1_supported_graph_contract를 canonical path로 정리하고 stage1 alias는 deprecated
4. TypeScript gate에 nodes.length > 64 bound 추가
5. TypeScript leaf glyphRange도 non-empty range로 검증
6. CanvasKit affine matrix helper 추출
7. defensive renderer branch에서 unsupportedColorGlyph diagnostic 기록
8. gradient f64 JSON output을 fixed precision으로 정리
9. feature scan early-return 복구
10. stage1 terminology에 compatibility alias 설명 추가
```

현재 코멘트는 실질적으로 응답/수정된 상태로 보인다.

## 4. 코드 검토 결과

### 4.1 Rust payload contract

`src/paint/paint_op.rs`에 다음 contract가 추가된다.

```text
LayerGlyphOutlinePaint:
  payload_kind
  color_layers
  bitmap_glyph
  svg_glyph

ColorLayersPayload:
  colrV0 resolved layer contract
  colrV1 bounded graph contract

BitmapGlyphPayload:
  strict visual contract

SvgGlyphPayload:
  static sanitized contract
```

COLRv1 graph contract는 다음 방어선을 가진다.

```text
- nodes 비어 있으면 reject
- nodes.len() > 64 reject
- duplicate node id reject
- missing node reject
- cycle reject
- depth > 64 reject
- unsupported node kind reject
- leaf metadata/source range/glyph range 검증
- gradient stop 검증
```

이 구조는 payload vocabulary만 열고 producer/renderer가 임의 graph를 조용히 replay하지 못하게 막는 방향이라 적절하다.

### 4.2 TextRun fallback 유지

`src/renderer/layer_renderer.rs`의 `TextVariantSelectionOptions` 기본값은 advanced payload gate를 닫아 둔다.

```text
allow_colrv0_color_layers = false
allow_colrv1_stage1_color_graph = false
allow_bitmap_glyph = false
allow_svg_glyph = false
```

따라서 public renderer 기본 동작은 바뀌지 않고, CanvasKit policy에서만 COLRv1 stage1 subset을 명시적으로 연다.

```rust
allow_colrv1_stage1_color_graph: true
```

이 설계는 PR 설명의 compatibility 주장과 일치한다.

### 4.3 Rust/TypeScript gate parity

`rhwp-studio/src/view/glyph-outline-payload-status.ts`는 Rust contract와 맞춰 다음 항목을 검증한다.

```text
- payload family exclusivity
- COLRv1 node kind whitelist
- nodes.length <= 64
- graph depth <= 64
- top-level/leaf glyphRange non-empty
- finite affine transform
- gradient stop offset/color validity
- full-circle sweep gradient
```

초기 Copilot 지적이었던 TS gate의 node bound와 empty glyphRange 문제는 현재 head에서 반영되어 있다.

### 4.4 JSON/schema

`src/paint/schema.rs`는 schema minor를 14로 올린다.

```text
schemaVersion: 1
schemaMinorVersion: 14
```

기존 schema major를 유지하고 additive vocabulary만 추가하므로 호환성 방향은 맞다.

`src/paint/json.rs`는 advanced payload를 JSON으로 출력하고, `usedFeatures`/`optionalFeatures`에 payload family를 추가한다.

확인한 안정성 보강:

```text
- gradient numeric fields fixed precision
- feature scan early return 복구
- colorLayers/bitmapGlyph/svgGlyph serialization 분리
```

## 5. 로컬 검증

별도 worktree:

```text
/tmp/rhwp-pr1117
```

실행한 검증:

```text
cargo fmt --check
cargo test --lib colrv1
cargo test --lib serializes_advanced_glyph_outline_payload_gate_metadata
npm --prefix rhwp-studio test -- render-backend
git diff --check devel...origin/pr/1117
```

결과:

```text
success
```

참고:

```text
npm --prefix rhwp-studio run build
```

별도 worktree에서는 generated wasm alias `@wasm/rhwp.js`가 없어서 `tsc`가 중단되었다.
PR CI의 `npm --prefix rhwp-studio run build`는 pass 상태이므로 코드 자체 실패로 보지는 않는다.
실제 반영 후에는 기존 프로젝트 루트에서 wasm 빌드 절차까지 게이트로 확인하는 것이 좋다.

## 6. 위험 평가

낮은 위험:

```text
- default renderer path는 advanced payload를 직접 선택하지 않는다.
- unsupported payload는 TextRun fallback으로 남긴다.
- schema major는 유지되고 additive minor만 증가한다.
- CanvasKit 직접 replay는 COLRv1 제한 subset에만 열려 있다.
```

주의할 점:

```text
- 변경량이 크고 Rust/TS schema parity를 함께 건드린다.
- PR은 full COLRv1/SVG/bitmap glyph replay가 아니라 gate와 subset replay다.
- writer emission은 아직 열지 않는 것이 맞다.
- 반영 후 wasm build와 Studio 쪽 타입 빌드 확인이 필요하다.
```

## 7. 판단

PR #1117은 수용 가능하다.

이유:

```text
1. Compatibility boundary가 명확하다.
2. 기본 renderer selection은 바뀌지 않는다.
3. CanvasKit에서만 제한된 COLRv1 subset을 명시적으로 연다.
4. unsupported payload family는 fallback과 diagnostic으로 처리한다.
5. Copilot의 실질 지적이 후속 커밋에서 반영되었다.
6. CI와 핵심 로컬 검증이 통과했다.
```

## 8. 권장 처리

권장안:

```text
체리픽 수용
```

처리 절차:

```text
1. PR #1117의 6개 커밋을 local/devel에 cherry-pick
2. cargo fmt --check
3. cargo check
4. cargo test --lib colrv1
5. cargo test --lib serializes_advanced_glyph_outline_payload_gate_metadata
6. npm --prefix rhwp-studio test -- render-backend
7. wasm build
8. 통과 후 완료 보고서 작성
```

현재 검토 기준으로는 추가 maintainer-side 코드 수정은 필요하지 않다.
