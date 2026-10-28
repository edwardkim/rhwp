# Task #1280 1단계 완료보고서 — 프런트엔드 핵심 수정

## 목표

`enterTextboxPlacementMode()`가 도형 타입을 `'rectangle'`로 전달하여 글상자가 text_box 없이 생성되는
버그(#1280)를 프런트엔드에서 수정한다.

## 변경 내용

**파일**: `rhwp-studio/src/engine/input-handler.ts`

1. `enterTextboxPlacementMode()` (L512~): `this.shapePlacementType = 'rectangle'` → `'textbox'`.
   - 의도를 명시하는 주석 추가(#1280 참조: rectangle 전달 시 text_box 없는 Rectangle 생성 → 입력·붙여넣기 실패).
2. `shapePlacementType` 필드 주석(L150): 가능한 값 목록을
   `'rectangle' | 'ellipse' | 'line' | 'arc' | 'polygon' | 'textbox' | 'connector-*'`로 정합.

수정 후 자동 정상화(코드 추가 없이 기존 분기가 의도대로 동작):
- `input-handler.ts:874` `if (shapePlacementType !== 'textbox')` → textbox는 종이 기준 offset 계산 스킵
  (offset=0) → 백엔드가 vertRel=Para/horzRel=Column/treat_as_char=true로 커서 위치에 인라인 배치.
- `:915` `createShapeControl({ shapeType: 'textbox' })` → 백엔드 `wasm_api.rs:2865`/`object_ops.rs`의
  `== "textbox"` 게이트 통과 → text_box(내부 문단) + margin(283) 정상 구성.
- `:922` `selType`: textbox는 `'line'`/`'connector-*'`가 아니므로 `'shape'` 분류(정상).

일반 사각형 삽입은 별도 함수 `enterShapePlacementMode('rectangle')`(L520)을 통하므로 영향 없음.

## 검증

| 항목 | 결과 |
|------|------|
| `node_modules/.bin/tsc --noEmit` (input-handler.ts) | **0 오류** ✓ |
| 잔여 tsc 오류 2건 (`@wasm/rhwp.js` 미해결) | 기존·환경적 — `pkg/`(Docker WASM 빌드 산출물) 미생성으로 발생, 본 수정과 무관 |

`@wasm/*`는 tsconfig에서 `../pkg/*`로 매핑되며 `pkg/`는 Docker WASM 빌드(3단계 e2e 시 수행)에서 생성된다.
본 단계 수정 파일에는 타입 오류가 없다.

## 다음 단계

2단계: `object_ops.rs`에 `issue_1280_*` 단위 테스트를 추가하여 백엔드 글상자 생성 계약을 고정한다.

## 승인 대기

본 보고서와 소스 커밋 후 승인을 요청한다. 승인 후 2단계로 진행한다.
