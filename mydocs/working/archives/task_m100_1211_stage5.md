# Task M100-1211 Stage 5 완료 보고 — IME 조합 입력 경로 보완

## 배경

PR #1212 생성 후 로컬 확인에서 한글 텍스트 입력이 여전히 느리게 느껴진다는 피드백이 있었다.

기존 Stage 2~3 구현은 `executeOperation({ kind: 'command' })`를 타는 일반 `insertText` / `deleteText` 명령만 narrow invalidation으로 보냈다. 하지만 한글 IME 조합 중 입력은 `input-handler-text.ts`에서 `insertTextAtRaw()` / `deleteTextAt()`로 WASM을 직접 갱신한 뒤 `afterEdit()`를 호출한다.

따라서 IME 조합 중에는 여전히 다음 full refresh 경로가 반복되었다.

```text
raw insert/delete
  -> afterEdit()
  -> document-changed
  -> CanvasView.refreshPages()
  -> resetImageRetryState()
  -> visible page 재렌더
```

## 변경 내용

- `InputHandler.afterTextInputEdit(beforePos, afterPos)`를 추가했다.
  - command를 거치지 않는 raw 텍스트 입력도 `isPageLocalTextEditCommand()` 판정을 재사용한다.
  - 셀 내부 같은 `cellPath`에 남아 있으면 `afterPageLocalEdit()`를 호출한다.
  - header/footer, footnote, 본문 문단 등은 기존 `afterEdit()` full refresh를 유지한다.

- `input-handler-text.ts`
  - IME 조합 중 raw insert/delete 후 `afterEdit()` 대신 `afterTextInputEdit(anchor, currentPosition)`를 호출한다.
  - iOS composition fallback의 debounce 후 렌더도 같은 라우터를 사용한다.

## 검증

```text
cd rhwp-studio && npm test
cd rhwp-studio && npm run build
```

결과: 모두 통과.

브라우저 수동 확인:

- `http://127.0.0.1:7700/?url=/samples/exam_social.hwp&filename=exam_social.hwp`
- 한글 텍스트 입력 후 browser console error 없음.

## 결론

Stage 5 보완으로 일반 command 입력뿐 아니라 한글 IME 조합 중 raw 텍스트 편집도 같은 narrow invalidation 정책을 사용한다. 사용자가 체감한 한글 입력 지연의 직접 원인이던 `afterEdit()` full refresh 반복을 셀 내부 입력 경로에서 제거했다.
