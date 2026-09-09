---
kind: working
status: manual-verification-pending
issue: 3604
stage: 8
last_verified: 2026-08-01
---

# #3604 Stage 8: 암호 문서 드롭 열기 안정화

## 관측

- 암호 HWPX는 파일 메뉴 열기에서 암호 dialog를 거쳐 정상 로드된다.
- 같은 파일을 Finder에서 Studio로 드롭하면 Chrome renderer가 `RESULT_CODE_KILLED_BAD_MESSAGE`로
  종료된다.
- 두 경로의 document bytes와 `loadDocumentForOpen()`은 같고, 드롭 경로만 macOS Chromium
  `DataTransferItem.getAsFileSystemHandle()`을 같은 event tick에 호출해 save handle을 보관한다.

## 구현 계획

1. 드롭 문서 열기에서 저장용 File System Access handle capture를 제거하고 `File` bytes만
   `loadFile()`에 전달한다.
2. 파일 메뉴 열기와 동일하게 암호 document detection 뒤 password dialog로 전환되는 source
   contract를 추가한다.
3. 일반 문서 drag/drop 및 파일 메뉴의 handle 저장 경로에는 영향을 주지 않는 범위를 확인한다.
4. Studio production build와 실제 Finder 드롭 수동 검증으로 확인한다.

## 안전성 경계

- 드롭으로 연 문서는 이전과 같이 사용자의 열기 확인 뒤에만 `arrayBuffer()`를 읽는다.
- 드롭 문서는 save handle을 보관하지 않으므로, 이후 Ctrl+S는 save-as 경로를 선택한다.
  이는 Chromium renderer 안정성을 우선한 의도된 동작이다.
- 암호와 File System Access 권한 상태는 local storage, recent metadata, log에 추가하지 않는다.

## 테스트 결과

| 검증 | 결과 |
| --- | --- |
| `npx --yes tsx --test tests/file-system-access.test.ts tests/hwp-password-open.test.ts tests/hwp-password-save.test.ts` | 통과: 26 tests, 0 failures |
| `npx tsc --noEmit` | 통과 |
| `npm run build` | 통과: Studio production bundle 생성 |
| `git diff --check` | 통과 |
| source scan (`src/`) | 통과: 드롭 열기 경로에 `captureDroppedFileHandle` 및 `getAsFileSystemHandle` 호출 없음 |

이 검증 호스트에는 headless Chrome 실행 파일이 없고, 기존 Windows Chrome CDP endpoint도 연결할 수
없어 실제 OS Finder drag/drop은 자동화하지 못했다. 합성 `DataTransfer`는 native File System Access
IPC를 만들지 않으므로 이 renderer 종료 문제의 종단 검증으로는 충분하지 않다.

사용자 수동 확인 항목:

1. 암호 HWPX를 Finder에서 Studio 창으로 드롭한다.
2. `열기` 확인 뒤 암호 입력 dialog가 보이는지 확인한다.
3. 올바른 암호 입력 뒤 문서와 페이지 canvas가 표시되고 Chrome renderer가 종료되지 않는지 확인한다.
4. 드롭으로 연 문서의 Ctrl+S가 기존 원본을 덮어쓰지 않고 save-as 흐름으로 전환되는지 확인한다.

수동 확인이 통과하면 이 stage의 상태를 `completed`로 갱신한다.
