---
PR: #797
제목: render — add Text IR v2 compatibility contract (P11)
컨트리뷰터: @seo-rii (Seohyun Lee) — 10번째 사이클 (Skia 핵심 컨트리뷰터)
처리: 옵션 A — 1 commit cherry-pick + no-ff merge
처리일: 2026-05-11
머지 commit: 098db015
Refs: #536
---

# PR #797 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (1 commit cherry-pick + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `098db015` (--no-ff merge) |
| Cherry-pick commit | `795a132d` |
| Refs | Issue #536 (Skia native raster 트래킹) |
| 시각 판정 | 면제 (작업지시자 결정 — contract 정합 단계 + 결정적 검증) |
| 자기 검증 | cargo build/test/clippy + native-skia 28/28 + sweep 170/170 same |

## 2. 본질

P11 단계 — Text IR v2 **compatibility contract** 추가. P9 영역 영역 text replay parity 후속 영역 영역 GlyphRun / font resource / native glyph replay 영역 영역 schema contract 미리 여는 단계.

**중요**: 본 PR 영역 영역 GlyphRun 영역 영역 기본 경로 전환 부재 — TextRun fallback 유지 + Additive schema. 실제 GlyphRun 영역 영역 P12+ 영역 영역 분리 (Still designing 명시).

### 2.1 Skia native raster 단계적 진전 (Issue #536)
| 단계 | PR | 본질 |
|------|-----|------|
| P4 | #599 | native Skia PNG raster backend |
| P5 | #626 | equation replay |
| P6 | #720 | raw SVG fragment replay |
| P8 | #761 | schema/resource hardening |
| P9 | #769 | text replay parity + module split |
| **P11** | **#797 (5/11)** | **Text IR v2 compatibility contract** |

## 3. 4 본질 원칙 (PR 본문 명시)

| 원칙 | 본질 |
|------|------|
| **Compatibility first** | 모든 backend 가 TextRun fallback 으로 렌더링 가능 |
| **Additive schema** | schemaMinorVersion + feature negotiation + text metadata 추가 (기존 consumer 미파괴) |
| **Source traceability** | PageLayerTree.text_sources + TextRun.source span (text op 원문 범위 추적) |
| **Placement/cluster metadata** | paintStyle / projectionKind / orientation / placement / clusterBasis / clusters / legacyVisuals |

## 4. 정정 본질 — 12 files, +1173/-35

### 4.1 핵심 변경
| 파일 | 변경 | 본질 |
|------|------|------|
| `src/paint/json.rs` | +687/-14 | schemaMinorVersion + feature negotiation + text metadata + special visual ops export |
| `src/paint/builder.rs` | +151/-10 | LayerBuilder 영역 영역 explicit visual op 생성 |
| `src/paint/layer_tree.rs` | +170/-1 | Text IR v2 metadata + text_sources + TextSourceTable |
| `src/paint/paint_op.rs` | +45 | 4 신규 PaintOp variant |
| `src/paint/schema.rs` | +15/-2 | schemaMinorVersion + resourceTableMinorVersion |
| `src/paint/mod.rs` | +5/-2 | 재export |

### 4.2 Renderer 정정 — 4 backend 동기 (`feedback_image_renderer_paths_separate` 권위 사례)
| 파일 | 변경 |
|------|------|
| `src/renderer/svg_layer.rs` | +13/-4 |
| `src/renderer/canvas.rs` | +4 |
| `src/renderer/skia/renderer.rs` | +4 |
| `src/renderer/web_canvas.rs` | +10/-1 |

→ 신규 special visual op (CharOverlap / TextControlMark / TabLeader / TextDecoration) 영역 영역 기존 backend 영역 영역 skip — double-painting 방지 (기존 TextRun payload 영역 영역 이미 같은 visual 렌더링).

### 4.3 문서
- `docs/text-ir-v2.md` (+67) — Text IR v2 migration contract 명시
- `README.md` (+2/-1) — P11 범위 + GlyphRun 후속 분리 명시

## 5. 신규 인프라

### 5.1 PaintOp 4 신규 variant
- `PaintOp::CharOverlap`
- `PaintOp::TextControlMark`
- `PaintOp::TabLeader`
- `PaintOp::TextDecoration`

### 5.2 Text IR v2 metadata
- `PageLayerTree.text_sources` + `TextSourceTable` (export-local)
- `TextRun.source` span
- `TextRun.{paintStyle, projectionKind, orientation, placement, clusterBasis, clusters, legacyVisuals}` (7 신규)
- `schemaMinorVersion`, `resourceTableMinorVersion`
- `usedFeatures`, `requiredFeatures`, `optionalFeatures`, `knownFeatures`, `text` metadata
- `docs/text-ir-v2.md` migration contract 문서

### 5.3 LayerBuilder 변경
- char overlap / tab leader / decoration: visual payload 항상 존재 영역 영역 explicit op 생성, 기존 TextRun payload 영역 영역 legacy mirror 보존
- control mark: `showParagraphMarks` 또는 `showControlCodes` 옵션 켜진 경우만 external op 영역 영역 낮춤 (기존 renderer output option 정합)

## 6. Non-goals (PR 본문 — Still designing)

P12+ 영역 영역 분리:
- `GlyphRun` eligibility — guarded variant 작업
- font resource table
- cluster basis
- fallback diagnostics

→ P11 영역 영역 contract 영역 영역만 정합, 실제 GlyphRun 영역 영역 후속.

## 7. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` (1 commit) | ✅ auto-merge 충돌 0건 |
| `cargo build --release` | ✅ 통과 |
| `cargo test --release` | ✅ ALL GREEN |
| `cargo clippy --release --features native-skia --lib -- -D warnings` | ✅ 통과 |
| **`cargo test --release --features native-skia --lib skia`** | ✅ **28/28 PASS** (PR #769 인프라 보존) |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** (Compatibility first 원칙 입증) |

## 8. 시각 판정 면제 (작업지시자 결정)

contract 정합 단계 (PR 본문 명시 — "GlyphRun 을 기본 경로로 만드는 PR 이 아니라 TextRun v2 compatibility contract 를 완성하는 PR") + 결정적 검증 + 회귀 가드 (sweep 170/170 same 입증) + clippy 통과 — 시각 판정 게이트 면제 합리.

## 9. 영향 범위

### 9.1 변경 영역
- Rust paint 모듈 (Text IR v2 contract — json + builder + layer_tree + paint_op + schema + mod)
- Rust renderer (4 backend — svg_layer + canvas + skia + web_canvas) — special visual op skip
- 문서 (docs/text-ir-v2.md + README)

### 9.2 무변경 영역
- TypeScript / rhwp-studio (변경 부재)
- HWP3/HWPX 변환본 시각 정합 (sweep 170/170 same 입증)
- 기존 TextRun payload — legacy mirror 보존 (Compatibility first)
- GlyphRun 기본 경로 (PR 본문 Non-goals 명시 — P12+)

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @seo-rii **10번째 사이클** (Skia 핵심) |
| `feedback_image_renderer_paths_separate` | **권위 사례 강화** — 4 backend (svg/canvas/skia/web_canvas) 동기 정정 영역 영역 special visual op skip (double-painting 방지) |
| `feedback_pr_supersede_chain` 권위 사례 강화 | PR #599 (P4) → #626 (P5) → #720 (P6) → #761 (P8) → #769 (P9) → **#797 (P11)** Issue #536 트래킹 단계적 진전 |
| `feedback_process_must_follow` | Compatibility first + Additive schema + Source traceability + Placement metadata 4 본질 원칙 + Still designing 명시 (P12+ 분리) — 위험 좁힘 |
| `feedback_visual_judgment_authority` | contract 정합 단계 영역 영역 결정적 검증 + sweep 통과 영역 영역 시각 판정 면제 합리 |

## 11. 잔존 후속

- 본 PR 본질 정정의 잔존 결함 부재
- 후속 PR (PR 본문 Still designing 명시):
  - GlyphRun eligibility (guarded variant)
  - font resource table
  - cluster basis
  - fallback diagnostics

---

작성: 2026-05-11
