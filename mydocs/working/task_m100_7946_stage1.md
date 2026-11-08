# #7946·#7855·#7856 1단계 — 빌드 의존성 호환성 보정

- Issue: [#7946](https://github.com/edwardkim/rhwp/issues/7946), [#7855](https://github.com/edwardkim/rhwp/issues/7855), [#7856](https://github.com/edwardkim/rhwp/issues/7856)
- 승인: 2026-10-16 공동 범위 지정 후 사용자의 “시작하세요” 지시
- 기준: `4662c069e557e928ed7968d061121fde90b1f027`; 브랜치 `task_m100_7946`
- 증적: `output/pr-review/issue7946/dependencies/` (ignored)
- 판정: 의존성 단계의 아래 로컬 검사 충족. PDF UI·최종 통합·CI·이슈 종료는 다음 단계다.

## 1. 원인과 수정 계약

#7855의 Studio 구형 경로는 EJS → Jake → FileList 1 → minimatch 5 → brace-expansion 2였다.
공식 FileList 2.0.2는 minimatch 10 소비 코드로 수정됐으며 Jake의 `FileList` 공개 계약을
보존한다. Jake 하위 FileList를 2.0.2로 갱신하고 Studio·metrics의 호환되는 5.x 경로를
5.0.12로 고정했다. 구형 minimatch에 5.x export를 강제 주입하지 않았다.
공식 registry의 목표 버전·engine·integrity를 조회했고 실제 FileList 파일 선택을 대조했다.

#7856의 최신 Ajv 8.20.0도 fast-uri `^3.0.1`을 요구한다. 공식 배포본의 원 파일 해시·MIT
LICENSE를 `vendor/npm/ajv/`에 보존하고 의존을 4.2.2로 변경했다. 실제 소비하는 세 URI
함수를 전용 어댑터로 연결하여 Ajv codegen 표식을 공용 fast-uri namespace에 쓰지 않는다.
오류를 삼키거나 이전 파서로 되돌리지 않는다. 변경한 upstream 파일은 package manifest,
TS/JS/d.ts 어댑터와 해당 JS의 낡은 map 5개뿐이며 원장으로 대조했다.

두 프로젝트가 로컬 Ajv를 직접 devDependency로 지정하고 Ajv 8에만 `$ajv` override를
적용한다. `.npmrc`의 `install-links=true`로 실제 패키지를 복사 설치한다. Studio의 기존
`legacy-peer-deps` 설정은 보존했다. VS Code의 기존 Ajv 6.14.0 소비 경로도 그대로 유지했다.
일반 상대 file override가 transitive 소비자 기준 경로로 해석되는 npm 오류는 직접 dependency
참조로 해결했다(`studio-ajv-resolve*.log`). 수동 node_modules 변경은 없다.

## 2. 실행 결과

| 검사 | 결과·증거 |
| --- | --- |
| 새 소비 경로 회귀 | `frontend-dependency-compat.test.mjs` 7건, 기존 설치에서 7 FAIL → 새 설치에서 7 PASS. 버전·공용 namespace·URI 거절 계약으로 실패하며 환경 실패가 아니다 |
| Node 지원 대조 | Node 24.15.0 및 22.18.0 각각 7 PASS, `compat-final.log`, `compat-node22.log` |
| URI 타입 계약 | fast-uri 4와 전용 어댑터를 Ajv `Options.uriResolver`에 대입하는 strict TypeScript 검사 PASS (`uri-types.log`) |
| 세 패키지 설치 | 각 `npm ci` 및 재설치 성공. Studio·VS Code의 실제 설치 어댑터가 vendor 소스와 일치. lockfile 버전·출처·integrity 및 `npm ls` invalid 없음 |
| Studio | `npm test`: 2,014 PASS·2 skip, `npm run build`: PWA 2.0.0 및 service worker 생성 성공 |
| VS Code | `npm run compile` 성공. 별도 검증 도구 `@vscode/vsce@4.0.0`의 `package --no-dependencies` 성공, 55파일·18.57 MB |
| metrics | 실제 실행·JSON/summary 생성 성공. 같은 입력의 포함 파일·그룹 합계·파일 메트릭·인지 복잡도·export surface·ESLint 진단 모두 전후 동일 (`metrics-comparison.json`) |

회귀는 brace 대안·중첩·숫자/문자 범위·escape·빈/잘못된 패턴·개수/길이 한도, Jake 파일
선택·제외, Ajv 스키마와 상대/fragment 참조, URI escape·host/port 오류를 실제 설치 경로로 확인한다.
타입 검사 최초 시도는 불필요한 Node type root 부재로 실패해 빈 ambient type root로
교정했다. 패키지 타입 오류나 검사 완화가 아니다. 설치 중의 일시적 모듈 부재 실행은 최종
회귀로 세지 않고 설치가 끝난 뒤 7건을 다시 확인했다.

metrics 실행은 기존 2.6 MB Git 파일 목록이 Node의 기본 1 MiB 수집 한도를 넘어 `ENOBUFS`였다.
파일 선택을 바꾸지 않고 해당 읽기 버퍼만 16 MiB로 명시했다. 변경 전 분석 대조는 기본
작업공간의 이전 의존성에서 동일 버퍼만 보정하는 ignored preload 진단으로 실행했으며,
이를 원래 명령의 성공으로 표시하지 않는다. 실제 수정 명령은 preload 없이 통과했다.
재현·보정 전후 로그와 `metrics-buffer-probe.cjs`를 증적에 보존했다.

현재 frontend 빌드는 기존 #7913에서 Docker로 만든 PDF-on pkg를 재사용했다.
새 Rust capability를 포함한 fresh Docker WASM과 실제 PDF UI 검증을 대신하지 않는다.

## 3. 보안·라이선스 확인과 남은 범위

개발 의존성을 포함한 `npm audit --json` 전후는 Studio 5 → 3, VS Code 3 → 2,
metrics 1 → 0개 패키지 발견이다. 대상 두 패키지는 세 그래프 모두에서 사라졌으며
아래 기존 발견은 이번 범위 밖으로 구분한다. 전체 무취약 또는 실제 악용 불가를 주장하지 않는다.

| 패키지 | 확인 결과 | 근거 |
| --- | --- | --- |
| brace-expansion | 5.0.12 알려진 권고 없음, MIT·설치 lifecycle 없음 | [NPMScan](https://npmscan.com/package/brace-expansion), [공식 릴리스](https://github.com/juliangruber/brace-expansion/releases/tag/v5.0.12) |
| fast-uri | 4.2.2 알려진 권고 없음, BSD-3-Clause·설치 lifecycle 없음 | [NPMScan](https://npmscan.com/package/fast-uri), [공식 릴리스](https://github.com/fastify/fast-uri/releases/tag/v4.2.2) |
| browserslist | 기존 4.28.2 high 발견 유지 | [NPMScan](https://npmscan.com/package/browserslist) |
| baseline-browser-mapping | 기존 2.10.29 moderate 발견 유지 | [NPMScan](https://npmscan.com/package/baseline-browser-mapping) |
| @babel/core | 기존 7.29.0 low 발견 유지 | [NPMScan](https://npmscan.com/package/%40babel%2Fcore) |

FileList 2.0.2와 upstream Ajv 8.20.0도 알려진 권고 없음·각 Apache-2.0/MIT를 확인했다.
로컬 Ajv는 registry 감사만으로 검증되지 않으므로 원본 파일 원장·실제 패치·타입·소비 회귀를
별도로 대조했다. NPMScan의 maintainer/provenance 확인에서 fast-uri는 과거 maintainer 제거
신호가 있지만 새 계정 유입·공식 출처 불일치 증거는 없고 4.2.2 provenance 검사는 통과했다.
기존 browserslist의 provenance 부재 신호도 남겨 두며 침해로 단정하지 않는다.
정확한 원 응답은 `package-*`, `maintainers-*`, `provenance-*`, `before/*-npmscan.json`이다.

Git whitespace 검사는 upstream LICENSE의 끝 빈 줄과 `dist/runtime/quote.js`의 기존 공백
2건을 검출했다. 두 파일은 원본 원장과 바이트가 같아 그대로 보존했고, 그 외 변경 파일의
whitespace 검사는 통과했다(`staged-whitespace.log`). 배포본을 일괄 재포맷하지 않았다.

UI 연결 후 최종 source에서 Studio·빌드·정식 CI 연결을 재확인한다. GitHub push·PR 생성·
이슈 종료는 수행하지 않았다.
