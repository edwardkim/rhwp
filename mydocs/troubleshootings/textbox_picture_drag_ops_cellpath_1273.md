---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 글상자/셀 중첩 그림 마우스 드래그 조작 실패 — 드래그 상태 staging ref 소비처 누락 (#1273)

- **작성일**: 2026-06-03
- **관련**: #1273, 선행 #1171(hit-test, CLOSED)·PR #1254·보완 커밋 `5453b254`,
  메인테이너 사후분석 `nested_picture_selection_ref_consumers_1171.md`
- **분류**: 재발 방지 (공유 ref 소비처 감사 누락 — 한 단계 심화)

## 현상

글상자(Shape text_box)·셀 안 picture 를 **마우스 핸들 드래그**로 리사이즈/회전/이동하면:

```
[InputHandler] 개체 리사이즈 실패: 렌더링 오류: 컨트롤 인덱스 1 범위 초과
[InputHandler] 개체 리사이즈 실패: 렌더링 오류: 지정된 Shape 컨트롤이 그림이 아닙니다
[InputHandler] 개체 회전 드래그 실패: 렌더링 오류: 컨트롤 인덱스 1 범위 초과
```

선택·대화상자·키보드는 정상. 마우스 드래그만 실패(단일 선택 한정, 다중 선택은 정상).

## 근본 원인 — 선택 ref 와 소비처 사이의 "드래그 상태 staging 객체"가 cellPath 를 떨어뜨림

`5453b254`(메인테이너)는 선택 ref 생성 → 조작 헬퍼(`get/setObjectProperties`) →
리사이즈 커맨드 타깃까지 **양 끝**에 cellPath 를 배선했다. 그러나 마우스 드래그는 선택 ref 를
곧장 쓰지 않고, **중간에 드래그 상태 객체**(`pictureResizeState`/`pictureRotateState`/
`pictureMoveState`)를 새로 만들어 그것을 소비한다. 그 객체를 만들 때
`ref: { sec, ppi, ci, type }` 리터럴로 재조립하면서 `cellPath`/`headerFooter` 를 누락
(`input-handler-mouse.ts` L307/L323/L369).

→ 드래그 중/종료 시 path 가 없어 scalar API(`setPictureProperties(sec,ppi,ci)`)로 폴백 →
중첩 picture 를 본문 문단으로 오해석 → 위 오류.

즉, **하류(소비처)는 cellPath 를 받을 준비를 끝냈는데 상류(드래그 상태 생성)가 흘려버려**
연결이 끊긴 것. `5453b254` 의 `input-handler-mouse.ts` 변경은 +2줄(선택 호출만)이고,
**드래그 상태 staging 객체는 감사 범위 밖**이었다.

## 메인테이너 1171 사후분석과의 관계

`nested_picture_selection_ref_consumers_1171.md` 의 교훈 #2("공유 ref 소비처 grep 감사")가
**한 단계 더** 적용됐어야 했다. 그 문서는 *조작 엔드포인트*(delete/resize 커맨드/dialog)를
감사 대상으로 봤지만, 그 사이에서 ref 를 **재materialize 하는 드래그 staging 객체**(resize/
rotate/move 3곳)는 목록에 없었다. → "공유 ref 의 소비처"에는 **최종 연산뿐 아니라 중간
staging 도 포함**해야 한다.

## 해결 (#1273, TypeScript 전용)

- `input-handler.ts`: 3개 드래그 상태 `ref` 타입에 `cellPath?`/`headerFooter?` 가산.
- `input-handler-mouse.ts` L307/L323/L369: 리터럴에 `cellPath: ref.cellPath, headerFooter: ref.headerFooter` 복사.
- `command.ts`: `MovePictureCommand`/`MoveShapeCommand` 에 cellPath by-path 지원(undo/redo) + `finishPictureMoveDrag` 전달.

## 재발 방지 — 검증 공백도 동시 해소

기존 E2E(`textbox-picture-1171`)는 `wasm.setCellPicturePropertiesByPath` 를 **직접** 불러
드래그 staging 경로를 **우회**했기에 버그가 있어도 통과했다(교훈 #3 그대로). 신규
`e2e/textbox-picture-ops-1273.test.mjs` 는 **InputHandler 의 실제 드래그 경로**(onClick →
mousemove → mouseup, 실제 핸들 좌표 + 스크롤 진입)를 구동하여:
- `pictureResizeState.ref.cellPath`/`pictureRotateState.ref.cellPath` 보존을 직접 검증,
- 리사이즈 by-path 반영 + undo 원복, 회전 각도 반영, 조작 중 콘솔 오류 0건 확인.

**red-green 검증 완료**: Stage1 수정을 되돌리면 동일 콘솔 오류로 FAIL, 적용 시 PASS.

## 체크리스트 (다음에 위치 개체를 1급화할 때)

1. 공유 ref 의 **모든 소비처**를 grep — 최종 연산뿐 아니라 **드래그/이동 staging 객체**까지.
2. ref 를 리터럴로 재조립하는 지점은 `cellPath`/`headerFooter`/`noteRef` 등 위치 식별자를 **전부 spread/복사**.
3. 검증은 WASM API 직접 호출이 아니라 **사용자 연산 표면(InputHandler 드래그 경로)** 으로.
