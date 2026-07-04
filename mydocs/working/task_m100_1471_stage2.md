# Task M100 #1471 Stage 2 완료 보고서

- 이슈: #1471 `onDeterminingFilename 인터셉터가 다른 확장의 chrome.downloads.download({filename}) 하위폴더 저장을 무효화`
- 브랜치: `local/task1471`
- 작업일: 2026-06-22
- 단계: Stage 2 — Chrome downloads mock 테스트 추가

## 1. 완료 내용

Chrome 다운로드 관찰자 전환 동작을 검증하는 Node 테스트를 추가했다.

신규 테스트는 Chrome downloads API를 mock으로 구성한 뒤 `setupDownloadInterceptor()`를 실행하여 다음 계약을 확인한다.

- `onDeterminingFilename` 리스너를 등록하지 않는다.
- `onCreated` / `onChanged` 관찰자만 등록한다.
- 비-HWP blob PDF 다운로드는 `openViewer`, `cancel`, `erase`를 호출하지 않는다.
- HWP 다운로드는 뷰어를 한 번만 연다.
- `autoOpen=false`에서는 뷰어를 열지 않는다.
- `onCreated` 시점에 판정 불가인 다운로드는 `onChanged` + `downloads.search()` 이후 filename 기준으로 재판정한다.
- `file://` HWP는 뷰어를 열고 `cancel`/`erase` best-effort 억제를 시도한다.

## 2. 변경 파일

| 파일 | 변경 |
|------|------|
| `rhwp-chrome/sw/download-interceptor.test.mjs` | 신규 Chrome downloads mock 테스트 6개 추가 |

## 3. 검증 결과

| 명령 | 결과 |
|------|------|
| `node --check rhwp-chrome/sw/download-interceptor.test.mjs` | 통과 |
| `node --test rhwp-shared/sw/download-interceptor-common.test.js` | 26 passed |
| `node --test rhwp-chrome/sw/download-interceptor.test.mjs` | 6 passed |

## 4. 남은 작업

다음 Stage 3에서 실제 Chrome 확장 로드와 수동 재현 검증을 진행한다.

- rhwp Chrome 테스트 빌드 또는 압축해제 로드 상태 확인
- Mac Chrome 재현 확장으로 #1471 경로 보존 확인
- HWP/HWPX 자동 열기 회귀 수동 확인
- 필요 시 확장 빌드 검증

Stage 3 승인 전에는 빌드/수동 검증 단계로 진행하지 않는다.
