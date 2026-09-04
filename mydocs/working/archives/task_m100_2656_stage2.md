---
kind: working-note
status: complete
issue: 2656
stage: 2
---

# Task #2656 Stage 2: 다운로드 탭 안전성 사후 보강

## 배경

작업지시자가 수정 빌드와 스토어 v0.2.8의 정상 수명주기 테스트가 모두 통과함을 확인했다.
따라서 원 제보는 재현 완료 버그가 아니라 저장 실패·초기화 경쟁에 대한 방어적 개선으로
재분류했다. PR 작성 전에 과거 #1498/#1515의 다운로드 탭 다발 회귀와 이번 설정 loader 변경의
접점을 별도로 감사했다.

과거 탭 다발은 Service Worker 재시작 뒤 과거 다운로드 항목을 새 항목으로 오인한 것이 원인이었다.
현재의 신선도 가드, `storage.session` 상태, `onChanged` 단독 항목 무시 계약은 이번 변경에서
수정하지 않았고 관련 회귀 테스트도 유지된다.

## 추가로 재현한 위험

### 동일 download id 동시 이벤트

설정 storage read를 20ms 지연한 상태에서 동일 id의 `filename`과 `finalUrl` 변경 이벤트를
동시에 전달했다. 두 handler가 모두 미처리 상태를 읽은 뒤 각각 탭을 열어, 보강 전 테스트는
기대 1개 대비 실제 2개로 실패했다.

이 경쟁 구간은 기존 sync read에도 있었지만, 이번 settings loader의 sync/local read와 최초
snapshot 보강이 await 구간을 늘릴 수 있어 PR 전에 차단해야 하는 위험으로 판정했다.

### handled/terminal 상태 덮어쓰기

첫 HWP 판정 이벤트가 `state=complete`를 함께 포함하면 처리 함수가 기록한 `handledAt`을 handler
시작 시점의 오래된 state로 만든 terminal 기록이 덮었다. 그 뒤 `finalUrl` 이벤트를 전달하자 같은
download id가 다시 열려 보강 전 테스트에서 탭 2개를 재현했다.

### sync 장애 중 local true 자동 허용

sync read 실패와 local snapshot `autoOpen=true`를 함께 주입하면 기존 보강안은 자동 탭을 열었다.
이는 UI 복구와 사용자 비동의 자동 동작을 같은 정책으로 취급한 결과다.

## 보강

- Chrome download adapter에 Service Worker 실행 중인 download id별 in-flight Promise 선점을
  추가했다. 후발 handler는 동일 Promise의 최신 handled 결과를 이어받는다.
- 설정 storage await 전에 id를 선점하고 완료 후 해제하며, 처리 직전에 session 최신 상태를 다시
  읽어 먼저 완료된 handler의 handled marker를 존중한다.
- `processDownloadCandidate`가 최신 handled state를 반환하고 terminal 기록이 이를 이어받게 했다.
- 자동 동작 전용 loader를 추가해 sync read 실패 시 local 값과 무관하게 `autoOpen=false`로
  fail-closed 처리했다.
- 옵션 UI는 local snapshot을 복구 표시할 수 있어 데이터 복구 기능을 유지했다.
- sync를 권위 저장소로 먼저 기록하고 local snapshot은 best-effort 백업으로 갱신한다. local만
  실패해도 성공한 sync 사용자 설정을 잃지 않는다.
- 확장 개발 가이드의 오래된 Chrome `onDeterminingFilename` 설명을 현재 양 브라우저의
  `onCreated`/`onChanged` 상태 머신 구조로 정정했다.

## 검증

- 보강 전 red test
  - 동시 `filename`/`finalUrl`: 탭 2개로 실패
  - 첫 complete 판정 후 추가 이벤트: 탭 2개로 실패
  - sync 실패 + local true: 원치 않는 탭 1개로 실패
- 보강 후 Chrome 확장 테스트: 37 passed, 0 failed
- 공통 다운로드 판정·상태 머신 테스트: 40 passed, 0 failed
- 변경 JavaScript `node --check`: 통과
- `npm --prefix rhwp-chrome run build`: 통과, 167 modules transformed
- Chrome/Firefox/Safari dist 계약: 3 passed, 0 failed
- source/dist settings-store, download-interceptor, options byte 비교: 통과
- `git diff --check`: 통과
- `onDeterminingFilename.addListener` source/dist 검색 결과 없음

## 잔여 위험과 수동 인수

- in-flight Promise는 동일 Service Worker 안의 동시 처리만 담당한다. Service Worker 재시작 경계는 기존
  `storage.session` handled state가 담당한다.
- sync read 장애 중에는 기본값이 true인 사용자도 자동 열기가 한 번 누락될 수 있다. 원치 않는 탭
  생성보다 안전한 의도적 fail-closed다.
- local backup만 실패하면 다음 정상 설정 load 때 snapshot 보강을 다시 시도한다.
- 작업지시자가 별도 Chrome 프로필에서 1~7 정상 수명주기 테스트를 통과했다.
- 보강 후 최신 빌드에서도 자동 열기를 켠 상태로 과거 HWP 다운로드 기록을 남긴 뒤 확장
  Reload/Chrome 재실행 시 탭 0개, 새 HWP 다운로드 시 탭 정확히 1개임을 확인했다.
- Chrome 완전 종료 뒤 프로필 선택 화면이 나타나기까지의 지연은 확장을 제거한 상태에서도 동일하게
  재현됐다. 특정 프로필과 그 확장이 활성화되기 전 단계의 지연이므로 이번 변경과 무관한 현상으로
  판정했다.

## 판정

과거 다운로드 기록 전체를 다시 여는 #1498 경로는 유지된 가드와 회귀 테스트로 차단되어 있다.
감사에서 발견한 동일 id 동시 처리와 terminal state 드리프트도 재현 테스트와 함께 보강했다.
자동 검증과 최신 빌드 수동 인수를 모두 통과했다. PR은 “재현되지 않은 제보에 대한 방어 로직과
관측 가능한 오류 처리”로 설명할 수 있으며, PR 준비가 완료됐다.
