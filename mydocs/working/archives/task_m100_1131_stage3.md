# 단계 3 완료보고서 — task_m100_1131_stage3

- **이슈**: edwardkim/rhwp#1131
- **브랜치**: `feature/1131-file-url-access-guidance`
- **단계**: 3/4 — `file://` 중복 다운로드 억제
- **작성일**: 2026-05-26

## 변경 내용

**`rhwp-chrome/sw/download-interceptor.js`**:

- `isLocalFileDownload(item)`: `item.url`이 `file:`로 시작하는지 판별.
- `onDeterminingFilename` 분기:
  - HWP 판정 + `file://` → `handleHwpDownload(item)`로 뷰어를 열고, `suggest` 대신
    `suppressLocalDownload(item)` 호출.
  - HWP 판정 + 원격 → 기존 `suggest({ filename })` 유지 (회귀 없음).
- `suppressLocalDownload(item)`: `chrome.downloads.cancel(id)` → `erase({ id })`
  (best-effort, 각 단계 예외 무시).
- 판정 로직(`shouldInterceptDownload`, 공유 모듈)은 변경하지 않음.

## 검증

- 공유 판정 테스트 `download-interceptor-common.test.js`: **26/26 통과** (회귀 없음).
- `node --check sw/download-interceptor.js`: 문법 OK.

## 한계 (수동 검증 필요)

- 로컬 복사는 거의 즉시 완료되어 `cancel`이 늦을 수 있음 → Downloads 폴더에 파일이
  남을 가능성. 작업지시자 환경(Windows Chrome)에서 실제 억제 여부 검증 필요.
- `cancel` 실패 시에도 뷰어 동작에는 영향 없음.

## 다음 단계

단계 4 — 빌드 검증 + 작업지시자 수동 검증 + 최종 보고서 → PR.
