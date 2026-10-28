# Task M100 #1448 Stage 2: 복구 안내 UI와 복구 실행

- 이슈: #1448 `rhwp-studio: 미저장 문서 자동 백업 및 복구 기능`
- 기준 브랜치: `upstream/devel`
- 작업 브랜치: `local/task_m100_1448`
- 작성일: 2026-06-21

## 1. 목표

Stage 1에서 저장한 자동 백업 draft를 앱 시작 시 조회하고, 사용자가 복구/삭제/나중에 결정을 선택할 수 있게 한다.

## 2. 구현 범위

- 자동 백업 draft 목록을 표시하는 복구 대화상자 추가
- 복구 후보 선택 후 문서 로드
- 복구본 로드 후 dirty 상태 유지
- 복구 성공 시 기존 draft 삭제
- 복구 거절 시 draft 삭제
- 나중에 결정 선택 시 draft 보존
- URL 파라미터 자동 로드가 있는 경우 복구 대화상자 자동 표시를 건너뛰기

## 3. 제외 범위

- 복구 후보 관리 전용 화면
- 여러 후보 개별 삭제 UI 고도화
- HWPX 출처 전용 상세 안내
- 자동 백업 주기 설정 UI

## 4. 진행 기록

- 2026-06-21: Stage 2 착수.
- 2026-06-21: 복구 후보 대화상자와 복구 실행 경로 구현 완료.

## 5. 구현 내용

### 5.1 복구 후보 대화상자

- `rhwp-studio/src/recovery/recovery-ui.ts` 추가
- 앱 시작 시 복구 후보가 있으면 `문서 복구` 대화상자를 표시한다.
- 사용자는 다음 중 하나를 선택할 수 있다.
  - `복구`: 선택한 draft를 복구본 문서로 연다.
  - `삭제`: 모든 복구 후보를 삭제한다.
  - `나중에`: 복구 후보를 보존하고 대화상자를 닫는다.
- 대화상자 문구에 원본 파일을 자동으로 덮어쓰지 않는다고 명시했다.

### 5.2 복구 표시 helper

- `rhwp-studio/src/recovery/recovery-format.ts` 추가
- 복구본 파일명은 원본을 덮어쓰지 않도록 `복구본` 접미사를 붙인다.
  - 예: `sample.hwp` → `sample 복구본.hwp`
- 후보 표시용 저장 시각, 크기, 원본 포맷 표시 함수를 분리했다.

### 5.3 앱 시작 연결

- `rhwp-studio/src/main.ts`
  - 초기화 후 자동 백업 draft를 조회한다.
  - URL 파라미터 자동 로드(`?url=`)가 있는 경우 복구 대화상자 자동 표시를 건너뛴다.
  - 이미 문서가 로드됐거나 dirty 상태면 복구 대화상자를 띄우지 않는다.
  - 복구 선택 시 draft 데이터를 `loadBytes`로 열고, 원본 draft를 삭제한다.
  - 복구본 로드 후 `documentState.markDirty('autosave-recovered')`를 호출해 저장 전 상태로 유지한다.
  - 복구 데이터 로드 실패는 기존 `showLoadError` 경로로 사용자에게 표시한다.

## 6. 테스트와 시각 확인

- `cd rhwp-studio && npm test`
  - 83개 테스트 통과
- `cd rhwp-studio && npx tsc --noEmit`
  - 통과
- `cd rhwp-studio && npm run build`
  - 통과
- `git diff --check`
  - 통과
- Headless Chrome / `http://localhost:7700/` 시각 확인
  - IndexedDB에 테스트 draft를 주입한 뒤 새로고침
  - `문서 복구` 대화상자 표시 확인
  - `복구`, `삭제`, `나중에` 버튼 표시 확인

## 7. 남은 작업

- 실제 HWP/HWPX 샘플 기반 복구 로드 수동 검증
- HWPX 출처 문서의 복구 안내 문구와 저장 정책 보강
- 자동 백업 저장 주기/최대 draft 수 사용자 설정 여부 검토
