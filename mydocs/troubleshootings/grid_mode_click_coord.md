# rhwp-studio 그리드 모드 (zoom ≤ 0.5) click 좌표 단일 컬럼 가정 결함

## 본질

[rhwp-studio/src/view/virtual-scroll.ts](../../rhwp-studio/src/view/virtual-scroll.ts)는 줌 ≤ 0.5일 때 **다중 열 그리드 배치**로 분기 (`gridMode = zoom <= 0.5 && viewportWidth > 0 && pages.length > 1`). 그리드 활성 시 각 페이지의 X 좌표는 `pageLefts[i] = marginLeft + col * (pw + gap)` 로 열별 분리 저장되며, [getPageLeft(pageIdx)](../../rhwp-studio/src/view/virtual-scroll.ts#L155) 로 노출된다.

그러나 [rhwp-studio/src/engine/input-handler-mouse.ts](../../rhwp-studio/src/engine/input-handler-mouse.ts)는 `getPageLeft` 를 **단 한 곳도 호출하지 않고** 14개 분기 모두 단일 컬럼 가정 공식 사용:

```ts
const pageLeft = (scrollContent.clientWidth - pageDisplayWidth) / 2;
const pageX = (contentX - pageLeft) / zoom;
```

그리드 모드에서 페이지가 좌/우 열에 분산 배치되므로 위 공식의 `pageLeft` 는 모든 페이지를 "중앙 정렬" 로 가정 → `pageX` 가 실제 페이지 내 좌표와 어긋남 → 모든 마우스 인터랙션 어긋남.

## 영향 분기 (14곳)

| Line | 함수 | 트리거 | 영향 |
|------|------|--------|------|
| 23 | onConnectorMouseDown | 연결선 모드 click | 연결선 시작/끝 좌표 |
| 129 | onMouseDown (표 객체 선택 중) | 선택된 표 click | 표 이동 드래그 시작 hit |
| 176 | onMouseDown (다중 그림 선택) | multi-picture handle click | 합산 BBOX 좌표 |
| 279 | onMouseDown (직선 끝점 드래그) | line endpoint handle click | drag init coord |
| 296 | onMouseDown (회전 드래그) | rotate handle click | rotate center coord |
| 357 | onMouseDown (단일 그림 선택) | 선택된 그림 본체 click | move drag init |
| 431 | onMouseDown (표 리사이즈) | 표 border click | resize drag hit |
| **475** | **onMouseDown (일반 click)** | **left mousedown 전반** | **메인 click 좌표 (cursor.moveTo)** |
| **811** | **onDblClick** | **left dblclick** | **dblclick hit (머리말/꼬리말)** |
| **889** | **onContextMenu** | **right click** | **context menu 표 셀 판정** |
| 931 | onMouseMove (연결선 모드) | connector drawing mousemove | preview 좌표 |
| 1146 | onMouseMove (그림 hover) | mousemove during picture hover | hover cursor |
| 1196 | onMouseMove (표 hover) | mousemove during table hover | hover cursor |
| 1243 | handleResizeHover | 표 border mousemove | hover cursor |

핵심 영향 (사용자 직접 인지): **475 / 811 / 889** — 일반 click / dblclick / context menu 모두 그리드 모드에서 엉뚱한 위치 처리.

## 트리거 조건

`virtual-scroll.ts:29`:
```ts
this.gridMode = zoom <= GRID_ZOOM_THRESHOLD && viewportWidth > 0 && pages.length > 1;
```

- `GRID_ZOOM_THRESHOLD = 0.5`
- 페이지 수 > 1
- 뷰포트 폭 > 0

→ 줌 25%/50% (실제 사용자 시나리오: "전체 보기" 또는 "다중 페이지 미리보기") 에서 활성.

## 영향 범위 (사용자 직접 측정, 2026-05-07)

**한컴**: 그리드 모드 / 일반 모드 모두 **정상 클릭 동작**. ※ 한컴 오피스도 다중 페이지 그리드 모드 (여러 페이지 동시 보기) 가 존재하며, 거기서도 click 좌표는 정확히 처리됨.

**RHWP 그리드 모드 (zoom ≤ 0.5)**: **모든 열에서 click 어긋남** (좌측 열 + 가운데 열 + 우측 열 전부). 일반 모드 (zoom > 0.5) 는 정상.

→ **한컴 호환 결함** (한컴은 정상, RHWP만 어긋남).

원인: 페이지 element 의 실제 left 좌표는 `pageLefts[i]` ([canvas-view.ts:156-163](../../rhwp-studio/src/view/canvas-view.ts#L156-L163) 에서 `style.left = ${pageLeft}px`) 인데, input-handler-mouse 의 14곳 모두 `(clientWidth - pageDisplayWidth) / 2` 단일 컬럼 가정. 그리드 모드에서는 모든 열의 페이지가 슬롯 좌표에 배치되므로 단일 컬럼 가정 공식과 어긋남 (좌/우 방향 + 정도가 열별로 다름).

## 정정 범위

**`getPageLeft(pageIdx)` 호출로 14개 분기 일괄 정정**. 단, 단일 컬럼 모드에서 `getPageLeft = -1` 반환 (CSS 중앙 정렬 sentinel) 이므로 fallback 처리 필요.

권장 패턴:
```ts
const pl = virtualScroll.getPageLeft(pi);
const pageLeft = pl >= 0 ? pl : (scrollContent.clientWidth - pageDisplayWidth) / 2;
```

또는 `virtual-scroll.ts` 에 단일 헬퍼 추가:
```ts
getPageLeftResolved(pageIdx: number, containerWidth: number): number {
  const pl = this.pageLefts[pageIdx] ?? -1;
  if (pl >= 0) return pl;
  const pw = this.pageWidths[pageIdx] ?? 0;
  return (containerWidth - pw) / 2;
}
```

→ `input-handler-mouse.ts` 14곳 모두 `virtualScroll.getPageLeftResolved(pageIdx, sc.clientWidth)` 한 줄로 치환.

## 회귀 영역

- 단일 컬럼 모드 (zoom > 0.5) → `pageLefts[i] = -1` → 기존 `(clientWidth - pw) / 2` 공식 그대로 적용 → 무회귀
- 그리드 모드 → 새 공식 적용 → 사용자 시각 검증 필요 (e2e 줌 25%/50% 클릭 시 cursor 위치 정확)

## 재현 절차

1. `cd rhwp-studio && npx vite --host 0.0.0.0 --port 7700 &`
2. 다중 페이지 HWP 파일 (예: `samples/exam_kor.hwp`) 열기.
3. 줌 25% 또는 50% 로 변경 → 그리드 활성 (좌/우 열 페이지 다중 배치).
4. **2열째 페이지** 본문 텍스트 클릭 → 커서가 엉뚱한 위치 (1열째 페이지 영역 처럼 처리됨) 에 떨어지는지 확인.
5. 또는 머리말 영역 dblclick → 머리말 편집기 미진입 (좌표 어긋남으로 hit_test_header_footer = false 반환).

## 한컴 호환 결함

한컴 오피스는 다중 페이지 그리드 모드 (여러 페이지 동시 보기) 가 존재하며, 그 모드에서 click 좌표는 정확히 처리됨 (사용자 직접 시연 확인, 2026-05-07). RHWP 만 그리드 모드에서 click 좌표 어긋남 → **한컴 호환 결함**.

본 정정의 기대 동작: 한컴 그리드 모드와 동일하게 클릭 → 클릭한 페이지 안의 정확한 위치에 cursor 배치.

## 우선순위

- **High** (UX-blocking): 줌 25%/50% 사용 시 모든 열의 마우스 동작이 어긋남 (사용자 직접 측정 확인).
- 정정 범위는 file 1개 (input-handler-mouse.ts) + 1개 헬퍼 추가 → 작업량 작음.
- 시각 검증은 e2e (Vite dev server + 줌 변경) 으로 빠르게 가능.

## 등록 전 추가 검증 권장

본 노트의 본질 (단일 컬럼 가정 공식 vs 그리드 pageLefts 불일치) 은 코드 + CSS evidence 로 확정. 다만 등록 전 다음 사항 보강 가능:

1. **수치 정량화**: 줌 50% + columns=2 환경에서 좌측 열 click X 좌표 어긋남 정도 (예: ~px 단위) e2e 계측.
2. **다른 input 경로 확인**: keyboard 입력 / IME / textarea 좌표 변환에 동일 결함 영향 있는지 (현 진단은 mouse 만).
3. **Touch / 펜 입력**: touch 이벤트도 같은 14개 분기 사용하는지 별도 확인 (touch handler 별도 가능).

## 정량 측정 결과 (e2e, 2026-05-07)

`rhwp-studio/e2e/grid-mode-click-coord.test.mjs` 실행 결과 (`samples/exam_kor.hwp` 20p, viewport 1600x1000, headless Chrome).

### zoom=0.5 (columns=2)

| page | col | pw | correct (pageLefts[i]) | buggy (단일 컬럼 공식) | delta_px |
|------|-----|-----|------------------------|------------------------|----------|
| 0 | 0 | 561.3 | 223.8 | 509.4 | **+285.6** |
| 1 | 1 | 561.3 | 795.0 | 509.4 | **−285.6** |
| 2 | 0 | 561.3 | 223.8 | 509.4 | +285.6 |
| 3 | 1 | 561.3 | 795.0 | 509.4 | −285.6 |
| ... | ... | ... | ... | ... | ... |

→ 좌측 열 (col 0) 모두 +285.6px, 우측 열 (col 1) 모두 −285.6px 어긋남.

### zoom=0.25 (columns=5)

| page | col | pw | correct | buggy | delta_px |
|------|-----|-----|---------|-------|----------|
| 0 | 0 | 280.6 | 68.4 | 649.7 | **+581.3** |
| 1 | 1 | 280.6 | 359.1 | 649.7 | **+290.6** |
| 2 | 2 | 280.6 | 649.7 | 649.7 | **0.0** |
| 3 | 3 | 280.6 | 940.3 | 649.7 | **−290.6** |
| 4 | 4 | 280.6 | 1230.9 | 649.7 | **−581.3** |

→ 가운데 열 (col 2) 만 우연히 0px 정합. 양 끝 열은 ±581.3px 어긋남.

### zoom=1.0 (단일 컬럼, baseline)

| page | col | pw | correct | buggy | delta_px |
|------|-----|-----|---------|-------|----------|
| 모두 | 0 | 1122.5 | −1.0 | 20.3 | **0.0** |

→ 단일 컬럼 모드는 정합 (correct=−1 sentinel → fallback 공식 = buggy 공식 결과).

### 실제 click 동작 검증

zoom=0.5, page 1 (col 1) 에서 hwpX=100 좌표를 의도한 click:
- CORRECT click @(865.0, 242.0) → cursor.pos = `{sec:0, para:39, char:70}` (정상)
- BUGGY click @(579.4, 242.0) → cursor.pos = `{sec:0, para:31, char:0}` (page 0 의 영역으로 잘못 떨어짐)

**결론**: 그리드 모드에서 input-handler-mouse 의 14개 분기는 모든 페이지에서 ±수백 px 단위로 click 좌표를 어긋나게 만듬. 가운데 열 (홀수 columns 일 때 mid-col) 만 우연히 정합.
