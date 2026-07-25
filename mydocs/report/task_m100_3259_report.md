# 처리 결과 보고서 — Task M100 #3259

## 이슈와 결론

- 이슈: [#3259 — macOS 최근 문서 재열기 시 Finder 재선택 요구](https://github.com/edwardkim/rhwp/issues/3259)
- 결론: Finder/Explorer 드롭에서 제공하는 `FileSystemFileHandle`을 drop event의 같은 tick에
  확보하고, 사용자가 열기를 확인한 뒤에만 문서 로드·최근 문서 기록으로 전달하도록 구현했다.
  이전에는 드롭 `File`만 `loadFile()`에 전달해 recent entry가 항상 `handle: null`이었다.

## 변경

- `captureDroppedFileHandle()`이 선택된 `DataTransferItem`에 대해
  `getAsFileSystemHandle()` Promise를 동기적으로 시작한다.
- 미지원 API, Promise 거부, null, directory handle, 선택 파일 불일치는 기존 메타-only
  `null` fallback으로 정규화한다.
- drop 확인 대화상자 전에는 handle Promise만 확보하고 bytes를 읽거나 IndexedDB recent entry를
  기록하지 않는다. 확인 뒤 `loadFile(file, { fileHandle })`로 전달한다.
- `loadFile()`은 받은 handle을 기존의 단일 수렴점 `loadBytes()`에 넘긴다. 따라서
  `wasm.currentFileHandle`과 `addRecentDoc()`의 기존 권한·재열기 정책은 변하지 않는다.

## 검증

| 명령 | 결과 |
|---|---|
| `node --test tests/file-system-access.test.ts` | 19 passed |
| `npm test` | 568 passed |
| `wasm-pack build --target web --dev` | 성공 |
| `npm run build` | 성공 (`tsc`, Vite production build) |
| `git diff --check` | 통과 |

추가한 단위 테스트는 handle API가 `await` 전에 같은 tick에서 호출되는지, 미지원·오류·directory
fallback 및 다른 파일 item을 capture하지 않는지를 고정한다.

## macOS 실동작 확인

로컬 Chrome의 Studio 기동과 production build까지 확인했다. 자동화 세션은 Finder 창의 파일을 Chrome
웹 콘텐츠 영역으로 안정적으로 드롭하지 못했으나, 작업지시자가 실제 macOS Finder에서 아래 흐름을
직접 확인해 통과했다.

1. Finder에서 문서 A를 Studio로 드롭하고 열기 확인을 승인한다.
2. 문서 B를 연다.
3. 파일 → 최근 문서에서 A를 선택한다.
4. 파일 재선택 picker나 `핸들 없이 열려` toast 없이 A가 다시 열린다.

지원하지 않는 브라우저와 과거 handle 없는 recent entry는 의도대로 재선택 fallback을 유지한다.

## PR 준비 상태

#3257과 함께 단일 브랜치에서 준비한 [PR #3265](https://github.com/edwardkim/rhwp/pull/3265)에 포함했다.
오늘할일 문서는 최초 PR diff에만 추가했으며 PR 번호 발급 후에는 다시 수정하지 않는다.
