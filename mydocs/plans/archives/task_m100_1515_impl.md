# Task M100 #1515 구현계획서 — 공통 다운로드 상태 머신과 Chrome adapter

- 이슈: #1515
- 브랜치: `codex/1515-download-state-machine`
- 작성일: 2026-06-24
- 수행계획서: `mydocs/plans/task_m100_1515.md`

## 구현 개요

#1513의 긴급 완화 로직을 단순히 확장하지 않고, 다운로드 관찰자 생명주기를 공통 상태 머신으로
분리한다. Chrome은 이 상태 머신을 먼저 소비하는 adapter가 된다. Firefox는 #1516에서 같은
상태 머신을 재사용한다.

핵심 원칙:

- 브라우저 API 의존은 adapter에만 둔다.
- 상태 전이와 중복 방지 판단은 `rhwp-shared` 순수 모듈에서 테스트한다.
- `onDeterminingFilename`은 계속 사용하지 않는다.
- service worker 재시작을 견디도록 임시 상태는 `chrome.storage.session`에 둔다.

## 1단계 — 공통 상태 머신 설계와 단위 테스트

대상:

- `rhwp-shared/sw/download-observer-state.js`
- `rhwp-shared/sw/download-observer-state.test.js`

구현:

1. 다운로드 상태 구조를 정의한다.
   - `id`
   - `firstSeenAt`
   - `itemStartTime`
   - `itemEndTime`
   - `handledAt`
   - `terminalAt`
   - `lastReason`
2. 시간 파싱 유틸을 상태 머신 내부 또는 별도 순수 함수로 둔다.
3. `onCreated` 항목을 평가하는 함수를 만든다.
   - 과거 완료 항목이면 `ignore`.
   - 새 다운로드 후보이면 `track`.
   - 즉시 HWP 판정이 가능한 경우 adapter가 `open` 판단을 수행할 수 있도록 후보 item을 유지한다.
4. `onChanged` + 재조회 항목을 평가하는 함수를 만든다.
   - 저장 상태가 없고 과거 항목으로 보이면 `ignore`.
   - 저장 상태가 있고 아직 처리 전이면 재판정 후보로 반환한다.
5. `markHandled`, `markTerminal`, `expireOldStates` 계열 함수를 둔다.

테스트:

- 과거 완료 항목 `onCreated`는 무시.
- 새 항목 `onCreated`는 추적.
- 저장 상태가 없는 과거 `onChanged`는 무시.
- 저장 상태가 있는 새 다운로드는 filename 확정 후 처리 후보.
- 이미 handled인 download id는 다시 open 후보가 되지 않음.
- TTL 지난 상태는 정리.

완료 기준:

- `node --test rhwp-shared/sw/download-observer-state.test.js` 통과.

## 2단계 — Chrome storage/downloads adapter 적용

대상:

- `rhwp-chrome/sw/download-interceptor.js`
- 필요 시 `rhwp-chrome/sw/download-state-storage.js`

구현:

1. 현재 파일의 메모리 `seen`, `handled`, `workerStartedAt`, `isFreshDownloadItem()` 의존을 제거하거나
   공통 상태 머신 호출로 대체한다.
2. `chrome.storage.session` 기반 상태 저장 helper를 만든다.
   - key prefix 예: `rhwpDownloadState:${id}`
   - 저장/조회/삭제/TTL 정리 함수 제공
   - `storage.session` 부재 시 테스트 가능한 메모리 fallback은 adapter 내부에 제한적으로 둔다.
3. `onCreated` 흐름:
   - 상태 머신으로 새 항목 여부 평가
   - 추적 상태 저장
   - `shouldInterceptDownload(item)` true이고 `autoOpen` true이면 한 번만 open
4. `onChanged` 흐름:
   - 상태 조회
   - 필요한 경우 `downloads.search({ id })`
   - 상태 머신 재판정
   - `shouldInterceptDownload(item)` true이고 아직 handled가 아니면 open
5. terminal delta에서 `markTerminal` 후 TTL 정리 예약.
6. `file://` HWP의 `cancel` / `erase` 유지.

완료 기준:

- Chrome 다운로드 감시기 mock 테스트 통과.
- `node --check rhwp-chrome/sw/download-interceptor.js` 통과.

## 3단계 — Chrome 회귀 테스트 강화

대상:

- `rhwp-chrome/sw/download-interceptor.test.mjs`

추가/수정 테스트:

- 기존 10개 케이스 유지.
- `onCreated` 후 모듈 재로드, 이후 `onChanged` filename 확정 시 새 HWP가 정확히 1회 열림.
- 동일 download id에 `onCreated`, filename 변경, finalUrl 변경, complete 이벤트가 모두 와도 탭 1개.
- `storage.session`에 handled 상태가 있으면 worker 재시작 후 중복 open 없음.
- `state: complete`와 과거 `endTime`이 있는 `onCreated` 항목은 무시.
- `startTime`이 없는 `onChanged` 단독 항목은 저장 상태 없으면 보수적으로 무시.
- 비-HWP blob PDF는 탭/cancel/erase/search 부작용 없음.

완료 기준:

- `node --test rhwp-chrome/sw/download-interceptor.test.mjs` 통과.
- #1471 `onDeterminingFilename` 미등록 검증 유지.

## 4단계 — 빌드와 수동 검증 준비

검증:

- `node --test rhwp-shared/sw/download-observer-state.test.js`
- `node --test rhwp-chrome/sw/download-interceptor.test.mjs`
- `node --check rhwp-chrome/sw/download-interceptor.js`
- `npm run build` in `rhwp-chrome`

수동 검증:

- #1471 blob PDF 재현 확장으로 filename/subdirectory 유지 확인.
- 기존 HWP 다운로드 기록이 있는 상태에서 Chrome 재시작/확장 reload 후 자동 재오픈 없음.
- direct `.hwp/.hwpx` 다운로드 1회당 탭 1개.
- extensionless `/download?id=...`가 filename 확정 후 탭 1개.
- `autoOpen=false`에서 자동 open 없음.

문서:

- `mydocs/working/task_m100_1515_stage1.md`
- `mydocs/report/task_m100_1515_report.md`
- `mydocs/orders/20260624.md` 상태 갱신

## 위험 / 주의

- `storage.session`은 적절한 임시 상태 저장소지만, 브라우저별 지원 차이는 adapter에서 감춘다.
- `startTime`이 없는 항목을 무조건 새 다운로드로 보면 과거 항목 재오픈이 재발할 수 있으므로,
  저장 상태 없는 `onChanged` 단독 경로는 보수적으로 처리한다.
- #1513은 긴급 배포용이므로, #1515 브랜치는 #1513 위에 쌓인 상태다. #1513 병합 후 base를
  `devel`로 rebase하는 것을 전제로 한다.
- Firefox는 #1516에서 별도 적용한다. #1515에서 Firefox 파일을 직접 수정하지 않는다.

