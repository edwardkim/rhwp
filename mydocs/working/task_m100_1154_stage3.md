# Task #1154 Stage 3 완료 보고서 — 렌더러 호출 지점 통합 + 시각 검증

## 1. 목표

- Stage 2 에서 도입한 `PageRenderTree::clip_overlapping_same_bin_images()` 의
  호출 지점을 결정/통합.
- exam_eng.hwp page 2 의 잔상(이중 라인) 제거 시각 확인.
- 의도적 효과 sample(test-image, 3-10월_교육_통합) 회귀 무영향 확인.

## 2. 호출 지점 분석 (렌더러 7곳 vs. 중앙 1곳)

조사 결과 PageRenderTree 를 소비하는 렌더러 진입부는 총 7곳:

| 위치 | 시그니처 | 입력 |
|---|---|---|
| `svg.rs:173` | `render_tree(&PageRenderTree)` | PageRenderTree |
| `svg_layer.rs:240` | `render_tree(&PageRenderTree)` | PageLayerTree → 재조립 |
| `canvas.rs:83` | `render_tree(&PageRenderTree)` (legacy) | PageRenderTree |
| `canvas.rs:317` | `render_page(&PageLayerTree)` | PageLayerTree |
| `web_canvas.rs:308` | `render_tree(&PageRenderTree)` (WASM) | PageRenderTree |
| `web_canvas.rs:1860` | `render_page(&PageLayerTree)` | PageLayerTree |
| `html.rs:44` | `render_tree(&PageRenderTree)` | PageRenderTree |

이 모든 렌더러는 `DocumentCore::build_page_tree()` 또는 (캐시 경유)
`build_page_tree_cached()` 가 만든 PageRenderTree 를 소비. 또한
`build_page_layer_tree()` 도 내부에서 `build_page_tree_cached()` 를 호출하고
`LayerBuilder` 가 ImageNode bbox / crop 을 그대로 PaintOp::Image 에 복사
(`src/paint/builder.rs:86`)하므로 layer 경로도 동일하게 영향을 받는다.

### 설계 결정 — 중앙 1곳 통합

수행계획서/구현계획서의 권장(“PageRenderTree 빌드 직후 1 회만, finalize 시점”)에
따라 **`build_page_tree()` 의 종단**(`extra_mps` 머지 후, `Ok(tree)` 직전)에
1 회 호출.

근거:
- 단일 진실 공급원 — 신규 렌더러 추가 시 호출 누락 위험 없음.
- 캐시(`build_page_tree_cached`) 도 자동으로 clip 적용된 트리를 보관.
- WASM `getPageRenderTree` JSON 도 정정된 트리 반환 → 외부 소비자 일관성.
- IR 단계 후처리(픽셀이 아닌 IR 에서 정정) — 자연스러운 추상화.

## 3. 변경 사항

`src/document_core/queries/rendering.rs:2585-2591`:

```rust
// 확장 바탕쪽 추가 렌더링
for ext_mp in &extra_mps {
    self.layout_engine.build_master_page_into(...);
}
// Task #1154: 동일 bin_data_id Pic 컨트롤이 수직으로 인접 겹쳐 그려질 때
// 두 그림의 미세한 세로 스케일 차이로 인한 잔상(이중 라인) 제거.
// build 직후 1회만 적용 — 모든 렌더러(SVG/Canvas/Skia/HTML/Layer) 공통.
tree.clip_overlapping_same_bin_images();
Ok(tree)
```

## 4. 검증 결과

### 4.1 단위 테스트 / clippy / fmt

```
cargo test --release --lib
  test result: ok. 1318 passed; 0 failed; 6 ignored

cargo clippy --release --lib -- -D warnings
  Finished — no warnings

cargo fmt — applied to rendering.rs only
```

### 4.2 exam_eng.hwp page 2 (대상 케이스)

`output/svg/task1154_baseline/exam_eng_hwp/exam_eng_002.svg`(stale text rendering
diff 포함)이 아닌 “same-commit no-clip 산출물”로 공정 비교 수행:

- no-clip: `RHWP_TASK1154_NOCLIP=1 ...` 임시 가드 경유 산출 (검증용, 본 커밋엔 제외)
- with-clip: 일반 산출

LOWER (z=2):
- before: `y=243.59, height=256.09, viewBox="0 0 2532 1612.77"`
- after:  `y=243.59, height=**219.59**, viewBox="0 0 2532 **1382.87**"`
- height 변화 = `B.y(=463.17) − A.y(=243.59) = 219.58` ✓
- crop bottom 비례 축소: `1612.77 × (219.59 / 256.09) ≈ 1382.87` ✓

UPPER (z=3):
- before/after 모두 `y=463.17, height=70, viewBox="0 1412.77 2532 434.43"` (불변) ✓

### 4.3 의도적 효과 회귀 무영향

| Sample | 페이지 수 | no-clip vs with-clip diff |
|---|---|---|
| test-image.hwp | 1 | **동일** |
| 3-10월_교육_통합_2022.hwp | 16 | **모든 페이지 동일** |
| exam_eng.hwp | 8 | page 2 만 차이 |

→ strict 5 조건이 의도된 회귀 보호(x/width 동일 가드)를 정확히 수행.

### 4.4 LayerBuilder 검증

`src/paint/builder.rs:86` 에서 `RenderNodeType::Image(image)` 는 `node.bbox` 와
`image.clone()` (crop 4 필드 포함)을 그대로 `PaintOp::Image` 에 복사하므로
PageRenderTree 단계 clip 은 layer 트리에 그대로 전파. CanvasRenderer /
SvgLayerRenderer / 기타 layer 기반 백엔드도 fix 의 혜택을 자동으로 받는다.

## 5. 산출물

- 코드: `src/document_core/queries/rendering.rs` (build_page_tree 끝에 1줄 추가)
- 보고서: `mydocs/working/task_m100_1154_stage3.md`
- 검증 산출물: `/tmp/task1154_no_clip/`, `/tmp/task1154_with_clip/` (임시)

## 6. 다음 단계 (Stage 4)

- 전체 sample sweep (Stage 1 식별 17 sample) 회귀 시각/diff 검증.
- 일반 회귀: exam_kor, exam_math, biz_plan, 통합재정통계, 시험지 일반,
  HWPX 변환본 sweep.
- WASM 빌드 (`docker compose --env-file .env.docker run --rm wasm`).
- 최종 결과 보고서 `mydocs/report/task_m100_1154_report.md`.

승인 후 Stage 4 진행.
