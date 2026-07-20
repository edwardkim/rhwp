# PR #2638: editor loadFile suppressDialogs 기본값 true 테스트 추가

## 이슈
- **Issue**: #2637 — suppressDialogs 기본값 true 회귀 방지 테스트 누락

## 분석

PR #2602에서 `npm/editor/index.js`의 `suppressDialogs` 기본값을
`true`로 변경했다. 그러나 이 동작을 단언하는 자동화 테스트가 없어
향후 변경 시 회귀를 감지할 수 없다.

## 변경

`npm/editor/tests/suppress-dialogs-default.test.mjs` 신규 생성:

1. **옵션 생략 시 true**: `loadFile(data, name)` → suppressDialogs: true
2. **명시적 false 유지**: `{ suppressDialogs: false }` → suppressDialogs: false
3. **명시적 true 유지**: `{ suppressDialogs: true }` → suppressDialogs: true
4. **skipUnsavedGuard 조합**: 두 옵션의 독립성 검증

## 결과
- `node --test tests/suppress-dialogs-default.test.mjs` → 4/4 pass
- Closes #2637
