# 수정 수행계획 — Task M100 #4969 W10-Q4-D3 atomic vertical GlyphRun publication

- **상위 계획**: [`task_m100_4969_w10_q4_d.md`](task_m100_4969_w10_q4_d.md)
- **D2 checkpoint**: `992fe4acaf92`
- **최신 devel merge**: `049b73d0cd80` (`upstream/devel@bd78a53122e4`)
- **D2 재자격화 기록**: `7bfefff07a99`
- **기계 판독 계획**:
  [`w10_q4_d3_revised_execution_plan.json`](../../tech/investigations/issue-4969/w10_q4_d3_revised_execution_plan.json)
- **상태**: 계획 checkpoint `e64d6fd67`; D3-A checkpoint `bb334756b`; D3-B checkpoint
  `d49400aad`; D3-C `qualified-publication-parity` checkpoint `43d50ad99`; 최신 devel merge
  `bcaf86e18`; Q4-D4 상세 수행계획 승인 대기
- **작성일**: 2026-08-30 KST
- **제품 출력 변경**: 승인된 D2 target의 layer tree에 fallback과 함께 portable `GlyphRun` 대안을 게시
- **backend 선택 변경**: 없음 — Q4-D4 전까지 모든 backend는 `TextRun`을 선택

## 1. 재검토 결론

Q4-D2 target의 한 shaped line은 render tree에서 문자별 `TextRun` leaf 두 개를 fallback으로 유지한다.
각 leaf는 layer JSON에서 서로 다른 `TextSourceId`를 받는다. 반면 기존 D3 계획은 줄 전체를 하나의
equivalence group(같은 원문을 그리는 대안 묶음)으로 게시하도록 적었다.

현재 variant 계약은 한 equivalence group이 여러 leaf를 가로지르면 `CrossLeafGroup`으로 거부한다. 줄 전체
`GlyphRun` 하나를 어느 한 leaf에 붙이면 다른 fallback source까지 소유한다고 거짓 주장하게 되고, 새 집계
`TextRun`을 만들면 승인된 D2 fallback을 중복 그리게 된다. 따라서 다음 두 원자성을 구분한다.

1. **publication transaction 원자성**: 한 D2 line의 모든 glyph 대안과 portable font resource를 전부 준비한
   뒤 모두 함께 게시하거나 전부 게시하지 않는다.
2. **variant leaf-scope 원자성**: 기존 문자별 `TextRun` leaf마다 그 source와 정확히 대응하는 `GlyphRun`
   한 개와 고유 `text-N` equivalence group을 둔다.

이 정정은 D2 render tree를 다시 쓰지 않으며 `text_variants`의 leaf-scope, `TextSourceTable`의 source 순서,
fallback 보존 계약을 모두 지킨다.

## 2. D3 권위와 비권위

### D3가 여는 권위

- D2 line sidecar의 exact source certificate를 portable font blob/face resource로 한 번 준비한다.
- Q4-C의 동일 geometry `Arc`에서 문자별 glyph id·origin·advance·bbox·cluster를 읽는다.
- 각 fallback leaf에 같은 source id의 vertical-upright `GlyphRun` 대안을 게시한다.
- line subtree와 resource delta를 validation 뒤 한 commit 경계에서 적용한다.

### D3가 열지 않는 권위

- Rust `layer_renderer`의 vertical strict 선택 조건
- `text_v2`의 `verticalGlyphOrientationAuthorityPending` 및 `writingModeAuthorityPending`
- Studio CanvasKit feature detection·font construction·`drawGlyphs`
- Native Skia·SVG·Canvas2D의 glyph replay
- Latin sideways, punctuation, mixed run, variation, HWPX, 다중 line/run/column

D3 산출 `GlyphRun`은 정확한 portable candidate지만 Q4-D4 전에는 선택 불가다. 따라서 실제 화면은 모든
backend에서 기존 `TextRun` fallback과 같아야 한다.

## 3. 게시 계약

한 D2 target line에 glyph가 `N`개이면 다음 조건을 전부 만족해야 batch를 준비한다.

1. line group의 `source_node_id`가 sidecar `line_node_id`와 같다.
2. line subtree에 visible vertical `TextRun` leaf가 정확히 `N`개이고 다른 paint op가 없다.
3. traversal 순서의 `TextSourceId`, render node id, fallback 문자, UTF-8/UTF-16 길이와 Q4-C cluster가
   일대일로 대응한다.
4. sidecar의 transaction, geometry owner, source certificate `Arc`, generation, digest, byte length,
   face index, units-per-em이 D2 attach 때의 값과 같다.
5. glyph id는 유효 범위이고 origin·advance·bbox는 finite이며 leaf/line geometry와 허용 오차 안에서 일치한다.
6. 기존 page resource와 합친 font source 개수·byte 상한을 넘지 않는다.

각 leaf에 게시하는 candidate는 다음과 같다.

- `source.id`: 해당 fallback `TextSourceId`
- source range: 그 leaf 문자 전체의 local UTF-8·UTF-16 범위
- `equivalenceGroup`: `text-{sourceId}`; `variantId`: `verticalGlyphRun`
- glyph/cluster: 해당 source에 대응하는 한 glyph와 local range
- `writingMode`: `vertical-rl`; `orientation`: `vertical-upright`; `glyphTransforms`: 없음
- direction/bidi: 현 schema의 bounded pure-CJK 계약인 `ltr`/0
- font: certificate의 exact portable face, variation·synthetic style 없음
- diagnostic reason: `boundedVerticalHwp5TableCellV1`

여러 leaf의 candidate는 같은 line publication id와 source certificate를 공유하지만 equivalence group은 공유하지
않는다. 모든 glyph op와 font resource가 준비된 뒤에만 line subtree를 교체한다. validation 실패 시 resource,
leaf op, source claim을 하나도 추가하지 않는다.

## 4. 구현 절편

### Q4-D3-A — leaf/source mapping shadow

1. 새 integration source를 만들지 않고 기존
   `tests/cases/issue_4969_shaping_atomic_activation.rs`에 D2 target의 line group, 두 fallback leaf,
   text source 순서와 sidecar glyph/cluster 일대일 mapping을 고정한다.
2. `src/paint/shaping_glyph_vertical.rs`에 line subtree를 읽기 전용으로 감사하는 bounded preparation DTO와
   typed rejection을 만든다.
3. product `LayerBuilder` caller는 연결하지 않고 resource·layer mutation 0을 증명한다.
4. 기존 D3 source red와 D4 red는 모두 유지한다.

**종료 게이트**: target mapping mismatch 0, no-source/non-Noto control rejection, 제품 layer hash 변화 0.

**결과 후보**: [Q4-D3-A 결과 보고서](../../working/task_m100_4969_w10_q4_d3_a.md)는 D2 line과 두
fallback leaf/source/glyph를 일대일로 매핑하고, 같은 길이의 다른 문자도 비직렬화 source-text SHA-256으로
거부한다. Q4 green 36/36, atomic activation 9 pass/1 ignore, integration 정책 19/19와 canonical controls를
통과했으며 product publication과 backend 선택은 0이다. 메인테이너 결과 승인·checkpoint 전 D3-B는 닫힌다.

### Q4-D3-B — atomic resource + leaf publication

1. exact certificate에서 portable blob/face metadata와 모든 leaf `GlyphRun`을 mutation 없이 준비한다.
2. 기존 `ResourceArena` key·intern API를 재사용하되 horizontal lowerer를 리팩터링하거나 vertical 권위를
   horizontal `VerticalPositioningAuthorityPending` 경로에 섞지 않는다.
3. line subtree clone에 모든 leaf 대안을 먼저 적용·검증하고, resource budget 대사 뒤 원 subtree와 resource
   delta를 한 infallible commit 함수에서 교체·등록한다.
4. 성공한 line만 claim하고 nominal lowerer가 같은 `TextSourceId`에 중복 `GlyphRun`을 만들지 않게 한다.
5. 기존 fallback `TextRun`은 각 leaf의 첫 op로 그대로 남긴다.
6. `LayerBuilder`는 vertical lowerer 뒤 horizontal lowerer·nominal lowerer 순서를 명시하고 서로의 claim
   집합을 합쳐 nominal 중복만 막는다.

**종료 게이트**: target은 `N TextRun + N vertical GlyphRun`, leaf별 group 1, portable blob/face 1,
nominal duplicate 0; rejected batch publication residue 0.

**결과 후보**: [Q4-D3-B 결과 보고서](../../working/task_m100_4969_w10_q4_d3_b.md)는 target의 두 fallback
leaf에 vertical `GlyphRun`을 하나씩 게시하고 portable blob/face를 한 번만 등록했다. 같은 길이 문자 변조는
line 전체를 거부해 glyph/resource residue 0을 유지했고, D4 전 selector 다섯 종류는 모두 `TextRun`을
선택했다. 판정은 `qualified-atomic-leaf-publication`으로 승인됐으며 checkpoint 승인 대기 상태다.

### Q4-D3-C — publication 검증·결과 판정

1. `validate_text_variant_scope`와 layer JSON source/variant mapping을 통과시킨다.
2. Rust selector·`text_v2`가 vertical/writing-mode authority pending으로 fallback을 선택하는 것을 확인한다.
3. D0~D2, horizontal Q2/Q3, #6029, 두 canonical sample을 재실행한다.
4. native·격리 Node WASM·표준 Docker WASM에서 동일 layer JSON glyph/source/resource 값을 대사한다.
5. source preparation 횟수, font payload bytes와 layer JSON 증가량은 기록하되 D5 성능 결론으로 과장하지 않는다.

**종료 게이트**: publication 값 mismatch 0, fallback disappearance 0, backend false selection 0,
canonical mismatch 0, 회귀 0. 결과 승인·checkpoint 전 Q4-D4는 시작하지 않는다.

**최종 결과 후보**: [Q4-D3-C 결과 보고서](../../working/task_m100_4969_w10_q4_d3_c.md)는 native와 격리
Node WASM의 source/variant/glyph/resource 수치가 일치하고, selector·`text_v2`가 두 leaf 모두 fallback을
유지하며 canonical SVG 두 건과 관련 회귀가 불변임을 확인했다. font payload와 layer JSON 증가량은 기준선으로만
기록했다. 메인테이너가 `qualified-publication-parity-pre-docker` 사전 판정을 승인한 뒤 표준 Docker WASM과
post-build native 영수증도 통과했다. 최종 판정 후보는 `qualified-publication-parity`이며 별도 결과 승인과
checkpoint 승인 전에는 D3-C를 고정하지 않는다.

## 5. 실패 원자성

| 실패 지점 | 결과 |
| --- | --- |
| line/source/leaf count 불일치 | `TextRun`만 유지, resource·claim 0 |
| cluster와 leaf 문자 범위 불일치 | batch 전체 거부, glyph op 0 |
| certificate generation/digest/face 불일치 | batch 전체 거부, resource 0 |
| glyph geometry·advance·bbox malformed | batch 전체 거부, line subtree 변화 0 |
| resource 상한 초과·기존 key 충돌 | batch 전체 거부, arena 변화 0 |
| staged subtree variant 검증 실패 | 원 subtree 유지, resource 0 |
| D4 selector 미지원 | 게시된 candidate는 남되 `TextRun` fallback 선택 |

## 6. 보호 불변식

1. D2 render tree geometry·node id·fallback text는 바꾸지 않는다.
2. 한 줄의 일부 glyph/leaf/resource만 게시하지 않는다.
3. equivalence group은 leaf를 가로지르지 않는다.
4. parser, font name, fallback matrix는 exact source authority가 아니다.
5. horizontal lowerer의 `VerticalPositioningAuthorityPending`을 변경하지 않는다.
6. D4 Rust·Studio selector와 backend draw path를 변경하지 않는다.
7. 새 integration `.rs`, generated suite·manifest, Cargo marker를 source commit에 추가하지 않는다.
8. private corpus·Hyper-V·GitHub mutation은 사용하지 않는다.

## 7. 검증 게이트

- D3-A shadow: line/source/leaf/glyph/cluster mapping 및 typed rejection
- D3-B behavior: leaf별 `TextRun + GlyphRun`, one resource, one line commit, failed residue 0
- `validate_text_variant_scope`, layer JSON source table·variant anchor·font digest
- D3 상태의 Rust CanvasKit/Browser/NativeSkia selector 모두 fallback
- Q4 request contract와 atomic activation, Q2/Q3 horizontal publication 회귀
- #6029 vertical cell 및 `table-004`·#6029 canonical SVG byte/SHA-256
- `cargo fmt --all`, `cargo fmt --all -- --check`, focused Clippy
- review worktree의 integration manifest prepare/check; generated 산출물은 stage하지 않음
- 결과 승인 뒤 표준 Docker WASM; D4 Studio pixel E2E는 아직 실행·판정하지 않음

## 8. 중단 조건과 다음 승인 경계

- leaf별 source span으로 Q4-C cluster를 손실 없이 표현할 수 없으면 D2 tree를 즉석 변경하지 않고 중단한다.
- staged line subtree만으로 atomic commit을 증명할 수 없으면 page tree 전체 clone을 성능 근거 없이 채택하지 않는다.
- D3에서 vertical candidate를 선택하려면 계획 위반이므로 selector를 열지 않고 D4로 넘긴다.
- public schema 변경, D2 target 확대, horizontal resource helper 대규모 리팩터링이 필요하면 별도 수정 계획을 낸다.

메인테이너가 수정 수행계획을 승인하고 계획 checkpoint `e64d6fd67`을 고정했다. Q4-D3-A 결과는
`qualified-shadow-mapping`으로 승인돼 checkpoint `bb334756b`로 고정됐다. D3-B는
`qualified-atomic-leaf-publication`으로 승인돼 checkpoint `d49400aad`로 고정됐다. 별도 승인을 받아 D3-C의
native·격리 Node WASM·회귀·canonical 검증을 마치고 `qualified-publication-parity-pre-docker` 사전 결과를
승인받았다. 이어서 표준 Docker WASM과 post-build native 영수증도 통과했다. 현재
`qualified-publication-parity` 최종 결과를 승인받고 checkpoint `43d50ad99`로 고정했다. Q4-D4 착수 승인을
받아 최신 devel을 merge commit `bcaf86e18`로 병합하고 D3-C·incoming 회귀와 canonical을 재통과했다.
Q4-D4 상세 수행계획 승인 전에는 Rust selector 구현을 시작하지 않는다.
