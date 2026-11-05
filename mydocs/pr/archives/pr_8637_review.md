# PR #8637 리뷰 — 웹폰트 요청 수명과 초기화 복구

## 최종 판정

**승인 — 검토한 범위에서 수정이 필요한 결함을 발견하지 않았다.**
작성자 self-review이며 GitHub의 독립 reviewer 승인이나 merge를 뜻하지 않는다.
검토 기준은 `9321d60930a8c22357e919cd44ff66e3b66015fe`다.
사용자가 Firefox 검증 결과의 PR 본문 반영과 리뷰 문서 push를 승인했다.
이 기록은 코드 변경 없는 trailing 문서이며, 새 head CI 확인은 push 뒤 수행한다.
GitHub 승인 리뷰와 merge는 수행하지 않았다.

## 접수와 범위

- PR: [#8637](https://github.com/edwardkim/rhwp/pull/8637), 작성자 `postmelee`.
- Base: `a95d1d5afad1bfd61998794fa78fb0db100d690a`, 대상 `devel`.
- 2026-11-05 확인 시 Open, non-draft, MERGEABLE이며 실패·대기 check가 없었다.
- 관련 이슈: [#8614](https://github.com/edwardkim/rhwp/issues/8614).
- 검토 범위는 파일별 fetch 공유·AbortController, FontFace deadline,
  실패 파일의 별칭·CSS·face·blob 정리, 늦은 완료, 반복 호출과 외부 비활성 경로다.

## 코드 검토

`loadFontResource`의 바이트 요청은 헤더와 본문 대기를 함께 제한한다.
`loadFontFace`는 등록 전에 실패 파일 상태를 다시 확인하며, 시간 제한에서 진 작업의
늦은 성공은 등록 경로로 복귀하지 않는다. 같은 파일의 바이트와 같은 별칭의 Promise를
각각 공유한다. `removeFailedFontFile`은 기존 공급을 함께 제거한다.
CSS는 성공한 바이트의 blob URL만 사용하므로 원격 FontFace 요청을 다시 시작하지 않는다.

기존 글꼴 registry와 치환 순서, 로컬 글꼴 감지, CanvasKit 공급 계획 함수의 구현은 유지한다.
외부 비활성 경로는 요청 후보를 미리 걸러낸다. 정상 공급과 fallback을 동일한 외형이라고
해석하지 않았으며, 페이지 수명 동안 실패 파일을 재시도하지 않는 정책은 PR에 명시되어 있다.

## 검증 결과

- 검토 시 집중 검사 재실행: `node --test rhwp-studio/tests/font-loader-deadline.test.ts
  rhwp-studio/tests/font-loader-offline-mode.test.ts` — 16 PASS, 실패·건너뜀 0.
- 동일 코드의 제출 전 검증: TypeScript, production 빌드, 전체 npm 2,737 PASS,
  최적화 WASM, 실제 Chrome 7개 시나리오와 E2E manifest 186/186 PASS.
- 최신 base 원본 로더의 무응답은 30초 초기화 실패, 후보는 약 5.5초 후 초기화 완료.
  정상 응답의 base/후보 Canvas·배치 해시와 측정값은 일치했다.
- 리베이스로 들어온 #8630의 실제 묶음 글상자 클릭·입력·Undo·Esc 여정은 base/후보
  양쪽에서 통과했고 결과가 동일했다. 코드 SHA·입력·소비 경로의 상세 구분은
  [작업 보고서](../../report/task_m100_8614_report.md)의 최신 devel 반영 절을 따른다.
- `git diff --check` PASS.
- [CI / Build & Test](https://github.com/edwardkim/rhwp/actions/runs/42006131581)
  및 Frontend package gates SUCCESS.
- [Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/42006131443),
  [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/42006132020),
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42006131647),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/42006131714)의 실행 대상 job SUCCESS.
  정책상 SKIPPED인 Rust/WASM job을 새 실행 성공으로 표기하지 않는다.

## 조판 원칙과 검증 한계

줄 구성·측정 계산·배치·분할·저장 LineSeg 수용 조건 변경은 비해당이다.
DOM 글꼴 공급의 수명 계약을 수정하며 좌표 보정이나 문서별 예외를 추가하지 않는다.
새 테스트는 공급 상태와 완료·내용 보존 관계를 검사하며 실물 페이지의 절대 좌표나 golden을
추가하지 않는다. 한컴 Print PDF 피델리티 개선이나 전쪽 실루엣 통과를 주장하지 않는다.

최초 검토 이후 Firefox 159.0.1에서 아래 추가 검증을 완료했다. Safari 실제 실행은 포함되지 않는다. 공식 확장 manifest의
CSP는 확인했으나 각 확장 설치 실행까지 검증했다고 보지 않는다. 사용자 지정 CSP에는
폰트 출처의 `connect-src`와 CSS의 `font-src blob:` 허용이 필요하며 PR에 안내되어 있다.

## 후속 조건

원격 리뷰 게시·문서 trailing push·merge는 별도 단계다. head가 바뀌면 변경 범위와 최신 CI를
다시 확인한다. 본인 PR이므로 이 검토만으로 독립 승인을 충족했다고 처리하지 않는다.


## Firefox 추가 검증 — 2026-11-05

동일 head와 base의 원본 로더를 Firefox 159.0.1 headless에서 비교했다.
검증 프로필·하네스·로그는 T7의 `output/pr-review/web-font-deadline/firefox/`에 보존했다.

- Base 정상 응답: 초기화·문서 열기·입력·Undo PASS.
- Base 네트워크 무응답: 실제 HTTP 서버에 연결한 URL FontFace가 완료되지 않아
  30초 뒤에도 state=null. 요청 interception과 실제 HTTP 서버 양쪽에서 재현했다.
- 후보 7개 시나리오 모두 PASS. 외부 무응답 초기화 5,440ms, 문서 열기 5,159ms.
  실제 HTTP 서버의 응답 연결 close 이벤트로 클라이언트 취소를 확인했다.
- 정상 응답의 base/후보 배치·측정값 동일, 대표 캡처의 디코딩된 픽셀 차이 0.
  Canvas data URL 문자열의 해시는 달랐으므로 그 해시의 동일성을 주장하지 않는다.
- 후속 입력·Undo, 실패 후 재요청 없음, 늦은 성공·실패 후 공급·배치·화면 유지 확인.
- Chrome용 하네스를 그대로 옮긴 첫 실행은 Firefox BiDi의 interception 요청에 대한
  `requestfailed` 관측 assertion에서 실패했다. 제품 초기화·문서 열기는 진행됐으며,
  실제 HTTP 서버의 연결 종료 관측으로 대체한 검증에서 취소를 직접 확인했다.
  제품 코드와 정식 테스트의 assertion을 약화하거나 수정하지 않았다.

핵심 증거는 `verification.json`, `candidate-http.log`, `base-http.log`다.
이 결과는 Firefox 웹 Studio 검사이며 Firefox 확장 설치 검증이나 자연 발생 CDN 장애
원인 확정을 의미하지 않는다.
