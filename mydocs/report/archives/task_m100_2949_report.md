# task-m100-2949 처리 결과 보고

## 이슈

- #2949 수식 속성 대화상자 글자 크기 입력에 min/max clamp 누락 — Enter 제출 시 범위 밖 값이 CTRL_DATA에 기록될 수 있음

## 문제 요약

`rhwp-studio/src/ui/equation-props-dialog.ts`의 `EquationPropertiesDialog`는 수식 속성 대화상자의 "크기"
입력(`fontSizeInput`)에 `min="1"` / `max="127"` HTML 속성을 부여해 두었으나, 확인 처리 메서드인
`handleOk()`에서는 이 범위를 다시 강제하지 않았다.

```ts
const fontSize = (parseInt(this.fontSizeInput.value, 10) || 10) * 100;
```

`min`/`max` 속성은 브라우저의 number-input 스피너 화살표 조작에만 영향을 주고, 사용자가 입력창에 직접
숫자를 타이핑한 뒤 확인(OK)을 누르거나 프로그램적으로 `.value`를 설정하는 경로에는 아무 영향을 주지
않는다. `page-setup-dialog`(#2845), `table-cell-props-dialog`(#2847), `numbering-dialog`(#2938),
`char-shape-dialog`(#2908), `section-settings-dialog`(#2915), `style-edit-dialog`(#2928) 등 형제
다이얼로그에서 이미 여러 차례 발견·수정된 것과 동일한 패턴이다.

범위를 벗어난 값(예: 음수, 수만 pt)이 그대로 `* 100`을 거쳐 `wasm.setEquationProperties` /
`wasm.setNoteEquationProperties`로 전달되면, 수식 컨트롤의 CTRL_DATA에 비정상적인 fontSize 값이
기록되어 이후 저장·재로드·렌더링 단계에서 문제를 일으킬 수 있다.

## 수정 내용

`handleOk()`에서 `fontSizeInput.value`를 파싱한 뒤 `Math.max(1, Math.min(127, ...))`로 clamp 하도록
1줄을 추가했다 (형제 다이얼로그들과 동일한 clamp 패턴).

- `rhwp-studio/src/ui/equation-props-dialog.ts` — `handleOk()`의 `fontSize` 계산부 clamp 추가 (실질 diff 2줄).

## 테스트

기존 `rhwp-studio/tests/equation-props-undo.test.ts`가 취하는 방식(빌드된 dialog 소스를 문자열로 읽어
정규식으로 handleOk 본문을 검증)을 그대로 따라, 새 테스트 파일을 추가했다.

- `rhwp-studio/tests/equation-props-fontsize-clamp.test.ts`
  - `handleOk()` 본문에서 `Math.max(1, Math.min(127, parseInt(this.fontSizeInput.value, 10) || 10))`
    패턴이 존재하는지 검증한다.
  - 수정 전 원본 코드(`(parseInt(this.fontSizeInput.value, 10) || 10) * 100`)에는 해당 패턴이 없어
    테스트가 실패(RED)했고, clamp 추가 후 통과(GREEN)함을 확인했다.

```
node --test 결과:
✔ 수식 속성 다이얼로그 handleOk는 글자 크기를 1~127 범위로 clamp 한다
✔ 수식 속성 다이얼로그 apply는 스냅샷으로 undo 기록된다 (기존 테스트, 회귀 없음 확인)
✔ 수식 속성 다이얼로그 생성자는 CommandServices를 전달받는다 (기존 테스트, 회귀 없음 확인)
```

## 영향 범위

- `rhwp-studio/src/ui/equation-props-dialog.ts`의 `handleOk()` 메서드 1개 지점만 수정했다.
- 기존 undo/서비스 라우팅 테스트(`equation-props-undo.test.ts`)가 여전히 통과함을 확인해 회귀가 없음을
  검증했다.
