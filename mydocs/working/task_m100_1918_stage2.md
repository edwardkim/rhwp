# Stage 2 완료보고서 - Task #1918

**단계**: Stage 2 - overlay summary 경량화
**이슈**: #1918 표 입력/삭제 시 워터마크·정적 이미지 레이어 포함 페이지가 매 입력마다 고비용 재렌더링됨
**브랜치**: `local/task1918`
**작성일**: 2026-07-05

---

## 1. 작업 요약

Stage 1 계측 결과를 반영해 구현계획서를 보정했다. `samples/복학원서.hwp`는 text edit refresh에서
`flow/background/behind/front`가 모두 반복 렌더링되는 overlay 재생성 병목이고,
`samples/253E164F57A1BC6934-empty.hwp`, `samples/143E433F503322BD33.hwp`는 overlay가 없거나 제한적인
상태에서 flow 내부 정적 이미지/OLE·RawSvg 비용이 큰 별도 부류로 정리했다.

이번 Stage 2에서는 매 page-local refresh마다 전체 `PageLayerTree` JSON을 가져와 파싱하던 summary 경로를
경량 overlay summary 우선 경로로 바꿨다. 실패하거나 구버전 WASM처럼 필요한 필드가 없으면 기존
`PageLayerTree` fallback을 유지한다.

## 2. 변경 내용

### Rust query

- `DocumentCore::get_page_overlay_images_native` 반환 JSON에 다음 필드를 추가했다.
  - `rawSvgCount`
  - `hasBehind`
  - `hasFront`
- `hasBehind`/`hasFront`는 이미지 자체의 `text_wrap`만 보지 않고, 부모 `RenderLayerInfo`를 상속한
  `paint_op_replay_plane_with_layer` 결과로 계산하도록 변경했다.
- `PaintOp::Image`는 `imageCount`에 포함하고, behind/front overlay 배열 emit 여부도 replay-plane 기준으로
  판단한다.
- `PaintOp::RawSvg`는 `imageCount`에 섞지 않고 `rawSvgCount`로 분리했다.

### Studio renderer

- `LayerPlaneSummary`에 `signature`를 추가했다.
- `PageRenderer.getLayerPlaneSummary`가 먼저 `WasmBridge.getPageOverlayImages(pageIdx)`를 호출해
  `hasBehind`/`hasFront`/`imageCount`/`rawSvgCount`를 읽는다.
- overlay summary가 없거나 형식이 맞지 않으면 기존 `getPageLayerTree` 기반 summary로 fallback한다.
- Stage 3의 overlay canvas 재사용 cache key에서 사용할 수 있도록 summary signature를 준비했다.

### 테스트

- `rhwp-studio/tests/render-backend.test.ts`에 `PageRenderer`가 overlay summary를 우선 사용하고
  `PageLayerTree` fallback을 유지하는 정적 계약 검사를 추가했다.
- `tests/issue_938.rs`의 복학원서 overlay 테스트에 새 summary 필드 검증을 추가했다.

## 3. 검증 결과

```bash
node --test tests/render-backend.test.ts
```

- 결과: 통과
- 세부: 24 passed, 0 failed

```bash
cargo test issue_938_overlay_watermark_is_hancom_baked_png --test issue_938
```

- 결과: 통과
- 세부: 1 passed, 0 failed

## 4. 남은 범위

- Stage 3에서 `reason: 'text-edit'` refresh에 한해 기존 overlay canvas를 재사용하도록 연결해야 한다.
- Stage 2는 summary 경량화 단계이므로 `복학원서.hwp`의 background/behind/front 재렌더 자체는 아직 줄이지 않았다.
- `253E164F57A1BC6934-empty.hwp`, `143E433F503322BD33.hwp`의 flow 내부 정적 이미지/OLE 비용은 Stage 4에서
  별도 완화 대상으로 다룬다.
