# Task M100 #4739 Stage 5 - WASM·Studio·Chrome 검증

## 범위

Stage 1~3의 exact local face와 단일 repaint 구현을 검증한다. Stage 4의 KoPub/정부상징 layout metric
A/B와 전역 metric 변경은 별도 승인 전이므로 수행하지 않았다.

## 자동 검증

- `npm test`: 호스트 실행 930건 중 929 통과, 1 skip, 실패 0
  - sandbox 실행의 자식 Node 드라이버 6건 무출력 실패는 호스트 재실행에서 모두 통과했다.
- `npm run build`: 통과
- `npx tsc --noEmit`: 통과
- focused TypeScript: 초기화 순서 6/6, 글꼴 해소 9/9, local-font 14/14 통과
- Rust focused: style resolver 29/29, renderer chain·CanvasKit primary family·DocumentInfo 접합면 각
  1/1 통과
- `cargo fmt --check`, `git diff --check`: 통과
- 변경 Markdown의 상대 링크 검사: 이상 없음

## WASM 빌드

- 표준 Docker service는 현재 WSL 배포판에 `docker` 명령이 없어 실행하지 못했다.
- 매뉴얼의 진단용 native 경로인
  `CARGO_TARGET_DIR=target/pr-review wasm-pack build --target web --out-dir pkg --no-opt`는 통과했다.
  첫 sandbox 실행은 Rust wasm32 컴파일 완료 뒤 `wasm-bindgen` 임시 설치 경로가 read-only라 멈췄고,
  동일 명령의 호스트 실행이 성공했다.
- `pkg/rhwp_bg.wasm`과 7700 Vite의 실제 `@fs` 제공 WASM SHA-256은 모두
  `71f5f52db1698c1ae269017d976fd00855af8cf906577389cdcdc297caadbe7a`였다.
- fresh WASM 뒤 production `npm run build`도 다시 통과했다. 이는 표준 Docker 최적화 빌드 통과를
  뜻하지 않는다.

## Chrome CDP - Canvas2D

- Windows Chrome 151, CDP 1.3, local-font permission `granted`에서 확인했다. 계획 당시 Chrome 150은
  검증 중 메인테이너가 Chrome을 재시작하며 151로 바뀌었다.
- 메인테이너가 직접 연 HWP는 383쪽이었다. 환경 설정의 재감지는 KoPub 6 face와 ROKG successor
  1 face를 snapshot에 저장했고, 문서 쪽수와 backend를 유지한 채 `document-view-changed`를 정확히
  한 번 발행했다.
- 별도 검증 탭은 Windows Chrome에 WSL 절대경로를 넘기지 않고 Vite가 제공한
  `/samples/2025 행정업무운영 편람(최종).hwp`와 같은 이름의 `.hwpx`를 percent-encode한 HTTP
  URL로 가져왔다. 응답은 각각 10,687,488 bytes와 13,639,865 bytes였다.
- 저장 snapshot을 가진 새 탭의 첫 HWP load는 383쪽, 추가 감지·repaint 이벤트 0회, 글꼴 prompt
  없음이었다.
- 물리 11쪽 Canvas font setter에서 `KoPub바탕체 Light`가 첫 face이고 proportional serif portable
  chain이 뒤따르는 것을 확인했다. Bold/Medium 및 KoPub돋움체 style face도 각각 첫 face였다.
- ROKG-only인 물리 145쪽 HWPX의 정부상징 chain은 다음 순서를 사용했다.
  `정부상징 부처명_16040911` → `ROKG`/`ROKG R`/`대한민국정부상징체`/
  `대한민국정부상징체 R`/`ROKGR` → 문서 `한컴바탕` → portable sans-serif.

## Chrome CDP - CanvasKit

- 메인테이너의 7700 Vite는 `canvaskit-wasm.js?v=57f6e03c`에 `504 Outdated Optimize Dep`를 반환해
  첫 강제 CanvasKit 시도가 `canvaskitInitializationFailed`로 Canvas2D fallback했다.
- 사용자 서버를 종료하지 않고 별도 7701 Vite dev를 `--force`로 재최적화해 검증했다. 앞서 preview가
  만든 7701 test-origin Service Worker와 Cache Storage만 제거했고 7700 저장소에는 손대지 않았다.
- 7701에서 local-font snapshot 저장 뒤 reload하고 같은 HTTP HWPX를 열었다. 결과는 383쪽,
  `explicitCanvasKit`, fallback 없음, local Typeface 12개, load failure 0, pending 0이었다.
- local Typeface 준비 전 중간 paint의 unregistered fallback은 준비 완료 뒤 0이 되었고, helper가
  `document-view-changed`를 한 번 발행했다. 물리 145쪽 최종 진단은 render error 0, 예상 밖
  unsupported op 0, readiness gate 통과였다.

## 판정

자동·접합면 검증은 통과했다. 메인테이너는 7701 CanvasKit 탭의 물리 145쪽을 직접 확인하고 최종 시각
판정도 통과로 선언했다. Stage 4 metric 결정은 별도 승인 대상으로 보류 상태를 유지한다.
