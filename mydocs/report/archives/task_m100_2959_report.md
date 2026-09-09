# 완료 보고서 — Task M100-2959

- 이슈: #2959
- 제목: 문단 모양 대화상자 줄 간격 입력이 min/max 범위를 벗어나도 확인 시 그대로 적용됨
- 작성일: 2026-07-22
- 브랜치: `task/m100-2959-line-spacing-clamp`

## 1. 배경

`rhwp-studio/src/ui/para-shape-dialog.ts`의 줄 간격 입력란(`lineSpacingInput`)은
`numberInput(0, 9999, 1)`로 생성되어 HTML `min=0, max=9999` 속성을 갖지만, 값을
확정해 모델에 반영하는 `collectMods()`에서는 이 범위를 검증하지 않았다. 브라우저
number input의 `min`/`max`는 스핀 버튼·마우스 휠 조작에만 클램프를 적용하고
키보드 직접 입력 후 바로 확인을 눌렀을 때는 강제되지 않으므로, 사용자가 예를
들어 `999999`를 입력하고 확인을 누르면 그 값이 그대로 `mods.lineSpacing`에
담겨 저장될 수 있었다.

이는 #2845, #2908, #2915, #2928, #2938, #2949 등에서 이미 여러 차례 발견·수정된
"HTML min/max 속성은 있지만 확인 로직에서 클램프가 빠진" 동일 계보의 버그다.

## 2. 완료 내용

`para-shape-dialog.ts`에 `clampLineSpacing(value)` 헬퍼(0~9999 클램프)를
추가하고, `collectMods()`의 `Percent`/그 외 두 분기 모두에서 `lineSpacingInput`
값을 이 헬퍼로 감싸도록 수정했다.

## 3. 주요 변경

- `rhwp-studio/src/ui/para-shape-dialog.ts`
  - `clampLineSpacing(value: number): number` 헬퍼 함수 추가 (`Math.max(0, Math.min(9999, value))`)
  - `collectMods()`의 `newLS` 계산 두 분기에 `clampLineSpacing` 적용
- `rhwp-studio/tests/para-shape-line-spacing-clamp.test.ts` (신규)
  - `clampLineSpacing` 헬퍼 정의와 `collectMods()` 내 실제 사용을 소스 검사 방식으로 검증

## 4. 검증 결과

통과:

```
node --test --experimental-strip-types tests/para-shape-line-spacing-clamp.test.ts
```

- tests 1, pass 1, fail 0

## 5. 참고

같은 대화상자의 `marginLeftInput`/`marginRightInput`/`indentInput`/
`spacingBeforeInput`/`spacingAfterInput`도 동일한 클램프 누락 패턴을 공유한다.
diff를 최소화하기 위해 이번 이슈에서는 영향이 가장 큰 줄 간격 필드 하나만
다뤘으며, 나머지 필드는 별도 이슈로 후속 처리하는 것을 권장한다.
