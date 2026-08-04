# 완료 보고서 — Task M100-2941

- 이슈: #2941
- 제목: [rhwp-studio] 표/셀 속성 다이얼로그 배경 탭 무늬색·무늬모양 populate 동기화 결함
- 작성일: 2026-07-22
- 브랜치: `task/m100-2938b-bg-pattern-sync`

## 1. 완료 내용

`rhwp-studio/src/ui/table-cell-props-dialog.ts`의 `populateBgFromTarget()`에서 배경 무늬색
(`bgPatternColorPicker`)과 무늬모양(`bgPatternTypeSelect`)이 모델 값이 없을 때 이전에 열려
있던 다른 셀/표의 값을 그대로 유지하던 populate 동기화 결함을 수정했다.

## 2. 원인

```ts
if (props.patternColor) this.bgPatternColorPicker.value = props.patternColor;
if (props.patternType != null) this.bgPatternTypeSelect.value = String(props.patternType);
```

두 줄 모두 값이 있을 때만 UI를 갱신하는 조건부 대입이었다. `fillColor`는 매번 무조건
갱신되는 반면, `patternColor`/`patternType`이 없는 셀(단색 채우기만 있고 무늬는 없는 셀)을
열면 두 컨트롤은 직전에 열었던 다른 셀·표의 값을 그대로 유지했다. 이는 이미 수정된
표/셀 속성 다이얼로그 테두리 탭 select 동기화 결함(#2908 → PR #2913), 문단모양 다이얼로그
테두리 탭 select 동기화 결함(#2915 → PR #2921), 문자모양 다이얼로그 외곽선/그림자 토글
populate 동기화 결함(#2928 → PR #2933)과 동일한 클래스의 버그다.

`onConfirm()`은 `bgColorRadio.checked`일 때 `bgPatternTypeSelect.value`를 그대로 읽어
`newCellProps.patternType`에 저장하므로(1376번째 줄 부근), 사용자가 아무 것도 건드리지
않고 확인만 눌러도 스테일 무늬 값이 모델에 잘못 저장될 수 있었다. 미리보기
(`updateBgPreview()`)도 같은 스테일 값을 사용해 잘못된 무늬를 보여줬다.

## 3. 주요 변경

- `rhwp-studio/src/ui/table-cell-props-dialog.ts`
  - `populateBgFromTarget()`에서 무늬색/무늬모양 대입을 조건부(`if`)에서 널 병합
    연산자(`??`) 기반 무조건 대입으로 변경. 모델에 값이 없으면 각각 `'#000000'`,
    `'0'`(없음)으로 명시적 리셋한다.

```diff
-      this.bgColorRadio.checked = true;
-      this.bgColorPicker.value = props.fillColor;
-      if (props.patternColor) this.bgPatternColorPicker.value = props.patternColor;
-      if (props.patternType != null) this.bgPatternTypeSelect.value = String(props.patternType);
+      this.bgColorRadio.checked = true;
+      this.bgColorPicker.value = props.fillColor;
+      this.bgPatternColorPicker.value = props.patternColor ?? '#000000';
+      this.bgPatternTypeSelect.value = props.patternType != null ? String(props.patternType) : '0';
```

diff 4줄 변경.

## 4. 검증 결과

TS 전용 변경이며 로직이 단순(대입 조건 제거)하므로 별도 빌드 없이 수동 코드 리뷰로 검증했다.

- `props.patternColor`가 `undefined`인 경로: 이전에는 대입을 건너뛰어 스테일 값 유지 →
  수정 후 `'#000000'`으로 명시적 리셋됨을 코드상 확인.
- `props.patternType`이 `null`/`undefined`인 경로: 이전에는 대입을 건너뛰어 스테일 값 유지 →
  수정 후 `'0'`(없음)으로 명시적 리셋됨을 코드상 확인.
- 값이 정상적으로 있는 경로(`patternColor`/`patternType` 둘 다 존재)는 이전과 동일하게
  모델 값을 그대로 대입하므로 회귀 없음을 확인.
- 같은 파일의 다른 필드(`cellFieldNameInput.value = cp.fieldName ?? ''`, 1263번째 줄)가
  이미 동일한 `??` 패턴을 사용하고 있어 코드베이스 관례와 일치함을 확인.

## 5. 남은 이슈

없음. 이번 수정 범위 밖에서 발견된 별도 결함은 없다.
