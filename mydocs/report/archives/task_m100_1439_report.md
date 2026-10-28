# Task M100 #1439 최종 보고서 — 드래그&드롭 로컬 파일 로딩 보안 게이트

- 이슈: #1439 "드래그&드롭 로컬 파일 로딩을 기본 동작에서 제외 + 드롭 시 모달 확인 대화상자 (보안 UX)"
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1439`
- 작성일: 2026-06-19

## 1. 개요

rhwp-studio 가 HWP/HWPX·이미지를 `#scroll-container` 에 드롭하면 확인·안내 없이 즉시
로딩하던 것을, **모달 확인 대화상자(opt-in)** 로 사용자가 명시적으로 [열기]를 눌러
동의한 경우에만 로딩하도록 보안 게이트를 추가했다. 크롬 확장/웹 양쪽에서 동작한다.

## 2. 변경

### 신규 `rhwp-studio/src/ui/drop-confirm-dialog.ts`

`unsaved-changes-dialog.ts` 동형 (`ModalDialog` 상속 + `showAsync(): Promise<boolean>`).
"드래그한 로컬 파일을 엽니다" + 파일명 + 고지, [열기]/[취소]. 미동의(취소/×/Escape/밖
클릭) → `false` → 미로딩. `chrome` API 의존 없는 순수 DOM 모달.

### `rhwp-studio/src/main.ts` drop 핸들러

지원 형식(HWP/HWPX·이미지) 판별 후 `loadFile`/이미지 배치 **전에**
`showDropConfirmDialog(file.name)` 확인. `!confirmed` 시 early return.
순서: **드롭 보안 확인 → (HWP/HWPX) loadFile 내부 unsaved 가드**.

## 3. 크롬 확장 모드 동작 보장

- 드롭·대화상자 모두 `main.ts` 같은 페이지 컨텍스트. 확장도 `/rhwp/` 를 standalone 탭
  (manifest display=standalone + file_handlers)으로 열어 popup 제약 없이 모달 정상 렌더.
- `chrome` API 의존 없는 단일 코드 경로 — 빌드 산출물에 게이트 포함(확장/PWA 공통).
- 드롭은 `dataTransfer.files` 라 file:// 권한(#1131)과 무관.

## 4. 검증

- `npx tsc --noEmit`: exit 0.
- `npm run build` (확장/PWA): 성공, 산출물 `dist/assets/index-*.js` 에 게이트 문구 포함.
- e2e `e2e/drop-confirm.test.mjs` (host CDP localhost:19222) **전부 PASS**:
  - 드롭 후 확인 대화상자 표시(즉시 로딩 안 됨).
  - [취소] 후 문서 로드 상태 불변(미로딩).
  - 재드롭 시 대화상자 표시 + [열기] 후 대화상자 닫힘(로딩 분기 진입).
- 기존 e2e `unsaved-changes-guard.test.mjs` 전부 PASS — 회귀 0.
  - e2e 인프라 의존성(pixelmatch/pngjs) 미설치 정정(`npm install`).

## 5. 문서

- 보안 가이드 `mydocs/manual/browser_extension_dev_guide.md` "3. 보안" 에 드롭 게이트
  규칙 추가 (opt-in 패턴, 확장/웹 공통, unsaved 순서).

## 6. 산출물

- 수행계획서: `mydocs/plans/task_m100_1439.md`
- 구현계획서: `mydocs/plans/task_m100_1439_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_1439_stage{1,2}.md`
- 최종 보고서: 본 문서
- 코드: `rhwp-studio/src/ui/drop-confirm-dialog.ts`, `rhwp-studio/src/main.ts`
- e2e: `rhwp-studio/e2e/drop-confirm.test.mjs`
- 문서: `mydocs/manual/browser_extension_dev_guide.md`
