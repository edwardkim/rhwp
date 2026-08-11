# [#4577] Subsecond 핫패치 렌더 경계 — 처리 결과 (stage 1)

> 이슈: [#4577](https://github.com/edwardkim/rhwp/issues/4577) · 브랜치:
> `fix/issue-4577-subsecond-boundaries` (base: `upstream/devel`)

## 1. 무엇이 문제였나

`HotFn::current(f)` 는 `f` 하나만 리다이렉트한다. 점프 테이블은 `map.get(&real)` 단일 키
조회이고(`subsecond-0.7.10/src/lib.rs:919-940`), wasm `apply_patch` 는 `memory.grow`/`funcs.grow`
로 **덧붙이기만** 한다(`:628-632`). 전이적 재링크가 없으므로 JS 가 부르는 export 마다 경계가
있어야 그 아래가 새 코드로 돈다.

`upstream/devel` 의 경계는 둘뿐이었고, **타이핑 중 실제로 쓰는 경로**가 그 밖에 있었다.

## 2. 무엇을 했나

### 2.1 경계 목록을 한 곳으로 (`src/wasm_api/subsecond_boundary.rs`, 신규)

`#[cfg(feature = "subsecond-dev")]` 4줄 블록을 export 마다 복사하던 모양을 없앴다. 목록에 한
항목을 적으면 세 가지가 **한 선언에서** 나온다.

1. dispatcher — `#[cfg]` 분기는 매크로 안에 한 벌만 있다.
2. `patch_revision()` 의 한 칸 — `getSubsecondPatchRevision` 은 이 함수 하나만 부른다.
   경계를 더할 때 리비전을 따로 고칠 자리가 없으므로 **놓칠 수 없다.**
3. `hot_render_exports()` 의 export 이름 — 테스트가 이것으로 검사한다.

반대 방향은 dispatcher 마다 붙인 `#[deny(dead_code)]` 가 막는다. 목록에 올려 놓고 export 를
배선하지 않으면 빌드가 깨진다(저장소 전역은 `Cargo.toml:208` 에서 `dead_code = "allow"` 라
국소 deny 가 없으면 조용히 통과한다 — 실측으로 확인했다).

### 2.2 새로 경계 뒤로 들어간 export

| export | 왜 |
|---|---|
| `renderPagePatchToCanvasFilteredWithProfile` | 타이핑 중 부분 재도색 페인트. 이슈의 본 증상 |
| `getPageOverlayImages` | `getLayerPlaneSummary` → 합성·**부분 재도색 자격** 판정 |
| `getPageFlowImageOps` | 본문 그림 DOM 배치(같은 평면 분류·조상 clip 접기) |
| `getPageLayerTree` | 이슈에 없던 구멍. `getPageLayerTreeWithProfile` 옆에서 같은 `_impl` 로 가는데 경계를 지나지 않았다. `getLayerPlaneSummaryFromTree`(page-renderer.ts:893) 등 3곳이 쓴다 |

기존 둘(`renderPageToCanvasFilteredWithProfile`, `getPageLayerTreeWithProfile`)은 그대로
말단에 남는다 — 깔때기(`build_page_layer_tree_with_profile`)로 옮기면 페인트를 잃는다는 이슈의
판단을 유지했다. 깔때기 경계는 **추가하지 않았다**: 부분 재도색 export 자체가 경계 뒤로
들어가면서 `PageRenderer` 의 모든 트리 빌드 경로가 이미 경계 아래가 되어, 안쪽에 하나 더 두면
재도색마다 점프 테이블 조회만 한 번 늘고 새로 덮이는 코드가 없다.

### 2.3 인자 9개 상한 (경계가 없던 진짜 이유)

`subsecond` 는 `HotFunction` 을 인자 9개까지만 구현한다(`impl_hot_function!` 의 `Fn9Marker`).
`render_page_patch_to_canvas_filtered_with_profile_impl` 은 인자가 10개라 `HotFn::current` 가
**컴파일되지 않는다.** `x/y/width/height` 를 `BoundingBox` 하나로 접어 7개로 줄였다.
wasm_bindgen export 의 JS 시그니처는 그대로다.

## 3. 경계 밖에 그대로 둔 것

| export | 이유 |
|---|---|
| `getPageSourceImageKeys` | 그림 신원 키만 만든다 — 아래에 페인트도 평면 분류도 없다. 캐시 신원을 핫패치하면 이미 저장된 서명과 비교가 불가능해져 재디코드만 는다 |
| `getSourceImageBytes` | 결과를 studio 가 키별 object URL 로 메모이즈한다(`FlowImageUrlCache`, digest+generation 키). 핫패치는 신원을 바꾸지 않으므로 다시 들어오지 않는다 — 경계를 두어도 관측되지 않는다 |
| `getPageInfo` | 이미 계산된 pagination(`find_page`)을 읽고 PageDef 여백을 px 로 환산할 뿐이다. 낡을 수 있는 값(page 크기·단 영역)은 `invalidateSubsecondRenderCaches` 가 비우지 않는 pagination 에서 오므로 경계를 두면 "새 산술 + 옛 페이지네이션" 반만 새 기록이 된다. 목록에서 가장 잦은 질의이기도 하다 |
| `getCanvasKitReplayPlan(WithProfile)` | `PageRenderer` 가 부르지 않는다 — 소비자는 `e2e/renderer-baseline.mjs:486` 뿐이다. CanvasKit 페인트는 TypeScript 이고 입력 트리는 이미 경계 뒤에서 온다 |

판단은 `src/wasm_api/subsecond_boundary.rs` 의 `DELIBERATELY_COLD_EXPORTS` 에 이유와 함께
적혀 있고, 테스트가 "경계 목록과 겹치지 않는다 + 이유가 비어 있지 않다"를 지킨다.

## 4. 테스트 — 고치기 전 RED 실측

`every_render_path_export_sits_behind_a_hot_patch_boundary`. 목록에서 새 경계 셋을 뺀
(= 고치기 전) 트리에서:

```
thread '...::every_render_path_export_sits_behind_a_hot_patch_boundary' panicked at
src/wasm_api/subsecond_boundary.rs:211:9:
이 export 들은 PageRenderer 의 렌더 경로에 있는데 핫패치 경계 뒤에 없다 — 패치를 걸어도
base 모듈의 옛 코드가 그린다:
["renderPagePatchToCanvasFilteredWithProfile", "getPageOverlayImages", "getPageFlowImageOps"]
test result: FAILED. 2 passed; 1 failed
```

리비전 쪽도 물린다. 생성기가 첫 칸만 쓰도록 일부러 망가뜨리면
`patch_revision_covers_every_compiled_boundary` 가 `리비전 칸은 16자리 함수 주소여야 한다:
0000000100b83dcc::` 로 실패한다.

## 5. 검증하지 못한 것

**end-to-end 는 검증하지 않았다.** `dx serve --hot-patch` + 실제 패치로 "표 셀에 한 글자 쳐도
새 색으로 그려진다"를 확인하려면 구동 환경이 필요하고, 이 작업에서는 돌리지 못했다. 확인한
것은 컴파일 계약(경계가 붙는다·리비전이 경계 수만큼 나온다·주소가 서로 다르다)까지다.

덧붙여 `subsecond` 의 `HotFn::try_call` 은 `!cfg!(debug_assertions)` 이면 점프 테이블을 아예
건너뛴다(`lib.rs:412-414`). 핫패치는 debug 빌드에서만 동작한다 — 기존 성질이고 이번 변경과
무관하다.

## 6. 발견했지만 고치지 않은 것

- `CanvasView.refreshPages()` 는 `getPageInfo` 를 다시 모으지만 `this.pageRenderer` 의
  `FlowImageUrlCache` 는 digest+generation 이 그대로라 그대로 남는다. 즉 워터마크 bake
  (`renderer::image_resolver::emitted_image_bytes`, `rendering.rs:1750`)를 고쳐도 본문 그림은
  옛 바이트를 쓴다. 경계 배치 문제가 아니라 캐시 수명 문제다 (#4579 계열).
- `body_page_border_outset`(`src/renderer/layout/border_rendering.rs:963`)을 고치면 그려진
  페이지 테두리는 새 코드, `getPageInfo` 의 `pageBorderLeft/Right/Top/Bottom`
  (`rendering.rs:2261`)은 옛 코드가 되어 여백 가이드가 어긋난다. 위 3절의 이유로 이번에는
  경계를 두지 않았다.
- `cargo check --target wasm32-unknown-unknown` 은 `rhwp` **bin** 타깃에서 깨진다
  (`src/main.rs:4455`, `:4599` 등 7건). `upstream/devel` 에서도 같으므로 이번 변경과 무관하다.
  wasm 게이트는 `--lib` 로 돌려야 한다(`wasm-pack build` 가 하는 것과 같다).
- `rhwp-studio/tests/subsecond-runtime.test.ts` 의 마지막 테스트는 `src/wasm_api.rs` 를 문자열
  정규식으로 검사한다. `getSubsecondPatchRevision` 이라는 이름이 있는지만 보므로 이번 이슈의
  구멍(경계가 두 개뿐)을 전혀 보지 못했다. 이번에 추가한 Rust 쪽 소진 테스트가 그 자리를
  대신하지만, 정규식 테스트 자체는 남겨 두었다(범위 밖).

## 7. 범위

`src/wasm_api.rs` 와 신규 `src/wasm_api/subsecond_boundary.rs` 만 건드렸다.
`src/document_core/queries/rendering.rs` 는 **읽기만 했다** — #4576 이 같은 두 파일을 고치므로
충돌 면적을 줄이려고 렌더 본체(`*_native`)는 그대로 두고 wasm_api 층에서만 경계를 붙였다.
무효화 계약(#4576)·진단(#4578)·수명(#4579)·번들링(#4580)은 손대지 않았다.
