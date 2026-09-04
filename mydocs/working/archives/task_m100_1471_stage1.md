# Task M100 #1471 Stage 1 완료 보고서

- 이슈: #1471 `onDeterminingFilename 인터셉터가 다른 확장의 chrome.downloads.download({filename}) 하위폴더 저장을 무효화`
- 브랜치: `local/task1471`
- 작업일: 2026-06-22
- 단계: Stage 1 — Chrome 인터셉터 관찰자 전환

## 1. 완료 내용

`rhwp-chrome/sw/download-interceptor.js`에서 Chrome filename 결정 이벤트 의존을 제거했다.

- 제거: `chrome.downloads.onDeterminingFilename.addListener(...)`
- 추가: `chrome.downloads.onCreated.addListener(...)`
- 추가: `chrome.downloads.onChanged.addListener(...)`
- 추가: 다운로드 ID 중복 처리용 `handled` 집합
- 유지: `shouldInterceptDownload()` 기반 HWP/HWPX 판정
- 유지: `autoOpen` 설정 확인, 대용량 파일 경고, `openViewer()` 호출
- 유지: `file://` HWP의 `cancel`/`erase` best-effort 억제

핵심 변화는 rhwp Chrome 확장이 더 이상 filename 결정 단계에 참여하지 않는다는 점이다.
따라서 비-HWP blob/PDF 다운로드에서 다른 확장의 `download({ filename })` 경로 결정을 무효화하지 않아야 한다.

## 2. 변경 파일

| 파일 | 변경 |
|------|------|
| `rhwp-chrome/sw/download-interceptor.js` | `onDeterminingFilename` 기반 인터셉터를 `onCreated`/`onChanged` 관찰자로 전환 |

## 3. 확인 결과

| 확인 | 결과 |
|------|------|
| JS 문법 확인: `node --check rhwp-chrome/sw/download-interceptor.js` | 통과 |
| Chrome 코드 내 `chrome.downloads.onDeterminingFilename` 등록 검색 | 없음 |
| Chrome 코드 내 `onDeterminingFilename.addListener` 검색 | 없음 |

## 4. 남은 작업

다음 Stage 2에서 테스트를 추가한다.

- Chrome downloads API mock 테스트 추가
- `setupDownloadInterceptor()`가 `onDeterminingFilename`을 등록하지 않는지 검증
- 비-HWP PDF/blob 다운로드가 `openViewer`/`cancel`/`erase`를 호출하지 않는지 검증
- HWP/HWPX 다운로드가 중복 없이 처리되는지 검증
- `autoOpen=false` 회귀 검증
- 기존 `rhwp-shared/sw/download-interceptor-common.test.js` 유지 검증

Stage 2 승인 전에는 테스트 추가와 추가 소스 수정을 진행하지 않는다.
