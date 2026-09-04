# Task #3041 처리 결과

## 이슈
#3041 미주 모양 대화상자 간격 입력값 비우면 NaN이 mmToHwp에 전달됨

## 원인
`rhwp-studio/src/ui/endnote-shape-dialog.ts`의 `onConfirm()`에서
`separatorLength`/`separatorMarginTop`/`noteSpacing`/`separatorMarginBottom` 4개 필드가
`mmToHwp(parseFloat(input.value))` 형태로 계산됐다. 입력 필드를 비운 채(빈 문자열) 확인을
누르면 `parseFloat('')`가 `NaN`을 반환하고, 그대로 `mmToHwp(NaN)` → `NaN`이 되어
`EndnoteShapeSettings`에 저장된 뒤 `wasm.applyEndnoteShape`로 전달된다.

같은 저장소의 `column-settings-dialog.ts`, `page-setup-dialog.ts` 등은 동일 패턴에서
`parseFloat(input.value) || 0`으로 NaN을 0으로 폴백 처리하는데, `endnote-shape-dialog.ts`만
이 폴백이 빠져 있었다.

## 수정
4개 파싱 라인에 `|| 0` 폴백을 추가해 다른 다이얼로그와 동일한 관례를 따르도록 했다.

- `rhwp-studio/src/ui/endnote-shape-dialog.ts`

## 테스트
- `rhwp-studio/tests/endnote-shape-dialog-nan-fallback.test.ts` 신규 추가 —
  4개 필드 모두 `parseFloat(...) || 0` 폴백을 갖추는지 정적 검사.
- `node --experimental-strip-types --test tests/endnote-shape-dialog-nan-fallback.test.ts` 통과.
- 기존 `undo-number-dialogs.test.ts`, `grid-settings-dialog.test.ts` 회귀 없음 확인.

## 범위
diff는 4줄(수정) + 신규 테스트 파일 1개로 최소화했다.
