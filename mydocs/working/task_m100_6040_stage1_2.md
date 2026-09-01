# 작업 기록 — Task M100 #6040 Stage 1.2

- **이슈**: [#6040](https://github.com/edwardkim/rhwp/issues/6040)
- **PR**: [#6458](https://github.com/edwardkim/rhwp/pull/6458)
- **브랜치**: `codex/issue-6040-zoom-topology`
- **최신 기준**: `upstream/devel@0d1540931`
- **최신 devel 통합 commit**: `a19020085`
- **작성일**: 2026-09-01 KST
- **Stage 범위**: live 자동 열 commit 연결, 문서·배치 경계 reset, resize 뒤 오버레이 재투영

## 문제와 판단

Stage 1의 순수 `resolveAutoPageColumns()`는 이전 확정 열을 입력받아 히스테리시스를 적용했지만,
실제 `VirtualScroll.setPageDimensions()` 호출은 그 값을 보존하거나 전달하지 않았다. 따라서 같은
인스턴스에 `814 → 806 → 818 → 806px` 같은 경계 입력이 연속되면 순수 helper의 계약과 달리 열 수가
매 호출의 단발 후보로 다시 계산될 수 있었다.

최신 devel을 통합한 뒤 실제 브라우저에서 resize를 반복하자 페이지와 눈금자는 새 행·열 좌표를 사용하지만
캐럿 오버레이는 이전 슬롯에 남는 별도 회귀도 확인됐다. 열 commit은 `VirtualScroll`이 소유하고,
CanvasView의 기존 settled 재레이아웃·Canvas 반환 경로는 유지한다. 캐럿·선택 오버레이는 CanvasView가
같은 `viewport-resize` 이벤트에서 레이아웃을 확정한 다음 tick에 authoritative 좌표를 다시 읽는다.

PR #6438에서 지적된 progressive Canvas 교체와 active pool/DOM 이중 소유권 문제를 되살리지 않기 위해
`releaseAllRenderedPages() → updateVisiblePages()` 계약은 변경하지 않는다.

## 변경

- `VirtualScroll`이 마지막 자동 열 commit을 보존하고 다음 auto 계산에 전달한다.
- horizontal 또는 명시 배치로 전환하면 자동 열 commit을 지운다.
- `CanvasView.reset()`에서 문서 교체 전 commit을 지워 새 문서의 첫 후보에 섞이지 않게 한다.
- 기존 zoom 오버레이 갱신을 한 메서드로 모으고, viewport resize에서는 레이아웃 확정 다음 tick에
  캐럿·필드·텍스트/셀 선택·그림/표 선택을 같은 좌표로 재투영한다.
- 실제 `CanvasView.onZoomChanged()`를 Vite SSR로 실행해 zoom event당 레이아웃 commit 1회와
  CanvasPool active page/DOM canvas의 단일 소유권을 검증한다.

## 자동화 검증

- 자동 열 focused test: 같은 `VirtualScroll`에서 `800 → 814 → 806 → 818 → 806 → 801px`를 넣어
  `1 → 1 → 1 → 2 → 2 → 1열`로만 확정되는 계약을 고정했다.
- 명시 배치 전환과 문서 reset이 이전 auto commit을 제거하는 회귀 test를 추가했다.
- 실제 CanvasView zoom test에서 1→2열 경계를 왕복하고, 초기 1회와 zoom event당 1회만 레이아웃을
  commit하며 매 정착 뒤 DOM canvas 수·고유 page slot·CanvasPool active page가 일치함을 확인했다.
- InputHandler source contract test로 zoom과 resize가 같은 overlay 재투영 경로를 사용함을 고정했다.
- focused test: 34/34 pass
- TypeScript `npx tsc --noEmit`: 통과
- 전체 Studio: 1,352건 중 1,351 pass·1 policy skip·0 fail
- production build: 246 modules, 통과. 기존 CanvasKit browser externalize와 대형 chunk 경고만 확인
- `git diff --check`: 통과

## 실제 브라우저 검증

77쪽 `kps-ai.hwp`를 Canvas2D, 자동 배치, 77%에서 열고 뷰포트 폭을 왕복했다.

| 편집 viewport 폭 | 1225px | 1235px | 1240px | 1235px | 1220px |
| --- | ---: | ---: | ---: | ---: | ---: |
| 확정 열 수 | 1 | 1 | 2 | 2 | 1 |
| DOM canvas / 고유 page slot | 3/3 | 3/3 | 4/4 | 4/4 | 3/3 |
| 현재 쪽 | 2/77 | 2/77 | 2/77 | 2/77 | 2/77 |

- 증가·감소 경계 안에서는 기존 열 commit이 유지되고 dead band 밖에서만 1↔2열로 전환됐다.
- 캐럿은 매 단계에서 현재 2쪽 안에 있었고 1열로 돌아오면 처음 좌표로 복귀했다.
- 수평 눈금자 canvas 폭은 매 단계 편집 viewport 폭과 정확히 일치했다.
- 클릭 hit-test 뒤 현재 쪽은 2/77로 유지됐다.
- DOM canvas 수와 고유 page slot 수가 항상 같아 orphan/중복 Canvas가 없었다.

## 범위 감사와 다음 게이트

이 Stage는 live 열 commit과 오버레이 좌표 동기화만 보정한다. 폐기된 Canvas 전용 zoom preview,
점진 Canvas 교체, #6041 render scale, #6042 LRU·scheduler는 포함하지 않는다.

#6444는 contributor credit과 head 계보를 남기고 #6458→#6467→#6042로 인계한 뒤 닫았다. #6438은
이 Stage를 원격 #6458에 게시해 maintainer가 대체 구현을 볼 수 있게 한 뒤 같은 방식으로 인계·종료한다.
그 전에는 contributor PR을 닫지 않는다.
