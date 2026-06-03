# 단계 1 완료보고서 — task_m100_1131_stage1

- **이슈**: edwardkim/rhwp#1131
- **브랜치**: `feature/1131-file-url-access-guidance`
- **단계**: 1/3 — 권한 감지 + 안내 토스트 헬퍼 추가
- **작성일**: 2026-05-26

## 변경 내용

**`rhwp-studio/src/main.ts`** — `showLoadError` 바로 위에 헬퍼 2개 추가:

1. `isFileSchemeAccessAllowed(): Promise<boolean | null>`
   - `chrome.extension.isAllowedFileSchemeAccess()`를 Promise로 호출.
   - 허용=true / 미허용=false / API 부재·예외=null(판정 불가).
2. `showFileUrlAccessGuidance(): void`
   - `showToast`로 안내 메시지(한국어) + `confirmLabel: '확인'` + `durationMs: 0`.
   - `action: { label: '설정 열기', onClick }` → `chrome.tabs.create({ url: 'chrome://extensions/?id=' + chrome.runtime.id })`.
   - `chrome.tabs`/`chrome.runtime.id` 부재 대비 가드 포함.
   - 상태 표시줄에도 짧은 메시지 표기.

이 단계에서는 **헬퍼 정의만** 추가했고 호출부 연결은 단계 2에서 수행한다.

## 검증

- `npx tsc --noEmit` 실행:
  - 본 변경 관련 타입 에러 **없음**.
  - 남은 2건(`@wasm/rhwp.js` 모듈 미발견)은 WASM `pkg/` 미빌드로 인한 **기존 문제**이며 본 변경과 무관.
- `rhwp-studio` 의존성은 fresh clone이라 `npm install`로 설치 완료.

## 다음 단계

단계 2 — `loadFromUrlParam` 실패 `catch`에 `file://` + 권한 미허용 분기 통합.
