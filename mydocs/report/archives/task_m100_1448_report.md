# Task M100 #1448 최종 보고서

- 이슈: #1448 `rhwp-studio: 미저장 문서 자동 백업 및 복구 기능`
- 브랜치: `local/task_m100_1448`
- 기준 브랜치: `upstream/devel` `23dc197f`
- 최종 HEAD: PR 생성 직전 `git rev-parse --short HEAD`로 재확인
- 상태: PR 준비 완료

## 1. 해결 범위

사용자 리뷰로 접수된 “작성 중 컴퓨터가 꺼지면 저장 안 된 문서를 복구할 수 있는가” 요구에 맞춰
rhwp-studio의 미저장 문서 자동 백업과 복구 흐름을 추가했다.

- dirty 문서를 IndexedDB 기반 자동 백업 draft로 저장한다.
- IndexedDB를 사용할 수 없는 테스트/제한 환경에서는 메모리 저장소로 폴백한다.
- dirty 이벤트와 문서 변경 이벤트를 debounce해 복구용 HWP 스냅샷을 저장한다.
- 정상 저장, 새 문서/파일 교체, 변경 버림 시 현재 draft를 정리한다.
- 앱 시작 시 복구 후보가 있으면 `문서 복구` 대화상자로 안내한다.
- 사용자가 `복구`, `삭제`, `나중에` 중 하나를 선택할 수 있게 했다.
- 복구본은 원본 파일을 자동으로 덮어쓰지 않으며, 복구 후 dirty 상태를 유지한다.
- HWPX 출처 문서는 autosave 데이터가 HWP export 결과임을 명확히 하도록 `.hwp` 복구본으로 연다.
- 새 문서, 기존 HWP 문서, HWPX 출처 문서 복구 정책을 E2E로 검증했다.

## 2. 주요 구현 요약

### 2.1 자동 백업 저장소

- `rhwp-studio/src/recovery/autosave-store.ts`
  - `rhwpStudioAutosave` IndexedDB와 `drafts` store 추가.
  - draft id, 원본 파일명, 원본 포맷, 저장 시각, byte length, dirty reason, 복구 바이트를 저장.
  - 오래된 draft 정리와 데이터 복사로 외부 변경과 분리.

### 2.2 자동 백업 매니저

- `rhwp-studio/src/recovery/autosave-manager.ts`
  - dirty/document-mutated/document-changed 이벤트 기반 저장 예약.
  - 기본 debounce 2초, 최소 저장 간격 10초 정책.
  - clean 전환과 문서 교체 시 draft 삭제.

### 2.3 앱 연결과 저장 정책

- `rhwp-studio/src/main.ts`
  - 앱 초기화 시 `AutosaveManager` 연결.
  - 파일 로드와 새 문서 생성 시 autosave 문서 세션 시작.
  - 앱 시작 후 idle 상태에서 복구 후보를 조회하고 복구 UI 표시.
  - URL 파라미터 자동 로드가 있으면 복구 대화상자를 자동 표시하지 않음.
  - 복구 성공 시 기존 draft 삭제, 문서는 dirty 상태 유지.
- `rhwp-studio/src/command/commands/file.ts`
  - 다른 이름으로 저장 성공 시 dirty clean 전환을 보강해 draft가 정리되게 했다.

### 2.4 복구 UI와 표시 정책

- `rhwp-studio/src/recovery/recovery-ui.ts`
  - 복구 후보 목록 대화상자 추가.
  - `복구`, `삭제`, `나중에` 동작 추가.
  - 원본 파일 자동 덮어쓰기 금지 안내 추가.
- `rhwp-studio/src/recovery/recovery-format.ts`
  - 복구본 파일명 생성, 저장 시각/크기 표시 helper 추가.
  - HWPX 출처 draft는 `HWPX → HWP 복구본`으로 표시.

## 3. 테스트와 검증

### 3.1 단위 테스트

- `rhwp-studio/tests/autosave-store.test.ts`
  - IndexedDB 미사용 환경 메모리 폴백
  - draft 데이터 복사/분리
- `rhwp-studio/tests/autosave-manager.test.ts`
  - dirty 이벤트 후 draft 저장
  - clean 전환 시 draft 삭제
  - 새 문서 세션 시작 시 이전 draft 정리와 새 draft id 사용
- `rhwp-studio/tests/recovery-ui.test.ts`
  - 복구본 파일명 생성
  - 복구 후보 크기/시각 표시
  - HWPX 출처 draft가 HWP 복구본으로 열림 표시

### 3.2 E2E

- `rhwp-studio/e2e/autosave-recovery.test.mjs`
  - 새 문서 draft 복구
  - `samples/셀보호2.hwp` draft 복구
  - `samples/셀보호2.hwpx` 출처 draft를 HWP 복구본으로 복구
  - 복구 후 dirty 상태 유지
  - 복구 성공 후 원본 draft 삭제
  - 원본 파일 자동 덮어쓰기 금지 안내 확인

### 3.3 실행한 검증

- `cargo build --release`
  - 통과
- `cargo test --release --lib`
  - 통과: 1879 passed, 0 failed, 6 ignored
- `cargo test --profile release-test --tests`
  - 통과
- `cargo fmt --check`
  - 통과
- `cargo clippy --all-targets -- -D warnings`
  - 통과
- `cd rhwp-studio && npm test`
  - 통과: 84 passed
- `cd rhwp-studio && npx tsc --noEmit`
  - 통과
- `cd rhwp-studio && npm run build`
  - 통과
- `cd rhwp-studio && CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" VITE_URL=http://localhost:7700 node e2e/autosave-recovery.test.mjs --mode=headless`
  - 통과
- `git diff --check`
  - 통과

## 4. 수용 기준 판정

- dirty 문서 변경 후 일정 간격으로 복구용 스냅샷 저장: 완료
- 정상 저장 또는 사용자가 변경을 버린 경우 복구 스냅샷 정리: 완료
- 앱 시작 시 복구 가능한 문서 안내: 완료
- 복구 실행 시 문서 내용, 파일명 후보, dirty 상태 복원: 완료
- 자동 백업은 원본 파일을 자동 덮어쓰지 않음: 완료
- 저장 전 경고/수동 저장/파일 열기 기존 동작 회귀 방지: 완료
- 새 문서, 기존 HWP 문서, HWPX 출처 문서 정책 테스트: 완료

## 5. Git 상태

- `upstream/devel...HEAD`: PR 생성 직전 `git rev-list --left-right --count upstream/devel...HEAD`로 재확인
- 워크트리: PR 생성 직전 `git status --short --branch`로 재확인
- PR 생성 전 권장 원격 브랜치:
  - `task_m100_1448`

## 6. PR 생성 메모

PR 본문에는 `Closes #1448`를 포함해 merge 시 issue가 자동 close되도록 한다.
