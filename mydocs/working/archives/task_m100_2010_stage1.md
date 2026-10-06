# Task #2010 Stage 1

## 목표

Studio 대형 문서 편집 후 자동저장 때문에 UI가 멈추는 체감 문제를 줄인다.

## 원인 가설

현재 autosave는 문서 변경 이벤트마다 저장 예약을 걸고, 기본 2초 debounce 뒤 `wasm.exportHwp()`를 동기 실행한다.
100쪽 이상 문서에서는 한 글자만 수정해도 전체 HWP 직렬화가 예약되어 저장 시점에 긴 main thread block이 생길 수 있다.

## Stage 1 범위

- 사용자 환경설정에 autosave 설정을 추가한다.
- 환경 설정 대화상자에 `파일` 탭과 자동저장 옵션을 추가한다.
- `AutosaveManager`가 설정값을 반영해 저장 예약 간격을 조절하도록 한다.
- 자동저장 실행 중 상태창에 저장 중 메시지를 표시한다.
- 기존 복구 draft 저장/삭제 동작을 유지한다.
- `rhwp-studio` 단위 테스트를 보강한다.

## 비범위

- `exportHwp()` worker화 또는 chunked export.
- 실제 byte/paragraph 단위 저장 진행률.
- 브라우저 IndexedDB 저장소 구조 변경.

## 검증 계획

- `cd rhwp-studio && npm test`
- `cd rhwp-studio && npm run build`
- `git diff --check`

## 구현 결과

- `userSettings`에 복구용 자동저장 설정을 추가했다.
  - 복구용 자동저장: 기본 사용, 10분
  - 쉴 때 자동저장: 기본 사용, 10초
- 환경 설정 대화상자에 `파일` 탭을 추가하고 자동저장 간격을 조절할 수 있게 했다.
- `AutosaveManager`를 idle 저장 타이머와 복구 주기 타이머로 분리했다.
- 자동저장 시작/완료/실패 상태를 상태창에 표시하도록 연결했다.
- 기존 `debounceMs`, `minSaveIntervalMs` 테스트 호환성은 유지했다.

## 검증 결과

- `cd rhwp-studio && node --test tests/autosave-manager.test.ts tests/user-settings.test.ts`: 통과
- `cd rhwp-studio && npm test`: 통과, 171 tests
- `cd rhwp-studio && npm run build`: 통과
- `git diff --check`: 통과
