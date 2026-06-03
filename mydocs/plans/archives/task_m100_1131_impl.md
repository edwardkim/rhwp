# 구현계획서 — task_m100_1131_impl

- **이슈**: edwardkim/rhwp#1131
- **브랜치**: `feature/1131-file-url-access-guidance`
- **선행**: `task_m100_1131.md`(수행계획서) 승인 완료
- **작성일**: 2026-05-26

수행계획서의 설계를 3단계로 구현한다. 단계마다 커밋하고 단계별 완료보고서를 작성한다.

---

## 단계 1 — 권한 감지 + 안내 토스트 헬퍼 추가

**대상**: `rhwp-studio/src/main.ts`

- `file://` URL 로드 실패 시 사용할 안내 헬퍼 `showFileUrlAccessGuidance()` 추가.
  - `showToast` 사용 (`durationMs: 0`).
  - 메시지(한국어 하드코딩, 기존 뷰어 문자열 관례 일치):
    > 로컬 파일을 열려면 확장 프로그램의 "파일 URL에 대한 액세스 허용"을 켜야 합니다.
    > 설정에서 권한을 허용한 뒤 파일을 다시 열어 주세요.
  - `action: { label: "설정 열기", onClick }` →
    `chrome.tabs.create({ url: "chrome://extensions/?id=" + chrome.runtime.id })`.
    `chrome.tabs`가 없을 가능성에 대비해 안전 가드(존재 시에만 호출).
- 권한 상태 조회 헬퍼: `chrome.extension?.isAllowedFileSchemeAccess`를 Promise로 감싸
  `false`(미허용) 여부 판정. API 부재 시 `null`(판정 불가) 반환.

**완료 기준**: 헬퍼 함수 추가, 타입체크 통과. 호출부 연결은 단계 2.

## 단계 2 — `loadFromUrlParam` 실패 분기 통합

**대상**: `rhwp-studio/src/main.ts` `loadFromUrlParam` (현 L730), 최종 `catch`(L767).

- 로드 실패 `catch`에서 조건 분기:
  1. `fileUrl`이 `file:`로 시작하고
  2. 확장 환경(`chrome` 존재)이며
  3. 권한 조회 결과가 **미허용(false)** 이면
     → `showFileUrlAccessGuidance()` 호출 후 종료.
  4. 그 외(권한 허용/판정 불가/비-file URL) → 기존 `showLoadError(error)` 유지.
- 기존 정상 경로(권한 ON, 원격 URL)는 변경 없음 — 회귀 차단.

**완료 기준**: 분기 로직 반영, 타입체크 통과.

## 단계 3 — `file://` 중복 다운로드 억제

**대상**: `rhwp-chrome/sw/download-interceptor.js` `onDeterminingFilename` 리스너.

- HWP/HWPX 판정 후 `item.url`이 `file:`로 시작하면:
  - `suggest` 호출 대신 `handleHwpDownload(item)`로 뷰어를 열고,
  - `chrome.downloads.cancel(item.id)` → 성공/실패 무관하게 `chrome.downloads.erase({ id: item.id })`로 정리(best-effort, 예외 무시).
- `file://`이 아니면 기존 로직(`suggest`) 유지 → 원격 파일 동작 무변경.
- 판정 로직(`shouldInterceptDownload`)은 변경하지 않는다.

**완료 기준**: `file://` 분기 추가, 원격 경로 회귀 없음(코드 리뷰로 확인).

## 단계 4 — 빌드 검증 + 수동 검증 안내

- `rhwp-studio` 타입체크 / Vite 빌드 또는 `rhwp-chrome` `build.mjs` 빌드 성공 확인.
- 기존 테스트 무영향 확인.
- 작업지시자 수동 검증 절차 안내(Windows Chrome):
  - 권한 OFF → 로컬 `.hwpx` 열기 → 안내 토스트 + "설정 열기" 버튼 표시.
  - "설정 열기" → `chrome://extensions/?id=...` 이동.
  - 권한 ON 후 재시도 → 문서 정상 표시(회귀 없음).
  - 로컬 `.hwpx` 열기 시 **Downloads 폴더에 중복 파일이 생기지 않는지** 확인(다운로드 억제).
  - 원격(http) HWP 링크는 기존대로 다운로드되는지 확인(회귀 없음).
- 최종 결과보고서(`task_m100_1131_report.md`) 작성 → 승인 → upstream `devel`로 PR.

---

## 커밋 전략

- 단계 1·2는 단일 논리 변경이므로 묶어서 1커밋 가능(`Task #1131: file:// 권한 안내`).
  단, 단계별 완료보고서 규칙에 따라 단계 단위로 커밋 분리도 허용.
- 커밋 메시지: `Task #1131: ...` 형식 + 본문에 `refs edwardkim/rhwp#1131`.

## 위험 점검

- WASM 빌드(`pkg/`)가 없으면 `build.mjs` 전체 빌드는 실패할 수 있음 → 타입체크/Vite
  단독 빌드로 검증 대체 가능. 본 변경은 WASM 비의존.
- `chrome://` URL은 링크로 직접 못 열지만 `chrome.tabs.create`로는 열림(확장 컨텍스트).
