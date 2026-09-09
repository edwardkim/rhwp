# Task #1318 Stage 1 — 진단 및 기준선 확정

## 범위

- 이슈: #1318 `한컴식 Shift+Tab 커서 기준 내어쓰기 설정 구현`
- 브랜치: `local/task1318`
- 수행계획서: `mydocs/plans/task_m100_1318.md`
- 단계 목표: 소스 수정 없이 키 처리 경로, 커서 좌표계, 문단 속성 스케일 계약을 확정한다.

## 현행 키 처리

`rhwp-studio/src/engine/input-handler-keyboard.ts`의 `case 'Tab'`는 `shiftKey`를 보지 않고 항상
`InsertTabCommand`를 실행한다.

```ts
case 'Tab': {
  e.preventDefault();
  // 탭 문자 삽입 (본문·표 셀·글상자 공통)
  this.executeOperation({ kind: 'command', command: new InsertTabCommand(this.cursor.getPosition()) });
  break;
}
```

따라서 현재 rhwp-studio에서 `Shift+Tab`은 한컴식 내어쓰기 설정이 아니라 일반 탭 삽입 경로로
처리될 가능성이 높다. 이번 구현은 이 위치에서 `e.shiftKey` 분기를 먼저 추가하는 것이 자연스럽다.

## 커서 좌표계

프론트엔드 타입:

- `CursorRect`: `{ pageIndex, x, y, height }`
- `LineInfo`: `{ lineIndex, lineCount, charStart, charEnd }`

WASM API:

- 본문: `getCursorRect(sec, para, charOffset)`, `getLineInfo(sec, para, charOffset)`
- 셀: `getCursorRectInCell(...)`, `getLineInfoInCell(...)`
- 중첩/글상자 path: `getCursorRectByPath(...)`
- 머리말/꼬리말: `getCursorRectInHeaderFooter(...)`
- 각주/미주: `getCursorRectInNote(...)` 또는 fallback 계열

`src/document_core/queries/cursor_rect.rs`를 보면 커서 x는 `node.bbox.x + x_in_run`으로 만들어진다.
즉 반환 x는 문단-local이 아니라 **페이지-local 절대 x 좌표**다. 셀 내부도 동일하게 렌더 트리
TextRun bbox 기준으로 페이지-local x를 반환한다.

## 문단 속성 계약

`ParaProperties`의 `marginLeft`, `indent`는 프론트엔드에서 px 단위로 조회된다.

```ts
marginLeft?: number; // px
indent?: number;     // px
```

반면 적용 시에는 `ParaShapeDialog.collectMods()`가 pt 입력값을 raw 2x HWPUNIT으로 변환한다.

```ts
ptToRaw2x(pt) = round(pt * 100 * 2)
pxToRaw2x(px) = px -> pt -> raw 2x
```

WASM 쪽 `getParaPropertiesAt`은 #1172 이후 margin/indent의 IR 2x 스케일을 조회 시 px로 되돌려
프론트엔드에 제공한다. 따라서 새 `Shift+Tab` 구현에서도 최종 적용 `mods.indent`는 raw 2x
HWPUNIT이어야 하며, UI 헬퍼와 같은 변환 규칙을 공유해야 한다.

## 한컴식 내어쓰기 해석

한컴에서 첫 줄 특정 문자 위치에 커서를 두고 `Shift+Tab`을 누르면:

1. 첫 줄 시작 위치는 기존 왼쪽 여백 위치를 유지한다.
2. 두 번째 줄 이후 시작 위치가 커서 x 위치로 이동한다.
3. 문단 속성은 "내어쓰기"로 표시된다.

rhwp의 조판 규칙은 이미 다음 형태다.

- `indent > 0`: 첫 줄만 `marginLeft + indent`
- `indent < 0`: 두 번째 줄 이후 `marginLeft + abs(indent)`
- `indent = 0`: 모든 줄 `marginLeft`

따라서 목표 산식은 다음과 같이 잡을 수 있다.

```text
base_x = 첫 줄 시작 커서 x
target_x = 현재 커서 x
hanging_px = max(0, target_x - base_x)
new_indent = -px_to_raw2x(hanging_px)
```

`base_x`는 단순히 `page.marginLeft + para.marginLeft`로 계산하지 않는 것이 안전하다. 다단, 표 셀,
글상자, 머리말/꼬리말 등에서는 page/body 기준점이 달라지기 때문이다. 1차 구현은 같은 문맥에서
`lineInfo.charStart`의 `CursorRect.x`를 다시 조회하여 `base_x`로 삼는 것이 가장 안정적이다.

## 구현 가능성 판단

### 본문

- `cursor.getPosition()`
- `cursor.getRect()` 또는 `wasm.getCursorRect(...)`
- `wasm.getLineInfo(...)`
- `inputHandler.applyParaPropsToRange(...)`

위 조합으로 구현 가능하다.

### 표 셀

셀 문단도 `getCursorRectInCell`, `getLineInfoInCell`, `applyParaFormatInCell` 경로가 있으므로 같은
산식으로 구현 가능하다. 다만 `cellPath`가 있는 경우 `getCursorRectByPath`는 있으나
`getLineInfoByPath`는 현재 확인되지 않았다. 중첩 표/글상자 path 문맥은 Stage 2에서 지원 범위를
분리해야 한다.

### 머리말/꼬리말

`getCursorRectInHeaderFooter`와 `applyParaFormatInHf`는 존재한다. 하지만 line info API는 별도로
확인되지 않았다. 1차 범위에서 제외하거나, 별도 line info API 추가가 필요하다.

### 각주/미주

`getCursorRectInNote`/`getCursorRectInFootnote`와 `applyParaFormatInFootnote`는 존재한다. 그러나
일반화된 note line info API 확인이 필요하다. #1308/#1310에서 미주 커서 이동을 다뤘으므로
회귀 위험이 있어 1차 구현은 본문/일반 셀 우선이 안전하다.

## undo/redo 쟁점

현재 문단 모양 대화상자는 `InputHandler.applyParaPropsToRange()`를 통해 직접 WASM `applyParaFormat*`
API를 호출하고 `afterEdit()`를 실행한다. 글자 서식처럼 명시적인 `ApplyParaFormatCommand`는 확인되지
않았다.

따라서 #1318의 1차 구현은 기존 문단 모양 대화상자와 같은 적용 경로를 재사용하는 것이 일관적이다.
다만 사용자가 요구한 undo/redo 품질까지 맞추려면 별도 `ApplyParaFormatCommand` 도입 여부를 Stage 2에서
결정해야 한다.

## 권장 구현 방향

1. `input-handler-keyboard.ts`의 `case 'Tab'`에서 `e.shiftKey` 분기 추가
2. `InputHandler`에 `applyHangingIndentAtCursor()` 계열 메서드 추가
3. 현재 문맥에 맞는 line info와 cursor rect를 조회
4. `lineInfo.charStart` 위치의 cursor rect를 기준점으로 사용
5. 현재 cursor rect x와 기준점 x의 차이를 raw 2x HWPUNIT `indent` 음수값으로 변환
6. 기존 `applyParaPropsToRange()` 또는 내부 `applyParaFormat()` 경로로 적용
7. `afterEdit()` 후 커서/toolbar/ruler 갱신 확인

## Stage 2 구현 계획서에 포함할 결정 사항

- `Shift+Tab`이 첫 줄이 아닌 곳에서 눌렸을 때:
  - 후보 A: 아무 동작 하지 않음
  - 후보 B: 현재 visual line 시작점을 기준으로 내어쓰기 설정
  - 후보 C: 일반 탭 삽입과 동일 처리
- `lineCount === 1`인 문단에서도 내어쓰기 설정을 허용할지 여부
- 중첩 표/글상자 path, 머리말/꼬리말, 각주/미주의 1차 지원 범위
- 문단 서식 undo/redo 커맨드 도입 여부

## 결론

새 렌더러 조판 알고리즘을 건드리지 않고도 1차 구현은 가능하다. 핵심은 기존 렌더 트리 기반
커서 좌표를 이용해 `base_x`와 `target_x`를 같은 좌표계에서 비교하는 것이다.

다음 단계에서는 위 권장 방향을 기반으로 구현 계획서(`mydocs/plans/task_m100_1318_impl.md`)를 작성한다.
