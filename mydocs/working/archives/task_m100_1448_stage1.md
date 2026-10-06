# Task M100 #1448 Stage 1: 자동 백업 기반 구축

- 이슈: #1448 `rhwp-studio: 미저장 문서 자동 백업 및 복구 기능`
- 기준 브랜치: `upstream/devel`
- 작업 브랜치: `local/task_m100_1448`
- 작성일: 2026-06-21

## 1. 목표

복구 UI를 만들기 전에, dirty 문서를 로컬 복구 스냅샷으로 저장하고 정상 저장/폐기 시 정리할 수 있는 기반을
구축한다.

## 2. 구현 범위

- 자동 백업 전용 IndexedDB 저장소 추가
- IndexedDB 미사용 환경용 메모리 폴백 추가
- dirty 이벤트 기반 debounce 자동 백업 매니저 추가
- 문서 로드/새 문서 초기화 시 autosave 문서 세션 갱신
- 정상 저장 또는 `저장 안 함` 선택 시 현재 draft 정리
- 단위 테스트 추가

## 3. 제외 범위

- 앱 시작 시 복구 안내 UI
- 복구 후보 목록과 복구 실행 UX
- 원본 파일 자동 덮어쓰기
- 파일 핸들 저장
- HWPX 출처 전용 상세 안내

## 4. 진행 기록

- 2026-06-21: Stage 1 착수.
- 2026-06-21: 자동 백업 저장소와 매니저 구현 완료.

## 5. 구현 내용

### 5.1 자동 백업 저장소

- `rhwp-studio/src/recovery/autosave-store.ts` 추가
- 자동 백업 전용 IndexedDB DB 이름을 `rhwpStudioAutosave`로 분리했다.
- `drafts` store에 다음 메타데이터와 HWP 바이트를 저장한다.
  - draft id
  - 파일명
  - 원본 포맷
  - 저장 시각
  - 바이트 길이
  - dirty reason
  - 복구용 데이터
- IndexedDB를 사용할 수 없는 테스트/제한 환경에서는 메모리 저장소로 폴백한다.
- 최대 draft 수는 12개로 제한하고 오래된 draft를 정리한다.

### 5.2 자동 백업 매니저

- `rhwp-studio/src/recovery/autosave-manager.ts` 추가
- `document-mutated`, `document-changed` 이벤트를 감지해 dirty 상태가 이미 true인 문서의 후속 변경도 저장 예약한다.
- `document-dirty-changed`가 clean으로 바뀌면 현재 문서 draft를 삭제한다.
- 기본 정책은 debounce 2초, 최소 저장 간격 10초다.
- 저장 중 추가 변경이 발생하면 pending reason을 보존해 다음 저장을 다시 예약한다.

### 5.3 앱 연결

- `rhwp-studio/src/main.ts`
  - `AutosaveManager`를 생성하고 event bus에 연결했다.
  - 파일 로드와 새 문서 생성 성공 직후 새 autosave 문서 세션을 시작한다.
  - 새 문서/파일로 교체가 성공한 경우 이전 draft를 정리한다.
  - 개발 모드에서 `window.__autosaveManager`를 노출했다.
- `rhwp-studio/src/command/commands/file.ts`
  - `다른 이름으로 저장` 성공 시 `documentState.markClean('save-as')`를 호출하도록 보강했다.
  - 이 clean 전환으로 autosave draft도 함께 정리된다.
- `rhwp-studio/tsconfig.json`
  - Node 네이티브 TS 테스트와 Vite/TypeScript를 함께 만족하도록 `allowImportingTsExtensions`를 활성화했다.

## 6. 테스트

- `cd rhwp-studio && npm test`
  - 80개 테스트 통과
- `cd rhwp-studio && npx tsc --noEmit`
  - 통과
- `cd rhwp-studio && npm run build`
  - 통과
- `git diff --check`
  - 통과

## 7. 남은 작업

- 앱 시작 시 복구 후보를 조회하고 사용자에게 안내하는 UI는 Stage 2에서 진행한다.
- 복구본 로드 후 파일명/dirty 상태 정책은 Stage 2에서 확정한다.
- HWPX 출처 문서의 복구 안내 문구는 Stage 3에서 정리한다.
