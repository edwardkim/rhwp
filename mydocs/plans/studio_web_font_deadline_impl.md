# Studio 웹폰트 초기화 대기 제한 구현 계획

기준: `e96b500e3ea259c821f05440f0388355916a9e9b`.
이슈: [#8614](https://github.com/edwardkim/rhwp/issues/8614).
#8104는 완료된 편집 오류 계약이며 이번 초기화 결함과 분리한다. #8614가 이미 등록되어 있으므로
중복 이슈를 만들지 않는다. 특정 호스트 전용 처리를 추가하지 않는다.

인계 브랜치: `codex/studio-web-font-deadline`, 계획 커밋:
`9a54d8ac280645dc83bd984dd612ef4a999ae824`. 사용자 지시에 따라 현재 checkout과 미커밋 수정을
먼저 보존했다. 작업 브랜치는 `task/8614-studio-web-font-deadline`로 분리하며 기존 브랜치는
보존한다. 2026-11-05 원격 `upstream/devel`도 위 기준 SHA와 같음을 확인했다.

## 원인과 선행 구현

`main.ts initialize → await loadWebFonts → FontFace.load`가 무기한 pending이면 WASM/InputHandler
생성까지 도달하지 않는다. 문서 열기의 같은 로더도 영향을 받는다. 전달받은 실제 호스트 실험은
외부 URL의 FontFace 응답을 pending으로 주입했을 때 60초 뒤에도 초기화 상태가 null인 사례다.
자연 발생 네트워크 지연의 원인까지 확정한 것은 아니다.

#4823 → PR #4857 (`9a40da8818c44d3a12df18be6dbf80f9b54426b3`)은 요청한 글꼴만 공급하고
실패해도 fallback으로 계속 진행하는 계약이다. #3585 → #3586의 외부 웹폰트 비활성 옵션,
#8191 이후 공통 registry projection과 현재 base의 로컬 face 초기화 계약을 보존한다.
최신 base에서 로더의 무제한 await는 그대로이며 제공된 후보를 그대로 이식하지 않고 검토한다.

## 구현·검증

- 요청 시작부터 FontFace 완료까지 5초 deadline, timeout/실패 파일은 같은 페이지에서 자동 재시도하지 않는다.
  재시도는 페이지 재시작으로 하며 전역·영구 캐시는 만들지 않는다.
- 실패 파일의 CSS 등록과 공급 상태를 일치시키고, 늦은 성공은 등록하지 않는다.
- 같은 face의 동시 요청을 공유하고 같은 파일 별칭의 실패 경합도 점검한다.
- 정상 응답, 실패, 무응답, 늦은 성공/실패, 반복·동시 호출, 별칭, 외부 비활성 및 로컬 face를 검사한다.
- TypeScript·전체 npm·실제 브라우저 초기화/문서 열기를 검증한다. 외부 설치 글꼴에 의존하지 않는다.
- Rust 메트릭·fallback 순서는 수정하지 않는다. 실패 공급 제거의 Canvas paint/metrics 소비 영향을
  추적하고 정상 대조군 및 적용되는 시각 검증을 완료한 뒤 PR 제출 여부를 판단한다.
- 결과는 새 code head 및 최종 PR/merge SHA·release 포함 여부와 구분해 기록한다. 릴리즈는 발행하지 않는다.

다른 작업의 검증 자료와 소스는 보존하며 새 산출물은 T7로 연결된 output 아래 별도 경로에 둔다.

## 2026-11-05 인계 확인과 재개 조건

- #8614의 본문과 현재 HEAD·diff를 확인했다. 로더·단위 검사·브라우저 검사·E2E manifest의
  미커밋 구현은 인계 상태로 보존했다. #8104 관련 문서 3곳의 표현 수정은 별도 기존 변경이다.
- `/Volumes/T7`가 연결되지 않았다. `output`, `pkg`, `target`의 기존 라우팅 symlink는 유지했다.
  로컬 대체 디렉터리를 만들지 않았으며 빌드·검증을 실행하지 않았다.
- 이전 집중 검사 29 PASS와 WASM wrapper exit 0은 인계 보고다. 이번 작업에서 산출물·로그를
  확인하지 못했으므로 현재 검증 성공으로 재기록하지 않는다. 연결 후 실행 중인 작업과 원본 SHA,
  `pkg`/Studio public 해시·빌드 출처를 확인하고 재사용 가능 여부를 판단한다.
- 실제 호출 위치는 `main.ts`의 `initialize → loadWebFonts → wasm.initialize` 및
  `initializeDocument → loadWebFonts → canvasView.loadDocument`다. 두 경로를 모두 검사한다.
- 기존 브라우저 검사는 초기화 상태만 확인한다. 문서 열기·후속 편집과 실패 fallback 결과를 추가로
  확인해야 한다. 성공 모드의 테스트 글꼴 치환은 네트워크·등록 계약 증거이며 실제 글꼴 외형의
  일치 증거가 아니다.
- 브라우저 검사의 `document.fonts.check = () => false`만으로 설치 글꼴 영향을 제거할 수 없다.
  `isSystemFontAvailable`은 body가 있으면 DOM 폭 비교를 먼저 사용한다. OS 감지 대조군을 보존하면서
  무응답 대상이 실제 요청됐는지 입증하도록 검사 환경을 보완한다.
- E2E package script 등록, 동시 별칭 실패 경합, 늦은 성공/실패, 정상 공급과 실패 공급의 실제
  Canvas 소비·메트릭 영향, TypeScript·전체 npm·production build·브라우저 및 적용되는 시각 검증이
  남아 있다. 완료·PR 제출·merge·release 포함을 선언하지 않는다.

## 재개 후 확인한 요청 수명 결함과 보완

T7 재연결 후 기존 WASM wrapper의 release/wasm-opt 완료 로그와 이전 실행의 exit 0을 확인했다.
현재 Rust 소스는 당시와 동일하며 `pkg`와 Studio public의 JS·WASM SHA-256도 일치한다.
원래 전체 테스트의 누락 API 오류는 해당 fresh WASM으로 재실행해 해소했다.

인계 후보는 `FontFace.load()` Promise만 제한했다. 실제 HTTP 응답을 미완료로 두자 후보에서도
로더 반환 후 `페이지 렌더 준비 중`에 남았다. URL FontFace만 요청하고 등록하지 않은 최소 재현도
document `readyState=interactive`, load 미발생, `document.fonts.ready` 미완료였다.
[CSS Font Loading 사양](https://www.w3.org/TR/css-font-loading/#fontfaceset-pending-on-the-environment)은
문서 로딩을 FontFaceSet ready의 환경 대기 조건으로 정의한다. CSS 삭제만으로 네트워크 요청을
취소할 수 있다는 전제를 제거한다.

- 공통 로더에서 AbortController를 가진 파일별 fetch를 공유하고 응답 본문까지 deadline에 포함한다.
- 받은 바이트로 FontFace를 만들고, 로드 완료한 face만 document.fonts에 등록한다.
- CSS는 같은 바이트의 페이지 수명 blob URL을 사용한다. 성공 이전의 원격 CSS 요청 및
  cache-control과 무관한 별칭 재요청을 방지한다. 실패 시 모든 별칭·CSS·blob 자원을 함께 해제한다.
- 외부 비활성 옵션은 fetch 이전에 적용한다. 로컬 파일의 무응답에도 같은 deadline을 적용한다.
- 바뀌는 값은 공급의 완료/실패 수명이다. registry의 파일·family·format·unicode-range,
  fallback 후보 순서, Rust 메트릭·LineSeg·분할·paint 알고리즘은 그대로다.
- 소비 경로는 `loadWebFonts → document.fonts/CSS → prepareCanvasMetrics →
  CanvasMetricSession.prepare(document.fonts.ready) → Canvas 측정/실제 paint`다.
  URL 요청을 계속 남긴 채 metrics 대기만 건너뛰는 보정은 하지 않는다.
- 정식 E2E는 실제 네트워크 취소, 공개 MessagePort loadFile, 후속 편집/Undo 및 늦은 완료 전후의
  공급·측정·배치·Canvas 보존을 검사한다. 합성 입력의 수명 계약과 한컴 조판 일치는 구분한다.

최신 제출 기준은 `a95d1d5afad1bfd61998794fa78fb0db100d690a`이며, 리베이스 후 검증은
[최종 보고서](../report/task_m100_8614_report.md)의 최신 devel 반영 절에 기록한다.
