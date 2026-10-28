# Task m100 #1964 Stage 1

## 목표

GitHub Issue #1964의 `raw IME/iOS 입력 경로 page-local refresh 안전 가드`를 보강한다.

PR #1952 이후 command 기반 텍스트 입력은 `insertedText`, `beforePageIndex`,
`afterPageIndex`를 `shouldUsePageLocalRefresh()`에 전달하지만, raw IME/iOS 경로는
위 힌트 없이 `insertText` 판정을 호출하고 있었다. 이 때문에 줄바꿈, 탭, 긴 입력,
쪽 이동 입력이 page-local refresh로 잘못 분류될 수 있다.

## 수정

- `InputHandler.afterTextInputEdit()`가 `PageLocalTextEditOptions`를 받도록 확장했다.
- IME 조합 중 raw 입력 경로에서 실제 조합 문자열과 전후 page index를 전달하도록 했다.
- iOS fallback 경로에서 첫 입력 앵커 기준 `beforePageIndex`를 보존하고, 마지막 입력 후
  디바운스 렌더링 시 `insertedText`, `beforePageIndex`, `afterPageIndex`를 함께 전달하도록 했다.
- 기존 `isPageLocalTextEditCommand()` 계약을 재사용해 줄바꿈, 탭, 긴 입력, page 이동은
  full refresh로 남도록 했다.

## 검증

```bash
cd rhwp-studio
node --test tests/input-edit-invalidation.test.ts
npm run build
cd ..
git diff --check
```

결과:

- `node --test tests/input-edit-invalidation.test.ts`: 7개 통과
- `npm run build`: 통과
- `git diff --check`: 통과

보조 확인:

- `npm test -- input-edit-invalidation.test.ts`: 166개 통과

## 시각 검증

이번 변경은 Studio 입력 이벤트 이후의 refresh 라우팅 계약 보강이며 PDF/SVG 출력 배치 변경이 아니다.
따라서 `visual_sweep_guide.md` 기반 시각 비교 대신 입력 invalidation 단위 테스트와 Studio build로 검증했다.
