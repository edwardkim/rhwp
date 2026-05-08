# Task M100 #661 구현계획서

## 타이틀

rhwp-studio 텍스트 드래그 선택 중 커서와 스크롤 위치가 튀는 현상 정정

## 기준

- 수행계획서: `mydocs/plans/task_m100_661.md`
- 작업 브랜치: `local/task661`
- 기준 커밋: `upstream/devel` `2fe386c`
- 승인 상태: 수행계획서 승인 완료
- 구현계획서 작성일: 2026-05-08

## 구현 원칙

1. 본 작업은 #658의 selection rect overflow 본질을 다시 수정하지 않는다.
2. #664가 아직 `devel`에 병합되지 않았으므로, #664와 같은 파일을 수정하는 영역은 충돌 가능성을 명시적으로 관리한다.
3. 먼저 프론트엔드 드래그 입력/스크롤 루프를 안정화한다.
4. Rust `getCursorRect*` page hint API 확장은 프론트엔드 안정화만으로 부족한 경우에만 진행한다.
5. 각 단계 완료 후 단계별 완료보고서를 작성하고 승인 요청한다.

## 핵심 설계

### 드래그 중 스크롤 정책

일반 커서 이동:

- caret rect가 화면 밖이면 `scrollCaretIntoView(rect)`로 스크롤한다.

드래그 선택:

- caret rect 기준 자동 스크롤을 기본 금지한다.
- 포인터가 `#scroll-container`의 상/하단 margin 안으로 들어온 경우에만 명시적으로 auto-scroll한다.
- 포인터 기반 auto-scroll은 한 프레임당 작은 delta만 적용하고, 적용 후 최신 포인터 snapshot으로 hitTest를 다시 수행한다.

### rAF 입력 정책

현재 문제 경로는 rAF 콜백이 원본 `MouseEvent`를 뒤늦게 해석하는 구조다.

정정 방향:

- `mousemove` 발생 시점에 `clientX`, `clientY`를 plain object로 snapshot한다.
- rAF 콜백은 DOM 이벤트 객체가 아니라 최신 snapshot만 사용한다.
- 같은 rAF 안에서는 스크롤 적용 순서를 명확히 한다.
  - 포인터 edge auto-scroll 판단
  - 필요 시 scrollTop 조정
  - 조정 후 snapshot 좌표 기준 hitTest
  - cursor/caret/selection 갱신

### caret rect 정책

드래그 중에는 `hitTest`가 제공한 `cursorRect`를 우선 신뢰한다.

- `CursorState.moveTo(hit)`는 현재처럼 WASM `getCursorRect*`를 호출할 수 있다.
- 다만 `updateCaretDuringDrag()`에서 `cursor.getRect()`가 page mismatch 폴백을 거친 결과라면 스크롤 판단에는 사용하지 않는다.
- 드래그 중 caret 표시에는 `hit.cursorRect` 또는 `cursor.getRect()`를 사용하되, 스크롤은 포인터 기준으로만 수행한다.

## 단계 계획

## Stage 1. 기준 정리 및 계측

### 목표

최신 `devel`, PR #664 변경분, #661 재현 경로를 한 번에 확인할 수 있는 기준 로그를 확보한다.

### 작업 내용

- `local/task661` 기준으로 #664 병합 여부를 재확인한다.
- #664가 여전히 미병합이면 다음 둘 중 하나를 선택한다.
  - 우선 구현은 `devel` 기준 최소 diff로 진행하되 #664와 충돌하는 지점을 문서화한다.
  - 필요하면 작업지시자 승인 후 `local/task661`을 #664 위에 스택한다.
- `samples/exam_social.hwp` page 2 구조를 다시 확인한다.
- 임시 진단 스크립트 또는 console 계측으로 아래 값을 수집한다.
  - mouse snapshot `clientX/clientY`
  - `container.scrollTop` before/after
  - `hitTestFromEvent` 또는 snapshot 기반 hitTest 결과
  - `hit.cursorRect.pageIndex`
  - `cursor.getRect().pageIndex`
  - `CursorState` pageIndex mismatch 경고 발생 여부

### 산출물

- `mydocs/working/task_m100_661_stage1.md`
- 필요 시 커밋하지 않는 임시 진단 스크립트 경로 기록

### 완료 기준

- 재현 좌표 또는 대표 드래그 영역이 문서화된다.
- 스크롤 점프가 `scrollCaretIntoView` 또는 rAF 재해석과 연결되는지 판단할 수 있는 로그가 확보된다.

## Stage 2. 드래그 전용 caret 갱신 경로 분리

### 목표

드래그 선택 중 일반 `updateCaret()` 전체 경로를 호출하지 않도록 분리한다.

### 작업 내용

- `rhwp-studio/src/engine/caret-renderer.ts`
  - #664의 `updateLive()`와 동등한 드래그용 caret 표시 경로가 없으면 추가한다.
  - 이미 #664가 적용된 경우 기존 메서드를 재사용한다.

- `rhwp-studio/src/engine/input-handler.ts`
  - `updateCaretDuringDrag()`를 추가 또는 정정한다.
  - 이 경로는 아래만 수행한다.
    - IME 조합 중이면 기존 `updateCaret()` fallback
    - caret 표시 갱신
    - selection 갱신
    - `cursor-rect-updated` emit
  - 이 단계에서는 `scrollCaretIntoView()`를 호출하지 않는다.

- `rhwp-studio/src/engine/input-handler-mouse.ts`
  - 드래그 중 `this.updateCaret()` 호출을 `this.updateCaretDuringDrag()`로 교체한다.

### 산출물

- `mydocs/working/task_m100_661_stage2.md`

### 완료 기준

- 드래그 중 caret 기준 자동 스크롤이 호출되지 않는다.
- 단일 클릭, 키보드 커서 이동, 문서 로드 후 caret 배치의 기존 스크롤 동작은 유지된다.

## Stage 3. 포인터 기반 auto-scroll 도입

### 목표

드래그 중 필요한 auto-scroll을 caret 기준이 아니라 포인터 위치 기준으로 제한한다.

### 작업 내용

- `rhwp-studio/src/engine/input-handler.ts` 또는 mouse handler에 드래그 전용 helper를 추가한다.
  - 예: `autoScrollDuringDrag(clientY: number): boolean`
- `#scroll-container` viewport 기준 상/하단 margin을 정의한다.
  - 후보: 24px 또는 기존 caret margin 20px와 동일 계열
- 포인터가 margin 안에 있을 때만 `container.scrollTop`을 조정한다.
- 조정량은 급격한 점프가 아니라 edge distance 기반 작은 delta로 제한한다.
- 포인터가 margin 밖이면 `scrollTop`을 건드리지 않는다.

### 산출물

- `mydocs/working/task_m100_661_stage3.md`

### 완료 기준

- 포인터가 auto-scroll 영역 밖에 있을 때 드래그 중 `scrollTop` 변화가 없다.
- 포인터가 상/하단 edge 영역에 있을 때만 연속 선택을 위한 auto-scroll이 동작한다.

## Stage 4. rAF 입력 snapshot 정합화

### 목표

rAF 콜백에서 원본 `MouseEvent`를 재해석하지 않도록 한다.

### 작업 내용

- `rhwp-studio/src/engine/input-handler.ts` 또는 mouse module 상태에 `lastDragPointer`를 둔다.
  - 예: `{ clientX: number; clientY: number }`
- `mousemove` 즉시 snapshot을 갱신한다.
- rAF 콜백에서는 snapshot만 사용한다.
- snapshot 기반 hitTest helper를 추가하거나, 기존 `hitTestFromEvent()`를 `hitTestFromClientPoint(clientX, clientY)`로 분리한다.
- Stage 3 auto-scroll이 scrollTop을 바꾼 경우, 같은 snapshot의 화면 좌표를 현재 DOM 기준으로 다시 page 좌표화한다.

### 산출물

- `mydocs/working/task_m100_661_stage4.md`

### 완료 기준

- rAF 지연 중 `MouseEvent` 객체를 참조하지 않는다.
- 드래그 중 scrollTop 변경 전후 hitTest 순서가 코드상 명확하다.

## Stage 5. pageIndex mismatch 잔여 보강

### 목표

Stage 2~4 이후에도 pageIndex mismatch가 남는지 확인하고, 필요 시 최소 보강한다.

### 작업 내용

- `CursorState.updateRect()` 경고 발생 빈도를 재확인한다.
- 반복 경고가 남지만 스크롤 점프가 사라졌다면 경고는 후속 Rust API 개선 후보로 문서화한다.
- 반복 경고가 caret 표시 튐을 계속 유발하면 다음 중 하나를 적용한다.
  - 드래그 중 `hit.cursorRect`를 caret 표시 rect로 우선 사용하는 lightweight 경로
  - `getCursorRect*`에 page hint 보조 API 추가
- Rust API 확장이 필요하면 영향 범위를 별도 문서화하고 단위 테스트를 추가한다.

### 산출물

- `mydocs/working/task_m100_661_stage5.md`

### 완료 기준

- `exam_social.hwp` 재현 영역에서 caret이 다른 페이지/줄로 튀지 않는다.
- Rust API 변경 여부와 잔여 위험이 명확히 정리된다.

## Stage 6. 검증 및 최종 보고

### 목표

자동 검증과 시각 검증을 완료하고 최종 결과를 문서화한다.

### 작업 내용

- TypeScript 빌드:
  - `cd rhwp-studio && npm run build`
- Rust 변경이 있는 경우:
  - `cargo test --lib --release`
  - 추가 테스트 실행
- 가능하면 e2e 또는 진단 스크립트 실행:
  - `samples/exam_social.hwp`
  - zoom 120%
  - page 2 하단 좌측 박스 드래그
- 최종 보고서 작성:
  - `mydocs/report/task_m100_661_report.md`
- 오늘 할일 상태 갱신:
  - `mydocs/orders/20260508.md`

### 산출물

- `mydocs/working/task_m100_661_stage6.md`
- `mydocs/report/task_m100_661_report.md`

### 완료 기준

- 빌드/테스트 결과가 기록된다.
- 작업지시자 시각 검증 요청에 필요한 관찰 포인트가 정리된다.
- 미커밋 파일 확인 후 task branch commit 단위가 정리된다.

## 예상 변경 파일

| 파일 | 예상 변경 |
|------|-----------|
| `rhwp-studio/src/engine/input-handler-mouse.ts` | 드래그 rAF snapshot, `updateCaretDuringDrag()` 호출 |
| `rhwp-studio/src/engine/input-handler.ts` | 드래그 전용 caret 갱신, 포인터 기반 auto-scroll helper |
| `rhwp-studio/src/engine/caret-renderer.ts` | `updateLive()` 없으면 추가 |
| `rhwp-studio/src/engine/cursor.ts` | 필요 시 드래그 중 hitTest rect 우선 경로 |
| `src/document_core/queries/cursor_rect.rs` | 필요 시 page hint API |
| `src/wasm_api.rs` | Rust page hint API가 필요한 경우만 |
| `rhwp-studio/e2e/*.mjs` | 가능하면 회귀/진단 스크립트 |
| `mydocs/working/task_m100_661_stage*.md` | 단계별 완료보고서 |
| `mydocs/report/task_m100_661_report.md` | 최종 결과보고서 |
| `mydocs/orders/20260508.md` | 상태 갱신 |

## 검증 명령

기본 검증:

```bash
cd rhwp-studio && npm run build
```

Rust 변경 시:

```bash
cargo test --lib --release
```

E2E 또는 진단 스크립트 추가 시:

```bash
cd rhwp-studio
npx vite --host 0.0.0.0 --port 7700
node e2e/{추가_스크립트}.mjs --mode=headless
```

## 승인 요청

위 구현계획서 기준으로 Stage 1을 시작해도 되는지 승인 요청한다.

승인 후 Stage 1 재현/계측을 수행하고, `mydocs/working/task_m100_661_stage1.md` 완료보고서를 작성한 뒤 다시 승인 요청한다.
