# task_m100_2857: 문자표 최근 사용 문자 하이라이트 버그 수정

## 이슈
#3052 — [문자표] 최근 사용 문자 선택 시 엉뚱한 그리드 셀이 하이라이트됨

## 원인
`rhwp-studio/src/ui/symbols-dialog.ts`의 `selectChar()`는 그리드 하이라이트 인덱스를
`codePoint - this.currentBlock.start`로 계산한다. [최근 사용한 문자] 목록에서 문자를
클릭하면 그 문자가 **현재 화면에 표시된 블록**에 속하지 않을 수 있는데도 동일한 계산을
그대로 적용해, charGrid의 관계없는 셀(또는 우연히 범위 안에 들어간 엉뚱한 문자)에
`selected` 클래스를 붙였다.

## 수정
- `isCodePointInBlock(codePoint, block)` 헬퍼를 추가하고 export.
- `selectChar()`에서 이 헬퍼로 문자가 현재 블록에 속할 때만 그리드 하이라이트를 적용하도록
  가드.

## 검증
- 신규 단위 테스트 `rhwp-studio/tests/symbols-dialog.test.ts` 추가:
  현재 블록 내부 코드포인트는 true, 다른 블록(한글 음절) 코드포인트는 false를 확인.
- `npx tsx --test tests/symbols-dialog.test.ts` 통과.

## 영향 범위
UI 표시(하이라이트)만 변경, 문자 선택/삽입 로직(`selectedChar`, `codeLabel`, `previewCell`)은
변경 없음 — 최소 diff.
