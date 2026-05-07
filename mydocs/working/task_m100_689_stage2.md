# Task #689 Stage 2 단계 보고서 — `getPageAtY` 18곳 → `getPageAtPoint` 치환 + Task #685 누락 buggy `pageLeft` 10곳 동반 정정

- **이슈**: [#689](https://github.com/edwardkim/issues/689)
- **수행계획서**: [task_m100_689.md](../plans/task_m100_689.md)
- **구현계획서**: [task_m100_689_impl.md](../plans/task_m100_689_impl.md)
- **단계 위치**: 3 단계 중 2/3
- **변경 성격**: 본질 정정 (그리드 모드 모든 col click 정합) + Task #685 sweep 누락분 동반 정정
- **작성일**: 2026-05-08

---

## 변경 요약

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/engine/input-handler-mouse.ts` | 12곳 `getPageAtY` → `getPageAtPoint` 치환 (1:1 라인 변경) |
| `rhwp-studio/src/engine/input-handler.ts` | 4곳 `getPageAtY` → `getPageAtPoint` + 4곳 buggy `pageLeft` → `getPageLeftResolved` |
| `rhwp-studio/src/engine/input-handler-table.ts` | 1곳 `getPageAtY` 치환 + 1곳 buggy `pageLeft` (L400) + **추가 2곳 buggy `pageLeft`** (L74, L111) |
| `rhwp-studio/src/engine/input-handler-picture.ts` | 1곳 `getPageAtY` 치환 + 1곳 buggy `pageLeft` (L594) |
| `rhwp-studio/src/engine/input-handler-connector.ts` | **추가 2곳 buggy `pageLeft`** (L85, L152) |

총 변경: **18곳 `getPageAtY` 치환** + **10곳 buggy `pageLeft` 정정** = 코드 변경 ~+28/-28 LOC.

---

## 1. `getPageAtY` → `getPageAtPoint` 치환 (18곳)

### 1.1 input-handler-mouse.ts (12곳, 3 회 `replace_all`)

| 라인 | 함수 | 변수 패턴 | 치환 후 |
|------|------|-----------|---------|
| 20 | onClick (연결선 모드) | cy | `getPageAtPoint(cx, cy)` |
| 126 | onClick (선택된 표 이동) | cy | 동일 |
| 173 | onClick (다중 그림 선택) | cy | 동일 |
| 354 | onClick (단일 그림 본체) | cy | 동일 |
| 928 | onMouseMove (연결선 미리보기) | cy | 동일 |
| 1143 | onMouseMove (그림 hover) | y | `getPageAtPoint(x, y)` |
| 1193 | onMouseMove (표 hover) | y | 동일 |
| 428 | onClick (셀 선택 표 리사이즈) | contentY | `getPageAtPoint(contentX, contentY)` |
| 470 | onClick (일반 click main path) | contentY | 동일 |
| 807 | onDblClick (머리말/꼬리말 진입) | contentY | 동일 |
| 886 | onContextMenu | contentY | 동일 |
| 1240 | handleResizeHover | contentY | 동일 |

`replace_all` 3 회 (cy / y / contentY) — 각 패턴 안에서 X 변수 (cx/x/contentX) 가 모두 정의되어 있어 안전.

### 1.2 input-handler.ts (4곳, 2 회 `replace_all`)

| 라인 | 함수 | 변수 패턴 | 치환 후 |
|------|------|-----------|---------|
| 612 | 그림 객체 중심 좌표 → 페이지 | cY | `getPageAtPoint(cX, cY)` |
| 875 | 표 객체 중심 좌표 → 페이지 | cY | 동일 |
| 972 | hitTest helper | contentY | `getPageAtPoint(contentX, contentY)` |
| 1542 | hitTest helper (표 동일성 검증) | contentY | 동일 |

### 1.3 input-handler-table.ts:400 + input-handler-picture.ts:594 (2곳)

각 1건씩 컨텍스트 단건 Edit. `cy` → `cx, cy` 패턴 동일.

---

## 2. Task #685 sweep 누락 buggy `pageLeft` 동반 정정 (10곳)

### 2.1 마우스 컨텍스트 (`getPageAtY` 와 동일 함수, 6곳)

| 파일 | 라인 | pageIdx 출처 | 치환 후 |
|------|------|--------------|---------|
| input-handler.ts | 612 | `getPageAtPoint(cX, cY)` | `getPageLeftResolved(pageIdx, (scrollContent as HTMLElement).clientWidth)` |
| input-handler.ts | 875 | `getPageAtPoint(cX, cY)` | `getPageLeftResolved(pageIdx, scrollContent.clientWidth)` |
| input-handler.ts | 972 | `getPageAtPoint(contentX, contentY)` | 동일 |
| input-handler.ts | 1542 | `getPageAtPoint(contentX, contentY)` | 동일 |
| input-handler-table.ts | 400 | `getPageAtPoint(cx, cy)` | `getPageLeftResolved(pi, sc.clientWidth)` |
| input-handler-picture.ts | 594 | `getPageAtPoint(cx, cy)` | 동일 |

### 2.2 추가 발견 사이트 (`getPageAtY` 없음, pageIdx 신뢰값, 4곳) — 작업지시자 결정 (2026-05-08)

`getPageAtY` 사용 안 함 (state 객체 또는 함수 매개변수에서 pageIdx 획득) 이지만 buggy `pageLeft` 패턴 사용 중. #689 의 본질 결함 (`getPageAtY` X 무시) 과 무관하나, **#685 sweep 누락분의 같은 카테고리** — 작업지시자 추가 승인으로 동반 정정.

| 파일 | 라인 | pageIdx 출처 | 함수 | 치환 후 |
|------|------|--------------|------|---------|
| input-handler-table.ts | 74 | `this.resizeDragState.edge.pageIndex` | updateResizeDrag (드래그 마커) | `getPageLeftResolved(pageIdx, scrollContent.clientWidth)` |
| input-handler-table.ts | 111 | `state.edge.pageIndex` | finishResizeDrag | 동일 |
| input-handler-connector.ts | 85 | 함수 매개변수 `pageIdx` | onConnectorClick (연결점 후보 검색) | `getPageLeftResolved(pageIdx, sc.clientWidth)` |
| input-handler-connector.ts | 152 | 함수 매개변수 `pageIdx` | renderConnectorPreview | 동일 |

→ 본 추가 정정으로 그리드 모드에서 표 리사이즈 드래그 / connector 좌표 어긋남도 함께 해소.

---

## 3. 검증 결과

### 1. grep sweep — buggy pattern 잔여 0건

```
$ grep -rnE "clientWidth\s*-\s*\w+\)\s*/\s*2" src/
(0 매칭, exit=1)
```

### 2. 헬퍼 호출 수 카운트

```
$ grep -r "getPageAtPoint" src/ | wc -l → 19   (정의 1 + 사용 18)
$ grep -r "getPageLeftResolved" src/ | wc -l → 26 (정의 1 + 사용 25)
```

사용 25 분포:
- #685 정정분: input-handler-mouse 14 + input-handler.ts 1 (formBboxToOverlayRect) = **15**
- #689 정정분: input-handler.ts 4 (L612/875/972/1542) + input-handler-table 3 (L400/74/111) + input-handler-picture 1 (L594) + input-handler-connector 2 (L85/152) = **10**

→ 정확히 일치 (15 + 10 = 25).

### 3. typecheck

```
$ npx tsc --noEmit
(무에러)
```

### 4. vite build

```
$ npx vite build
✓ 85 modules transformed.
✓ built in (정상)
PWA v1.2.0 — generateSW 정상
```

### 5. e2e — `body-outside-click-fallback.test.mjs --mode=headless`

```
$ exit=0
- ERROR / FAIL / 에러 / 가설.yes: 0 매칭
```

→ 단일 컬럼 모드 click 좌표 무회귀 확인 (헬퍼 두 종 모두 sentinel/Y-only fallback 경로 정상).

---

## 4. 회귀 위험 점검

| 영역 | 위험 | 결과 |
|------|------|------|
| 단일 컬럼 모드 click | `getPageAtPoint` 가 `getPageAtY` 동치 안 되면 회귀 | OK — 명시 분기 + body-outside-click-fallback e2e 무회귀 |
| 단일 컬럼 모드 pageLeft | `getPageLeftResolved` fallback 식 무회귀 | OK — #685 에서 이미 검증, 본 작업 추가 사이트도 동일 헬퍼 |
| `pw` / `pageDisplayWidth` 변수 | hit test bbox 등 다른 사용처 | OK — 모두 보존, typecheck 통과 |
| 변수명 mismatch (`cx`/`x`/`contentX`/`cX`) | 페어링 잘못 | OK — `replace_all` 패턴 분리 + 각 그룹 변수명 일관 (e.g., `cy` → `cx, cy` 매칭) |
| viewport-center 영역 (canvas-view, keyboard) | 헬퍼 통일 욕심 시 회귀 | OK — 미수정 명시 |
| 추가 4 사이트 (table L74/111, connector L85/152) | 신뢰 pageIdx 인 곳에 헬퍼 적용 시 회귀 | OK — `getPageLeftResolved` 가 신뢰값 pageIdx 받아도 정상 (sentinel 검사 후 pageLefts[i] 반환) |

---

## 5. 다음 단계

Stage 3 — `grid-mode-click-coord.test.mjs` 의 non-last col SKIP 분기를 strict assert 로 활성화 + 추가 col probe + 시각 검증.

승인 요청 → 승인 시 Stage 3 진행.
