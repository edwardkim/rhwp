# Task M100 #1273 Stage 4 완료 보고서

floating(글자처럼취급 해제) 중첩 picture 리사이즈 시 글상자 이탈 수정 — offset 페이지절대값 → 델타 기반.

## 1. 배경 (Stage 1~3 후 노출된 잠재 결함)

Stage 1 이전에는 중첩 picture 리사이즈가 아예 실패했으나, 수정 후 리사이즈가 동작하면서
**단일 선택 리사이즈의 offset 좌표계 버그**가 드러났다. "개체 속성에서 글자처럼 취급 해제(floating)
후 축소하면 글상자 밖으로 벗어나 이동"하는 현상(작업지시자 보고).

## 2. 근본 원인 (코드 + 런타임 확정)

`finishPictureResizeDrag` 단일 선택 경로(`input-handler-picture.ts`)가 offset 을
**페이지 절대 좌표**로 기록:
- `before['horzOffset'] = state.origHorzOffset`(실제 저장값, 컨테이너 상대)
- `updated['horzOffset'] = newBbox.x * PX2HWP`(페이지 절대) ← 좌표계 불일치

본문 그림(보통 HorzRelTo=page)은 저장값≈페이지좌표라 우연히 맞지만, 글상자/셀 중첩 picture 는
offset 이 **컨테이너 상대(작은 값)** 이라 페이지 절대값으로 덮으면 글상자 밖으로 튕긴다.
바로 위 **다중 선택 경로는 이미 델타 기반**(`origHorzOffset + deltaH`)으로 올바름.

**런타임 증거(probe/E2E)**: floating 전환 후 nw 축소 시 vertOffset 59925 → **122100**(페이지절대 ≈119850),
bbox y 1598 → **2427**(829px 점프, 글상자 이탈). 수정 후: 59925 → 62175(델타 2250), bbox 1598 → 1628(30px).

## 3. HWP5 스펙 정합 (한글문서파일형식_5.0_revision1.3.md)

- **표 69 개체 공통 속성**: "세로 오프셋 값"/"가로 오프셋 값"(각 HWPUNIT) 저장.
- **표 70 개체 공통 속성의 속성**: bit 3~4 `VertRelTo`(paper/page/para), bit 8~9 `HorzRelTo`
  (page/column/para). → **offset 은 기준(RelTo) 프레임에 대한 상대값**이며 페이지 절대 픽셀이 아니다.
- **표 83 개체 요소**: "개체가 속한 그룹 내에서의 X/Y offset" — 컨테이너 상대 좌표.

→ 리사이즈 코드는 객체의 RelTo 프레임을 알지 못하므로 **페이지 절대값으로 변환해 기록하면 스펙 위반**
(특히 RelTo=para 인 중첩 객체). **저장된 상대 offset 을 유지하고 드래그 델타만 더하는** 본 수정은
RelTo 값과 무관하게 항상 스펙에 정합한다(프레임 불가지). 본문 그림이 버그 코드에서도 우연히 동작한
이유(page 상대) 와, 중첩/para 상대 객체가 깨진 이유를 모두 설명한다.

## 4. 변경

`rhwp-studio/src/engine/input-handler-picture.ts` — **동일 버그가 2곳**에 있어 모두 델타 기반으로 수정:
- `finishPictureResizeDrag`(드래그 종료/확정) 단일 선택
- `updatePictureResizeDrag`(라이브 드래그) 단일 선택 ← 작업지시자 추가 보고
  ("리사이즈 중 검정 예비박스가 이미지 축소와 정합하지 않음")의 원인

```ts
const newHorzOffset = Math.round(newBbox.x * PX_TO_HWP);   // 페이지좌표
const origHorzOffset = Math.round(state.bbox.x * PX_TO_HWP);
const beforeHorzOffset = state.origHorzOffset ?? origHorzOffset; // 저장 offset
// before(저장 offset) + 페이지좌표 델타
updated['horzOffset'] = ((beforeHorzOffset + (newHorzOffset - origHorzOffset)) >>> 0);
```
다중 선택 경로와 동일한 델타 방식. 본문 그림은 before≈orig 이므로 동작 불변.

라이브 드래그(`updatePictureResizeDrag`)도 페이지 절대값을 쓰면, `renderDragPreview` 가 그린
예비 테두리(검정 박스)는 올바른 위치인데 **이미지만 페이지 절대 offset 으로 이동**해 드래그 중
박스와 이미지가 어긋났다. finishPictureResizeDrag(Stage4 초안)만 고치면 최종 결과는 맞지만
**드래그 중 프레임은 계속 어긋난다** → 라이브 경로도 동일 수정 필요.

## 5. 검증 (red-green)

`e2e/textbox-picture-ops-1273.test.mjs` 에 FLOATING 리사이즈 시나리오 추가:
글자처럼취급 해제 → nw 축소 → (a) vertOffset 변화가 델타 크기(<20000, 페이지절대 아님),
(b) bbox y 점프 <300px(글상자 유지), (c) **라이브 드래그 중(mouseup 전) 이미지 bbox 가
예비박스 추적**(bbYMid 점프 <300px) — `updatePictureResizeDrag` 라이브 fix 커버.
- **green**(수정): vertOffset 59925→62175, bbox 1598→1628, bbYMid 799→829(델타 30px) PASS.
- **red**(finish 되돌림): vertOffset 59925→122100, bbox 1598→2427 FAIL.
- **red**(live 되돌림): bbYMid 799→1628(829px 점프) → 라이브 추적 assertion FAIL(검정박스 어긋남 재현).
- tsc 신규 오류 0건. 기존 리사이즈/회전 시나리오 및 textbox-picture-1171 E2E 회귀 없음.
