# 최종 결과보고서 — 글상자/셀 중첩 그림 마우스 드래그 조작 실패 수정 (M100 #1273)

- **이슈**: #1273 / **브랜치**: `local/task1273` (`local/devel` 분기)
- **선행/관계**: #1171(hit-test, CLOSED)·PR #1254·보완 커밋 `5453b254`, #1229, #1231, #1230
- **성격**: 버그 수정 (TypeScript 전용, WASM/Rust 무변경)
- **커밋**: `f4ae3bf2`(S1) · `d6f4d9ad`(S2) · `2a22849a`(S3) · `41c6465d`(S4)

## 1. 문제

글상자(Shape text_box)·셀 안의 **중첩 그림**을 마우스 핸들 드래그로 리사이즈/회전/이동하면
다음 오류가 발생(보고): `컨트롤 인덱스 1 범위 초과` / `지정된 Shape 컨트롤이 그림이 아닙니다` /
`개체 회전 드래그 실패`. 추가로, 글자처럼취급 해제(floating) 후 리사이즈 시 그림이 글상자 밖으로
이탈하는 현상.

## 2. 근본 원인

1. **드래그 상태 ref의 cellPath 소실**: 단일 개체 마우스 드래그 시작 시 드래그 상태 객체
   (`pictureResizeState`/`pictureRotateState`/`pictureMoveState`)의 `ref`를 `{sec,ppi,ci,type}`
   리터럴로 재구성하며 `cellPath`/`headerFooter`를 떨어뜨림 → 조작 헬퍼가 body-level scalar API로
   폴백 → 중첩 그림을 본문으로 오해석. (PR #1254/`5453b254`가 양 끝은 배선했으나 드래그 staging은 누락)
2. **이동 커맨드 body-level**: `MovePictureCommand`/`MoveShapeCommand`가 scalar API만 호출 →
   undo/redo 시 동일 결함 재현.
3. **리사이즈 offset 페이지절대값**: 단일 선택 리사이즈(라이브+확정)가 offset을 페이지 절대좌표로
   기록 → 중첩 그림(offset이 컨테이너 상대, HWP5 표69/70 VertRelTo/HorzRelTo)은 글상자 밖으로 이탈.

## 3. 수정 (Stage)

| Stage | 내용 | 파일 |
|------|------|------|
| 1 | 드래그 상태 ref 타입에 `cellPath?`/`headerFooter?` 추가 + 3개 생성 지점(L307/323/369)에서 복사 | `input-handler.ts`, `input-handler-mouse.ts` |
| 2 | `MovePicture/MoveShapeCommand`에 cellPath by-path 지원(execute/undo/mergeWith) + 종료 전달 | `command.ts`, `input-handler-picture.ts` |
| 3 | 드래그 lifecycle E2E 추가(실제 onClick→drag 경로) + 재발방지 문서 | `e2e/textbox-picture-ops-1273.test.mjs`, troubleshootings |
| 4 | 리사이즈 offset을 (저장값 + 페이지좌표 델타)로 — 라이브+확정 모두 | `input-handler-picture.ts` |

전부 TypeScript 한정. 필요한 by-path WASM 함수는 이미 번들에 존재(무변경).

## 4. 검증

- `npx tsc --noEmit`: 편집 파일 신규 오류 0건(기존 무관 오류 `canvaskit-renderer.ts`만 base와 동일).
- E2E `textbox-picture-ops-1273` (실제 InputHandler 드래그 경로, tac-img-02):
  - 리사이즈 by-path + cellPath 보존 + undo 원복, 회전 각도 반영, floating 리사이즈 글상자 유지 +
    라이브 추적, 조작 중 콘솔 오류 0건.
  - **red-green 검증**: Stage1 되돌리면 보고와 동일 오류(`지정된 Shape 컨트롤이 그림이 아닙니다`)로 FAIL,
    Stage4 되돌리면 글상자 이탈(offset 페이지절대값)로 FAIL. 적용 시 전부 PASS.
- 기존 `textbox-picture-1171` / `textbox-picture-insert-1171` E2E 회귀 없음.

## 5. 해결 결과 (당초 보고 항목)

- ✅ 글상자/셀 중첩 그림 드래그 **리사이즈·회전·이동** 정상화 (+ 이동 undo/redo).
- ✅ floating 해제 후 리사이즈 시 **글상자 이탈** 정상화.

## 6. 세션 중 발견된 별개 이슈 (#1273 범위 밖)

1. **그림을 글상자보다 크게 리사이즈 → 글상자가 렌더 이미지를 클리핑(컨테이너 미확장) → 선택 박스 >
   보이는 이미지**. HWP 동작도 애매(컨테이너 lazy 확장). 검증에 #2 선행 필요(새 문서 작성 차단) →
   **보류**.
2. **새로 만든 글상자에 커서 진입/붙여넣기 실패** (`글상자 없음` — paste-into-shape가 `text_box`
   미발견, `clipboard.rs:512`; 셀 커서 위치 실패). #1229(글상자 이미지 선택·복사, 읽기)와 **같은 영역
   형제이나 메커니즘은 별개(쓰기/커서 진입)**. → #1229 연관 또는 신규 이슈. 근본원인(글상자 생성 시
   text_box 누락 vs 커서 control 오지정) 별도 조사 필요.

> 별개 이슈 등록·`local/devel` merge·#1273 close 는 작업지시자 결정에 따른다.
