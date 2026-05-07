# rhwp-studio 본문 외곽 클릭 fallback 한컴 mismatch 결함

## 본질

`samples/hwpctl_Action_Table__v1.1.hwp` (16p, landscape, `margin_bottom=0`, `footer_area.height=0`) 의 페이지 16 꼬리말 영역 클릭 시 한컴과 RHWP 동작 mismatch.

| 환경 | 동작 |
|------|------|
| 한컴 | 문서 마지막 `}` 문자에 캐럿 배치 (= 본문 외곽 → 가장 가까운 본문 캐럿 fallback) |
| RHWP | **페이지 2, 3 으로 뷰 점프** + 캐럿 / 선택 상태 부정합 |

## 사전 정합 사실

- `PageAreas::from_page_def` 의 `footer_area.height = margin_bottom` 수식은 HWP spec 정합 (한컴도 동일).
- `margin_bottom=0` → footer_area.height=0 → `hit_test_header_footer = false` 는 한컴과 정합 (양쪽 모두 그 영역을 꼬리말로 인식 안 함).
- 결함은 **`hit_test_header_footer = false` 이후 onMouseDown fallback 분기** 에 한정.

## 코드 경로 trace

[input-handler-mouse.ts:onMouseDown](../../rhwp-studio/src/engine/input-handler-mouse.ts#L460-L781) 의 흐름 (page 16 꼬리말 영역 click 시 점진적 분기):

1. (line 470-479) `pageX/pageY` 계산 — 정상.
2. (line 482-491) `tableResizeRenderer` 표 경계 hit — `cachedCellBboxes` 없음 → skip.
3. (line 494-516) `cursor.isInHeaderFooter()` — 첫 클릭이라 false → skip.
4. (line 519-540) `cursor.isInFootnote()` — false → skip.
5. (line 543-595) 각주 마커 / 영역 hit — 없음 → skip.
6. (line 597) **`hit = wasm.hitTest(pageIdx=15, pageX, pageY)`** — 핵심 분기점.
7. (line 601) `paragraphIndex >= 0xFFFFFF00` (머리말/꼬리말 sentinel) → `textarea.focus()` + return. **scroll 변화 없음.** footer_area.height=0 이므로 sentinel 반환 가능성 낮음.
8. (line 607-622) 표 내부 + 표 경계 click → 표 객체 선택 + return.
9. (line 625-640) `findTableByOuterClick` (표 외곽 click 휴리스틱) → 표 객체 선택 + return.
10. (line 643-657) **`hit.isTextBox === true` → `cursor.moveTo(hit)` + `this.updateCaret()` + 드래그 시작.** `updateCaret` 은 [input-handler.ts:1505](../../rhwp-studio/src/engine/input-handler.ts#L1505) 에서 `scrollCaretIntoView(rect)` 호출 → **여기서 페이지 점프 발생 가능.**
11. (line 660-732) `findPictureAtClick` (그림/글상자/수식) → 객체 선택 + return.
12. (line 736-742) form object → handler + return.
13. (line 754-781) **일반 click fallback** — `cursor.moveTo(hit)` + `caret.show(rect, zoom)` + isDragging=true. **`scrollCaretIntoView` 호출 안 함**, `caret.show` 만 — scroll 변화는 없어야 정상.

## 후보 (a/b/c) 좁히기 — **e2e 측정으로 (b) 확정 (2026-05-07)**

| 후보 | 가설 | e2e 측정 결과 |
|------|------|---------------|
| (a) cursor.moveTo({sec:0, para:0, ...}) invalid fallback | hit 결과 invalid → 기본값 fallback | **부정** — wasm.hitTest 가 valid hit 반환 (cursorRect.pageIndex=15 정상) |
| **(b) 바탕쪽 (master page) 글상자 hit → isTextBox=true 분기** | line 643-657 분기 → `updateCaret` → `scrollCaretIntoView` 발생 | **확정 — dblclick 시 정확히 재현** (delta=−11288, page 0~1 jump) |
| (c) 본문 paragraph hit 휴리스틱이 잘못된 paragraphIndex 반환 | hit.paragraphIndex 가 다른 페이지로 매핑 | **부정** — hit.paragraphIndex 정상, cursorRect.pageIndex=15 (click 페이지) 정상 |

### 결정적 측정 데이터 (2026-05-07)

`rhwp-studio/e2e/body-outside-click-fallback.test.mjs` 변종 + 정밀 측정 (페이지 번호 textbox 정확한 위치 click) 결과:

**Step 1 — Layout dump (page 15 의 control 위치)**:
```json
{ "controls": [
    {"type": "shape", "x": 113.4, "y": 740, "w": 75.6, "h": 21.5, "secIdx": 0, "paraIdx": 0, "controlIdx": 0},  // ← 자동번호 글상자
    {"type": "image", "x": 1752.9, "y": 1480.5, ...},  // off-page
    {"type": "shape", "x": 113.4, "y": 736.8, "w": 895.7, "h": 4, ...}  // 단 구분선
]}
```

자동번호 글상자: `secIdx=0, paraIdx=0, controlIdx=0` — section 0 의 paragraph 0 (= master page) 에 anchored.

**Step 2 — 글상자 정확한 위치 (151.2, 750.75 = bbox 중앙) 단일 click**:
```
scroll: 12023 → 12023 (delta=0)
pos = {sec:0, para:0, char:0}
isInPictureObjectSelection=true
selectedPicRef = {sec:0, ppi:0, ci:0, type:'shape'}
```
→ shape 객체 선택. **scroll 변화 없음.** 선택 상태로만 진입.

**Step 3 — 같은 위치 더블 click (사용자 정확한 시나리오)**:
```
scroll: 12023 → 735 (delta=−11288)
pos = {sec:0, para:0, char:0, ppi:0, ci:0, cellIdx:0}
isInTextBox=true  ★
rectPg=0  ★
visible pages after dblclick: [0, 1]  ★
```

→ **isInTextBox=true 분기 진입**, cursor 가 sec 0 paragraph 0 의 control 0 cell 0 (= master page 글상자 안) 으로 이동, **`cursor.getRect().pageIndex = 0`** (master page 가 first-attached 된 page 0 의 좌표 반환), `scrollCaretIntoView` 가 page 0 으로 스크롤 → **사용자 보고와 정확히 일치 (페이지 1, 2 visible)**.

**확정된 결함 본질**:
1. dblclick 분기는 [input-handler-mouse.ts:onDblClick](../../rhwp-studio/src/engine/input-handler-mouse.ts#L784) 직접 처리가 아닌, mousedown 의 textbox 분기 ([input-handler-mouse.ts:643-657](../../rhwp-studio/src/engine/input-handler-mouse.ts#L643-L657)) 안의 **`isInTextBox` 가 이미 true** 인 상태에서 두 번째 mousedown.
2. master page 의 자동번호 글상자는 `secIdx=0, paraIdx=0` 으로 attached → `cursor.moveTo(hit)` 후 `getRect()` 가 paraIdx=0 이 first 정의된 위치 (= page 0) 의 rect 를 반환.
3. `scrollCaretIntoView(rect)` 가 rect.pageIndex=0 의 좌표를 viewport 안으로 스크롤 → 마지막 페이지 → 첫 페이지로 점프.

**한컴 정합 정정 방향**: master page 글상자 (특히 paraIdx=0 으로 anchored 된 control) 는:
- A) `findPictureAtClick` 에서 마스터 페이지 출처 글상자는 hit 결과에서 제외 → 일반 본문 fallback 진행
- B) hit 의 cursorRect.pageIndex 가 click 페이지 (`pageIdx`) 와 다르면 fallback
- 한컴은 master page 위 dblclick 시 본문 가장 가까운 caret 으로 떨어짐 — 본 환경도 동일하게.

## 한컴 정합 fallback 동작

한컴은 **master page 위 click 을 textbox edit 으로 처리하지 않고**, 본문 영역 외곽 click 으로 인식 → 가장 가까운 본문 paragraph 캐럿 (이 fixture 의 경우 문서 마지막 `}` 문자) 으로 떨어짐.

본 환경 정합 정정 방향 (가설):
- A) `findPictureAtClick` / `isTextBox` 분기에서 **master page (바탕쪽) 출처의 글상자는 제외**하고 본문 fallback 으로 진행.
- B) hit 결과의 `cursor.getRect().pageIndex` 가 click 페이지 (`pageIdx`) 와 다르면 본문 fallback 으로 우회.
- 한컴 정합은 A 가 더 자연스러움 (의미적으로 master page 위 click은 본문 click).

## e2e 계측 (정정 task 에서 수행)

1. `cd rhwp-studio && npx vite --host 0.0.0.0 --port 7700 &`
2. `samples/hwpctl_Action_Table__v1.1.hwp` 로드.
3. 페이지 16 페이지 하단 (꼬리말 위치) 클릭.
4. `console.log` 또는 e2e 계측: `hit` 객체 (`sectionIndex`, `paragraphIndex`, `parentParaIndex`, `controlIndex`, `cellIndex`, `isTextBox`), `cursor.getPosition()`, `cursor.getRect().pageIndex`, click 직후 `scrollContainer.scrollTop` 변화.
5. 후보 (b) 확정: `hit.isTextBox === true` + `parentParaIndex` 가 master page 글상자 ppi.

## 우선순위

- **Medium**: 단일 fixture (`hwpctl_Action_Table__v1.1.hwp`) 에서만 재현. 다른 일반 문서는 footer_area.height > 0 이라 hit_test_header_footer = true 분기로 진입.
- 그러나 본질 (master page 글상자 위 본문 외곽 click) 은 **다른 양식** (페이지 양식 / 표지 양식 등) 에서도 잠재적 재현 가능. 등록 가치 있음.

## fixture

- `samples/hwpctl_Action_Table__v1.1.hwp`
  - 16 페이지, landscape (가로)
  - `margin_left=30, margin_right=30, margin_top=0, margin_bottom=0, margin_header=15, margin_footer=15` (mm)
  - 바탕쪽 2개 (Both, 자동번호 Page 글상자 포함)
  - section 0 dump: `cargo run --release --bin rhwp -- dump samples/hwpctl_Action_Table__v1.1.hwp`

## 우선 처리 후순위 (정정 task 진입 전)

1. e2e 계측으로 후보 (b) 확정 (5-10 분).
2. 정정 방향 (A vs B) 결정.
3. 한컴 정합 시각 검증 매뉴얼 준비.

## 등록 전 보류 (사용자 결정, 2026-05-07) → **e2e 확정으로 등록 가능**

작업지시자: "**모든 이슈 파악을 완벽하게 하고 이슈를 올리는게 좋을것 같음**". 본 노트는 진단 보강 단계로 간주하고 issue 등록 보류 → **2026-05-07 e2e 측정으로 가설 (b) 확정 + 정량 데이터 확보 → 등록 가능 상태**.

남은 횡적 검증 (선택):
- master page 구조 변환 (예: section 다중 + 표지 양식) 에서 동일 결함 재현되는지 — 본 fixture 외 추가 fixture 측정.
- 한컴 정합 정정 방향 (A vs B) 결정 — 회귀 영역 (정상 글상자 click 시나리오) 명시 후 정정.

## 사용자 측정 vs e2e 측정 정합

| 측정 항목 | 사용자 직접 (host browser) | e2e (headless) |
|-----------|----------------------------|-----------------|
| 페이지 번호 위치 | "16" 텍스트 | shape #1 bbox (113.4, 740, 75.6×21.5) |
| 동작 | 더블클릭 | mouse.click({clickCount:2, delay:80}) |
| 결과 (사용자) | "페이지 2, 3 으로 이동" | scroll delta=−11288, visible pages [0, 1] (= 1, 2 페이지) |
| 한컴 동작 | "문서 마지막 `}` 캐럿" | (e2e 미측정 — 한컴 자동화 불가) |

→ **본질 정합** (사용자 보고 "2, 3 페이지" 와 e2e "1, 2 페이지" 차이는 viewport 크기 / 첫 화면에 보이는 페이지 수 차이일 뿐, **첫 페이지 부근으로 점프** 라는 본질은 동일).
