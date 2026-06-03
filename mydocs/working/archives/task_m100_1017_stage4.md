# Task M100 #1017 Stage 4 보고서 — CanvasKit direct replay plane 적용

## 1. 범위

Stage 4에서는 `rhwp-studio` CanvasKit direct renderer에 Stage 2/3에서 확정한 replay plane 순서를 적용했다.

적용 순서:

```text
background -> behindText -> flow -> inFrontOfText
```

native Skia는 Stage 3에서 완료했고, replay plan 진단 갱신은 아직 하지 않았다.

## 2. 변경 파일

| 파일 | 내용 |
|------|------|
| `rhwp-studio/src/view/canvaskit/replay-plane.ts` | CanvasKit replay plane helper 추가 |
| `rhwp-studio/src/view/canvaskit-renderer.ts` | root를 plane별로 multi-pass 순회 |
| `rhwp-studio/tests/render-backend.test.ts` | plane 순서/분류/source regression 테스트 추가 |

## 3. 구현 내용

`CANVASKIT_REPLAY_PLANES`를 다음 순서로 정의했다.

```text
background, behindText, flow, inFrontOfText
```

`layerPaintOpReplayPlane()` 분류:

| LayerPaintOp | replay plane |
|--------------|--------------|
| `pageBackground` | `background` |
| `image` + `wrap="behindText"` | `behindText` |
| `image` + `wrap="inFrontOfText"` | `inFrontOfText` |
| 그 외 모든 op | `flow` |

`CanvasKitLayerRenderer.renderPage()`는 `CANVASKIT_REPLAY_PLANES` 순서로 root를 반복 순회한다.

`renderNode()` / `renderClipNode()` / `renderLeaf()`에는 현재 replay plane 인자를 전달한다. leaf에서는 `layerPaintOpReplayPlane(op) !== replayPlane`인 op를 건너뛴다.

## 4. 회귀 테스트

추가 테스트:

| 테스트 | 검증 |
|--------|------|
| `CanvasKit replay planes match native Skia direct z-order contract` | plane 순서가 native Skia와 동일 |
| `CanvasKit replay plane helper classifies PageLayerTree ops by wrap` | `pageBackground`, `behindText`, `inFrontOfText`, flow op 분류 |
| `CanvasKit renderer source replays the root once per replay plane` | renderer source가 plane별 root 순회와 leaf filtering을 포함 |

기존 source regression도 유지했다.

- `getContext('2d')` 미도입
- `renderPageToCanvas` 미도입
- `rhwpOverlay` 미도입

## 5. 검증

실행:

```text
npm --prefix rhwp-studio ci
npm --prefix rhwp-studio test
npm --prefix rhwp-studio run build
rhwp-studio/node_modules/.bin/tsc --project /private/tmp/rhwp-task1017-stage4-tsconfig.json
```

결과:

```text
npm --prefix rhwp-studio test
45 passed

수정 CanvasKit 소스 별도 타입 체크
passed
```

`npm --prefix rhwp-studio run build` 결과:

```text
src/core/wasm-bridge.ts(1,44): error TS2307: Cannot find module '@wasm/rhwp.js'
src/hwpctl/index.ts(377,57): error TS2307: Cannot find module '@wasm/rhwp.js'
```

해석:

- `rhwp-studio` 전체 build는 repo root의 `pkg/rhwp.js` WASM 산출물이 없어서 실패했다.
- `pkg/`는 `.gitignore` 대상이며, README/CLAUDE 기준으로 Docker WASM build가 생성한다.
- 실패 지점은 Stage 4 수정 파일이 아니라 기존 `@wasm/* -> ../pkg/*` alias 입력 부재다.
- Stage 4 수정 파일은 임시 tsconfig로 별도 타입 체크를 통과했다.

## 6. 다음 단계

Stage 5에서는 Rust `CanvasKitReplayPlan` 진단에 replay plane 정보를 추가한다.

Stage 5 진행 전 작업지시자 승인이 필요하다.
