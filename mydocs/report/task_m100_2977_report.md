# 완료 보고서 — Task M100-2977

- 이슈: #2977
- 제목: 그림 확대/축소 비율 입력이 min/max 범위를 벗어나도 확인 시 그대로 적용됨 (picture-props-dialog)
- 작성일: 2026-07-22
- 브랜치: `task/m100-2977-image-scale-clamp`

## 1. 문제

`rhwp-studio` 그림 속성 대화상자의 "확대/축소 비율" 탭 가로/세로 배율 입력 필드
(`picScaleXInput`, `picScaleYInput`)는 `picture-props-dialog.ts`에서 HTML
`min=1`, `max=1000` 제약으로 생성된다. 그러나 확인(OK) 시 실제 패치를 만드는
`appendImageScale`(`picture-props-apply-model.ts`)은 이 범위를 전혀 검증하지 않고
`numberOr`로 파싱한 값을 그대로 원본 크기에 곱해 폭/높이를 계산했다.

사용자가 네이티브 스피너가 아닌 직접 타이핑(예: `-10`, `5000`)으로 값을 입력하면
HTML `min`/`max` 속성이 값 검증에 관여하지 않으므로, 범위를 벗어난 배율이 그대로
적용되어 음수 높이나 1000%를 초과하는 비정상적인 크기가 patch에 들어갈 수 있었다.

같은 파일의 밝기/대비/투명도 클램프, 그리고 최근 회전각 클램프(#2954)와 동일한
"HTML min/max는 있지만 확인 시 검증 누락" 패턴이다.

## 2. 수정 내용

`appendImageScale`에서 `numberOr`로 읽은 가로/세로 배율을 UI와 동일하게
`Math.max(1, Math.min(1000, ...))`로 클램프한 뒤 폭/높이를 계산하도록 했다.

- `rhwp-studio/src/ui/picture-props-apply-model.ts`
  - `appendImageScale`에 `scaleX`, `scaleY` 클램프 추가
- `rhwp-studio/tests/picture-props-apply-model.test.ts`
  - `'image scale clamps to the 1..1000 HTML input range'` 픽스처 추가
    (`x: '5000', y: '-10'` → `width: 10000, height: 8`로 클램프되는지 검증)

## 3. 검증 결과

`npx tsx --test tests/picture-props-apply-model.test.ts` 전체 통과 (27 passed).

## 4. 참고

- 동일 패턴 선례: 밝기/대비 클램프(#2967), 회전각 클램프(#2954)
