# PR #????: hwpctl PageCount·Clear WASM 호출 예외 처리 추가

## 이슈
- **Issue**: #2684 — PageCount·Clear WASM 호출에 try-catch 누락

## 분석

`rhwp-studio/src/hwpctl/index.ts`의 `HwpCtrl` 클래스는 대부분의 WASM 호출 메서드(`Open`, `SaveAs`, `InsertText`, `SetCellText`, `GetCellText`, `EvaluateFormula`, `GetFieldList`, `MoveToField`, `PutFieldText`, `GetFieldText`, `MovePos`)가 try-catch로 예외를 처리하고 `console.error`로 기록한다.

그러나 다음 두 메서드는 try-catch 없이 WASM을 직접 호출한다:

```typescript
// Clear — 예외 발생 시 호출자까지 전파 + 커서 위치만 초기화되는 불일치
Clear(): void {
  this.wasmDoc.createBlankDocument();
  this.cursorSection = 0;
  this.cursorPara = 0;
  this.cursorPos = 0;
}

// PageCount — WASM 패닉이 스튜디오 전체로 전파 가능
PageCount(): number {
  return this.wasmDoc.pageCount();
}
```

### 영향

- `PageCount()` WASM 패닉 → 스튜디오 전체 비정상 종료
- `Clear()` 실패 후 커서 위치 초기화 → 문서 상태 불일치

## 변경

두 메서드에 try-catch를 추가하고 실패 시 안전한 기본값을 반환:

```typescript
// after
Clear(): void {
  try {
    this.wasmDoc.createBlankDocument();
    this.cursorSection = 0;
    this.cursorPara = 0;
    this.cursorPos = 0;
  } catch (e) {
    console.error('[hwpctl] Clear 실패:', e);
  }
}

PageCount(): number {
  try {
    return this.wasmDoc.pageCount();
  } catch (e) {
    console.error('[hwpctl] PageCount 실패:', e);
    return 0;
  }
}
```

## 검증

- 정상 경로: 기존 동작과 완전히 동일
- 예외 경로: `console.error` 출력 후 안전한 기본값(0 또는 void) 반환
- `PageCount()` 실패 시 `0` 반환 — 호출자는 페이지 없음으로 처리
- TypeScript 타입 체크 통과 (기존 `@wasm/rhwp.js` 미생성 오류 외 신규 오류 없음)

## 결과
- **Branch**: `pr/fix-issue-2684-hwpctl-trycatch`
- **PR**: https://github.com/edwardkim/rhwp/pull/2686
- **Closes**: #2684
