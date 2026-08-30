# 구현 계획 — Task M100 #6041

- **이슈**: [#6041](https://github.com/edwardkim/rhwp/issues/6041)
- **브랜치**: `codex/issue-6041-budget-first-render-scale`
- **stack base**: `codex/issue-6040-zoom-topology@dfe27e188`
- **설계 승인 출처**: 2026-08-30 작업지시자 대화·이슈 본문
- **문서 보정**: 최초 구현 뒤 저장소 계획 파일 누락을 발견해 실제 구현·검증 순서를 소급 없이 기록

## 정책 모델

페이지 비용은 열 수가 아니라 다음 값으로 계산한다.

```text
surfacePixels(page) = width × height × zoom² × effectiveDpr² × layerCount(page)
```

- 기본 visible 예산은 32M surface pixels, retained 예산은 40M이다.
- `layerCount(page)`는 Canvas2D plane 요약으로 계산한다. main=1, static-flow 가능 시 +1,
  BehindText가 있으면 background+behind로 +2, InFrontOfText가 있으면 +1이다.
- 아직 plane 요약을 읽을 수 없는 호출에는 Canvas2D 4를 fallback으로 사용하지만, 정상 문서 경로는
  페이지별 값을 전달한다. CanvasKit은 1이다.
- full-quality 비용이 예산 이내면 모든 페이지가 raw DPR을 유지한다.
- 초과하면 visible budget은 비포커스 visible만, retained budget은 화면 밖 페이지를 먼저 낮춘다.
- 단계는 raw→2→1.5→1이며 한 후보를 1까지 내리기 전에 같은 우선순위의 후보를 한 단계씩 낮춘다.
- release 기준은 예산의 88%로 두어 경계에서 승격/강등 반복을 막는다.

## 파일별 구현

### `rhwp-studio/src/view/render-surface-budget.ts`

- DOM과 독립된 순수 planner, DPR ladder, 예산/우선순위/hysteresis를 소유한다.
- 입력 페이지가 자기 `layerCount`를 제공하면 전역 fallback보다 우선한다.
- 결정값에 DPR, tier, layer count, page/visible/retained surface 비용을 반환한다.

### `rhwp-studio/src/view/page-renderer.ts`

- 페이지 overlay/tree 요약에서 Canvas surface 상한을 계산한다.
- 계산값은 zoom과 독립적이므로 페이지 단위로 캐시하고 backend/profile, 문서, revision, dispose 경계에서
  무효화한다.
- flow-static 지원 실패 시 과대 추정이 남지 않게 캐시를 비운다.
- 기존 render plane 생성, CanvasKit fallback, 출력 profile 동작은 바꾸지 않는다.

### `rhwp-studio/src/view/canvas-view.ts`

- 현재 retained/visible/focus 집합으로 planner를 호출한다.
- effective DPR 또는 tier가 달라진 active 페이지만 다시 그린다. 가시성 진단만 달라지면 raster하지 않는다.
- 기존 `clampRenderScale()` 앞에서 effective DPR을 선택해 브라우저별 hard cap을 그대로 보존한다.
- `data-rhwp-surface-layer-count`와 비용/tier 진단을 노출한다.

### `rhwp-studio/tests/render-surface-budget.test.ts`

- 열 수 독립, raw 유지, 단계적 강등, offscreen/far-first, focus/export 보존, hysteresis를 검증한다.
- fallback 4가 있어도 단일 Canvas 페이지는 `layerCount=1`로 계산해 모두 raw DPR을 유지함을 고정한다.
- CanvasView가 페이지별 layer count를 전달하는 wiring을 회귀 테스트한다.

## 검증과 rollback

- focused planner test, TypeScript, Studio 전체 test, production build, `git diff --check`
- 실제 Canvas2D 34% 4쪽과 100% 다중 쪽 일반 문서에서 raw DPR 보존
- `basic/KTX.hwp`의 main/background/behind/front 4-layer 식별
- CanvasKit에서 layerCount 1과 기존 raw DPR 보존
- 회귀가 확인되면 budget planner 호출을 제거하면 기존 `zoom×rawDpr→clampRenderScale` 경로로 즉시
  돌아갈 수 있다. #6040 배치 코드는 이 layer에서 수정하지 않는다.

