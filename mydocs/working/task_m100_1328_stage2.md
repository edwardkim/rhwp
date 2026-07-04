# Task M100-1328 Stage 2 완료 보고서 — 문서 글꼴 상태 분석과 안내 모달

- 이슈: https://github.com/edwardkim/rhwp/issues/1328
- 수행 계획서: `mydocs/plans/task_m100_1328.md`
- 구현 계획서: `mydocs/plans/task_m100_1328_impl.md`
- 작성일: 2026-06-21
- 브랜치: `codex/1328-local-font-consent`
- 기준 커밋: `1ce7b79d7466`

## 1. 완료 범위

Stage 2 범위인 문서 글꼴 상태 분석과 사용자 동의 기반 로컬 글꼴 감지 안내 모달을 구현했다.

변경 파일:

- `rhwp-studio/src/core/document-font-status.ts`
- `rhwp-studio/src/ui/local-fonts-modal.ts`
- `rhwp-studio/src/main.ts`
- `rhwp-studio/src/core/font-substitution.ts`
- `rhwp-studio/src/core/local-fonts.ts`
- `rhwp-studio/src/ui/options-dialog.ts`
- `rhwp-studio/tests/document-font-status.test.ts`
- `rhwp-studio/tests/local-fonts.test.ts`

## 2. 주요 변경

### 2.1 문서 글꼴 상태 분석 추가

`docInfo.fontsUsed`를 기준으로 문서에 사용된 글꼴을 다음 상태로 분류하는
`analyzeDocumentFonts()`를 추가했다.

- `available`
- `needs-local-check`
- `web-substitute`
- `missing`

저장된 로컬 글꼴 snapshot이 있으면 해당 결과를 우선 사용한다. 저장된 snapshot이 없고
Local Font Access API 또는 Firefox용 문서 후보 font presence probe를 사용할 수 있는 환경에서는
기본 지원 밖 글꼴을 `needs-local-check`로 분류해 사용자 동의 모달을 띄울 수 있게 했다.

Firefox fallback snapshot은 전체 로컬 글꼴 목록이 아니라 현재 문서에서 확인한 후보만 담으므로
`checkedFamilies`를 함께 저장한다. 이후 다른 문서에서 아직 확인하지 않은 글꼴이 나오면 다시 모달을 띄운다.

### 2.2 동의 기반 안내 모달 추가

`validation-modal.ts`와 비슷한 구조로 `local-fonts-modal.ts`를 추가했다.

모달은 다음 내용을 안내한다.

- 현재 문서에 rhwp 기본 지원 밖 글꼴이 있음
- 원본에 가깝게 표시하려면 로컬 글꼴 감지가 필요함
- 감지 결과는 브라우저/확장 로컬 저장소에만 저장함
- Firefox에서는 전체 로컬 글꼴 목록을 읽지 않고 현재 문서에 필요한 글꼴명만 확인함
- 서버로 전송하지 않음
- 거절해도 웹 대체 글꼴로 계속 볼 수 있음

버튼은 다음 두 가지로 구성했다.

- `로컬 글꼴 감지 (권장)`
- `웹 대체로 보기`

상세 영역에는 문서 글꼴별 상태와 대체 글꼴 후보를 표시한다.

### 2.3 문서 로드 흐름 연결

`main.ts`의 문서 초기화 흐름에 로컬 글꼴 안내 단계를 연결했다.

흐름:

1. 문서 로드 및 초기 렌더
2. HWPX 자동보정 modal 처리
3. 저장된 로컬 글꼴 snapshot 로드
4. 문서 글꼴 상태 분석
5. 필요한 경우 로컬 글꼴 감지 동의 modal 표시
6. 사용자가 승인한 경우에만 `detectLocalFonts({ force: true, includeRegistered: true, candidateFamilies: docInfo.fontsUsed })` 실행
7. 감지 후 상태 재계산, `local-fonts-changed` 이벤트 발행, canvas reload

승인 전에는 새 로컬 글꼴 목록을 조회하지 않는다.

### 2.4 Firefox fallback 추가

Firefox는 `queryLocalFonts()`를 제공하지 않으므로 Chrome과 동일한 전체 목록 감지는 불가능하다.
대신 같은 동의 모달 UX를 유지하고, 사용자가 승인한 뒤 현재 문서에 필요한 글꼴명만 canvas 측정 기반으로 확인한다.

이 방식의 특징:

- 전체 로컬 글꼴 목록을 열거하지 않는다.
- 문서에 등장한 후보 글꼴만 확인한다.
- 결과는 `source: 'font-presence-probe'` snapshot으로 저장한다.
- Firefox 확장 컨텍스트에서는 `browser.storage.local`을 우선 사용한다.
- 이미 등록된 웹폰트(`REGISTERED_FONTS`)는 probe 대상에서 제외해 웹폰트를 로컬 글꼴로 오인하지 않게 했다.

Firefox 한계와 대체 구현 방향은 이슈 코멘트로도 기록했다.

- https://github.com/edwardkim/rhwp/issues/1328#issuecomment-4759521169

### 2.5 테스트 보강

`rhwp-studio/tests/document-font-status.test.ts`를 추가했다.

검증 항목:

1. 저장된 snapshot이 없고 API 지원 환경이면 기본 지원 밖 글꼴을 `needs-local-check`로 분류한다.
2. 저장된 snapshot에 원본 글꼴이 있으면 `available/local`로 분류한다.
3. partial probe snapshot에 아직 확인하지 않은 글꼴이 있으면 다시 prompt 대상으로 분류한다.
4. API와 probe 모두 미지원인 환경에서는 modal prompt를 띄우지 않고 웹 대체 또는 누락 상태로 분류한다.

`rhwp-studio/tests/local-fonts.test.ts`도 함께 보강했다.

검증 항목:

1. 저장된 snapshot 로드는 `queryLocalFonts()`를 호출하지 않는다.
2. Chrome 확장 컨텍스트에서는 `chrome.storage.local`을 우선 사용한다.
3. Firefox 확장 컨텍스트에서는 `browser.storage.local`을 사용한다.
4. Local Font Access API가 없으면 문서 후보 글꼴만 probe snapshot으로 저장한다.

Node 직접 실행 테스트를 위해 `font-substitution.ts`의 내부 import 경로를 `.ts` 확장자 포함 형태로 정리했다.

## 3. 검증 결과

통과:

```bash
cd rhwp-studio && node --test tests/document-font-status.test.ts tests/local-fonts.test.ts
cd rhwp-studio && npm test
cd rhwp-studio && npm run build
git diff --check
```

결과 요약:

- `document-font-status.test.ts` + `local-fonts.test.ts`: 10개 통과
- `npm test`: 94개 통과
- `npm run build`: `tsc && vite build` 통과
- `git diff --check`: 출력 없음

미완료:

- `cd rhwp-firefox && npm run build`는 `rhwp-firefox/node_modules`에 `vite`가 설치되어 있지 않아 실패했다.
  `npm ci` 실행 승인은 현재 사용량 제한으로 거절되어 Firefox 확장 패키지 빌드는 후속 확인이 필요하다.

## 4. 남은 작업

다음 Stage 3에서 진행할 항목:

- 원본 글꼴명을 편집 값으로 보존하면서 표시/측정 단계에서만 font-family chain 적용
- 로컬 snapshot에 있는 원본 글꼴을 웹 대체 글꼴보다 앞에 배치
- `wasm-bridge.ts` 측정 경로와 `font-substitution.ts` fallback helper 정리
- alias 성격의 웹폰트 등록이 로컬 원본 글꼴을 가리는 문제 최소화

## 5. 승인 요청

Stage 2는 완료되었다. 승인 후 Stage 3 원본 글꼴명 보존과 표시용 font-family chain 구현으로 진행한다.
