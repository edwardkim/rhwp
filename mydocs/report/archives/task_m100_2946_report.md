# task/m100-2946 처리 결과 보고

## 관련 이슈

- https://github.com/edwardkim/rhwp/issues/2946

## 문제 요약

`rhwp-studio/src/ui/page-border-dialog.ts`의 쪽 테두리/배경 다이얼로그에서 굵기(`lineWidthSelect`)
`<select>`가 옵션 9개(코드 0~8, 0.1mm~0.6mm)만 제공했습니다. 형제 다이얼로그인
`endnote-shape-dialog.ts`는 동일한 목적의 select에서 코드 0~15(0.1mm~5mm), 총 16개 옵션을
제공하며, 이것이 HWPX/HWP5 테두리 굵기 값의 실제 범위와 일치하는 참조 구현입니다.

`populate()`가 문서에서 읽은 굵기 코드값을 `this.lineWidthSelect.value = String(firstBorder.width || 0)`로
할당하는데, 값이 9 이상이면 대응하는 `<option>`이 없어 할당이 무시되고 브라우저 기본 동작에 따라
select는 첫 번째 옵션(코드 0, 0.1mm)을 가리키게 됩니다. 이어서 `onConfirm()` → `currentBorder()`가
select의 현재 값을 그대로 읽어 저장하므로, 사용자가 굵기 필드를 전혀 조작하지 않고 다이얼로그를
열었다가 그대로 확인만 눌러도 0.7mm~5mm 범위의 원본 테두리 굵기가 조용히 0.1mm로 손실됩니다.

## 수정 내용

`buildLineWidthSelect()`의 옵션 라벨 배열에 코드 9~15에 해당하는 `0.7mm`, `1mm`, `1.5mm`, `2mm`,
`3mm`, `4mm`, `5mm`를 추가해 총 16개 옵션으로 확장했습니다. 라벨 텍스트는
`endnote-shape-dialog.ts`의 `LINE_WIDTH_OPTIONS` 라벨 형식과 동일하게 맞췄습니다.

- 변경 파일: `rhwp-studio/src/ui/page-border-dialog.ts` (1개 파일, 옵션 배열 확장 4줄 순증)

## 검증

- 별도 빌드 없이 TS 코드 리뷰로 검증: `buildLineWidthSelect()`가 생성하는 옵션의 `value`가
  이제 `"0"`~`"15"`까지 연속적으로 존재함을 확인했습니다.
- `populate()`의 `this.lineWidthSelect.value = String(firstBorder.width || 0)` 할당이 굵기 코드
  0~15 전 범위에서 대응하는 `<option>`을 찾도록 옵션 목록과 코드값이 1:1로 일치함을 확인했습니다.
- `onConfirm()` → `currentBorder()`가 `parseInt(this.lineWidthSelect.value, 10)`으로 굵기를 다시
  읽어 저장하는 경로도 동일한 옵션 목록을 사용하므로 왕복(round-trip) 시 굵기 손실이 없음을
  코드상으로 확인했습니다.
- 이 다이얼로그의 옵션 목록을 검증하는 기존 테스트 하네스가 없어 별도 테스트는 추가하지
  않았습니다.

## 남은 이슈

없음. 단일 파일의 옵션 목록 확장만으로 해결되며 IR/파서 변경은 필요하지 않습니다.
