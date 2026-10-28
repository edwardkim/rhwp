# PR #1416 검토 — strict bitmap/SVG glyph payload corpus 확장

- PR: https://github.com/edwardkim/rhwp/pull/1416
- 제목: `render: widen strict bitmap/svg glyph payload corpus`
- 작성일: 2026-06-16
- 작성자: `seo-rii`
- 관련 이슈: 없음
- base: `devel` (`aeba5db4`)
- head: `seo-rii:render-p24` (`b916a191`)
- 검토 브랜치: `local/pr1416-upstream`

## 1. 요약 판단

**수용 가능**으로 판단한다.

이 PR은 bitmap/SVG glyph replay를 기본 경로로 여는 변경이 아니라, 향후 backend replay를 열기 전에
strict payload가 실제 interned resource를 증명하도록 만드는 schema/selection 기반 공사다.
기본 `TextRun` fallback은 유지되고, backend option이 켜진 경우에도 image bytes 또는 static SVG
fragment가 `ResourceArena`에 없으면 strict variant를 선택하지 않는다.

검토 결과 차단 이슈는 발견하지 못했다. Copilot이 지적한 digest 재계산 비용과 `knownFeatures` 유지보수성
문제도 현재 head에서는 각각 `ResourceArena`의 precomputed resource key, `KNOWN_TEXT_FEATURES` 정적 배열로
반영되어 resolved/outdated 상태다.

다만 PR은 현재 `BEHIND` 상태이므로 승인 후 처리 시 admin merge 또는 maintainer cherry-pick 경로 중 하나를
선택해야 한다. 로컬 병합 시뮬레이션에서는 최신 `origin/devel`과 충돌이 없었다.

## 2. PR 정보

| 항목 | 값 |
|---|---|
| 상태 | open |
| draft | false |
| mergeable | `MERGEABLE` |
| mergeStateStatus | `BEHIND` |
| 변경량 | 6 files, +542 / -50 |
| 작성자 | `seo-rii` |
| closing issues | 없음 |

커밋:

- `f14a5097` — `test(render): widen strict glyph payload corpus`
- `2b94f3de` — `fix(render): require glyph payload resources`
- `b68aae66` — `fix(render): detect glyph payload digest features structurally`
- `4f132e2b` — `fix(render): cache glyph payload resource keys`
- `b916a191` — `fix(render): align glyph digest feature flags`

GitHub checks:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| Canvas visual diff | pass |
| CodeQL Analyze (javascript-typescript) | pass |
| CodeQL Analyze (python) | pass |
| CodeQL Analyze (rust) | pass |
| CodeQL | pass |
| WASM Build | skipped |

## 3. 변경 검토

### 3.1 ResourceArena 확장

`src/paint/resources.rs`:

- 기존 font blob interning 경로와 같은 방식으로 image bytes, SVG fragment interning 추가
- `ImageResourceId`, `SvgResourceId`에 대해 bytes/string 조회, hash, fingerprint, resource key, iterator API 제공
- resource key는 `kind:blake3:length:digest` 형식으로 유지
- 중복 리소스는 FNV hash 후보군 + 실제 byte/string 비교로 dedupe

이 구조는 Copilot의 per-glyph digest 재계산 지적을 적절히 해결한다. digest string은 `intern_*` 시점에
계산되어 저장되고, JSON export에서는 `image_resource_key(id)` / `svg_resource_key(id)` 조회만 수행한다.

### 3.2 Payload resource key와 JSON schema

`src/paint/paint_op.rs`, `src/paint/json.rs`, `src/paint/schema.rs`:

- `LayerGlyphOutlinePaint::payload_resource_key_with_resources()` 추가
- bitmap/SVG strict payload의 `payloadResourceKey`에 interned resource key를 `:resource:...`로 추가
- SVG glyph JSON에 `vectorResourceId`를 `svgRef`와 함께 출력
- `knownFeatures`를 정적 배열 `KNOWN_TEXT_FEATURES`로 분리
- `text.glyphOutline.payloadResourceDigestKey`, `text.glyphOutline.svgGlyph.vectorResourceId` feature 추가
- schema minor version `16 -> 17`

`payloadResourceDigestKey`는 실제 interned resource가 있는 bitmap/SVG glyph에서만 `usedFeatures`와
`optionalFeatures`에 나타난다. color layer의 문자열에 `:resource:`가 포함되어도 digest feature로 오인하지
않도록 테스트가 추가되어 있다.

### 3.3 Variant selection gate

`src/renderer/layer_renderer.rs`:

- `analyze_text_variant_selection()`이 `PageLayerTree.resources`를 variant evaluation에 전달
- bitmap/SVG glyph payload가 strict contract를 만족하고 backend option이 켜져 있어도, resource가 없으면
  `missingGlyphPayloadResource`로 reject
- invalid bitmap scaling policy나 unsafe SVG flag는 기존 family별 unsupported reason으로 fallback 유지

이 변경은 PR 본문 compatibility 설명과 일치한다. 즉 "resource identity를 증명한 strict sidecar만 선택 가능"이라는
계약을 selection 단계에 반영한다.

### 3.4 문서 변경

`docs/text-ir-v2.md`:

- P24 strict bitmap/SVG glyph producer corpus 섹션 추가
- image/SVG resource interning, digest key, vectorResourceId, strict 조건, fallback 유지 방침을 문서화

## 4. 로컬 검증

검토 기준:

- `origin/devel`, `devel`, `local/devel`: `aeba5db4`
- PR head: `local/pr1416-upstream` `b916a191`

| 명령 | 결과 |
|---|---|
| `git diff --check origin/devel...HEAD` | 통과 |
| 최신 `origin/devel` no-commit merge simulation | 통과, 충돌 없음 |
| `cargo fmt --check` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib paint::json -- --nocapture` | 통과, 13 passed |
| `CARGO_INCREMENTAL=0 cargo test --lib renderer::layer_renderer -- --nocapture` | 통과, 18 passed |
| `CARGO_INCREMENTAL=0 cargo check --lib` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib paint::resources -- --nocapture` | 통과, 4 passed |

## 5. 리스크

| 리스크 | 평가 | 비고 |
|---|---|---|
| schema minor bump | 낮음 | additive feature 확장, v1 fallback 유지 |
| JSON feature flag 불일치 | 낮음 | structural detection + `KNOWN_TEXT_FEATURES` 중복 테스트 있음 |
| strict bitmap/SVG selection 오동작 | 낮음~중간 | resource missing 시 fallback 테스트 추가됨 |
| resource memory 증가 | 낮음 | 현재 producer corpus/fixture 경로 중심. 향후 실제 문서 추출 연결 시 상한 정책 검토 필요 |
| PR `BEHIND` 상태 | 낮음 | 로컬 병합 시뮬레이션 충돌 없음 |
| 관련 이슈 미기재 | 낮음 | 내부 schema groundwork 성격. 자동 close 대상 없음 |

보안 관점:

- SVG glyph strict 조건은 `static_sanitized=true`, script/animation/external/interactivity false를 요구한다.
- 실제 resource가 없으면 strict variant를 선택하지 않는다.
- 이 PR 자체는 외부 SVG loading이나 직접 replay 경로를 열지 않는다.

## 6. 권장 처리

권장: **수용 가능**.

작업지시자 승인 후:

1. PR #1416 수용 절차 진행
2. `BEHIND` 상태이므로 admin merge 또는 local maintainer merge/cherry-pick 중 처리 방식 결정
3. 처리 후 `local/devel` 동기화
4. 처리 보고서 `mydocs/pr/pr_1416_report.md` 작성
5. 관련 이슈 자동 close 대상은 없으므로 PR 처리 코멘트 중심으로 종료

## 7. 승인 요청

위 검토 결과 기준으로 PR #1416 수용 절차를 진행해도 되는지 승인 요청한다.
