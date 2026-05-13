---
PR: #797
제목: render — add Text IR v2 compatibility contract (P11)
컨트리뷰터: @seo-rii (Seohyun Lee) — 10번째 사이클 (Skia 핵심 컨트리뷰터)
base / head: devel / render-p11
mergeStateStatus: BEHIND
mergeable: MERGEABLE
CI: SUCCESS (Build & Test + CodeQL js/ts/py/rust + Canvas visual diff)
변경 규모: +1173 / -35, 12 files
검토일: 2026-05-11
Refs: #536
---

# PR #797 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #797 |
| 제목 | render — add Text IR v2 compatibility contract (P11) |
| 컨트리뷰터 | @seo-rii — Skia 핵심 (10번째 사이클, PR #165/#419/#456/#498/#599/#626/#720/#761/#769/#797) |
| base / head | devel / render-p11 |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 부재 |
| CI | ✅ Build & Test + CodeQL (js/ts/py/rust) + Canvas visual diff 통과 |
| 변경 규모 | **+1173 / -35, 12 files** (대형 PR) |
| 커밋 수 | 1 (단일 commit) |
| Refs | #536 (Skia native raster 트래킹) |

## 2. 본질

P11 단계 — Text IR v2 **compatibility contract** 추가. P9 영역 영역 text replay parity 후속 영역 영역 GlyphRun / font resource / native glyph replay 영역 영역 schema contract 영역 영역 미리 여는 단계.

PR 본문 명시: **본 PR은 GlyphRun을 기본 경로로 만드는 PR이 아니라 TextRun v2 compatibility contract를 완성하는 PR**.

### 2.1 Skia native raster 단계적 진전 (Issue #536)
| 단계 | PR | 본질 |
|------|-----|------|
| P4 | #599 | native Skia PNG raster backend |
| P5 | #626 | equation replay |
| P6 | #720 | raw SVG fragment replay |
| P8 | #761 | schema/resource hardening |
| P9 | #769 | text replay parity + module split |
| **P11** | **#797 (5/11)** | **Text IR v2 compatibility contract** |

(P10 영역 영역 P9 영역 영역 통합됨)

## 3. 채택 접근 — Compatibility first + Additive schema

PR 본문 명시 영역 영역 4 본질 원칙:

| 원칙 | 본질 |
|------|------|
| **Compatibility first** | 모든 backend는 계속 TextRun fallback으로 렌더링 가능 |
| **Additive schema** | schemaMinorVersion + feature negotiation + text metadata 추가 (기존 consumer 미파괴) |
| **Source traceability** | PageLayerTree.text_sources + TextRun.source span (text op 원문 범위 추적) |
| **Placement/cluster metadata** | paintStyle / projectionKind / orientation / placement / clusterBasis / clusters / legacyVisuals |

### 3.1 External special visual ops 신규 (`PaintOp` enum 추가 4개)
- `PaintOp::CharOverlap`
- `PaintOp::TextControlMark`
- `PaintOp::TabLeader`
- `PaintOp::TextDecoration`

→ char overlap / control mark / tab leader / decoration 영역 영역 explicit visual op 영역 영역 낮춤 + 기존 renderer 영역 영역 skip 영역 영역 double-painting 방지.

### 3.2 LayerBuilder 변경
- char overlap / tab leader / decoration: visual payload 항상 존재 영역 영역 explicit op 생성, 기존 TextRun payload 영역 영역 legacy mirror 보존
- control mark: `showParagraphMarks` 또는 `showControlCodes` 옵션 켜진 경우만 external op 영역 영역 낮춤 (기존 renderer output option 정합)

## 4. PR 의 정정 — 12 files, +1173/-35

### 4.1 핵심 변경
| 파일 | 변경 |
|------|------|
| `src/paint/json.rs` | +687/-14 (schemaMinorVersion + feature negotiation + text metadata + special visual ops export) |
| `src/paint/builder.rs` | +151/-10 (LayerBuilder 영역 영역 explicit visual op 생성) |
| `src/paint/layer_tree.rs` | +170/-1 (Text IR v2 metadata + text_sources + TextSourceTable) |
| `src/paint/paint_op.rs` | +45 (4 신규 PaintOp variant) |
| `src/paint/schema.rs` | +15/-2 (schemaMinorVersion + resourceTableMinorVersion) |
| `src/paint/mod.rs` | +5/-2 |

### 4.2 Renderer 정정 — 기존 backend 영역 영역 special visual op skip
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

## 5. Non-goals (PR 본문 명시 — Still designing)
- `GlyphRun` eligibility — P12 이후 guarded variant
- font resource table — P12 이후
- cluster basis — P12 이후
- fallback diagnostics — P12 이후

→ P11 영역 영역 contract 영역 영역만 정합, 실제 GlyphRun 영역 영역 P12+ 영역 영역 분리.

## 6. 인프라 도입 / 재사용

### 6.1 신규 인프라
- `PaintOp::{CharOverlap, TextControlMark, TabLeader, TextDecoration}` — Text IR v2 영역 영역 explicit visual op
- `PageLayerTree.text_sources` + `TextSourceTable` (export-local) — text op 원문 범위 추적
- `TextRun.{paintStyle, projectionKind, orientation, placement, clusterBasis, clusters, legacyVisuals, source}` — text replay metadata 확장
- `schemaMinorVersion`, `resourceTableMinorVersion`, `usedFeatures`, `requiredFeatures`, `optionalFeatures`, `knownFeatures`, `text` metadata
- `docs/text-ir-v2.md` 영역 영역 migration contract 문서

### 6.2 재사용
- 기존 `PageLayerTree` (P8 #761)
- 기존 `LayerBuilder` (P5 영역 영역 도입)
- 기존 `TextRun` legacy mirror (P9 #769)

## 7. 충돌 / mergeable

mergeStateStatus = `BEHIND`, mergeable = `MERGEABLE`. paint 모듈 영역 영역 5/10 사이클 영역 영역 PR #761 (P8) + PR #769 (P9) 영역 영역 누적 — 본 PR 영역 영역 paint::json.rs + paint::builder.rs + paint::layer_tree.rs + paint_op.rs 영역 영역 신규 코드 추가 영역 영역 충돌 부재 예상.

## 8. 본 환경 점검

### 8.1 변경 격리
- Rust paint 모듈 영역 영역만 (Text IR v2 contract)
- Renderer 영역 영역 각 backend 영역 영역 새 PaintOp skip (compatibility first)
- TypeScript / rhwp-studio 무영향
- HWP3/HWPX 변환본 시각 정합 (sweep 170/170 same 예상 — contract 정합 + 기존 backend skip)

### 8.2 CI 결과
- 모두 ✅

### 8.3 시각 회귀 0 (예상)
- Compatibility first 원칙 — 모든 backend 영역 영역 TextRun fallback 유지
- 새 PaintOp 영역 영역 기존 backend 영역 영역 skip — double-painting 방지
- → 시각 출력 영역 영역 변경 부재 (예상)

## 9. 처리 옵션

### 옵션 A — 1 commit cherry-pick + no-ff merge

```bash
git checkout local/devel
git cherry-pick e6eb3992
git checkout devel
git merge local/devel --no-ff
```

→ **권장**.

## 10. 검증 게이트

### 10.1 자기 검증
- [ ] cherry-pick 충돌 0건
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release` ALL GREEN
- [ ] **`cargo test --release --features native-skia --lib skia`** 28/28 PASS (PR #769 인프라 보존)
- [ ] `cargo clippy --release --lib -- -D warnings` 통과
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (Compatibility first 원칙 영역 영역 시각 출력 무변경 예상)

### 10.2 시각 판정 게이트 — **면제 가능**

본 PR 본질 영역 영역 contract 정합 단계 — 시각 출력 변경 부재 (Compatibility first 원칙). PR #720 / #761 / #769 동일 패턴 영역 영역 시각 판정 면제 합리.

`feedback_visual_judgment_authority` 정합 — 결정적 검증 + sweep 통과 영역 영역 면제.

## 11. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @seo-rii **10번째 사이클** (Skia 핵심) |
| `feedback_image_renderer_paths_separate` | Skia + SVG + Canvas + WebCanvas 영역 영역 4 backend 영역 영역 special visual op skip — 모두 동기 정정 (권위 사례) |
| `feedback_pr_supersede_chain` 권위 사례 강화 | PR #599 (P4) → #626 (P5) → #720 (P6) → #761 (P8) → #769 (P9) → **#797 (P11)** Issue #536 트래킹 |
| `feedback_process_must_follow` | Compatibility first + Additive schema + Still designing 명시 — 위험 좁힘 |
| `feedback_visual_judgment_authority` | contract 정합 단계 영역 영역 결정적 검증 + sweep 통과 영역 영역 시각 판정 면제 합리 |

## 12. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 cherry-pick `e6eb3992` (auto-merge 정합 예상)
2. 자기 검증 (cargo build/test/clippy + native-skia feature 28/28 + 광범위 sweep)
3. 시각 판정 면제 합리 (작업지시자 결정)
4. 검증 통과 → no-ff merge + push + archives + 5/11 orders
5. PR #797 close

---

작성: 2026-05-11
