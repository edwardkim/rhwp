# PR #????: recoveryFileName 데드코드 제거

## 이슈
- **Issue**: #2687 — recoveryFileName 데드코드 제거

## 분석

`rhwp-studio/src/recovery/recovery-format.ts`의 `recoveryFileName()`에 불필요한 조건 분기가 존재한다.

### 문제

```typescript
export function recoveryFileName(fileName: string, sourceFormat = 'hwp'): string {
  const base = baseNameWithoutKnownExtension(fileName);
  if (sourceFormat.toLowerCase() === 'hwpx') return `${base} 복구본.hwp`;
  return `${base} 복구본.hwp`;  // hwpx 분기와 동일
}
```

두 분기가 완전히 동일한 값을 반환한다 (`복구본.hwp`). 주석에서도 "autosave draft는 exportHwp() 결과"라고 명시하고 있어, sourceFormat과 무관하게 항상 .hwp 확장자를 사용하는 것이 의도된 동작이다.

### 영향

- 호출처에서 `sourceFormat`을 넘기지만 실제로 사용되지 않음
- 불필요한 조건문으로 인한 코드 복잡도 증가

## 변경

1. `sourceFormat` 파라미터 제거
2. 단일 `return`으로 통일
3. 호출처(`main.ts:restoreAutosaveDraft`)에서 두 번째 인자 제거
4. 테스트(`recovery-ui.test.ts`)에서 불필요한 두 번째 인자 제거

```typescript
// after
export function recoveryFileName(fileName: string): string {
  const base = baseNameWithoutKnownExtension(fileName);
  return `${base} 복구본.hwp`;
}
```

## 검증

- `node --test tests/recovery-ui.test.ts` — 전과 동일하게 5/5 통과
- `recoveryFileName` 시그니처 변경에 따른 타입 오류 없음 (기존 호출처 모두 수정 완료)
- 함수 동작이 변경되지 않으므로 회귀 없음

## 결과
- **Branch**: `pr/fix-issue-2687-recovery-deadcode`
- **PR**: https://github.com/edwardkim/rhwp/pull/2688
- **Closes**: #2687
