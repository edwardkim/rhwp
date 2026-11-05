# #8614 Studio 웹폰트 초기화 대기 제한

Issue: [#8614](https://github.com/edwardkim/rhwp/issues/8614).
계획: [웹폰트 초기화 대기 제한](../plans/studio_web_font_deadline_impl.md).
작업 브랜치: `task/8614-studio-web-font-deadline`.
기준: `e96b500e3ea259c821f05440f0388355916a9e9b` — 2026-11-05 원격 devel과 일치.
#8104는 완료 상태를 유지하며 이번 결함과 분리한다.

## 원인과 구현

기존 `initialize → loadWebFonts → FontFace.load`는 외부 응답이 없으면 WASM 생성 이전에
멈췄다. 고정한 base 로더를 실제 Chrome에 주입하고 외부 요청을 미완료로 유지하자 30초 후에도
초기화가 끝나지 않았다. 자연 발생 네트워크 장애의 구체적 원인까지 확정한 것은 아니다.

인계 후보의 Promise deadline·CSS 제거만으로도 충분하지 않았다. 실제 요청은 계속 살아 있었고
문서의 load 완료를 막아 `prepareCanvasMetrics → document.fonts.ready`에서 다시 대기했다.
URL FontFace를 document.fonts에 등록하지 않은 최소 재현에서도 같은 현상을 확인했다.
이는 [CSS Font Loading의 환경 대기 조건](https://www.w3.org/TR/css-font-loading/#fontfaceset-pending-on-the-environment)과 연결된다.

[공통 로더](../../rhwp-studio/src/core/font-loader.ts)는 다음을 처리한다.

- 파일별 fetch와 바이트를 공유한다. 요청 시작부터 FontFace 완료까지 5초로 제한하며,
  응답 헤더 이후 본문 대기도 포함한다. 네트워크 제한 시 AbortController로 실제 요청을 취소한다.
- 받은 바이트로 FontFace를 생성하고 완료한 face만 등록한다. CSS도 같은 바이트의 blob URL을
  사용하므로 별칭·CSS가 원격 URL을 다시 요청하지 않는다.
- 실패 파일은 페이지를 다시 열 때까지 자동 재시도하지 않는다. 같은 파일의 모든 별칭,
  CSS 등록, 등록된 FontFace와 blob 자원을 해제한다. 늦은 완료로 공급을 다시 살리지 않는다.
- 외부 비활성 옵션은 요청 전에 적용하고 로컬 자원의 무응답에도 같은 제한을 적용한다.
- 동시 face 요청과 파일 요청을 각각 공유한다. 파일별 배치 크기는 기존 4개를 유지한다.
  5초는 개별 요청의 제한이며 여러 배치가 필요한 문서 전체의 5초 완료 보장은 아니다.

공급한 바이트 → FontFace/CSS → `WasmBridge.prepareCanvasMetrics` →
`CanvasMetricSession.prepare(document.fonts.ready)` → Canvas 측정·실제 paint의 소비를 확인했다.
metrics 대기를 생략하거나 틀린 좌표를 보정하지 않는다. 글꼴 registry의 파일·family·format·
unicode-range, 치환 후보 순서, Rust 메트릭·LineSeg·pagination·각 backend의 paint 규칙은 바꾸지 않았다.

배포 호스트가 CSP를 별도로 제한하면 웹폰트 출처의 `connect-src`와 CSS blob의 `font-src blob:`
허용을 확인해야 한다. 기존 Studio 및 브라우저 확장 설정에는 이 경로를 막는 제한이 없다.
VS Code는 이 로더의 CanvasKit 계획 함수만 가져오며 웹폰트 로드 함수를 사용하지 않는다.
새 외부 출처나 영구 저장소는 추가하지 않았다.

## 검증

2026-11-05 KST, macOS의 Node v24.15.0과 실제 headless Chrome에서 실행했다.
이 보고서와 함께 커밋하는 소스의 검사이며, 이전 실행·현재 실행을 구분한다.

| 검사 | 결과 |
| --- | --- |
| TypeScript `tsc --project rhwp-studio/tsconfig.json --noEmit` | PASS |
| `npm --prefix rhwp-studio test` | **2,734 PASS**, 실패·건너뜀 0 |
| `npm --prefix rhwp-studio run build -- --configLoader runner --outDir ../output/pr-review/web-font-deadline/resumed/dist --emptyOutDir` | PASS |
| `npm --prefix rhwp-studio run e2e:web-font-deadline` | 아래 7개 시나리오 PASS |
| 수정 전 base, 동일 네트워크 무응답 | 30초 초기화 timeout으로 FAIL |
| 수정 전 base, 정상 응답 대조군 | PASS; 후보와 Canvas·배치 해시 및 측정값 동일 |

브라우저 검사는 mock FontFace 성공만 확인하지 않는다. 실제 HTTP 요청을 보류·실패시키고,
공개 MessagePort `loadFile`로 새 글꼴이 필요한 문서를 연 뒤 후속 편집·Undo를 실행한다.
실제 초기화가 소비한 WASM 응답도 복사본의 SHA-256으로 확인했다.

| 시나리오 | 초기화 ms | 문서 열기 ms | 확인 |
| --- | ---: | ---: | --- |
| 정상 응답 | 893 | 362 | 공급 적재, 내용 보존, 편집·Undo |
| HTTP 404 | 502 | 329 | 실패 파일 제거, 재요청 없음 |
| 외부 네트워크 미응답 | 5,488 | 5,311 | 세 요청 실제 취소, 초기화·열기 완료 |
| FontFace 늦은 성공 | 5,479 | 5,311 | 뒤늦은 등록·재측정·paint 없음 |
| FontFace 늦은 실패 | 5,481 | 5,313 | 처리되지 않은 rejection 없음 |
| 외부 비활성 | 495 | 329 | 외부 요청 0, 로컬 공급 유지 |
| 로컬 자원 미응답·외부 비활성 | 5,503 | 327 | 로컬 요청 취소, 이후 재시도 없음 |

[단위 검사](../../rhwp-studio/tests/font-loader-deadline.test.ts)는 4,999/5,000ms 경계,
동시 요청, 별칭의 부분 성공 후 실패, 늦은 별칭 성공/실패, 생성자 동기 실패,
네트워크·본문 각각의 미응답, abort를 무시하는 fetch의 늦은 완료, 공유 blob 해제를 검사한다.
[기존 정상·비활성 검사](../../rhwp-studio/tests/font-loader-offline-mode.test.ts)의 OS 글꼴 우선,
문서별 선별 공급, WOFF/WOFF2 및 별칭 보호는 유지했다. 검사 기대값의 변화는 URL 직접 로드 대신
실제로 요청한 파일의 바이트와 CSS format을 관측하는 데 한정했다.

## 정상 출력과 실패 fallback의 구분

브라우저의 OS 탐지 probe만 동일 폭으로 고정했다. 실제 Canvas 측정·본문 paint는 수정하지 않았다.
외부 응답은 저장소의 라이선스 확인된 `NotoSerifKR-Regular.woff2`로 공급했다.
이는 네트워크·수명 계약용이며 원래 CDN 서체와 동일한 외형이라는 증거는 아니다.

- 정상 base/후보: 동일 Canvas hash
  `d570db7a2fcb3b4d939821a5ad12ad41b2dcbf95739f0d730298522706ccb80e`,
  동일 tree hash `8734add8449218fb1d6aca2df09b313bca51a1814cff501f5ac6b157f0d16d70`.
  같은 문자열의 Canvas 측정값은 양쪽 모두 221.2998504638672px였다.
- 후보의 HTTP 실패·네트워크 timeout·늦은 성공/실패·외부 비활성 대조군:
  Canvas hash `ba0e27164d479067cdaf567f98b1f04dcec22104469b2cdf08039a94fbba8fc7`,
  tree hash `1595cec4f3dc42d4450871b1dc053c6afd157d4f64bf4e7531a5faf296821049`.
  측정값은 모두 194.73985290527344px였다.
- 이 해시는 이번 실행의 대조 증거이며 회귀의 고정 golden으로 넣지 않았다.
  정식 검사는 같은 실행에서 늦은 완료가 기존 공급·측정·배치·Canvas를 바꾸지 않는 관계를 검사한다.
- 실제 화면 캡처에서 정상 공급과 fallback의 서체 차이를 직접 확인했다. 두 경로 모두
  영문·한글 내용이 빠지지 않았으며, 늦은 완료 전후에도 해당 화면을 유지했다.
  정상 공급과 fallback을 동일한 글꼴 출력이라고 보고하지 않는다.

이번 변경은 공급 실패의 수명과 초기화 완료 계약을 수정하며 조판 규칙·정상 공급의 글꼴 선택을
변경하지 않는다. Native/CanvasKit 공급 계획은 이 바이트 로더를 소비하지 않는다.
새 실물 조판 fixture·절대 좌표·페이지 수 golden은 추가하지 않았으며, 한컴 Print 일치율 개선이나
Native/fresh WASM PDF 실루엣 통과를 주장하지 않는다. 독립 PDF 비교를 합성 수명 검사로 대체한
것이 아니라, 위 정상 동작 보존과 실패 완료 계약을 검증한 범위다.

## 빌드 출처와 증거

인계된 `wasm-build.log`의 release·wasm-opt 성공 및 이전 실행 exit 0을 실물과 대조했다.
WASM 생성 시점의 HEAD는 `9a54d8ac280645dc83bd984dd612ef4a999ae824`이고,
해당 HEAD와 현재 작업 사이 Rust/WASM 소스 변경은 없다. 불필요한 재빌드 없이 재사용했다.

- JS: `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393`
- WASM: `1e644ae4fb3038e87722159300628c6b624549031ef02d888ec1f68124c79d13`
- 테스트 공급 글꼴: `c9a1e3ac69994d680883b53b1fd83d520733b704792522a932bfaa276117de45`
- 로더 소스: `ec8413b99d133b6d9e4a782405e3bdf251d299877f65012a21c29f158e1a80a0`

`pkg`와 Studio public의 JS·WASM 해시, 각 브라우저 초기화의 실제 응답 해시가 일치한다.
초기 인계 전체 테스트의 API 누락은 낡은 WASM 때문이었으며 fresh 산출물로 전체 재실행해 해소했다.
이후 브라우저 하네스의 T7 실제 경로 허용 및 큰 WASM 응답의 해시 관측도 보완했다.

임시 증거는 T7로 라우팅된 `output/pr-review/web-font-deadline/` 아래에 보존한다.
`resumed/npm-final-2734.log`, `typecheck-final.log`, `build-final.log`, `browser-final.log`,
`base-network-pending.log`, `base-success-final.log`와 `browser/results.json`,
`browser/*-before.png`·`*-after.png`가 현재 결과다.
이전 실패 로그도 원인 추적을 위해 보존하며 통과 로그로 바꾸어 부르지 않는다.

## 전달 상태

로컬 구현·위 범위의 검증을 완료했다. 본인 계정의 열린 upstream PR은 조회 당시 없었다.
remote push, PR 생성, CI, merge, 이슈 close, release는 아직 수행하지 않았다.
기존 #8104 문서 3개의 미커밋 표현 변경은 별도로 보존하고 이번 코드 커밋에 포함하지 않는다.
정식 배포 포함 여부는 이후 release tag와 resolved commit에서 별도로 확인한다.


## 최신 devel 반영과 제출 검증 — 2026-11-05

기존 base `e96b500e3ea259c821f05440f0388355916a9e9b`에서
`a95d1d5afad1bfd61998794fa78fb0db100d690a`로 충돌 없이 rebase했다.
검증한 코드 head는 `bd64bf13c069898b998db6e7199589914115696f`이며, 이후 변경은 이 검증 기록뿐이다.
기존 #8104 문서 세 개의 미커밋 변경은 보존했다.

- 최신 소스의 locked release WASM + wasm-opt 빌드: PASS, 5분 31초.
- TypeScript, 전체 npm 검사 **2,737 PASS / 실패·건너뜀 0**, production 빌드,
  E2E manifest **186/186**: PASS.
- 실제 Chrome의 폰트 수명 7개 시나리오: PASS. 외부 요청 무응답 시 초기화 5,543ms,
  문서 열기 5,312.8ms이며 후속 입력·Undo까지 완료했다.
- 최신 base의 원본 로더는 같은 무응답 조건에서 30초 후에도 state=null로 초기화 실패했다.
  정상 응답의 base/후보 Canvas·배치 해시와 측정값은 동일했다.
- 새 WASM SHA-256: `6967c416ed06dc9f226f135a23d550f59e8122911a9c6ab5361630492666a3ef`.
  동일 패키지를 base/후보 브라우저에 공급하고 실제 응답 해시를 확인했다.

리베이스 기능 보존 범위: #8630의 묶음 글상자 클릭·입력·Undo·Esc 복귀는
기존 `issue-8578-group-text-editing.test.mjs`의 실제 편람 383쪽 Canvas2D 여정으로
최신 base와 후보를 대조했다. 두 실행의 소유 위치·내용·오류 목록·쪽수 결과가 동일했다.
공용 실행기의 잔여 Node 핸들 때문에 검증 전용 래퍼에서 `runTest` 완료를 명시적으로
기다리도록 했으며 assertion과 제품 코드는 바꾸지 않았다.

#8609·#8622 통합의 Rust 편집·저장·조판과 Native/CanvasKit paint 변경은 이 PR에서
수정하지 않는다. 이 PR의 `loadFontResource`는 DOM FontFace/CSS 공급만 소비하며,
CanvasKit의 `resolveCanvasKitFontPlan`과 Native 공급 경로는 이를 호출하지 않는다.
새 devel의 CanvasKit 아래 일점쇄선 분기도 이 바이트 로더를 소비하지 않는다.
문서 분류 자동화 #8618 및 후속 기록 #8633·#8634는 Studio 런타임과 무관하다.
전체 회귀와 최신 WASM 브라우저 실행을 위 증거로 기록하며, 해당 통합 PR들의 독립 PDF
시각 검증을 이번 작업에서 다시 수행했다고 주장하지 않는다.

추가 증거: `output/pr-review/web-font-deadline/rebased/`의 `wasm-build.log`,
`typecheck.log`, `npm-test.log`, `build.log`, `browser.log`, `base-success.log`,
`base-pending.log`, `group-base-final.log`, `group-candidate-final.log`.
2026-11-05 사용자가 push와 Open PR 제출을 승인했다. 제출 직전 본인 열린 PR은 없었다.
