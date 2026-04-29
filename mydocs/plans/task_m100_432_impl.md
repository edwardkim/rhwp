---
타스크: #432 브라우저 확장 실제 HWP/HWPX 응답 검증 및 파일 URL 해석 계층 도입
브랜치: local/task432
작성일: 2026-04-29
선행: mydocs/plans/task_m100_432.md (수행계획서, 승인됨)
---

# 구현계획서: 브라우저 확장 URL 해석/응답 검증 계층

## 0. 작업지시자 결정 사항

수행계획서 승인 지시에 따라 다음 방향으로 진행한다.

| 질문 | 결정 |
|---|---|
| Q1. provider adapter 구조 | **도입**. GitHub 처리는 첫 provider adapter로 격리 |
| Q2. 단계 분할 | **4단계 유지** |
| Q3. GitHub 외 provider | 이번 타스크에서는 인터페이스/구조만 남기고 실제 구현은 후속으로 분리 |

## 1. 핵심 설계

### 1.1 URL 해석 계층

확장자 기반 링크 감지는 "후보" 판정으로 유지하되, viewer/thumbnail이 실제로 fetch할 URL은 공통 resolver를 거친다.

신규 공통 모듈:

```text
rhwp-shared/sw/document-url-resolver.js
```

역할:

- `resolveDocumentUrl(url)`:
  - 입력 URL을 파싱한다.
  - provider adapter를 순회하여 실제 파일 URL이 있으면 반환한다.
  - 매칭 adapter가 없으면 원본 URL을 반환한다.
- `resolveGithubBlobUrl(parsedUrl)`:
  - `https://github.com/{owner}/{repo}/blob/{ref}/{path}` 형식을 raw URL로 변환한다.
  - `path`가 `.hwp`/`.hwpx`로 끝나는 경우만 변환한다.
  - raw URL, release asset, query 확장자 위장 URL은 변환하지 않는다.

Chrome/Firefox 확장에는 기존 `download-interceptor-common.js`와 같은 방식으로 symlink를 둔다.

```text
rhwp-chrome/sw/document-url-resolver.js -> ../../rhwp-shared/sw/document-url-resolver.js
rhwp-firefox/sw/document-url-resolver.js -> ../../rhwp-shared/sw/document-url-resolver.js
```

### 1.2 응답 바이트 검증 계층

viewer에서 WASM 파서 호출 직전 실제 문서 바이트인지 확인한다.

판정:

- HWP: `D0 CF 11 E0 A1 B1 1A E1`
- HWPX: ZIP 계열 `50 4B 03 04`, `50 4B 05 06`, `50 4B 07 08`
- HTML: 앞부분 공백 제거 후 `<!doctype`, `<html`, `<?xml` 중 HTML 계열 오류 응답으로 판단 가능한 경우

실패 시 WASM 파서에 전달하지 않고 사용자 오류를 표시한다.

예상 오류 문구:

```text
파일 로드 실패: 실제 HWP/HWPX 파일이 아닙니다. 파일 미리보기/오류 페이지가 반환되었습니다.
```

### 1.3 fetch-file 경로

현재 viewer는 직접 `fetch(fileUrl)`가 성공하면 background `fetch-file` 경로를 사용하지 않는다. 따라서 이번 타스크의 핵심 게이트는 `rhwp-studio/src/main.ts`의 `loadFromUrlParam()` 안에 둔다.

background `fetch-file`은 직접 fetch 실패 시 fallback으로 유지하되, 반환된 바이트도 같은 검증 함수로 검사한다.

### 1.4 썸네일 경로

`extractThumbnailFromUrl(url)`은 fetch 전에 `resolveDocumentUrl(url)`을 호출한다. 그러면 GitHub wiki의 blob URL 카드에서도 raw HWP를 받아 `PrvImage`를 추출할 수 있다.

## 2. 파일 변경 계획

| 파일 | 변경 |
|---|---|
| `rhwp-shared/sw/document-url-resolver.js` | 신규 ESM 공통 URL resolver |
| `rhwp-shared/sw/document-url-resolver.test.js` | 신규 순수 함수 테스트 |
| `rhwp-chrome/sw/document-url-resolver.js` | shared 모듈 symlink 추가 |
| `rhwp-firefox/sw/document-url-resolver.js` | shared 모듈 symlink 추가 |
| `rhwp-chrome/sw/viewer-launcher.js` | viewer URL 파라미터에 resolver 적용 |
| `rhwp-firefox/sw/viewer-launcher.js` | viewer URL 파라미터에 resolver 적용 |
| `rhwp-chrome/sw/thumbnail-extractor.js` | fetch 전 resolver 적용 |
| `rhwp-firefox/sw/thumbnail-extractor.js` | fetch 전 resolver 적용 |
| `rhwp-studio/src/main.ts` | URL 로드 바이트 검증 + 오류 메시지 개선 |
| `rhwp-chrome/dist/`, `rhwp-firefox/dist/` | 빌드 산출물 갱신 필요 시 반영 |

주의:

- `rhwp-shared/security/file-signature.js`는 이미 시그니처 검증 개념이 있으나 CommonJS export라 확장 SW의 ESM import와 맞지 않는다. 이번 변경은 SW 공통 ESM helper와 viewer TypeScript 게이트를 우선한다.
- 보안 감사 문서의 `fetch-file` sender/URL 검증 hardening은 별도 큰 범위다. 이번 타스크에서 정책을 약화하지 않는다.

## 3. 단계 분할

### Stage 1 — URL resolver + 순수 함수 테스트

**변경**:

- `rhwp-shared/sw/document-url-resolver.js` 추가
- `rhwp-shared/sw/document-url-resolver.test.js` 추가
- Chrome/Firefox `sw/document-url-resolver.js` symlink 추가

**테스트 케이스**:

- GitHub blob HWP URL:
  - `https://github.com/edwardkim/rhwp/blob/devel/saved/pr360-edward.hwp`
  - → `https://raw.githubusercontent.com/edwardkim/rhwp/devel/saved/pr360-edward.hwp`
- GitHub blob HWPX URL도 변환
- GitHub raw URL은 변환하지 않음
- 일반 HWP URL은 변환하지 않음
- `.hwp`가 query에만 있는 URL은 변환하지 않음
- malformed URL은 원본 반환 또는 실패 reason 반환

**검증 명령**:

```bash
node --test rhwp-shared/sw/document-url-resolver.test.js
```

**완료 기준**:

- 순수 함수 테스트 그린
- 기존 `download-interceptor-common.test.js` 그린

### Stage 2 — viewer/thumbnail URL 해석 적용

**변경**:

- `rhwp-chrome/sw/viewer-launcher.js`
- `rhwp-firefox/sw/viewer-launcher.js`
- `rhwp-chrome/sw/thumbnail-extractor.js`
- `rhwp-firefox/sw/thumbnail-extractor.js`

**구현 방향**:

- `openViewer()`와 `openViewerOrReuse()`에서 `options.url`을 `resolveDocumentUrl()`로 변환한다.
- 썸네일 추출은 cache key를 원본 URL로 유지하되 fetch 대상은 resolved URL로 둔다.
  - 같은 페이지에서 카드 상태는 원본 링크 기준으로 관리한다.
  - 실제 네트워크 요청만 raw URL로 보낸다.

**완료 기준**:

- GitHub wiki blob URL이 viewer URL 파라미터에는 raw URL로 전달된다.
- raw URL/일반 URL은 기존 동작 유지.

### Stage 3 — viewer 응답 검증 + 사용자 오류 개선

**변경**:

- `rhwp-studio/src/main.ts`

**구현 방향**:

- `loadFromUrlParam()`의 `response.arrayBuffer()` 이후 `wasm.loadDocument()` 전에 검증한다.
- 직접 fetch 경로와 background `fetch-file` fallback 경로 모두 같은 검증을 통과해야 한다.
- 검증 실패 시 `showLoadError()`로 명확한 메시지를 표시한다.

**예상 helper**:

```typescript
type DocumentByteKind = 'hwp' | 'hwpx' | 'html' | 'unknown';

function detectDocumentByteKind(bytes: Uint8Array, contentType?: string | null): DocumentByteKind;
function assertDocumentBytes(bytes: Uint8Array, contentType?: string | null): void;
```

**완료 기준**:

- HTML 응답을 HWP 파서에 넘기지 않는다.
- 기존 CFB 오류 대신 원인 중심 메시지를 표시한다.
- 로컬 파일 열기와 drag/drop 흐름은 기존대로 `loadBytes()`를 사용한다.

### Stage 4 — 빌드/검증 + 보고서

**검증 명령**:

```bash
node --test rhwp-shared/sw/document-url-resolver.test.js
node --test rhwp-shared/sw/download-interceptor-common.test.js
cd rhwp-firefox && npm run build
cd rhwp-chrome && npm run build
```

**수동 검증**:

- Firefox 확장 dist 로드
- GitHub wiki의 `saved/pr360-edward.hwp` hover 카드 클릭
- viewer 로드 성공 확인
- hover 썸네일 표시 확인
- GitHub raw 링크는 기존처럼 성공 확인
- HTML 응답 URL은 명확한 오류 표시 확인

**문서**:

- `mydocs/working/task_m100_432_stage1.md`
- `mydocs/working/task_m100_432_stage2.md`
- `mydocs/working/task_m100_432_stage3.md`
- `mydocs/working/task_m100_432_stage4.md`
- `mydocs/report/task_m100_432_report.md`
- `mydocs/orders/20260429.md` 상태 갱신

## 4. 위험 요소와 완화

| 위험 | 단계 | 완화 |
|---|---|---|
| symlink가 dist 복사에서 깨짐 | Stage 1/4 | build.mjs의 `dereference: true` 복사 확인 |
| GitHub branch/ref 파싱이 slash 포함 ref를 처리하지 못함 | Stage 1 | 이번 구현은 GitHub URL 구조상 ref segment 1개를 우선 지원, slash 포함 ref는 후속 명시 |
| ZIP 매직만으로 일반 zip을 HWPX로 오탐 | Stage 3 | 기존 viewer도 HWPX ZIP 파서에서 최종 검증하므로 파서 전 게이트는 HTML 차단 목적. 상세 HWPX 구조 검증은 후속 |
| content-type이 `application/octet-stream`인 HTML 오류 페이지 | Stage 3 | content-type뿐 아니라 앞부분 HTML 매직도 검사 |
| Chrome/Firefox API 차이 | Stage 2/4 | 공통 helper는 순수 함수, 연결부는 각 확장별 import만 다르게 확인 |
| dist 갱신 범위가 커짐 | Stage 4 | 빌드 산출물은 필요한 경우만 포함하고 `git diff`로 확인 |

## 5. 검증 방법

### 자동 테스트

- `node --test rhwp-shared/sw/document-url-resolver.test.js`
- `node --test rhwp-shared/sw/download-interceptor-common.test.js`

### 빌드

- `cd rhwp-firefox && npm run build`
- `cd rhwp-chrome && npm run build`

### 수동 확인

- `error1` 재현 링크에서 viewer 로드 성공
- `error2` 원본/raw 링크 동작 유지
- HTML 응답 URL의 사용자 오류 문구 확인

## 6. 구현 후 커밋 단위

하이퍼-워터폴 규칙에 따라 각 Stage 완료 후 완료보고서를 작성하고 승인 요청한다. 단계별 완료보고서는 해당 단계 소스 변경과 함께 타스크 브랜치에서 커밋한다.

예상 커밋:

- `Task #432: Add document URL resolver`
- `Task #432: Validate remote document bytes before parsing`
- `Task #432: Build browser extension dist`
- `Task #432: Add stage reports and final report`

## 7. 승인 요청

본 구현계획서 승인 후 Stage 1에 착수한다.
