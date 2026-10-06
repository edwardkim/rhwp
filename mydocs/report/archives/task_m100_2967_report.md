# 완료 보고서 — Task M100-2967

- 이슈: #2967
- 제목: 사진 속성 밝기/대비 값이 HTML 입력 범위(-100~100)를 벗어나도 clamp 없이 그대로 적용됨
- 작성일: 2026-07-22
- 브랜치: `task/m100-2967-picture-brightness-contrast-clamp`

## 1. 완료 내용

`rhwp-studio/src/ui/picture-props-apply-model.ts`의 `appendImageEffects()`에서
밝기(brightness)·대비(contrast) 값을 UI가 선언한 HTML 입력 범위(-100~100)와 동일하게
clamp하도록 수정했다. 같은 함수 안의 투명도(transparency) 처리는 이미
`Math.max(0, Math.min(100, ...))`로 clamp되어 있었는데, 밝기·대비만 `integerOr()` 파싱
결과를 그대로 patch에 실었다. #2845/#2938/#2949/#2959에서 반복 확인된 "HTML min/max는
있지만 확인 처리 로직에는 clamp가 없는" 패턴과 동일한 결함이다.

## 2. 주요 변경

- `rhwp-studio/src/ui/picture-props-apply-model.ts`
  - `appendImageEffects()`에서 brightness/contrast를 `Math.max(-100, Math.min(100, ...))`로
    clamp 후 patch에 반영
- `rhwp-studio/tests/picture-props-apply-model.test.ts`
  - `image brightness and contrast clamp to the -100..100 HTML input range` 픽스처 추가
    (입력 `250`/`-999` → 결과 `100`/`-100` 검증)

## 3. 검증 결과

- `npx vitest run tests/picture-props-apply-model.test.ts`
  - 신규 테스트 포함 전체 통과 (기존 파일의 vitest 러너 특성상 파일 레벨에서
    "No test suite found" 경고가 함께 출력되나, 개별 테스트 케이스는 모두 통과로 표시됨 —
    같은 파일의 기존 테스트들과 동일한 현상)

## 4. 리스크

- 변경 범위가 `appendImageEffects()` 내부 2개 필드로 국한되어 다른 patch 필드나
  객체 타입(shape/line/ole)에는 영향이 없다.

## 5. 결론

Task M100-2967 구현과 테스트를 완료했다. PR 생성 후 이슈를 close할 수 있다.
