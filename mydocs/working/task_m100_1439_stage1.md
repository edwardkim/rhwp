# Task M100 #1439 — 1단계 완료 보고서 (drop-confirm-dialog + 게이트)

- 브랜치: `local/task1439`
- 작성일: 2026-06-19
- 추가/수정: `rhwp-studio/src/ui/drop-confirm-dialog.ts`(신규), `rhwp-studio/src/main.ts`

## 1. 신규 — `drop-confirm-dialog.ts`

`unsaved-changes-dialog.ts` 동형 (`ModalDialog` 상속 + `showAsync(): Promise<boolean>`):

- 본문: "드래그한 로컬 파일을 엽니다" + 파일명 + "로컬 파일 내용을 읽습니다. 계속?" 고지.
- 버튼: [열기](primary)/[취소]. `onConfirm` → `resolve(true)`, `hide`(취소/×/Escape/밖
  클릭) → `resolve(false)`. resolved 가드로 중복 방지.
- 기본값 안전: 미동의 → false → 미로딩.
- `chrome` API 의존 없음 — 확장(standalone 탭)/웹 공통 DOM 모달.

`export function showDropConfirmDialog(fileName): Promise<boolean>`.

## 2. drop 핸들러 게이트 (`main.ts`)

- import `showDropConfirmDialog` 추가.
- drop 핸들러 정정: 지원 형식(HWP/HWPX·이미지) 판별 후 **`loadFile`/이미지 배치 전에**
  `showDropConfirmDialog(file.name)` 확인. `!confirmed` 시 early return(미로딩).
- 순서: **드롭 보안 확인 → (HWP/HWPX) loadFile 내부 unsaved 가드**. 사용자가 "이 파일
  열기" 먼저 동의 → 그 다음 저장 안 된 변경 경고.
- 범위: HWP/HWPX + 이미지 드롭 모두 게이트(일관). 파일 메뉴/열기 버튼 경로 불변(미적용).

## 3. 검증

- `npx tsc --noEmit`: **exit 0** (타입 오류 없음).

## 4. 다음 단계

- 2단계: `npm run build`(확장/PWA) 성공 + 산출물에 게이트 포함 + 기존 e2e 회귀 0.
- 3단계: e2e(드롭 확인/열기/취소) + 보안 문서 반영 + 보고서.
