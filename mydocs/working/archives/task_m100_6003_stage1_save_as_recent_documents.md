# #6003 다른 이름 저장 문서명과 최근 문서 동기화

- 이슈: [#6003](https://github.com/edwardkim/rhwp/issues/6003)
- 브랜치: `fix/6003-save-as-recent-documents-20260824`
- 기준: `upstream/devel@13ae331db2b76c6fc2e1841df87aead8abcd7e5b`

## 배경

Studio에서 다른 이름으로 저장을 완료해도 하단 상태 표시줄은 불러올 때 만든 이전 파일명을
계속 표시한다. 저장한 새 문서는 최근 문서에도 즉시 나타나지 않는다. 최근 문서 저장소는 8개로
잘라 저장하므로 사용자가 더 오래된 문서를 확인할 방법도 없다.

## 기존 경로 분석

- `saveAsFormat`과 `completeHandleSave`는 `wasm.fileName` 및 file handle을 갱신하지만, 상태 표시줄은
  `main.ts`의 문서 초기화 경로에서만 만든다. 저장 완료가 그 상태를 다시 계산할 수 있는 연결점이 없다.
- 최근 문서는 문서를 열 때만 `addRecentDoc`에 기록한다. File System Access 저장과 다운로드 fallback
  저장에는 새 이름을 기록하는 경로가 없다.
- `recent-store.ts`의 영속 상한은 8개이고, 최근 문서 하위 메뉴도 전체 목록을 한 번에 표시한다.
- `saveAsFormat`은 암호 문자열을 잠시 보유하는 경로다. 기존 보안 계약은 이 함수 안에서
  `localStorage`, `sessionStorage`, `console.*`, 암호를 파일명으로 전달하는 코드를 금지한다.
  저장 결과 기록 실패도 이 경로에서 console로 보고하면 안 된다.

## 구현 범위

- 저장 성공 후 현재 파일명 기준으로 상태 표시줄을 다시 만든다.
- File System Access 저장은 새 handle과 저장 형식을, 다운로드 fallback은 handle 없는 메타데이터를
  최근 문서에 기록한다. 취소와 실패는 기존처럼 상태를 변경하지 않는다.
- 최근 문서 영속 상한은 20개로 늘리되, 메뉴는 기본 8개만 표시하고 `최근 문서 더보기`를 눌렀을 때
  최대 20개를 표시한다.
- 저장 경로에서 최근 문서 기록의 비핵심 실패는 암호 취급 함수 안에서 console로 출력하지 않고
  무시 가능한 비동기 실패로 처리한다.

## 범위 밖

- 기존 최근 문서의 정렬 방식과 중복 제거 규칙 변경
- 현재 문서 저장의 파일 선택 UX 변경
- 최근 문서의 별도 관리 화면 추가

## 완료 기준

1. 다른 이름 저장 뒤 상태 표시줄이 새 파일명과 현재 페이지 수를 표시한다.
2. 새 이름은 File System Access 저장과 다운로드 fallback 모두에서 최근 문서에 남는다.
3. 저장소는 최근 문서 20개를 보존하고, 메뉴는 8개와 명시적 더보기로 단계 표시한다.
4. 암호 저장 경로가 암호 문자열을 영속화·로그·파일명 경로로 전달하지 않는 기존 계약을 통과한다.
5. `npm --prefix rhwp-studio run build`, `npm --prefix rhwp-studio test`, 실제 브라우저 동작 검증을
   완료하고, push 전 `cargo fmt --all` 및 `cargo fmt --all -- --check`를 통과한다.
