# Task #1318 완료 보고서

## 요약

한컴에디터의 `Shift+Tab` 동작처럼, 커서가 위치한 x 좌표를 기준으로 현재 문단의 내어쓰기를 설정하는
기능을 rhwp-studio에 구현했다.

이번 구현은 일반 `Tab` 입력과 `Shift+Tab` 입력을 분리한다. 일반 `Tab`은 기존처럼 탭 문자를 삽입하고,
`Shift+Tab`은 현재 커서 위치를 기준으로 문단 내어쓰기 값을 계산해 문단 서식에 반영한다.

## 변경 내용

- `rhwp-studio/src/engine/input-handler-keyboard.ts`
  - `case 'Tab'`에서 `e.shiftKey` 분기 추가
  - `Shift+Tab` 입력 시 `applyHangingIndentAtCursor()` 호출
  - 일반 `Tab` 입력은 기존 `InsertTabCommand` 유지

- `rhwp-studio/src/engine/input-handler.ts`
  - `applyHangingIndentAtCursor()` 추가
  - 현재 visual line 시작 x와 현재 커서 x를 같은 렌더 좌표계에서 비교
  - `indent_raw_2x = -round(hanging_px * 150)` 산식으로 내어쓰기 적용
  - 본문 문단과 일반 표 셀 문단을 1차 지원
  - 머리말/꼬리말, 각주/미주, 글상자, 중첩 표는 no-op 처리

## 문단 속성 단위

한컴에디터와 동일하게 문단 모양 대화상자에서는 `pt` 기준으로 보이도록 기존 변환 계약을 유지했다.

```text
pxToPt(px) = px * 72 / 96
ptToRaw2x(pt) = round(pt * 100 * 2)
pxToRaw2x(px) = round(px * 150)
```

## 지원 범위

성공 판정 범위:

- 일반 본문 여러 줄 문단
- 일반 표 셀 문단
- 문단 모양 대화상자 `pt` 표시
- 일반 `Tab` 입력 유지

후속 확장 범위:

- 머리말/꼬리말 문단
- 각주/미주 문단
- 글상자 문단
- 중첩 표 문단

## Undo/Redo 후속 분리

동작 테스트 후 Undo/Redo 지원 필요성이 확인되었다. 다만 이는 `Shift+Tab` 전용 기능이 아니라 문단 속성
변경 전체의 공통 구조 문제다.

현재 문단 모양 대화상자, toolbar/menu 기반 정렬·줄간격 변경, `Shift+Tab` 내어쓰기 등이 모두
문단 서식 변경을 수행하지만 공통 `EditCommand` 기반 Undo/Redo 체계로 정리되어 있지 않다.

따라서 `Shift+Tab`에만 임시 Undo를 붙이지 않고 별도 이슈로 분리했다.

- 후속 이슈: #1319 `문단 서식 변경 Undo/Redo 커맨드 체계화`

## 검증

| 항목 | 결과 |
|---|---|
| `npm run build` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --lib` | 통과 |
| rhwp-studio 동작 테스트 | 통과 |

`cargo test --lib` 결과:

```text
test result: ok. 1602 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out; finished in 129.63s
```

## 판정

작업지시자가 rhwp-studio에서 `Shift+Tab` 커서 기준 내어쓰기 동작 테스트 성공을 확인했다.

이번 이슈는 성공으로 완료 판정한다.
