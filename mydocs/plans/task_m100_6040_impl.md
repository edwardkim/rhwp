# 구현 계획 — Task M100 #6040

- **이슈**: [#6040](https://github.com/edwardkim/rhwp/issues/6040)
- **브랜치**: `codex/issue-6040-zoom-topology`
- **PR 기준 commit**: `upstream/devel` `b9d408f0d`
- **최초 구현 기준**: `upstream/devel` `2deb3dd61`
- **문서 성격**: 구현 전 파일·상태 전이 설계
- **계획 승인**: 2026-08-30 작업지시자 승인, Stage 1 진행
- **현재 상태**: Stage 2·3 구현 폐기, Stage 1·1.1 유지, Stage 1.2 live 자동 열 commit 보정. 아래 줌
  상태 전이 설계는 역사적 폐기안이며 새 공유 좌표 계획 승인 전에는 구현하지 않는다.

## 자동 열 계약

### 후보 계산

자동 열 후보는 이미 zoom이 적용된 표시 폭을 입력으로 받아 다음 의미를 갖는다.

```text
fitColumns = floor((viewportWidth + pageGap) / (maxDisplayedPageWidth + pageGap))
candidate = clamp(fitColumns, 1, pageCount)
```

- 페이지가 없으면 1열의 빈 레이아웃, 한 쪽이면 1열을 반환한다.
- invalid/0 viewport와 invalid page width는 안전하게 1열로 수렴한다.
- 50%라는 절대 배율은 입력과 분기에서 제거한다.
- zoom과 resize는 동일 순수 계산을 호출한다.
- 같은 경계의 미세한 왕복 입력에는 현재 commit 열 수와 CSS px 단위 히스테리시스 여유를 함께 전달해
  candidate가 즉시 되돌아가지 않게 한다.

### 점유 그리드 중앙 정렬

- `auto`의 commit 열 수는 page count보다 클 수 없으므로 3쪽 문서의 첫 행이 3쪽이면 점유 열도 3이다.
- 중앙 정렬 폭은 `occupiedColumns × maxPageWidth + gaps`를 사용한다.
- 페이지별 폭이 다른 경우 각 slot 안의 기존 중앙 정렬을 유지한다.
- `double`, `facing`, `multiple`은 지정 topology 자체가 의미이므로 의도적인 빈 slot을 유지한다.
- 마지막 미완성 행은 이 작업에서 별도 가운데 정렬하지 않고 기존 `pageIdx % columns` 규칙을 보존한다.

## 줌 상태 전이

> **폐기된 설계**: 이 절의 Canvas 전용 preview는 눈금자·캐럿·선택·hit-test와 다른 좌표계를 만들어
> 회귀를 일으켰다. 현재 branch는 최신 PR 기준 대비 `CanvasView`·`ViewportManager`·caret/input 경로를
> 변경하지 않으며, `Ruler`는 Stage 1.1의 끝 라벨 경계 처리만 포함한다. 후속 설계는 모든 소비자가 하나의
> authoritative preview geometry를 사용하거나, 기존처럼 매 프레임 같은 `VirtualScroll`을 소비해야 한다.

```text
idle
  └─ 첫 animation zoom event → preview(snapshot)
preview
  ├─ animation frame → committed topology 유지 + active element CSS preview
  └─ settled event → commit(final candidate, one layout pass, anchor restore)
commit
  └─ visible active Canvas를 새 품질로 점진 교체 → idle
```

snapshot에는 최소한 다음 값을 둔다.

- 시작 zoom과 committed topology key/columns
- 기준 페이지와 그 페이지 안의 정규화된 x/y 앵커
- 시작 viewport와 scrollLeft/scrollTop
- active page별 기존 rendered zoom/epoch

애니메이션 프레임은 전체 page array의 `setPageDimensions()`를 다시 호출하지 않는다. committed
VirtualScroll 좌표와 snapshot으로 현재 active Canvas·overlay에만 preview box를 적용하고 배율 표시는 계속
갱신한다. 정착 event는 final zoom의 candidate로 `recalcLayout()`을 정확히 한 번 호출하고 새 page box에서
정규화 앵커를 복원한다.

## 파일별 구현

### `rhwp-studio/src/view/virtual-scroll.ts`

- `GRID_ZOOM_THRESHOLD`와 그 분기를 제거한다.
- auto candidate/commit에 필요한 순수 helper 또는 작은 value object를 추가한다.
- `VirtualScroll` 인스턴스가 auto의 이전 committed columns를 보존하고 다음 `setPageDimensions()` 계산에
  전달한다. horizontal·고정 배치·문서 교체에서는 이를 reset하며 fixed arrangement 호출 계약은 바꾸지
  않는다.
- `getLayoutTopologyKey()`에 commit된 auto columns만 반영해 preview 후보가 topology 변경으로 보이지 않게
  한다.
- auto의 grid width와 margin은 실제 점유 열을 사용한다.

### `rhwp-studio/src/view/canvas-view.ts`

- **Stage 1.2 실제 변경**: `reset()`이 `VirtualScroll.resetAutoColumnCommit()`을 호출해 이전 문서의
  경계 상태를 제거한다. settled zoom의 기존 전체 반환·재할당 경로는 변경하지 않는다.

아래 항목은 폐기된 Stage 2 설계이며 현재 code candidate에 포함되지 않는다.

- `ZoomLayoutPreviewSession` 상태와 render epoch를 관리한다.
- `onZoomChanged()`를 animation preview와 settled commit 경로로 분리한다.
- preview 경로에서는 `recalcLayout()`, `renderPage()`, `releaseAllRenderedPages()`를 호출하지 않는다.
- settled 경로는 final candidate를 commit한 뒤 한 번만 레이아웃을 계산하고 snapshot의 기준 페이지/정규화
  앵커로 scroll을 복원한다.
- resize는 같은 candidate resolver를 사용하되 독립적인 단일 commit으로 처리한다.
- fixed arrangement에서는 topology key가 바뀌지 않으므로 좌표 재배치와 품질 교체만 수행한다.

### `rhwp-studio/src/view/canvas-pool.ts`

- 현재 page→Canvas 소유권을 유지한 채 교체 후보 Canvas를 준비하고 성공 시 swap하는 최소 API를 둔다.
- swap 전까지 기존 Canvas는 DOM과 pool의 active entry로 남는다.
- 취소·실패한 후보는 DOM에 노출하지 않고 available pool로 돌린다.
- page 결과를 viewport 밖에 보존하는 LRU/eviction은 추가하지 않는다.

### `rhwp-studio/src/view/page-renderer.ts`

- 필요한 경우 staging Canvas 렌더가 기존 페이지 layer를 먼저 제거하지 않도록 render/commit 경계를
  분리한다.
- 최신 render epoch의 성공 결과만 Canvas와 부속 layer를 commit한다.
- render scale 계산은 현재 `zoom × rawDpr`와 `clampRenderScale()`을 그대로 사용한다. tier·surface 예산은
  #6041에서 구현한다.

### 진단 경로

- 기존 DEV 진단 패턴을 따라 제스처당 preview frame, layout commit, candidate change, full release,
  page replacement 횟수와 anchor CSS px 오차를 관찰 가능하게 한다.
- production 동작이나 공개 문서 모델에는 진단 상태를 저장하지 않는다.

### `rhwp-studio/src/engine/input-handler.ts`

- **Stage 1.2 실제 변경**: 기존 zoom overlay 갱신을 공통 메서드로 모으고, viewport resize에서는
  CanvasView가 레이아웃을 확정한 다음 tick에 캐럿·필드·텍스트/셀 선택·그림/표 선택을 다시 투영한다.
- 별도 preview 좌표를 만들지 않고 기존 cursor/VirtualScroll authoritative geometry만 소비한다.

## 테스트

### `virtual-scroll-page-arrangement.test.ts`

- 51%에서도 폭이 두 쪽만 허용하면 2열, 세 쪽을 허용하면 3열
- 50% 전후가 동일 geometry에서 동일 candidate
- 3쪽 27%·17%에서 columns=3이고 묶음 중심 오차 ≤1 CSS px
- page count보다 큰 fitColumns의 cap
- invalid/0 viewport의 1열 fallback
- single/double/facing/multiple과 horizontal movement의 topology 불변
- 페이지 폭이 다른 auto slot의 내부 중앙 정렬

### 줌 preview/commit focused test

- animation N frame에서 전체 layout commit 0, settled event에서 1
- animation frame에서 final-quality `renderPage`와 `releaseAllRenderedPages` 0
- 경계 하나를 넘는 제스처의 topology commit 1 이하
- 기준 페이지/정규화 앵커 오차 ≤2 CSS px
- zoom in/out 왕복과 resize가 같은 최종 candidate를 선택
- stale render epoch가 새 Canvas를 덮어쓰지 않고 실패 시 기존 Canvas 유지

### 기존 회귀

- #685, #689, #2560의 grid hit/current-page/row navigation
- #3244, #3245, #3246, #3591의 zoom sensitivity/anchor/horizontal pan
- page arrangement transaction이 최종 `recalcLayout()` 한 번만 수행하는 계약
- Canvas2D와 CanvasKit의 기존 render 성공·fallback 경로

## Stage별 예상 변경 경계

1. Stage 1: `virtual-scroll.ts`와 자동 배치 focused test, working 보고서
2. Stage 1.1: 수평 눈금자 끝 라벨 경계 처리와 실제 브라우저 검증
3. Stage 1.2: live auto commit·문서 reset·resize overlay 재투영과 actual CanvasView/CanvasPool test
4. Stage 2: **폐기**, Canvas 전용 줌 상태 전이와 앵커 설계를 현재 branch에서 제거
5. Stage 3: **폐기**, 점진 Canvas 교체와 계측을 현재 branch에서 제거
6. Stage 4: Stage 1~1.2 통합 test·build·browser evidence와 최종 보고서

각 Stage 결과 승인 뒤 해당 source·test·보고 문서를 하나의 검토 가능한 commit으로 고정한다. 실제 조사에서
파일 책임이 달라지면 source를 수정하기 전에 이 구현 계획과 승인 기록부터 갱신한다.

## native stacked PR 연결 계획

1. #6040의 네 Stage와 최종 승인을 마치고 bottom head를 고정한다.
2. 그 head에서 `codex/issue-6041-adaptive-render-scale`을 만들고 #6041 수행·구현 계획 승인을 받는다.
3. #6041 완료 head에서 `codex/issue-6042-page-virtualization`을 만들고 #6042 수행·구현 계획 승인을 받는다.
4. `gh stack view`에서 `devel ← #6040 ← #6041 ← #6042` 순서와 각 layer diff를 확인한다.
5. 세 head의 범위별·누적 검증, 한국어 PR 제목·본문·stack map을 사용자에게 보고하고 게시 승인을 받는다.
6. `gh stack submit --remote upstream`은 push와 PR 생성을 함께 수행하므로 승인 뒤 한 번 실행한다.
7. GitHub native stack public preview의 cascading rebase 뒤에는 영향받은 상단 head를 다시 동기화·검증한다.

## 커밋 경계 후보

1. `docs(studio): #6040 자동 줌 토폴로지 계획을 기록한다`
2. `fix(studio): 자동 페이지 열과 점유 그리드 정렬을 바로잡는다`
3. `perf(studio): 줌 preview와 토폴로지 commit을 분리한다`
4. `perf(studio): 줌 정착 Canvas를 점진 교체한다`
5. `docs(test): #6040 통합 검증 결과를 기록한다`

#6041의 render scale tier와 #6042의 LRU·scheduler 변경은 이 branch에 포함하지 않는다.
