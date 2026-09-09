# Task #2915 — 문단 모양 다이얼로그 테두리 탭 굵기/색/선종류 select 동기화

## 배경

이미 확립된 `#2908`(PR #2913)에서 **표/셀 속성 다이얼로그**의 "테두리" 탭이 다이얼로그를
다시 열 때 내부 상태·미리보기만 문서 값으로 복원하고 굵기/색/선종류 select 컨트롤은
하드코딩 기본값에 머무는 결함을 고쳤다. 본 작업은 **동일한 populate-sync 결함**이
`문단 모양 다이얼로그`(`para-shape-dialog.ts`)의 테두리/배경 탭에도 존재함을 확인하고,
같은 대표-값 동기화 패턴으로 수정한다.

## 근본 원인

`para-shape-dialog.ts`의 `populateFromProps()`는 각 변(side)의 문서 테두리를
`this.borderStates[side]`와 `this.bdSideToggles[side]`에 복원하고 미리보기만
`updateBdPreview()`로 갱신한다. 그러나 실제 화면의 세 컨트롤은
`para-shape-tab-builders.ts`의 `buildBorderTab()`이 생성한 하드코딩 기본값에 머문다:

- `bdTypeSelect` → 기본 선택 `'0'`(선 없음)
- `bdWidthSelect` → 기본 선택 `'0'`
- `bdColorInput.value = '#000000'`

`buildBorderTab()`의 `onBorderControlChange()`/`applyBorderPreset()`은 이 컨트롤들의
**현재 값**을 그대로 읽어 `borderStates`에 기록한다. 따라서 테두리가 설정된 문단에서
다이얼로그를 다시 열고 종류/굵기/색을 건드리지 않은 채 방향 토글이나 프리셋을 재적용하면,
기본값(선 없음/0/검정)이 기존 서식을 조용히 덮어써 **굵기·색·선종류가 유실**된다.

## 수정 내용

`populateFromProps()`에서 `updateBdPreview()` 직전, 테두리가 켜진 대표 변(첫 유효 변,
없으면 `left`)의 값으로 세 컨트롤을 동기화한다.

```ts
const repSide = (['left', 'top', 'right', 'bottom'] as const)
  .find(s => this.borderStates[s].type !== 0) ?? 'left';
const rep = this.borderStates[repSide];
this.borderResult.bdTypeSelect.value = String(rep.type);
this.borderResult.bdWidthSelect.value = String(rep.width);
this.borderResult.bdColorInput.value = rep.color;
```

- 파일: `rhwp-studio/src/ui/para-shape-dialog.ts` (주석 포함 +12줄)
- 테스트: `rhwp-studio/tests/para-shape-border-tab-sync.test.ts` (신규 소스 가드)

## 검증

- `node --test tests/para-shape-border-tab-sync.test.ts`
  - 수정 전(동기화 라인 제거): **fail 1** (red)
  - 수정 후: **pass 1** (green)
- `npm test`: 전체 500개 중 499 pass, 사전 실패 `cell-flow-boundary.test.ts` 1건만 실패
  (본 작업과 무관한 기존 실패).
- `npx tsc --noEmit`: 기존 baseline 2건(`@wasm/rhwp.js` TS2307)만 남고 신규 오류 없음.

## 참고

- 선례: #2908 / PR #2913 — 표/셀 속성 다이얼로그에 동일 패턴 적용.
- 이슈: #2915.
