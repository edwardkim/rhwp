# Task M100-1328 Stage 1 완료 보고서 — 로컬 글꼴 저장소와 상태 모델

- 이슈: https://github.com/edwardkim/rhwp/issues/1328
- 수행 계획서: `mydocs/plans/task_m100_1328.md`
- 구현 계획서: `mydocs/plans/task_m100_1328_impl.md`
- 작성일: 2026-06-21
- 브랜치: `codex/1328-local-font-consent`
- 기준 커밋: `1ce7b79d7466`

## 1. 완료 범위

Stage 1 범위인 로컬 글꼴 감지 결과 저장소와 상태 모델을 구현했다.

변경 파일:

- `rhwp-studio/src/core/local-fonts.ts`
- `rhwp-studio/tests/local-fonts.test.ts`

## 2. 주요 변경

### 2.1 저장 snapshot 모델 추가

`LocalFontSnapshot`을 추가했다.

- `version`
- `detectedAt`
- `families`
- `source: 'local-font-access'`

저장 key는 `rhwp-local-fonts`로 분리해 기존 `rhwp-settings`와 섞지 않는다.

### 2.2 저장소 adapter 추가

저장소 우선순위는 다음과 같다.

1. `chrome.storage.local`
2. `localStorage`
3. 저장소 없음 또는 접근 실패 시 메모리 캐시만 사용

Chrome 확장 컨텍스트에서는 `chrome.storage.local`을 우선 사용하므로 Service Worker/확장 환경에서
`localStorage`에 의존하지 않는 기반을 마련했다.

### 2.3 API 분리

다음 API를 추가했다.

- `loadStoredLocalFonts()`
- `clearStoredLocalFonts()`
- `getDetectedLocalFonts()`
- `getLocalFontState()`
- `resetLocalFontsForTests()`

기존 API는 유지했다.

- `isLocalFontSupported()`
- `detectLocalFonts()`
- `getLocalFonts()`

`detectLocalFonts()`는 기본적으로 기존 UI 호환을 위해 `REGISTERED_FONTS`에 등록된 글꼴을 제외해 반환한다.
다만 snapshot에는 전체 family를 저장하고, 후속 Stage에서 문서 상태 분석에 사용할 수 있도록
`includeRegistered` 옵션과 `getDetectedLocalFonts()`를 추가했다.

### 2.4 동의 전 조회 방어 기반

`loadStoredLocalFonts()`는 저장된 snapshot만 읽고 `queryLocalFonts()`를 호출하지 않는다.
새 로컬 목록 조회는 `detectLocalFonts({ force: true })` 같은 승인된 UI 흐름에서만 발생하도록
후속 Stage에서 연결할 수 있는 구조로 분리했다.

## 3. 테스트 보강

`rhwp-studio/tests/local-fonts.test.ts`를 추가했다.

검증 항목:

1. 저장된 `localStorage` snapshot 로드는 `queryLocalFonts()`를 호출하지 않는다.
2. Chrome 확장 컨텍스트에서는 `chrome.storage.local` snapshot을 `localStorage`보다 우선한다.
3. `detectLocalFonts()`는 전체 snapshot을 저장하고, 기본 반환은 웹 등록 글꼴을 제외한다.
4. 저장소 읽기 실패는 빈 상태로 처리하고 오류 상태만 기록한다.

## 4. 검증 결과

통과:

```bash
cd rhwp-studio && node --test tests/local-fonts.test.ts
cd rhwp-studio && npm test
cd rhwp-studio && npm run build
git diff --check
```

결과 요약:

- `local-fonts.test.ts`: 4개 통과
- `npm test`: 88개 통과
- `npm run build`: `tsc && vite build` 통과
- `git diff --check`: 출력 없음

## 5. 남은 작업

다음 Stage 2에서 진행할 항목:

- `docInfo.fontsUsed` 기반 문서 글꼴 상태 분석
- HWPX validation modal과 유사한 로컬 글꼴 안내 modal 추가
- 저장된 snapshot이 없을 때만 사용자에게 로컬 글꼴 감지 승인을 요청하는 흐름 연결

## 6. 승인 요청

Stage 1은 완료되었다. 승인 후 Stage 2 문서 글꼴 상태 분석과 안내 모달 구현으로 진행한다.
