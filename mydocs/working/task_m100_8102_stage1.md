# #8102 셀 편집 대상 표현 정규화 — 1단계

Issue: #8102

## 분석과 계획

- 기준: 최신 `upstream/devel` = `def947c2149df5bc812d801e6e79be19a403b4d1`.
- 독립 브랜치: `codex/issue-8102-cell-target`. 미병합 PR을 전제로 하지 않는다.
- 이슈 작성자 `postmelee`, 댓글 0건. #8102 참조 PR 검색 0건, 본인 열린 PR 0건.
- 원인: `table:create`는 flat 위치만 만들고 `selectAllInCell`도 flat 조회를 허용하지만,
  `DeleteSelectionCommand`의 `cellPathJson`은 없는 경로를 `[]`로 직렬화한다.
- 독립 기대 계약: 이슈의 생성 직후/클릭 대조군 및 기존 `DocumentPosition`의 최외곽 flat,
  최내곽 cellPath 규약. 같은 유효한 셀은 진입 방식에 관계없이 같은 범위를 지운다.
- 공통 경로 정규화로 완전한 flat 위치만 depth 1로 해석한다. 중첩 경로·글상자는 보존한다.
  누락 좌표를 0으로 채우지 않으며 잘못된 선택 대상은 mutation 전에 거부한다.
- 수정 범위: `table.ts`, `command.ts`, `cursor.ts` 및 해당 회귀·작업 기록.
- 제외: 호스트 #594, #8104 오류 계약, core 삭제 알고리즘, 조판/렌더러/저장 형식.
  기존 ByPath 편집 API에 전달할 주소만 일치시킨다. 조판 속성이나 배치 규칙은 바꾸지 않는다.
- 검증: 수정 전 실패/후 통과, 생성·클릭·키보드·undo 진입, 다중 문단·중첩·인접 셀·
  글상자·undo/redo·invalid target, fresh dev WASM/Studio E2E, TypeScript/npm/build,
  PR 범위와 작업 트리 `git diff --check`.
- 사용자 실제 문서는 제공되지 않았다. 사용자가 같은 셀 진입·선택 삭제·undo를 직접 확인해야 한다.

## 결과

- `cellPathForEdit`가 완전한 flat 주소를 depth 1로 정규화하고 기존 경로를 복사한다.
  선택 조회, ByPath 직렬화와 문단별 경로가 이를 공유한다.
- 표 생성 결과에 경로를 제공하며, 중첩 전체 선택은 최외곽 flat 문단을 보존한다.
- 선택 삭제는 같은 셀/컨테이너·순서·오프셋·실제 문단 범위를 snapshot·mutation 전에 확인한다.
- 제품 경로 단위 검사: 수정 전 flat/빈 경로/직접 flat 삭제/불완전 좌표 거부가 실패했고,
  클릭·중첩·글상자 대조군은 통과했다. 수정 후 최신 검사 10건 통과, skip 0건.
- fresh web dev WASM 빌드 성공. Studio public과 pkg의 JS/WASM SHA-256 각각 일치.
- Chromium 브라우저: 수정 전 생성 직후·키보드·legacy flat·다중 문단 삭제 실패,
  클릭 재진입은 삭제·undo/redo 성공. `경로가 비어있습니다` 오류를 실제 관측했다.
- 수정 후보 브라우저: 생성 직후·실제 클릭·Tab/Shift+Tab·undo 진입·legacy flat·다중 문단
  6가지 모두 삭제/undo/redo/undo 뒤 재삭제와 인접 셀 보존 통과. 페이지 오류 0건.
  별도 브라우저 WasmBridge에서 중첩 셀/바깥 내용·옆 셀 보존, 글상자, invalid target 3건
  거부 및 HWP 바이트 무변경을 확인했다. 합성 HML은 검사 소스에 포함한다.
- 초기 브라우저 검사 도구의 Puppeteer 키 조합 표기·줌 조회 오류를 보정해 재실행했다.
  기존 UI는 셀 선택 삭제 undo 때 선택 하이라이트를 복원하지 않으므로, 현재 계약인 내용과
  셀 위치 복원을 검사한다. 선택 하이라이트 복원은 이번 변경의 주장에 포함하지 않는다.
- 전체 npm 첫 실행은 실패 0/2370 통과/2 skip이었다. Node WASM 부재가 원인이므로
  동일 Rust source의 fresh `pkg-node`를 빌드했다. 최종 commit에서 전체 검사를 다시 실행한다.
- TypeScript 예비 검사 통과. 최종 commit package 검사·관련 기존 E2E·전체 npm 재검증은 남아 있다.
- 로그: ignored `output/pr-review/issue-8102/logs/`, 브라우저 결과 JSON과 화면 캡처.
- 조판 규칙·속성·저장 정보는 변경하지 않는다. 기존 core ByPath 삭제의 대상 주소만 일치시키며,
  Native Skia/Print PDF Visual Sweep은 이 주소 정규화의 검증 대상이 아니다.
- 보조 E2E manifest 검사에서 base부터 있던 #7998 파일의 원장 누락이 발견됐다.
  #8102 행/배선은 추가했다. 다른 이슈 구현은 섞지 않고 base/head 오류를 별도 대조한다.
- 사용자 실제 문서·macOS 네이티브 Command 키는 미검증이다. Linux Chromium의 실제 Ctrl 키와
  아래 사용자 직접 확인 절차를 구분한다.
