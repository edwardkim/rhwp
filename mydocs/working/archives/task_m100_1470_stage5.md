# Task M100 #1470 stage5 착수 문서

- 이슈: #1470 후속 검증 중 발견한 PR #1446 모양복사 회귀 후보
- 참조 PR: https://github.com/edwardkim/rhwp/pull/1446
- 기준 커밋: `c0e93033 task 1470: 표 캡션 번호 재정렬 보정`
- 작업 브랜치: `task_m100_1470`
- 작성일: 2026-06-22
- 상태: 구현 완료. 정적/단위 검증 및 사용자 직접 검증 완료.

## 1. 배경

작업지시자가 PR #1446에서 처리한 모양복사가 현재 정상 동작하지 않는다고 보고했다.
PR #1446은 #1443 작업으로 merge되었고, 표 셀 편집 보정 외에 다음 모양복사 항목을 포함했다.

- `Alt+C` 모양복사 단축키
- 툴바/메뉴의 `모양 복사` 기능 연결
- 본문 글자/문단 모양복사
- 표 셀 안 글자/문단 모양복사
- 표 셀 속성/테두리/배경 모양복사
- 복사한 모양의 일회성 적용

Stage 5는 Stage 4 캡션 보정과 섞지 않고, 모양복사 회귀 후보만 별도 스테이지로 다룬다.

## 2. 확인한 사실

### 2.1 PR #1446 문서 기준

`mydocs/working/task_m100_1443_stage5.md`:

- 본문/표 셀 내부 글자·문단 모양복사를 `Alt+C` 토글 방식으로 구현했다.
- 복사된 모양이 있으면 다음 `Alt+C`에서 선택 영역에 붙여넣는다.

`mydocs/working/task_m100_1443_stage6.md`:

- 표 셀 선택 범위에 셀 속성, 셀 테두리, 셀 배경을 복사/적용하도록 확장했다.

`mydocs/working/task_m100_1443_stage18.md`:

- `edit:format-paste` 커맨드 추가
- `InputHandler.performFormatPaste()` 추가
- `EditorContext.hasCopiedFormat` 추가
- 편집 메뉴와 우클릭 메뉴에 `모양 붙여넣기` 추가
- 기존 `Alt+C` 토글 동작은 유지하고, 새 `모양 붙여넣기` 메뉴는 붙여넣기만 수행하도록 분리

### 2.2 현재 코드 기준

현재 `rhwp-studio/src`에서 다음 심볼은 검색되지 않는다.

- `edit:format-paste`
- `performFormatPaste`
- `hasCopiedFormat`
- `모양 붙여넣기`

반면 다음 경로는 남아 있다.

- `edit:format-copy`
- `InputHandler.performFormatCopy()`
- `formatCopyState`
- `applyCopiedFormatToCurrentTarget()`
- 표 셀 선택 범위 `applyCopiedCellPropsToSelection()`

즉 현재 코드는 `Alt+C`/툴바의 `모양 복사` 토글 방식만 남고, PR #1446 Stage 18의 명시적 `모양 붙여넣기` 명령 경로가 사라진 상태다.

### 2.3 git 추적 결과

`git log -S` 기준:

- `3c897c48 task 1443: 모양 붙여넣기 메뉴 추가`
  - `edit:format-paste`, `performFormatPaste`, `hasCopiedFormat`, 메뉴/컨텍스트 메뉴 항목을 추가했다.
- `38ee2b05 task 1443: 일회성 모양복사 메뉴 정리`
  - 위 `format-paste` 경로를 제거했다.
  - 대신 툴바 `모양 복사 (Alt+C)` 버튼에 `data-cmd="edit:format-copy"`를 연결했다.

따라서 현재 회귀 후보 1순위는 `38ee2b05`에서 명시적 붙여넣기 경로를 정리하면서 실제 사용자 기대 동작까지 제거한 것이다.

### 2.4 현행 회귀 스크립트 결과

PR #1446 당시 사용한 스크립트를 현재 서버 `http://localhost:7700`에 재실행했다.

- `node /private/tmp/rhwp_1443_format_copy_check.mjs`
  - 통과
  - 본문 글자/문단 모양복사와 표 셀 안 글자/문단 모양복사는 `Alt+C` 토글 방식에서 동작한다.
- `node /private/tmp/rhwp_1443_cell_shape_format_copy_check.mjs`
  - 통과
  - 표 셀 선택 범위에 셀 속성/테두리/배경을 적용하는 경로는 동작한다.

따라서 현재 재현 범위는 "핵심 적용 엔진이 전부 깨짐"보다는 "PR #1446 Stage 18에서 제공했던 명시적 `모양 붙여넣기` UI/API 경로가 사라짐"으로 보는 것이 타당하다.

## 3. 후보정 방향

우선 후보:

1. `EditorContext.hasCopiedFormat`을 복구한다.
2. `InputHandler.hasCopiedFormat()`을 추가한다.
3. `InputHandler.performFormatPaste()`를 추가한다.
   - 복사 상태가 없으면 조용히 무시한다.
   - 복사 상태가 있으면 `applyCopiedFormatToCurrentTarget()`만 호출한다.
   - 붙여넣기 실패 시 현재 위치 모양을 새로 복사하지 않는다.
4. `edit:format-paste` 커맨드를 복구한다.
   - `ctx.hasDocument && ctx.hasCopiedFormat && !ctx.isFormMode && (ctx.hasSelection || ctx.inCellSelectionMode)` 조건으로 활성화한다.
5. 편집 메뉴와 기본/표 셀 컨텍스트 메뉴에 `모양 붙여넣기`를 복구한다.
6. 기존 `Alt+C` 일회성 토글 동작과 툴바 `모양 복사` 버튼은 유지한다.

대안:

- `모양 붙여넣기` 메뉴를 되살리지 않고 `Alt+C` 토글 방식만 공식 동작으로 유지한다.
- 이 경우 PR #1446 Stage 18 문서와 현재 구현이 계속 불일치하고, 메뉴에서 붙여넣기만 수행할 수 없어 사용자가 "정상 동작 안 함"으로 인식할 가능성이 크다.

## 4. 테스트 계획

Studio/브라우저 검증:

- 기존 회귀 스크립트 재실행
  - `node /private/tmp/rhwp_1443_format_copy_check.mjs`
  - `node /private/tmp/rhwp_1443_cell_shape_format_copy_check.mjs`
- 신규 검증 스크립트 추가 또는 임시 스크립트 작성
  - `edit:format-paste` 커맨드가 command registry에 존재하는지 확인
  - 모양 복사 전 `모양 붙여넣기` 비활성 확인
  - 모양 복사 후 선택 영역에서 `edit:format-paste`가 붙여넣기만 수행하는지 확인
  - 붙여넣기 실패 시 현재 커서 모양을 새로 복사하지 않는지 확인
  - 표 셀 선택 범위에서도 `edit:format-paste`가 셀 속성을 적용하는지 확인

정적 검증:

- `cd rhwp-studio && npx tsc --noEmit`
- `cd rhwp-studio && npm test`
- `git diff --check`

Rust/WASM:

- 이번 후보정이 Studio TypeScript/UI만 건드리면 Rust cargo 검증은 기계적으로 돌리지 않는다.
- WASM 경계 동작을 바꾸는 경우에만 `wasm-pack build --target web --out-dir pkg`를 추가한다.

## 5. 승인 게이트

작업지시자 승인 후 `rhwp-studio/src`와 `rhwp-studio/index.html` 소스 수정을 진행했다.
Stage 4 변경과 Stage 5 변경은 커밋을 분리했다.

## 6. 구현 결과

- `EditorContext.hasCopiedFormat`을 복구했다.
- `main.ts`의 `getContext()`가 `InputHandler.hasCopiedFormat()`을 반영하도록 했다.
- `InputHandler.hasCopiedFormat()`을 추가했다.
- `InputHandler.performFormatPaste()`를 추가했다.
  - 붙여넣기 전용 경로라서 실패 시 현재 위치 모양을 새로 복사하지 않는다.
- `edit:format-paste` 커맨드를 복구했다.
  - 복사 상태가 있고, 일반 편집 모드이며, 텍스트 선택 또는 셀 선택 모드일 때만 활성화된다.
- 편집 메뉴에 `모양 붙여넣기` 항목을 복구했다.
- 기본/표 셀 컨텍스트 메뉴에 `모양 붙여넣기` 항목을 복구했다.
- 기존 `Alt+C` 일회성 토글 동작과 툴바 `모양 복사` 버튼은 유지했다.
- `format-paste-availability.ts`와 `format-paste-command.test.ts`를 추가해 다음 회귀를 `npm test`로 고정했다.
  - 복사 상태/적용 대상/form mode에 따른 `모양 붙여넣기` 활성 조건
  - `edit:format-paste` 커맨드가 붙여넣기 전용 경로에 연결되는지
  - 메뉴/컨텍스트 메뉴/컨텍스트 상태 경로가 유지되는지

## 7. 검증 결과

- `cd rhwp-studio && npx tsc --noEmit`
  - 통과
- `cd rhwp-studio && npm test`
  - 통과: 110 passed
- 사용자 직접 검증
  - 완료

비고:

- IAB 추가 자동 검증을 시도했으나, 작업지시자가 직접 검증 완료를 알려 중단했다.
- 이번 Stage 5는 Studio TypeScript/UI 변경이므로 Rust cargo/wasm-pack 검증은 추가하지 않는다.
