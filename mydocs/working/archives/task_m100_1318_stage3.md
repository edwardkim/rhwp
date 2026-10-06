# Task #1318 Stage 3 — Shift+Tab 내어쓰기 구현 및 1차 검증

## 범위

- 이슈: #1318 `한컴식 Shift+Tab 커서 기준 내어쓰기 설정 구현`
- 브랜치: `local/task1318`
- 수행계획서: `mydocs/plans/task_m100_1318.md`
- 구현 계획서: `mydocs/plans/task_m100_1318_impl.md`

## 구현 내용

### 키 처리

`rhwp-studio/src/engine/input-handler-keyboard.ts`의 `Tab` 처리에서 `Shift+Tab` 분기를 추가했다.

- `Shift+Tab`: `InputHandler.applyHangingIndentAtCursor()` 실행
- 일반 `Tab`: 기존 `InsertTabCommand` 유지

따라서 일반 탭 문자 삽입 동작은 유지하고, 한컴식 내어쓰기 단축키만 별도 문단 서식 적용 경로로 분리했다.

### 내어쓰기 산식

`rhwp-studio/src/engine/input-handler.ts`에 `applyHangingIndentAtCursor()`를 추가했다.

기본 산식:

```text
line_start_x = CursorRect(lineInfo.charStart).x
cursor_x = CursorRect(currentOffset).x
hanging_px = max(0, cursor_x - line_start_x)
indent_raw_2x = -round(hanging_px * 150)
```

`px * 150`은 문단 모양 대화상자의 pt 바인딩과 같은 변환 계약이다.

```text
pxToPt(px) = px * 72 / 96
ptToRaw2x(pt) = round(pt * 100 * 2)
pxToRaw2x(px) = round(px * 150)
```

이렇게 저장하면 문단 모양 대화상자에서는 한컴에디터와 동일하게 `pt` 기준 내어쓰기 값으로 표시된다.

## 지원 범위

이번 1차 구현에서 지원한 문맥:

- 일반 본문 문단
- 일반 표 셀 문단

이번 1차 구현에서 no-op 처리한 문맥:

- 머리말/꼬리말
- 각주/미주
- 글상자
- 중첩 표/다중 `cellPath`

제외 문맥에서는 기존 탭 삽입으로 fallback하지 않고 콘솔 정보 로그만 남긴다. 한컴식 `Shift+Tab`의 의미가
다른 입력 동작으로 바뀌지 않게 하기 위한 결정이다.

## 검증

로컬 검증:

```text
npm run build                         PASS
cargo fmt --all -- --check            PASS
cargo test --lib                      PASS
```

`cargo test --lib` 결과:

```text
test result: ok. 1602 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out; finished in 129.63s
```

## 작업지시자 동작 판정

작업지시자가 rhwp-studio에서 Shift+Tab 내어쓰기 동작 테스트 성공을 확인했다.

## Undo/Redo 후속 분리

동작 판정 후 Undo 지원 필요성이 확인되었다. 다만 현재 문단 속성 변경 경로는 문단 모양 대화상자,
toolbar/menu 기반 정렬·줄간격 변경, Shift+Tab 내어쓰기 등이 공통 `EditCommand`로 체계화되어 있지
않다.

Shift+Tab 전용으로만 임시 Undo를 붙이면 문단 속성 변경 경로가 더 흩어질 수 있으므로, 문단 서식 변경
전체의 Undo/Redo 커맨드 체계화는 별도 이슈로 분리했다.

- 후속 이슈: #1319 `문단 서식 변경 Undo/Redo 커맨드 체계화`

## 확인 완료 항목

작업지시자 수동 검증으로 다음 항목을 확인했다.

- 여러 줄 본문 문단 첫 줄 중간에서 `Shift+Tab`
- 두 번째 줄 이후 시작 x가 커서 위치로 이동하는지 확인
- 문단 모양 대화상자에서 내어쓰기 값이 `pt`로 표시되는지 확인
- 일반 표 셀 문단에서도 동일하게 적용되는지 확인
- 일반 `Tab` 입력이 기존처럼 탭 문자 삽입으로 유지되는지 확인

## 다음 단계

Undo/Redo는 #1319로 넘기고, #1318은 현재 Shift+Tab 동작 범위에 대한 완료 보고서를 작성한다.
