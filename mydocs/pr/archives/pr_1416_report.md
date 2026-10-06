# PR #1416 처리 보고서 — strict bitmap/SVG glyph payload corpus 확장

## 1. 개요

| 항목 | 내용 |
|---|---|
| PR | #1416 |
| 제목 | `render: widen strict bitmap/svg glyph payload corpus` |
| 작성자 | `seo-rii` |
| 관련 이슈 | 없음 |
| PR base | `devel` (`aeba5db4`) |
| 원 PR head | `b916a191` |
| 처리 기준 | `local/devel` |
| 통합 방식 | PR 커밋 5개 cherry-pick |
| 리뷰/처리 문서 커밋 | `a3fc9be5` |
| devel merge | `a60ea2ad` |
| PR 처리 코멘트 | https://github.com/edwardkim/rhwp/pull/1416#issuecomment-4713571383 |
| PR close | `2026-06-16T00:07:07Z` |
| 처리 판정 | 수용 가능 |

## 2. 처리 내용

작업지시자 승인에 따라 PR #1416의 원 커밋 5개를 현재 `local/devel` 위에 cherry-pick했다.
충돌은 발생하지 않았다.

원 PR 커밋:

```text
f14a5097 test(render): widen strict glyph payload corpus
2b94f3de fix(render): require glyph payload resources
b68aae66 fix(render): detect glyph payload digest features structurally
4f132e2b fix(render): cache glyph payload resource keys
b916a191 fix(render): align glyph digest feature flags
```

`local/devel` 반영 커밋:

```text
c8368160 test(render): widen strict glyph payload corpus
ba4151a2 fix(render): require glyph payload resources
295e5b61 fix(render): detect glyph payload digest features structurally
db68ef6e fix(render): cache glyph payload resource keys
f83e25cc fix(render): align glyph digest feature flags
```

## 3. 변경 내용

`src/paint/resources.rs`:

- `ResourceArena`에 image bytes와 static SVG fragment interning 추가
- image/SVG resource key, hash, fingerprint, iterator API 추가
- resource key를 `kind:blake3:length:digest` 형식으로 precompute하여 저장

`src/paint/paint_op.rs`, `src/paint/json.rs`:

- bitmap/SVG glyph payload의 `payloadResourceKey`에 interned resource key를 결합
- SVG glyph JSON에 `vectorResourceId` alias 추가
- `knownFeatures`를 `KNOWN_TEXT_FEATURES` 정적 배열로 분리
- `text.glyphOutline.payloadResourceDigestKey`, `text.glyphOutline.svgGlyph.vectorResourceId` feature 노출

`src/renderer/layer_renderer.rs`:

- variant selection이 `PageLayerTree.resources`를 참조하도록 변경
- backend option이 켜져 있어도 bitmap/SVG payload resource가 없으면 `missingGlyphPayloadResource`로 reject
- invalid bitmap/SVG payload는 기존 fallback 경로 유지

`src/paint/schema.rs`, `docs/text-ir-v2.md`:

- schema minor version `16 -> 17`
- P24 strict bitmap/SVG glyph producer corpus 문서화

## 4. 검증 결과

GitHub checks:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| Canvas visual diff | pass |
| CodeQL | pass |
| Analyze rust | pass |
| Analyze javascript-typescript | pass |
| Analyze python | pass |
| WASM Build | skipped |

체리픽 전 로컬 검증:

| 명령 | 결과 |
|---|---|
| `git diff --check origin/devel...HEAD` | 통과 |
| 최신 `origin/devel` no-commit merge simulation | 통과, 충돌 없음 |
| `cargo fmt --check` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib paint::json -- --nocapture` | 통과, 13 passed |
| `CARGO_INCREMENTAL=0 cargo test --lib renderer::layer_renderer -- --nocapture` | 통과, 18 passed |
| `CARGO_INCREMENTAL=0 cargo check --lib` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib paint::resources -- --nocapture` | 통과, 4 passed |

체리픽 후 로컬 검증:

| 명령 | 결과 |
|---|---|
| PR 커밋 5개 cherry-pick | 통과, 충돌 없음 |
| `git diff --check origin/devel..HEAD` | 통과 |
| `git diff --check -- mydocs/orders/20260616.md mydocs/pr/pr_1416_review.md mydocs/pr/pr_1416_report.md` | 통과 |
| `cargo fmt --check` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib paint::json -- --nocapture` | 통과, 13 passed |
| `CARGO_INCREMENTAL=0 cargo test --lib renderer::layer_renderer -- --nocapture` | 통과, 18 passed |
| `CARGO_INCREMENTAL=0 cargo test --lib paint::resources -- --nocapture` | 통과, 4 passed |
| `CARGO_INCREMENTAL=0 cargo check --lib` | 통과 |

## 5. 판정

**수용 가능**.

이 PR은 bitmap/SVG glyph replay를 기본 활성화하지 않고, strict payload가 실제 interned resource를
증명해야 선택될 수 있도록 resource identity와 selection gate를 강화한다. 기본 `TextRun` fallback은
유지되며, resource가 없거나 payload strict 조건을 만족하지 못하면 안전하게 fallback한다.

보안 관점에서도 SVG glyph는 static sanitized contract와 script/animation/external/interactivity false 조건을
요구하고, 이 PR 자체가 외부 SVG loading 또는 직접 replay path를 열지 않는다.

## 6. 남는 범위

- 이 PR은 resource identity와 strict selection 조건을 고정하는 groundwork이다.
- CanvasKit/native Skia의 bitmap/SVG glyph 직접 replay는 열지 않는다.
- 실제 document image/SVG extraction 전체 연결과 resource table 상한 정책은 후속 단계에서 별도 검토가 필요하다.
- 연결된 issue가 없으므로 자동 close 대상은 없다.

## 7. 후속 절차

처리 완료:

- [x] 리뷰/처리 보고서와 주문서 갱신 커밋 — `a3fc9be5`
- [x] `local/devel`을 `devel`에 no-ff merge — `a60ea2ad`
- [x] `origin/devel` push — `a60ea2ad`
- [x] PR #1416에 cherry-pick 반영 및 검증 결과 코멘트 작성 — https://github.com/edwardkim/rhwp/pull/1416#issuecomment-4713571383
- [x] PR #1416 close — `2026-06-16T00:07:07Z`
