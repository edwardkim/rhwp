# task/m100-2928: 글자 모양 다이얼로그 외곽선/그림자 토글 populate 미동기화 수정

## 배경

이슈 지시에 따라 `rhwp-studio/src/ui/char-shape-dialog.ts`에서 #2908(표 셀 속성 다이얼로그
테두리 탭)·#2915(문단 모양 다이얼로그 테두리 탭)와 같은 populate 미동기화 버그 클래스가
밑줄/취소선 확장 탭 컨트롤에도 있는지 점검했다.

## 조사 결과

- 밑줄 확장 탭 컨트롤(`ulPosSelect`/`ulShapeSelect`/`ulColorInput`)과 취소선 확장 탭 컨트롤
  (`strikeShapeSelect`/`strikeColorInput`)은 이미 `populateFromProps()`에서 문서 값
  (`p.underlineType`, `p.underlineShape`, `p.underlineColor`, `p.strikeShape`, `p.strikeColor`)으로
  정상 동기화되어 있음을 확인했다(`char-shape-dialog.ts:864`-`868`). 이 부분은 버그 없음.
- 대신 같은 파일 기본 탭의 "외곽선"·"그림자" 아이콘 토글 버튼(`attrBtns['outline']`,
  `attrBtns['shadow']`)에서 동일한 버그 클래스를 발견했다. `collectMods()`는 이 버튼들의
  활성 클래스 상태를 `initialProps.outlineType`/`shadowType`(>0)과 비교해 변경분을 만드는데
  (`char-shape-dialog.ts:945`-`952`), `populateFromProps()`는 `bold`/`italic`/`underline`/
  `strikethrough`/`superscript`/`subscript`만 `setAttrBtn()`으로 초기화하고 `outline`/`shadow`는
  누락했다. 버튼은 항상 "꺼짐"으로 시작하므로, 이미 외곽선·그림자가 적용된 글자에서 다른
  값만 바꿔 저장해도 기존 외곽선·그림자 서식이 조용히 꺼진다.
- 이 조사 내용과 재현 경로는 이슈 #2928에 상세 기록했다.

## 수정

`populateFromProps()`에 아래 두 줄을 추가해, `collectMods()`의 비교 조건과 동일한 식으로
버튼을 초기화했다.

```ts
this.setAttrBtn('outline', (p.outlineType || 0) > 0);
this.setAttrBtn('shadow', (p.shadowType || 0) > 0);
```

## 테스트

`rhwp-studio/tests/char-shape-outline-shadow-sync.test.ts` 소스 가드 테스트 추가.
`populateFromProps()` 메서드 블록을 소스에서 잘라내 두 `setAttrBtn` 호출 패턴이 있는지 확인한다.

- 수정 전(red): `outline 버튼은 p.outlineType > 0 으로 초기화되어야 한다` assertion 실패로 확인
  (`git apply -R` 로 패치를 되돌린 뒤 재현, 이후 재적용).
- 수정 후(green): 통과.

```
npx tsx --test tests/char-shape-outline-shadow-sync.test.ts
✔ populateFromProps 가 outline/shadow 토글 버튼을 문서 값으로 동기화한다
```

## 전체 검증

```
npm test
ℹ tests 500
ℹ pass 499
ℹ fail 1   ← tests/cell-flow-boundary.test.ts (기존에도 실패하는 테스트, 본 변경과 무관)
```

```
npx tsc --noEmit
```
`@wasm/rhwp.js` 모듈 미해석으로 인한 기존 TS2307 오류 2건만 남고 신규 오류 없음(베이스라인과 동일).

## 변경 파일

- `rhwp-studio/src/ui/char-shape-dialog.ts` (populateFromProps 5줄 추가)
- `rhwp-studio/tests/char-shape-outline-shadow-sync.test.ts` (신규, 소스 가드 테스트)
- `mydocs/report/task_m100_2928_report.md` (본 문서)

## 관련

- 이슈 #2928
- 동일 버그 클래스 선례: #2908, #2915
