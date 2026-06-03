# 최종 결과보고서 — task_m100_1131_report

- **이슈**: edwardkim/rhwp#1131
- **브랜치**: `feature/1131-file-url-access-guidance` (upstream/devel 기준)
- **유형**: 버그 수정 (rhwp-chrome / rhwp-studio)
- **작성일**: 2026-05-26

## 1. 문제

파일 탐색기에서 로컬 `.hwp/.hwpx`를 Chrome으로 열면 rhwp 뷰어가 빈 화면 +
"파일 로드 실패: Failed to fetch"만 표시하고, 원본 파일은 다운로드됐다.

## 2. 근본 원인

- "파일 URL에 대한 액세스 허용(Allow access to file URLs)" 토글이 꺼져 있으면(기본 OFF)
  뷰어의 `fetch(file://)`가 실패한다. `host_permissions: <all_urls>`는 `file://`를 커버하지 않음.
- MV3 service worker 폴백(`fetch-file`)도 `file://`를 받지 못해 실패.
- `showLoadError`가 원본 에러("Failed to fetch")만 노출 → 원인/해결법 안내 부재.
- 또한 로컬 파일임에도 `suggest()`로 다운로드가 그대로 진행됨(중복).

## 3. 변경 사항

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/main.ts` | `isFileSchemeAccessAllowed()`, `showFileUrlAccessGuidance()` 추가. `loadFromUrlParam` 실패 시 `file://` + 권한 미허용이면 안내 토스트(+"설정 열기" 버튼 → `chrome://extensions/?id=...`) 표시. 그 외는 기존 `showLoadError` 유지. |
| `rhwp-chrome/sw/download-interceptor.js` | `file://` HWP는 `suggest` 대신 `cancel`+`erase`로 다운로드 억제. 원격(http)은 기존 동작 유지. |

## 4. 검증

- 타입체크(`tsc --noEmit`): 본 변경 관련 에러 없음(잔여 2건은 WASM `pkg/` 미빌드, 무관).
- 공유 판정 테스트(`download-interceptor-common.test.js`): 26/26 통과.
- `node --check`: `download-interceptor.js` 문법 OK.
- **작업지시자 수동 검증 (macOS Chrome)**:
  - ✅ 권한 OFF → 안내 토스트 + "설정 열기" 버튼 표시
  - ✅ 권한 ON → 문서 정상 표시 (회귀 없음)
  - ✅ 로컬 파일 열 때 중복 다운로드 발생 안 함

## 5. 커밋

- `a8f4f8b4` 단계1 — 권한 안내 헬퍼 추가
- `07085d99` 단계2 — `loadFromUrlParam` 분기 연결
- `fe13f2ee` 단계3 — `file://` 중복 다운로드 억제

## 6. 비고

- Windows Chrome 환경 추가 검증 권장(다운로드 억제는 best-effort).
- 안내 문구는 한국어 하드코딩(기존 뷰어 문자열 관례 일치). i18n은 후속 가능.

## 7. 후속

- upstream `edwardkim/rhwp`의 `devel`로 PR 생성.
