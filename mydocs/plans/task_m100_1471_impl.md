# Task M100 #1471 구현계획서

## 1. 분석 순서

1. Chrome 다운로드 인터셉터의 현재 진입점을 확인한다.
   - `rhwp-chrome/background.js`
   - `rhwp-chrome/sw/download-interceptor.js`
   - `rhwp-chrome/sw/viewer-launcher.js`
2. Firefox 관찰자 구현을 비교한다.
   - `rhwp-firefox/sw/download-interceptor.js`
   - `rhwp-shared/sw/download-interceptor-common.js`
3. Chrome `DownloadItem` 필드 사용 가능 범위를 확인한다.
   - `id`, `url`, `finalUrl`, `filename`, `mime`, `fileSize`, `referrer`
   - 필요 시 `byExtensionId` / `byExtensionName`
4. #1131 로컬 `file://` 중복 다운로드 억제 흐름을 Chrome 관찰자 구조에서 재평가한다.
5. 테스트 가능한 단위를 분리한다.
   - Chrome API mock 기반 인터셉터 테스트
   - 공통 판정 함수 기존 테스트
   - Mac Chrome 최소 재현 수동 테스트

## 2. 구현 방향

`rhwp-chrome/sw/download-interceptor.js`를 filename 결정 인터셉터에서 다운로드 관찰자로 전환한다.

- 제거: `chrome.downloads.onDeterminingFilename.addListener(...)`
- 추가: `chrome.downloads.onCreated.addListener(...)`
- 추가: `chrome.downloads.onChanged.addListener(...)`
- 중복 방지: 다운로드 ID별 처리 상태를 `Set` 또는 `Map`으로 관리한다.
- 판정: `shouldInterceptDownload(item)`만 사용한다.
- 최신 항목 조회: `onChanged`에서 `chrome.downloads.search({ id })`로 `DownloadItem`을 재조회한다.
- cleanup: `complete`/`interrupted`/`error` 계열 상태에서 처리 상태를 일정 시간 뒤 제거한다.

핵심 계약은 다음과 같다.

1. rhwp Chrome 확장은 `onDeterminingFilename` 리스너를 등록하지 않는다.
2. 비-HWP 다운로드에 대해서 rhwp는 파일명 결정 단계에 참여하지 않는다.
3. HWP/HWPX 다운로드 자동 열기 기능은 가능한 범위에서 유지한다.

## 3. 단계 계획

### Stage 1 — Chrome 인터셉터 관찰자 전환

- `rhwp-chrome/sw/download-interceptor.js`에서 `onDeterminingFilename` 의존을 제거한다.
- Firefox 구현과 유사하게 `onCreated` 1차 판정, `onChanged` 2차 판정 구조를 도입한다.
- `autoOpen`, 대용량 경고, `openViewer()` 호출은 기존 동작을 유지한다.
- `file://` 다운로드 억제는 `shouldInterceptDownload()` 이후 best-effort로 처리한다.

완료 기준:
- 코드에 `chrome.downloads.onDeterminingFilename` 등록이 남아 있지 않다.
- HWP/HWPX 판정과 뷰어 열기 경로가 유지된다.

### Stage 2 — 테스트 추가

- Chrome downloads API mock 테스트를 추가한다.
- 테스트 파일 후보:
  - `rhwp-chrome/sw/download-interceptor.test.mjs`
- 검증 항목:
  - `setupDownloadInterceptor()`가 `onDeterminingFilename.addListener`를 호출하지 않는다.
  - 비-HWP PDF/blob 다운로드는 `openViewer`, `cancel`, `erase`를 호출하지 않는다.
  - HWP/HWPX 다운로드는 한 번만 처리된다.
  - `autoOpen=false`에서는 `openViewer`를 호출하지 않는다.
  - `onCreated`에서 판정 불가인 항목은 `onChanged` + `downloads.search()` 이후 처리된다.
  - DEXT5 블랙리스트는 기존 공통 테스트로 유지된다.

완료 기준:
- 신규 테스트와 기존 공통 테스트가 통과한다.

### Stage 3 — 빌드와 수동 재현 검증

- Node 테스트를 실행한다.
  - `node --test rhwp-shared/sw/download-interceptor-common.test.js`
  - `node --test rhwp-chrome/sw/download-interceptor.test.mjs`
- Chrome 확장 빌드 또는 압축해제 로드 가능한 상태를 확인한다.
- Mac Chrome에서 재현 확장으로 검증한다.
  - rhwp ON 상태에서도 `Privacy Filter/Filtered/.../filtered_test.pdf` 경로가 유지되는지 확인한다.
  - rhwp OFF 정상 경로와 비교한다.
- HWP/HWPX 자동 열기 수동 검증을 수행한다.

완료 기준:
- #1471 최소 재현이 더 이상 발생하지 않는다.
- 일반 HWP/HWPX 자동 열기 기능이 회귀하지 않는다.

### Stage 4 — 보고서와 정리

- 단계별 완료 보고서 `mydocs/working/task_m100_1471_stage1.md`를 작성한다.
- 필요 시 후속 단계 보고서를 추가한다.
- 최종 완료 시 `mydocs/report/task_m100_1471_report.md`를 작성한다.
- 오늘할일 문서 상태를 완료 단계에 맞게 갱신한다.

완료 기준:
- 구현, 테스트, 수동 검증 결과가 문서화된다.
- 작업지시자 승인 후 다음 단계 또는 최종 정리로 진행한다.

## 4. 세부 설계 메모

### 4.1 처리 상태 관리

`handled` 집합으로 동일 다운로드 ID의 중복 처리를 막는다. `onCreated`에서 이미 처리된 ID는 `onChanged`에서 다시 열지 않는다.

`pending` 상태가 필요하면 `Map<id, reason>` 형태로 확장하되, 우선은 Firefox 구현과 같은 단순 `Set`을 기본으로 한다.

### 4.2 onCreated / onChanged 역할

- `onCreated`: `url`, `mime`, `referrer`만으로도 HWP/HWPX가 명확한 경우 즉시 처리한다.
- `onChanged`: filename이 확정되거나 상태가 바뀐 경우 `downloads.search({ id })`로 최신 항목을 가져와 재판정한다.

Chrome에서 filename 확정 이벤트 필드가 브라우저 버전별로 다를 수 있으므로, `delta.filename?.current`뿐 아니라 상태 변경 시점에도 필요한 경우 재조회할 수 있게 구현한다.

### 4.3 다른 확장 다운로드와의 관계

다른 확장이 시작한 다운로드라도 HWP/HWPX라면 사용자가 rhwp 자동 열기를 기대할 수 있다. 다만 #1471의 본질은 비-HWP 다운로드 개입이므로 다음 원칙을 둔다.

- 비-HWP는 무조건 무개입
- HWP/HWPX는 기존 `autoOpen` 설정을 따른다
- `byExtensionId`를 무조건 차단 조건으로 쓰지는 않는다
- 과도한 자동 열기 사례가 확인되면 별도 설정 또는 후속 이슈로 분리한다

### 4.4 file:// 중복 다운로드 억제

기존 #1131은 filename 결정 단계에서 로컬 파일 다운로드를 빠르게 억제했다. 관찰자 방식에서는 타이밍이 늦을 수 있으므로 다음 기준을 둔다.

- `file://`이면서 HWP/HWPX로 판정된 경우에만 `cancel`/`erase`를 시도한다.
- 실패는 무시한다.
- 일반 파일과 다른 확장 PDF/blob 다운로드에는 절대 `cancel`/`erase`를 호출하지 않는다.

## 5. 검증 명령

```bash
node --test rhwp-shared/sw/download-interceptor-common.test.js
node --test rhwp-chrome/sw/download-interceptor.test.mjs
```

필요 시:

```bash
cd rhwp-chrome
node build.mjs
```

수동 검증:

1. `output/poc/issue1471-download-repro-extension/`를 Chrome에 압축해제 로드한다.
2. rhwp 테스트 빌드를 로드한다.
3. 재현 확장에서 `Download 1`을 실행한다.
4. 실제 저장 경로가 `Downloads/Privacy Filter/Filtered/.../filtered_test.pdf` 계열인지 확인한다.
5. HWP/HWPX 다운로드 자동 열기 회귀가 없는지 확인한다.

## 6. 산출물 계획

- 수행계획서: `mydocs/plans/task_m100_1471.md`
- 구현계획서: `mydocs/plans/task_m100_1471_impl.md`
- 단계 보고서: `mydocs/working/task_m100_1471_stage1.md`
- 최종 보고서: `mydocs/report/task_m100_1471_report.md`

## 7. 승인 게이트

본 구현계획서 승인 후 Stage 1 구현을 시작한다.
Stage 1 완료 후 단계별 완료 보고서를 작성하고, 승인 전에는 다음 단계로 진행하지 않는다.
