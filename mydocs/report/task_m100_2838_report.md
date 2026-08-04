# Task m100-2838: numbering-dialog 시작 번호 clamp 누락 수정

## 관련 이슈

edwardkim/rhwp #2838

## 버그

`rhwp-studio/src/ui/numbering-dialog.ts`의 문단 번호 대화상자 "시작 번호" `<input type="number">`는
`min="1"` `max="999"` HTML 속성을 갖고 있었지만(364~365행), 이 다이얼로그는 `<form>` submit 흐름 없이
`input` 이벤트만으로 상태를 갱신하므로 브라우저가 min/max를 강제하지 않았다.

기존 핸들러:

```ts
startInput.addEventListener('input', () => {
  this.startNumber = parseInt(startInput.value) || 1;
  this.updatePreview();
});
```

`|| 1`은 `NaN`(빈 문자열 등)만 걸러내며, `0`/`-5`/`100000` 같은 유효한 정수는 그대로
`this.startNumber`에 반영되고, `onConfirm()`에서 `JSON.stringify({ ..., startNumber: this.startNumber })`로
`wasm.createNumbering()` 호출에 그대로 실려 Rust WASM 엔진까지 전달된다.

## 원인

입력 검증이 HTML 속성 선언에만 의존했고, 실제 상태를 갱신하는 JS 핸들러에는 범위 clamp 로직이
없었다. `formatNumber()`(93~120행)는 이미 여러 포맷 코드에서 `num >= 1 && num <= N` 가드를
반복적으로 두고 있어, 코드베이스가 "1 이상의 유효 범위"를 전제하고 있음을 보여준다 — 그런데
정작 값의 유일한 입력 지점에는 동일한 가드가 없었다.

## 수정

`rhwp-studio/src/ui/numbering-dialog.ts`의 `input` 핸들러에서 `parseInt` 결과를
`Math.min(999, Math.max(1, parsed))`로 clamp한 뒤 `this.startNumber`에 대입하도록 변경.

```ts
startInput.addEventListener('input', () => {
  const parsed = parseInt(startInput.value) || 1;
  this.startNumber = Math.min(999, Math.max(1, parsed));
  this.updatePreview();
});
```

## 테스트

`rhwp-studio/tests/numbering-dialog-start-clamp.test.ts` 신규 추가 — 소스 가드 테스트로,
`startInput` input 핸들러 본문에 `Math.min(999, Math.max(1, ...))` clamp 패턴이 존재하는지
정규식으로 확인한다.

### Red (수정 전, `git stash`로 원본 재현)

```
✖ numbering-dialog.ts 시작 번호 input 핸들러는 1~999로 clamp한다 (3.9077ms)
AssertionError: startNumber 대입 전에 Math.min(999, Math.max(1, ...)) clamp가 있어야 함
```

### Green (수정 후)

```
npm test
ℹ tests 500
ℹ pass 499
ℹ fail 1   (cell-flow-boundary.test.ts — 기존에 알려진 사전 실패, 이번 변경과 무관)
```

새로 추가한 `numbering-dialog-start-clamp.test.ts`는 통과.

## tsc 베이스라인 확인

```
npx tsc --noEmit
```

기존과 동일하게 TS2307 에러 2건만 존재 (`@wasm/rhwp.js` 모듈 미해결, WASM 빌드 산출물 부재로 인한
사전 존재 에러). 이번 변경으로 새로 발생한 에러 없음.
