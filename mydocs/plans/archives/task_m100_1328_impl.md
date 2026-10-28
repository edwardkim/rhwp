# 구현 계획서 — Task M100-1328: 로컬 글꼴 감지 동의 UX 및 표시 단계 font fallback 정리

- 이슈: https://github.com/edwardkim/rhwp/issues/1328
- 수행 계획서: `mydocs/plans/task_m100_1328.md`
- 작성일: 2026-06-21
- 브랜치: `codex/1328-local-font-consent` (원격 push 전 `codex/` 제거 여부 재판단)
- 기준 커밋: `1ce7b79d7466`

## 1. 구현 원칙

1. 사용자 승인 전에는 `queryLocalFonts()`를 호출하지 않는다.
2. 저장된 로컬 글꼴 감지 결과는 앱 시작/문서 로드 시 재사용할 수 있지만, 이것은 새 로컬 목록 조회가 아니다.
3. 문서의 원본 글꼴명은 편집 값으로 보존하고, 렌더링/측정 단계에서만 표시용 font-family chain을 만든다.
4. 기존 웹폰트 fallback 품질은 유지한다.
5. Local Font Access API 미지원, 권한 거절, 저장소 실패는 정상 fallback 경로로 처리한다.
6. Chrome 확장 컨텍스트에서는 `chrome.storage.local`, Firefox 확장 컨텍스트에서는 `browser.storage.local`을 우선 사용한다.
7. Firefox처럼 `queryLocalFonts()`가 없는 브라우저에서는 사용자 승인 후 현재 문서에 필요한 글꼴명만 font presence probe로 확인한다.

## 2. 현재 코드 기준 결정

최신 `upstream/devel` 기준으로 확인한 내용은 다음과 같다.

| 영역 | 현재 상태 | 구현 판단 |
|---|---|---|
| `local-fonts.ts` | `queryLocalFonts()` 결과를 메모리 캐시에만 보관 | 저장소 adapter, 감지 상태 모델, Firefox 문서 후보 probe를 이 파일 중심으로 추가 |
| `font-loader.ts` | OS 후보 감지, `REGISTERED_FONTS`, alias 성격의 `FONT_LIST`, selective webfont load 존재 | 기존 selective load는 유지하되 local snapshot과 alias masking 문제를 분리 |
| `font-substitution.ts` | `resolveFont()`가 원본을 등록 웹폰트명으로 조기 치환 가능 | 원본명 보존용 display chain helper를 추가하고 기존 API 영향 최소화 |
| `wasm-bridge.ts` | Canvas `measureTextWidth`에서 `substituteCssFontFamily()` 호출 | 표시용 chain helper를 측정 경로에 적용 |
| `main.ts` | 문서 초기화 후 HWPX validation modal 실행 | validation modal 이후 로컬 글꼴 안내 modal을 순차 실행 |
| `toolbar.ts` | 문서 로드 시 `initFontDropdown(docFonts)` 호출, local optgroup은 private helper | `lastDocFonts`와 public refresh 또는 event listener 추가 |
| `char-shape-dialog.ts` / `font-set-edit-dialog.ts` | 생성 시점의 `getLocalFonts()`만 반영 | 새로 여는 대화상자는 자동 반영, 열려 있는 대화상자는 필요 시 후속 보강 |
| `user-settings.ts` | 동기 `localStorage` 기반 싱글턴 | 로컬 글꼴 감지 결과는 별도 비동기 저장소로 분리 |
| `package.json` | `build`가 `tsc && vite build`, `test`가 Node test | 검증 명령은 `npm run build`, `npm test` 중심으로 확정 |

## 3. Stage 1 — 로컬 글꼴 저장소와 상태 모델

대상 파일:

- `rhwp-studio/src/core/local-fonts.ts`
- 신규 가능: `rhwp-studio/tests/local-fonts.test.ts`

작업:

1. 로컬 글꼴 감지 snapshot 타입 추가
   - `version`
   - `detectedAt`
   - `families`
   - `source: 'local-font-access' | 'font-presence-probe'`
   - Firefox fallback용 `checkedFamilies`
2. 저장소 key를 별도로 둔다.
   - 예: `rhwp-local-fonts`
   - `rhwp-settings`와 섞지 않는다.
3. 저장소 adapter를 분리한다.
   - Chrome 확장: `chrome.storage.local`
   - Firefox 확장: `browser.storage.local`
   - Web/PWA: `localStorage`
   - 저장소 접근 실패: 메모리 캐시만 사용하고 실패 상태 반환
4. API를 명확히 분리한다.
   - `loadStoredLocalFonts(): Promise<LocalFontSnapshot | null>`
   - `detectLocalFonts(options?: { force?: boolean }): Promise<string[]>`
   - `clearStoredLocalFonts(): Promise<void>`
   - `getLocalFonts(): string[]`
   - `getLocalFontState()` 또는 `hasStoredLocalFonts()`
5. `detectLocalFonts()`는 사용자가 승인한 UI 이벤트에서만 호출한다.
6. 중복 제거와 정렬은 기존 동작을 유지한다.

검증:

- 저장된 snapshot 로드 시 `queryLocalFonts()`가 호출되지 않는지 unit test
- `chrome.storage.local` mock 사용 시 localStorage를 쓰지 않는지 unit test
- 저장소 예외 발생 시 빈 배열/fallback으로 안전하게 끝나는지 unit test

## 4. Stage 2 — 문서 글꼴 상태 분석과 안내 모달

대상 파일:

- 신규 가능: `rhwp-studio/src/core/document-font-status.ts`
- 신규 가능: `rhwp-studio/src/ui/local-fonts-modal.ts`
- `rhwp-studio/src/main.ts`

작업:

1. `docInfo.fontsUsed` 기준 문서 글꼴 상태를 계산한다.
2. 상태 분류는 수행 계획서 기준을 따른다.
   - `available`
   - `needs-local-check`
   - `web-substitute`
   - `missing`
3. trigger 조건:
   - Local Font Access API 또는 문서 후보 font presence probe를 사용할 수 있고,
   - 저장된 로컬 감지 결과가 없거나 partial probe snapshot에서 아직 확인하지 않은 문서 글꼴이 있고,
   - 문서에 `needs-local-check` 또는 `web-substitute` 대상 원본 글꼴이 있을 때
4. 안내 modal은 `validation-modal.ts` 패턴을 따른다.
   - 제목: `로컬 글꼴 감지`
   - primary: `로컬 글꼴 감지 (권장)`
   - secondary: `웹 대체로 보기`
   - details: 문서 글꼴 상태 요약
5. 모달 설명에는 다음을 포함한다.
   - 로컬 글꼴 목록은 원본에 가까운 렌더링을 위해서만 사용
   - Firefox에서는 전체 로컬 글꼴 목록을 읽지 않고 현재 문서에 필요한 글꼴명만 확인
   - 서버로 전송하지 않음
   - 브라우저/확장 로컬 저장소에 저장
   - 거절해도 문서는 웹 대체 글꼴로 계속 표시
6. `main.ts`에서는 문서 초기 렌더와 HWPX validation modal 이후에 실행한다.
   - validation modal과 동시에 뜨지 않게 순차 처리
   - 사용자가 승인하면 `detectLocalFonts({ force: true, candidateFamilies: docInfo.fontsUsed })`
   - 감지 후 상태 재계산 및 UI 갱신 이벤트 발행

검증:

- 저장된 결과가 없고 대상 글꼴이 있으면 modal 생성
- 전체 snapshot에 저장된 결과가 있으면 modal 생략
- partial probe snapshot에 아직 확인하지 않은 새 글꼴이 있으면 modal 표시
- Local Font Access API 미지원이지만 canvas probe가 가능하면 Firefox용 안내 modal 표시
- `웹 대체로 보기` 선택 시 `queryLocalFonts()` 미호출

## 5. Stage 3 — 원본 글꼴명 보존과 표시용 font-family chain

대상 파일:

- `rhwp-studio/src/core/font-substitution.ts`
- `rhwp-studio/src/core/font-loader.ts`
- `rhwp-studio/src/core/wasm-bridge.ts`

작업:

1. 기존 `resolveFont()`는 큰 동작 변경 없이 유지한다.
2. 새 helper를 추가한다.
   - 예: `fontFamilyChainForDisplay(originalFont, altType, langId)`
   - 반환 chain: 원본 글꼴명 → 로컬 확인 글꼴명 → 웹 대체 글꼴명 → OS/system fallback → generic
3. `wasm-bridge.ts`의 `substituteCssFontFamily()`는 원본명을 대체명으로 단일 치환하지 않고 chain으로 확장한다.
4. `font-loader.ts`는 가능한 한 실제 번들 웹폰트 로드에 집중한다.
5. alias masking 위험을 줄인다.
   - 로컬 snapshot에 원본 글꼴명이 있으면 해당 원본명에 매핑된 웹폰트 파일 로드를 건너뛴다.
   - 표시 chain에서 원본명을 가장 앞에 두고, 웹 대체명은 뒤에 둔다.
   - 대규모 `FONT_LIST` 재작성은 피하되, 구현 중 alias masking이 확인되면 해당 alias만 최소 조정한다.
6. 원본 글꼴명이 toolbar/dialog의 편집 값에서 대체명으로 바뀌지 않도록 한다.

주의:

- 이미 `font-loader.ts`가 모든 `@font-face` 규칙을 등록하므로, exact local font와 동일한 family alias가
  웹폰트로 가려지는 문제가 생길 수 있다.
- 이 문제는 stage 3에서 실제 브라우저 동작을 확인하고, 최소 수정으로 해결한다.
- 대규모 폰트 목록 재분류가 필요하면 별도 후속 이슈로 분리한다.

검증:

- 원본 글꼴명이 toolbar/current format에 보존되는지 확인
- local snapshot에 있는 원본 글꼴이 web substitute보다 앞선 chain에 배치되는지 unit test
- 기존 등록 웹폰트 문서가 fallback 없이 깨지지 않는지 build/smoke 확인

## 6. Stage 4 — 감지 완료 이벤트와 UI 갱신

대상 파일:

- `rhwp-studio/src/main.ts`
- `rhwp-studio/src/ui/toolbar.ts`
- `rhwp-studio/src/ui/options-dialog.ts`
- 필요 시 `rhwp-studio/src/ui/char-shape-dialog.ts`
- 필요 시 `rhwp-studio/src/ui/font-set-edit-dialog.ts`

작업:

1. event name을 정한다.
   - 예: `local-fonts-changed`
2. `main.ts`는 감지 완료 후 다음을 수행한다.
   - `eventBus.emit('local-fonts-changed', { fonts, source: 'user-detect' })`
   - `toolbar?.initFontDropdown(docInfo.fontsUsed)` 또는 refresh method 호출
   - `canvasView?.loadDocument()`
   - 상태 표시줄 또는 toast로 감지 결과 안내
3. `Toolbar`는 마지막 문서 글꼴 목록을 보존한다.
   - `private lastDocFonts?: string[]`
   - `initFontDropdown()` 호출 시 저장
   - event 수신 시 현재 목록 재구성
4. `OptionsDialog`는 수동 재감지 후 같은 이벤트를 발행할 수 있도록 생성자에 선택적으로 `eventBus`를 받거나,
   command layer/main에서 재감지 흐름을 위임한다.
5. `char-shape-dialog.ts`와 `font-set-edit-dialog.ts`는 새로 열 때 최신 `getLocalFonts()`를 반영한다.
   열려 있는 dialog 즉시 갱신은 이번 구현에서 과도하면 후속으로 분리한다.

검증:

- 감지 직후 toolbar local optgroup이 추가되는지 확인
- 감지 직후 canvas reload가 1회 실행되는지 확인
- 환경설정 수동 재감지 후에도 같은 갱신 경로가 동작하는지 확인

## 7. Stage 5 — 환경설정 재감지/초기화와 회귀 테스트

대상 파일:

- `rhwp-studio/src/ui/options-dialog.ts`
- `rhwp-studio/tests/*.test.ts`
- 필요 시 `rhwp-studio/e2e/*.test.mjs`

작업:

1. 기존 `로컬 글꼴 감지하기` 버튼은 재감지 entry point로 유지한다.
2. 저장된 감지 결과가 있으면 detectedAt과 개수를 표시한다.
3. 필요하면 `감지 결과 초기화` 버튼을 추가한다.
4. 옵션 dialog에서 감지 성공 시 저장소에 반영하고 `local-fonts-changed` 이벤트를 발행한다.
5. 권한 거절과 API 미지원 문구를 구분한다.

검증 명령:

```bash
cd rhwp-studio && npm test
cd rhwp-studio && npm run build
```

가능하면 추가:

```bash
cd rhwp-studio && node e2e/<font-detection-flow>.test.mjs --mode=headless
```

수동 확인:

- Local Font Access API 지원 Chrome/Edge에서 문서 로드 후 modal 표시
- 승인 전 DevTools에서 `queryLocalFonts()` 미호출 확인
- 승인 후 저장 및 새로고침 재사용 확인
- API 미지원 환경 fallback 확인
- Chrome 확장 viewer에서 `chrome.storage.local` 경로 확인
- Firefox 확장 viewer에서 `browser.storage.local`과 문서 후보 probe 경로 확인

## 8. 커밋/브랜치 운영 메모

- 로컬 작업 브랜치는 현재 `codex/1328-local-font-consent`를 유지한다.
- 사용자가 원격 브랜치명에 `codex/`가 표시되지 않기를 원하므로, PR 생성 전 다음 중 하나를 판단한다.
  1. 로컬 브랜치를 `task_m100_1328`로 rename 후 push
  2. 로컬 브랜치는 유지하고 `git push origin HEAD:task_m100_1328`처럼 원격 브랜치명만 분리
- 저장소 관례상 `task_m100_1328` 계열 이름이 더 자연스러우므로, PR 직전 별도 승인 지점에서 최종 결정한다.

## 9. 승인 게이트

이 구현계획서 승인 전에는 `rhwp-studio/src` 및 테스트 소스를 수정하지 않는다.
승인 후 Stage 1부터 순서대로 구현하고, 각 stage 완료 후 문서/검증 결과를 보고한다.
