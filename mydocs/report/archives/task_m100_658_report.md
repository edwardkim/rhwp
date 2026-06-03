# Task M100 #658 최종 결과보고서

## 타이틀

rhwp-studio 마우스 드래그 텍스트 선택 하이라이트가 페이지 밖으로 튀고 버벅임

## 이슈

- GitHub Issue: #658
- 후속 분리 이슈: #661
- 작업 브랜치: `local/task658`

## 최종 결과

`rhwp-studio` web에서 `samples/exam_social.hwp` 오른쪽 자료 박스 내부 텍스트를 드래그 선택할 때 선택 하이라이트가 페이지 오른쪽 회색 영역까지 확장되던 문제를 정정했다.

수정 후 선택 rect는 페이지 폭 안에 머물며, 선택 하이라이트 DOM은 매 프레임 삭제/재생성되지 않고 재사용된다.

## 원인

### native selection rect 오류

줄 경계의 같은 `charOffset`이 이전 `TextRun`의 끝이면서 다음 `TextRun`의 시작일 수 있는데, 기존 `get_selection_rects_native()`는 render tree에서 먼저 발견된 run을 그대로 사용했다.

그 결과 선택 시작점은 다음 줄 시작이어야 하는데 이전 줄 끝 좌표로 잡히는 경우가 있었고, 다중 줄 선택 rect의 폭이 페이지 바깥까지 확장됐다.

### frontend DOM churn

`SelectionRenderer.render()`는 호출마다 기존 하이라이트 div를 모두 제거한 뒤 rect 수만큼 새 div를 생성했다. rect가 잘못 튀는 순간 긴 하이라이트 노드가 반복 생성/삭제되어 드래그 UX가 더 나빠졌다.

## 변경 내용

### 1. native selection rect 정합화

파일:

- `src/document_core/queries/cursor_nav.rs`

변경:

- `CursorBias` 도입
  - `Leading`: 선택 rect 시작점은 경계 offset에서 다음 run 시작을 우선
  - `Trailing`: 선택 rect 끝점은 경계 offset에서 이전 run 끝을 우선
- body/cell cursor hit 탐색을 “첫 hit 즉시 반환”에서 “후보 점수화 후 선택”으로 변경
- 셀 경로 접근을 `ctx.path.first()` 기반으로 방어

### 2. 회귀 테스트와 진단 도구

파일:

- `tests/issue_658_text_selection_rects.rs`
- `examples/inspect_658_selection.rs`

검증:

- `exam_social.hwp` 오른쪽 자료 박스 선택 rect가 페이지 폭을 넘지 않음
- 첫 셀 문단 3줄 선택에서 줄 y 좌표가 단조 증가
- 본문 다중 줄 선택에서 다음 줄이 이전 줄 y를 재사용하지 않음

### 3. 선택 하이라이트 DOM 재사용

파일:

- `rhwp-studio/src/engine/selection-renderer.ts`

변경:

- 하이라이트 div pool 재사용
- rect 수가 줄면 초과 div는 제거하지 않고 `display:none`
- 동일 rect layout signature 반복 시 DOM style 갱신 생략
- `clear()`는 active div만 숨기고 pool은 유지

### 4. 드래그 중 caret 갱신 경량화

파일:

- `rhwp-studio/src/engine/caret-renderer.ts`
- `rhwp-studio/src/engine/input-handler.ts`
- `rhwp-studio/src/engine/input-handler-mouse.ts`

변경:

- `CaretRenderer.updateLive()` 추가
- 드래그 rAF 경로에서 `updateCaret()` 대신 `updateCaretDuringDrag()` 호출
- 드래그 중 반복되는 `emitCursorFormatState()`와 `updateFieldMarkers()` 호출 제거
- mouseup 후 기존 전체 caret 갱신은 유지

## 검증 결과

```bash
cargo test --test issue_658_text_selection_rects
```

결과: 2개 통과

```bash
cargo test --lib --release
```

결과: 1141개 통과, 실패 0개, ignored 2개

```bash
cd rhwp-studio
npm run build
```

결과: 통과

```bash
cargo run --example inspect_658_selection
```

결과: 관찰 대상 모두 `overflow_count=0`

웹 계측:

- URL: `http://127.0.0.1:7701/`
- 샘플: `samples/exam_social.hwp`
- 전체 선택 rect 수: 18
- 전체 선택 rect 오른쪽 최대값: `956.6px`
- 페이지 폭: `1028.0px`
- selection div 재사용: 18개 pool 유지, visible 수만 18 → 3 → 0으로 변화
- console warn/error: 없음

## 시각 판정

작업지시자 직접 테스트 결과:

```text
드래그가 텍스트를 넘어가는 문제는 해결되었어.
```

따라서 #658의 핵심 증상은 해결된 것으로 판단한다.

## 후속 이슈

직접 테스트 과정에서 커서 및 페이지 스크롤 위치가 특정 상황에서 튀는 별도 문제가 발견되었다.

해당 증상은 selection rect overflow와 다른 원인으로 판단하여 #661로 분리했다.

후속 의심 지점:

- `input-handler-mouse.ts` 드래그 rAF 내부의 `hitTestFromEvent(e)` 재해석
- `updateCaretDuringDrag()`의 `scrollCaretIntoView()` 호출
- `CursorState.updateRect()`의 pageIndex mismatch 폴백
- Rust `getCursorRect*`의 page hint 없는 후보 페이지 순회

## PR 본문 초안

```markdown
## Summary

- Fix selection rect line-boundary ambiguity so drag selection highlights stay inside the rendered text/page bounds.
- Reuse rhwp-studio selection highlight DOM nodes during drag to reduce churn.
- Split the newly observed cursor/scroll jump behavior into follow-up issue #661.

## Verification

- cargo test --test issue_658_text_selection_rects
- cargo test --lib --release
- cd rhwp-studio && npm run build
- cargo run --example inspect_658_selection
- Local web instrumentation against samples/exam_social.hwp confirmed max selection right edge 956.6px within page width 1028.0px and stable selection-layer node reuse.
```

## 결론

#658은 PR 제출 가능한 상태로 정리됐다. #661은 별도 브랜치에서 후속 진행하는 것이 적절하다.
