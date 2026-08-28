# 수정 수행계획 — Task M100 #4969 W10-Q2-D4 same-boundary activation

- **선행 결과**: Q2-D0~D3 qualified, checkpoint `be88ec57e`
- **원인 보고서**:
  [`task_m100_4969_w10_q2_d3.md`](../working/task_m100_4969_w10_q2_d3.md)
- **기계 판독 계획**:
  [`w10_q2_d4_revised_execution_plan.json`](../tech/investigations/issue-4969/w10_q2_d4_revised_execution_plan.json)
- **상태**: D4-A·D4-B·D4-C qualified, D4 완료; D5는 별도 승인 대기
- **제품 변경**: D4-A는 0, D4-B의 승인된 최초 lane에서만 있음

## 1. 수정 이유

D3 lowerer는 Q2-B의 page-space glyph position·advance를 손실 없이 보존했지만, glyph outline을 실제로 그리는
크기까지 Q2-B의 `x=fontSize×ratio`, `y=fontSize`로 해석했다. 이는 #5821에서 확정한 압축 장평의 그리기 계약,
즉 `glyph size=fontSize×√ratio`, `horizontal scale=√ratio`와 다르다.

기존 계획대로 `GlyphTransform(xx=ratio, yy=1)`을 단순 활성화하면 총 폭은 맞지만 glyph 높이가 한컴보다 커진다.
반대로 Q2 measurement 전체를 `√ratio` 좌표로 바꾸면 line width·bbox·다음 origin의 page-space 계약이 깨진다.
따라서 layout 좌표와 replay-local 좌표를 구분하되, 둘 다 같은 shaping result `Arc`에서 한 번만 파생해야 한다.

## 2. 채택한 replay authority

### 2.1 기각안

| 안 | 판정 | 이유 |
| --- | --- | --- |
| Q2 page-space glyph + per-glyph `xx=ratio` | 기각 | #5821 세로 `√ratio` 축소를 잃고 현재 strict selector도 transform을 거부 |
| Q2 measurement 전체를 `√ratio` local 좌표로 변경 | 기각 | line width·bbox·next origin이 backend-local 좌표에 종속됨 |
| TextRun renderer가 GlyphRun을 다시 보정 | 기각 | Rust producer와 TypeScript consumer가 ratio를 이중 해석하고 backend별 분기 발생 |
| 한 shaping result에서 page layout과 local replay를 각각 투영 | **채택** | layout 폭은 유지하면서 #5821 SSOT와 CanvasKit affine 계약을 동시에 만족 |

### 2.2 유일한 투영 공식

`condensed_ratio_draw_params(font_size, ratio)`가 반환하는 값을 `(draw_fs, draw_x_scale)`이라 한다.

```text
layout page x      = design x × font_size × ratio / unitsPerEm
layout page width  = Σ design advance × font_size × ratio / unitsPerEm

replay local x     = design x × draw_fs / unitsPerEm
replay local y     = design y × draw_fs / unitsPerEm
replay placement   = scale(draw_x_scale, 1) + TextRun baseline translation

page(replay x)     = replay local x × draw_x_scale
                   = layout page x
```

압축 장평 `0 < r < 0.999`에서는 `draw_fs=fs×√r`, `draw_x_scale=√r`이므로 glyph 높이는 `×√r`, 최종 폭은
`×r`이다. `r>=0.999` 확대·항등의 기존 TextRun 규칙도 같은 SSOT가 반환하지만, 최초 D4-B lane은 오라클이 있는
압축 장평만 연다.

다만 D3 measurement의 y offset·advance는 `font_size / unitsPerEm`으로 환산하고 #5821 replay-local y는
`draw_fs / unitsPerEm`을 사용하므로, 비영(非零) GPOS y positioning에는 아직 두 좌표계 사이의 승인된 authority가
없다. D4-A/B 최초 lane은 raw design-unit `y_offset=0 && y_advance=0`을 기능 탐지하고, 하나라도 0이 아니면
`verticalPositioningAuthorityPending`으로 fail-closed한다. 이는 값을 버리는 보정이 아니라 mark positioning의
세로 의미를 후속 오라클 전까지 보류하는 보호 불변식이다.

GlyphRun에는 이미 소비된 장평을 다시 적용하지 않도록 다음을 고정한다.

- `shapeKey.fontInstance.sizePx = draw_fs`
- `paintStyle.fontSize = draw_fs`
- `paintStyle.ratio = 1`
- `placement.runToPage.a = draw_x_scale`, `d = 1`, rotation 성분 없음
- `glyphTransforms = None`
- round-trip 오차가 허용치 안일 때만 `strictVisualEligible = true`
- diagnostic reason은 `q2CommonShapingCondensedDrawProjectionV1`

### 2.3 Source Han 고정 오라클

font size 10px, ratio 0.8 fixture의 page-space vector는 D3와 같이
`x=[0, 7.728, 15.456, 15.456]`, advance `=[7.728, 7.728, 0, 0]`이다.

```text
draw_x_scale = √0.8 = 0.8944271909999159
draw_fs      = 8.94427190999916
local x      = [0, 8.640166665059187, 17.280333330118374, 17.280333330118374]
local advance= [8.640166665059187, 8.640166665059187, 0, 0]
```

affine 적용 뒤 page-space vector가 D3 값과 일치해야 한다. glyph ID와 UTF-8·UTF-16·glyph cluster range는
투영 전후 변하지 않는다.

## 3. 수정된 owner 구조

```text
one Arc<HorizontalShapingMeasurement>
  ├─ page layout projection
  │    line width = bbox width = next origin delta = total_advance_px
  └─ replay projection (#5821 SSOT)
       draw_fs + draw_x_scale + local glyph geometry
       + exact source Arc<[u8]> certificate
       -> page-local NodeId sidecar
       -> LayerBuilder common GlyphRun
```

exact font registry가 이미 소유한 `Arc<[u8]>`를 digest·length·face index·generation과 다시 대사한 뒤
page-local replay certificate에 `Arc`로 연결한다. bytes를 복사하지 않으며 sidecar·frame은 직렬화하지 않는다.
LayerBuilder가 같은 font를 이름이나 host 설치 상태로 다시 찾지 않고 certificate source를 사용하므로 layout 승인과
portable resource 등록 사이의 identity gap을 없앤다.

trace·JSON·public annotation에는 font bytes, raw text, family path를 기록하지 않는다. page certificate의 unique font
payload 합계는 64 MiB, 개별 face는 32 MiB 상한을 그대로 적용한다.

## 4. 최초 제품 activation lane

D4-B는 다음 상태를 모두 feature-detect한 문단만 연다.

- 문단 전체가 정확히 한 composed line, 그 line이 정확히 한 emitted text run이다.
- Q2-C final line·target scalar range와 현재 composed line·run range가 모두 같다.
- direct old-Hangul target이며 text 전체가 target이다.
- embedded exact portable source, 한 slot·한 face·한 style·한 registry generation이다.
- `0 < ratio < 0.999`, horizontal-tb, LTR, bidi level 0, left alignment이다.
- raw shaping glyph 전부의 `y_offset=0`, `y_advance=0`이다.
- `layout_positions=None`, model text와 replay text가 같고 display projection이 없다.
- numbering/prefix, TAC, footnote/endnote, field, tab/control, CharOverlap, rotation, border/background,
  underline/strike/emphasis, synthetic bold/italic, super/sub, 자간·분배·condense가 없다.
- node sidecar 한 건과 source certificate가 page 상한 안에서 원자적으로 attach된다.

버전·한컴 build number·HWP/HWPX 확장자 분기는 만들지 않는다. 다중 line/run, center/right/justify/distribute,
mixed target는 이 절편에서 fail-closed하고 후속 matrix로 넘긴다.

## 5. 구현 절편

### Q2-D4-A — replay projection·certificate shadow

1. `condensed_ratio_draw_params`를 사용하는 bounded internal replay projection을 만든다.
2. Q2 raw design-unit result에서 local position·advance와 draw font size·affine을 한 번 계산한다.
   최초 lane은 raw y offset·advance가 모두 0일 때만 계산한다.
3. affine round-trip을 D3 page-space position·advance·total width와 대사한다.
4. exact registry의 source `Arc`를 handle·generation과 대사해 non-serialized certificate에 연결한다.
5. D3 lowerer를 projection output으로 전환하되 제품 caller는 계속 0으로 유지한다.
6. ratio가 이미 투영된 paint style, transform 없음, strict eligibility 조건을 Rust·TypeScript selector와 대사한다.

**종료 게이트**: Source Han fixed oracle의 native·Node WASM round-trip mismatch 0, nonzero-y typed rejection,
source byte copy 0,
제품 layout·LayerBuilder caller 0, 출력 변화 0.

### Q2-D4-B — one-line/one-run atomic activation

1. paragraph 진입 시 위 최초 lane 전체를 preflight하고, 일부 line/run만 승인하지 않는다.
2. target run의 `NodeId`를 확정한 뒤 D2 mapping과 replay certificate를 page sidecar에 먼저 attach한다.
3. attach가 성공한 경우에만 measurement advance를 `full_width`·TextRun bbox·다음 x의 공통 값으로 사용한다.
4. `LayerBuilder`의 기존 public entrypoint는 보존하고, 내부 path만 page sidecar를 font lowerer에 전달한다.
5. font lowerer는 같은 `text_source_id`에서 common GlyphRun을 먼저 내린다. 성공 report의
   `claims_glyph_run_slot`이 nominal GlyphRun 생성을 건너뛰며 TextRun은 항상 첫 fallback으로 남긴다.
6. certificate source를 portable font resource로 한 번 등록하고 common GlyphRun 한 개만 추가한다.
7. CanvasKit/CanvasKitBrowser strict selector는 기존 조건으로 common GlyphRun을 선택한다. Canvas2D·legacy
   SVG와 blob typeface replay가 아직 없는 native Skia는 TextRun fallback을 유지한다.

**종료 게이트**: 최초 lane에서 TextRun 1 + common GlyphRun 1 + nominal duplicate 0, sidecar 1, font blob 1;
line width = bbox width = next origin delta = affine round-trip advance. reject lane의 render/layer hash 변화 0.

### Q2-D4-C — cross-backend·성능 판정과 보고

1. Rust native/WASM, studio selector/draw matrix, 기존 #5821 fixture를 함께 검증한다.
2. CanvasKit 실제 draw에서 glyph width·height와 cluster 배치를 오라클과 대사한다.
3. non-target·W9 `layout_positions`·K0·D0~D3 회귀를 실행한다.
4. page sidecar/Arc, layer build wall time, JSON·font payload bytes를 D3 dormant 기준과 분리해 기록한다.
5. 실패가 있으면 D4-B lane을 넓히지 않고 activation을 다시 닫은 뒤 원인 보고한다.

**종료 게이트**: backend별 선택 결과와 시각 오라클이 일치하고 전체 회귀 0. D5는 별도 승인 전 시작하지 않는다.

## 6. 원자성·실패 처리

| 실패 지점 | 결과 |
| --- | --- |
| paragraph/range/surface preflight 거부 | legacy width·bbox·x, TextRun만 유지, sidecar 0 |
| source handle/generation/digest/face 대사 실패 | geometry 게시 전 rollback, resource 0 |
| projection/round-trip malformed | geometry 게시 전 rollback, strict false |
| raw y offset/advance가 0이 아님 | `verticalPositioningAuthorityPending`, resource 0, TextRun 유지 |
| sidecar capacity/duplicate/generation attach 실패 | geometry 게시 전 rollback, partial publication 0 |
| common lowerer가 certificate와 불일치 | common GlyphRun을 선택하지 않고 TextRun 보존, typed report; invariant failure로 D4 lane 중단 |
| CanvasKit resource/typeface/selector 실패 | TextRun fallback 선택, 빈 텍스트 출력 금지 |

최초 lane은 문단당 target이 하나뿐이므로 “첫 run은 shaped, 뒤 run은 legacy”인 부분 게시 상태를 만들지 않는다.
여러 target의 batch reservation·rollback은 lane 확대 전에 별도 계획한다.

## 7. 검증 게이트

- D4-A source integration: fixed glyph IDs, page/local position·advance, draw fs, affine, cluster, source identity
- D4-B source integration: single-line/run activation, bbox·next origin, sidecar, TextRun/common GlyphRun/nominal count
- 음성 matrix: multi-line/run, mixed prefix/suffix, center/right/justify/distribute, ratio>=0.999, layout positions,
  source mismatch, stale generation, nonzero design y offset/advance, capacity exhaustion, decoration/control/TAC/note
- native·Node WASM parity와 native/WASM lib clippy
- Rust variant selection: CanvasKit strict GlyphRun, NativeSkia TextRun fallback
- studio unit/E2E: `replayStatus`, `drawGlyphs` local position과 `concat` affine, embedded face digest
- #5821 compressed glyph-height regression과 Source Han old-Hangul CanvasKit pixel bbox
- non-target render-tree·layer-tree·JSON hash, W9 K0/K1와 D0~D3 focused regression
- integration source는 `tests/cases/` 원본만 제출하고 generated suite·manifest·Cargo target은 stage하지 않음
- PR 준비 review worktree에서만 `--prepare`, `cargo fmt --all`, 전체 integration/nextest 수행

## 8. 중단·후속 조건

- existing schema로 draw fs·affine·source certificate를 표현할 수 없으면 public schema를 즉석 확장하지 않고 중단한다.
- CanvasKit 실제 glyph bbox가 #5821 draw contract와 맞지 않으면 selector를 열지 않는다.
- line boundary가 하나라도 다르면 D5로 넘기며 D4에서 재조판하지 않는다.
- native Skia blob typeface construction, vertical/RTL/variation, expansion ratio, multi-line/multi-run batch activation은
  본 절편에 포함하지 않는다.
- GPOS mark positioning처럼 nonzero y offset/advance가 필요한 run은 별도 세로 authority 오라클 전까지 포함하지 않는다.
- rejected attempt의 public annotation이 필요하면 schema minor 계획을 별도 등록한다.

## 9. 승인 게이트

Q2-D4-A/B/C는 각각 qualified다. D4-C는 실제 CanvasKit draw, native/WASM, backend fallback, 전체 회귀와 성능을
검증했다. 최초 strict lane은 유지하되 font payload 재사용·중복 제거를 입증하기 전에는 lane을 확대하지 않는다.
D5는 별도 승인 전 시작하지 않으며 push·PR·GitHub comment는 기존 승인 경계를 유지한다.
