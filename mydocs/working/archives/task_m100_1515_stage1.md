# Task M100 #1515 1단계 완료보고서 — 공통 다운로드 상태 머신과 Chrome adapter

- 이슈: #1515
- 브랜치: `local/task1515-download-state-machine`
- 작성일: 2026-06-24
- 단계: 1/1

## 1. 변경

### `rhwp-shared/sw/download-observer-state.js`

- 브라우저 API에 의존하지 않는 다운로드 관찰자 상태 머신을 추가했다.
- `evaluateDownloadCreated()`:
  - 과거 완료 항목 무시.
  - fresh 항목 추적 상태 생성.
  - 만료된 handled 상태는 새 다운로드를 영구 차단하지 않도록 TTL 처리.
- `evaluateDownloadChanged()`:
  - 저장 상태 없는 `onChanged` 단독 항목 무시.
  - 추적 상태가 있는 항목만 filename/finalUrl 확정 후 후보로 반환.
  - 과거 항목으로 재조회되면 무시.
- `markDownloadHandled()`, `markDownloadTerminal()`, `isDownloadStateExpired()` 추가.

### `rhwp-chrome/sw/download-interceptor.js`

- 메모리 `seen` / `handled` / `workerStartedAt` 기반 로직을 제거했다.
- `chrome.storage.session` 기반 TTL 상태 저장 helper를 추가했다.
- `storage.session`이 없는 테스트/호환 환경에는 adapter 내부 메모리 fallback을 둔다.
- `onCreated`는 공통 상태 머신으로 track 여부를 판단한 뒤 상태를 저장한다.
- `onChanged`는 저장 상태가 있는 download id만 `downloads.search({ id })`로 재조회한다.
- `autoOpen=false`, `file://` HWP cancel/erase, 대용량 경고, `openViewer()` 동작은 유지했다.
- terminal delta는 상태에 `terminalAt`을 기록한 뒤 30초 후 정리한다.

### `rhwp-chrome/sw/download-interceptor.test.mjs`

- `chrome.storage.session` mock을 추가했다.
- service worker 재시작을 모사해 `onCreated` 후 모듈 재로드, 이후 `onChanged` filename 확정 시 새 HWP가 1회 열리는지 검증했다.
- storage에 handled 상태가 남아 있으면 재시작 후에도 중복 open이 막히는지 검증했다.
- 동일 download id에 filename/finalUrl/complete 이벤트가 여러 번 와도 탭은 1개만 열리는지 검증했다.

## 2. 검증

| 명령 | 결과 |
|---|---|
| `node --test rhwp-shared/sw/download-observer-state.test.js` | 통과: 14 passed |
| `node --test rhwp-shared/sw/download-interceptor-common.test.js` | 통과: 26 passed |
| `node --test rhwp-chrome/sw/download-interceptor.test.mjs` | 통과: 13 passed |
| `node --check rhwp-shared/sw/download-observer-state.js` | 통과 |
| `node --check rhwp-chrome/sw/download-interceptor.js` | 통과 |
| `npm run build` (`rhwp-chrome`) | 통과 |
| dist `sw/download-observer-state.js` 포함 확인 | 통과 |
| dist/source `onDeterminingFilename.addListener` 잔여 검색 | 결과 없음 |

## 3. 비고

- Firefox 파일은 수정하지 않았다. Firefox 적용은 #1516에서 같은 shared 상태 머신을 재사용해 진행한다.
- Safari는 downloads API가 없어 이번 상태 머신 직접 적용 대상이 아니다.
- #1513 긴급 완화 패치 위에서 구조 개선을 수행했으며, #1513 merge 후 최신 `upstream/devel` 기준 브랜치로 재생성했다.

## 4. 수동 검증

작업지시자 환경의 unpacked Chrome 확장에서 다음 케이스를 확인했다.

- Kangnam `download.do?...` 링크: URL 확장자가 없는 다운로드 핸들러가 filename 확정 후 정상 처리됨.
- GitHub raw `.hwp` 링크: 직접 `.hwp` URL이 정상 처리됨.
- Chrome 재시작/확장 reload 후 과거 HWP 탭 자동 재오픈 없음.
- `autoOpen=false`에서 다운로드 관찰자 경로 자동 열림 없음.
- 같은 링크 반복 클릭 시 클릭 1회당 탭 1개.
