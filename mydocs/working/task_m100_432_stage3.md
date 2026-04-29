---
타스크: #432 브라우저 확장 실제 HWP/HWPX 응답 검증 및 파일 URL 해석 계층 도입
단계: Stage 3 — viewer 응답 검증 + 사용자 오류 개선
브랜치: local/task432
작성일: 2026-04-29
---

# Stage 3 완료 보고서

## 1. 작업 요약

원격 URL 로드 경로에서 fetch 응답을 WASM HWP 파서에 넘기기 전에 실제 문서 바이트인지 확인하는 검증 계층을 추가했다.

이번 변경은 특정 GitHub 링크만 예외 처리하는 방식이 아니라, 모든 `?url=` 기반 원격 로드에 대해 다음 일반 문제를 차단한다.

- URL 확장자는 `.hwp`/`.hwpx`처럼 보이지만 실제 응답은 HTML 미리보기 페이지인 경우
- 서버 오류/권한/미리보기 페이지가 `200 OK`로 반환되어 파서가 CFB 오류를 내는 경우
- 직접 fetch 실패 후 service worker `fetch-file` fallback에서 받은 바이트가 실제 문서가 아닌 경우

## 2. 변경 파일

| 파일 | 변경 내용 |
|---|---|
| `rhwp-studio/src/main.ts` | 원격 응답 바이트 판정 helper 추가 |
| `rhwp-studio/src/main.ts` | `loadFromUrlParam()` 직접 fetch 경로에서 `content-type`과 바이트 시그니처 검증 |
| `rhwp-studio/src/main.ts` | service worker `fetch-file` fallback 경로에서도 동일 검증 적용 |
| `rhwp-studio/src/main.ts` | `Error:` 접두어가 그대로 노출되지 않도록 오류 메시지 정리 및 토스트 표시 |

## 3. 판정 정책

| 종류 | 판정 |
|---|---|
| HWP | CFB 매직 `D0 CF 11 E0 A1 B1 1A E1` |
| HWPX | ZIP 매직 `50 4B 03 04`, `50 4B 05 06`, `50 4B 07 08` |
| HTML/오류 페이지 | `content-type: text/html`, 또는 앞부분이 `<!doctype`, `<html`, `<?xml` |
| 기타 | 실제 HWP/HWPX 파일 여부를 확인할 수 없는 응답으로 차단 |

## 4. 사용자 오류 메시지

기존에는 HTML 응답이 파서로 들어가 다음과 같은 내부 오류로 보였다.

```text
파일 로드 실패: 유효하지 않은 파일: CFB 오류
```

이제 HTML/미리보기 응답은 파서 호출 전에 다음처럼 원인 중심 메시지로 표시된다.

```text
파일 로드 실패: 실제 HWP/HWPX 파일이 아닙니다. 파일 미리보기/오류 페이지가 반환되었습니다.
```

## 5. 검증

### 통과

```bash
node --test rhwp-shared/sw/document-url-resolver.test.js
```

- 12개 테스트 통과

```bash
node --test rhwp-shared/sw/download-interceptor-common.test.js
```

- 26개 테스트 통과

### 확인 필요

```bash
cd rhwp-studio
npm run build
```

현재 빌드는 이번 Stage 3 변경부가 아니라 기존 생성 WASM 타입과 TypeScript 래퍼 사이의 불일치로 실패한다.

```text
src/core/wasm-bridge.ts(133,21): error TS2551: Property 'exportHwpx' does not exist on type 'HwpDocument'. Did you mean 'exportHwp'?
src/core/wasm-bridge.ts(137,22): error TS2339: Property 'getSourceFormat' does not exist on type 'HwpDocument'.
```

Rust 소스의 `src/wasm_api.rs`에는 `exportHwpx`, `getSourceFormat` wasm export가 존재하므로, 현재 로컬 `pkg/` 생성물이 최신 Rust WASM API와 맞지 않는 상태로 보인다. Stage 4에서 WASM 패키지 재생성 또는 빌드 산출물 갱신과 함께 재확인한다.

## 6. 완료 기준 대비

| 기준 | 결과 |
|---|---|
| HTML 응답을 HWP 파서에 넘기지 않는다 | 완료 |
| 기존 CFB 오류 대신 원인 중심 메시지를 표시한다 | 완료 |
| 직접 fetch와 service worker fallback 모두 검증한다 | 완료 |
| 로컬 파일 열기/drag-drop 흐름은 기존대로 유지한다 | 완료 |

## 7. 다음 단계

작업지시자 승인 후 Stage 4에서 확장 빌드와 수동 검증 절차를 진행한다.
