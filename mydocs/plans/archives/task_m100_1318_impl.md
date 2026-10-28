# Task #1318 구현 계획서

## 전제

- 수행계획서 승인 완료: `mydocs/plans/task_m100_1318.md`
- Stage 1 진단 완료: `mydocs/working/task_m100_1318_stage1.md`
- 작업지시자 추가 기준:
  - 한컴에디터는 문단 모양 설정창에서 들여쓰기/내어쓰기 값을 `pt`로 바인딩해 보여준다.
  - rhwp도 동일하게 `pt` 표시/입력 계약을 유지한다.

## 핵심 구현 원칙

1. `Shift+Tab`은 일반 탭 문자 삽입이 아니라 한컴식 커서 기준 내어쓰기 설정으로 처리한다.
2. 좌표 계산은 렌더 트리 기반 `CursorRect.x`의 page-local px 좌표를 사용한다.
3. 표시/해석 기준은 문단 모양 대화상자와 동일하게 `pt`다.
4. 적용 시에는 기존 `ParaShapeDialog`와 같은 raw 2x HWPUNIT 계약을 사용한다.

변환 기준:

```ts
pxToPt(px) = px * 72 / 96
ptToRaw2x(pt) = round(pt * 100 * 2)
pxToRaw2x(px) = round(px * 150)
```

## 구현 산식

본문/일반 셀 1차 구현의 기본 산식:

```text
line_start_x = CursorRect(lineInfo.charStart).x
cursor_x = CursorRect(currentOffset).x
hanging_px = max(0, cursor_x - line_start_x)
indent_raw_2x = -round(hanging_px * 150)
```

의미:

- 첫 줄 시작 위치는 그대로 둔다.
- 두 번째 줄 이후 시작 위치가 현재 커서 x에 맞춰지도록 `indent < 0`을 설정한다.
- 문단 모양 대화상자는 기존 경로로 `abs(indent)`를 `pt`로 표시한다.

## 지원 범위

### 1차 지원

- 일반 본문 문단
- 일반 표 셀 문단 (`parentParaIndex`, `controlIndex`, `cellIndex`, `cellParaIndex` 기반)

### 1차 제외

- 중첩 표/글상자 `cellPath` 문맥
  - `getCursorRectByPath`는 있으나 `getLineInfoByPath`가 없다.
- 머리말/꼬리말
  - cursor rect와 apply API는 있으나 line info API 확인이 필요하다.
- 각주/미주
  - #1308/#1310 커서 이동 회귀 위험이 있어 별도 확인 후 확장한다.

제외 문맥에서는 `Shift+Tab` 입력을 무시하거나 기존 탭 삽입 동작으로 fallback하지 않는다. 한컴식 단축키의 의미가 다른 동작으로 오인되지 않게 no-op 처리하고, 개발 로그만 남기는 방향을 우선한다.

## Stage 2 결정 사항

1. 첫 줄이 아닌 곳에서 `Shift+Tab`
   - 1차 구현에서는 현재 visual line의 시작 x를 기준으로 내어쓰기 값을 설정한다.
   - 이유: 사용자가 현재 보이는 줄의 특정 위치를 기준점으로 삼는 UX가 가장 직관적이다.
   - 한컴이 첫 줄 외 위치에서 정확히 어떤 기준을 쓰는지는 후속 시각 판정으로 보강한다.

2. 한 줄 문단
   - 내어쓰기 값을 설정한다.
   - 이유: 설정 직후 보이는 변화가 작더라도, 이후 줄바꿈이 생기면 같은 문단 속성이 적용되어야 한다.

3. `cursor_x <= line_start_x`
   - `hanging_px = 0`으로 처리하고 `indent = 0`을 적용한다.
   - 음수 내어쓰기의 역방향 값은 만들지 않는다.

4. undo/redo
   - 이번 1차 구현은 기존 문단 모양 대화상자와 동일한 `applyParaPropsToRange()`/`afterEdit()` 경로를 사용한다.
   - 별도 `ApplyParaFormatCommand` 도입은 후속 품질 개선으로 분리한다.
   - 작업지시자 동작 테스트 후, 문단 속성 변경 전체의 Undo/Redo 체계화는 #1319로 별도 등록했다.

## 구현 단계

### Stage 3. 변환 helper 및 키 분기

- `rhwp-studio/src/engine/input-handler-keyboard.ts`
  - `case 'Tab'`에서 `e.shiftKey` 분기 추가
  - `Shift+Tab`이면 `this.applyHangingIndentAtCursor?.()` 호출 후 종료
  - 일반 `Tab`은 기존 `InsertTabCommand` 유지
- `rhwp-studio/src/engine/input-handler.ts`
  - `applyHangingIndentAtCursor()` 추가
  - px→raw 2x 변환 helper는 문단 모양 대화상자와 동일 산식으로 구현

### Stage 4. 본문/일반 셀 좌표 산식 구현

- 본문:
  - `pos = cursor.getPosition()`
  - `lineInfo = wasm.getLineInfo(sec, para, charOffset)`
  - `lineStartRect = wasm.getCursorRect(sec, para, lineInfo.charStart)`
  - `cursorRect = cursor.getRect() ?? wasm.getCursorRect(...)`
- 일반 셀:
  - `lineInfo = wasm.getLineInfoInCell(...)`
  - `lineStartRect = wasm.getCursorRectInCell(..., lineInfo.charStart)`
  - `cursorRect = wasm.getCursorRectInCell(..., charOffset)`
- `indent: -pxToRaw2x(hanging_px)`를 적용한다.
- 적용은 현재 문단 하나를 대상으로 한다. 선택 영역이 있더라도 한컴식 커서 기준 동작이므로 커서 문단만 처리한다.

### Stage 5. 갱신/검증 보강

- 적용 후 `afterEdit()` 경로에서 reflow, toolbar, ruler 반영을 확인한다.
- 문단 모양 대화상자에서 내어쓰기 값이 pt로 표시되는지 확인한다.
- unsupported 문맥에서 콘솔 오류 없이 no-op 되는지 확인한다.

### Stage 6. 테스트 및 보고서

- 가능한 범위의 프론트엔드 타입/빌드 검증
- Rust 검증:
  - `cargo fmt --all -- --check`
  - `cargo test --lib`
- 필요 시 WASM 빌드:
  - `docker compose --env-file .env.docker run --rm wasm`
- rhwp-studio 수동 검증:
  - 여러 줄 문단 첫 줄 중간에서 `Shift+Tab`
  - 두 번째 줄 이후 시작 x 변경 확인
  - 문단 모양 대화상자 내어쓰기 pt 표시 확인
  - 표 셀 내부 문단 확인

## 회귀 위험

- `Tab` 입력과 `Shift+Tab` 분기가 꼬이면 일반 탭 삽입 UX가 깨질 수 있다.
- `cellPath` 문맥에서 잘못 일반 셀 API를 호출하면 최근 중첩 표/글상자 커서 회귀가 재발할 수 있다.
- `indent` raw 스케일이 잘못되면 문단 모양 대화상자 값이 2배/절반으로 표시될 수 있다.
- 선택 영역에 적용할 경우 사용자가 의도하지 않은 다중 문단 내어쓰기가 발생할 수 있으므로 1차는 커서 문단만 적용한다.
- 문단 속성 변경의 Undo/Redo는 공통 커맨드 체계가 필요하므로 #1319에서 별도 처리한다.

## 완료 기준

1. 일반 본문 여러 줄 문단에서 `Shift+Tab`으로 커서 위치 기준 내어쓰기가 적용된다.
2. 두 번째 줄 이후 시작 x가 커서 위치에 맞춰 조판된다.
3. 문단 모양 대화상자에서 내어쓰기 값이 `pt`로 자연스럽게 표시된다.
4. 일반 `Tab` 삽입은 기존처럼 동작한다.
5. 일반 표 셀 문단에서도 동일 동작을 확인한다.
6. unsupported 문맥에서 콘솔 오류가 발생하지 않는다.

## 승인 요청

본 구현 계획서를 승인하면 Stage 3부터 소스 수정을 시작한다.
