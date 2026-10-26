# #8153: 실제 Native·브라우저 회귀 실행 연결

## 확인한 누락

고정 base `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`와 현재 후보의
`.github/workflows/ci.yml`은 같다. 사용자 지시에 따라 upstream을 fetch하거나 동기화하지 않는다.
기존 Native job은 `native-skia` feature에서 명시한 논리 case를 실행하며, 기본
nextest suite의 수집만으로 feature 전용 검사가 실행되지는 않는다.

이번 작업의 `issue_8153_opaque_mixed_axis_image_sampling` 2건,
`issue_8153_print_glyph_geometry` 3건, `issue_8153_symbol_fallback` 1건은
기존 Native 호출 목록에 없다. 실제 Native 129건의 목록과 통과로 검사 본체의
실행 가능성은 확인했지만 자동 CI 배선이 있다는 뜻은 아니다.
나머지 새 일반 검사는 기존 suite 수집 경로를 유지한다.

실제 WASM의 `scripts/tests/print-text-geometry.browser.mjs` 1건은 단독 실행으로
통과했지만 CI의 `visual-sweep-canvas-fonts.browser.mjs`에서 import하지 않는다.
이 브라우저 검사는 기존 picture-tones/HFT 검사와 별개의 Screen/Print 기하 계약이다.

## 최소 연결과 검증

기존 Native job의 release-test/release 분기 각각에 위 세 논리 case만 연결한다.
기존 selector·feature·실행 프로필을 사용하고 새 job·step·target을 만들지 않는다.
두 프로필 분기는 상호 배타적이므로 실행당 신규 6건이며 중복 실행이 아니다.
기존 브라우저 진입점에는 원래 검사 1건을 import한다. sample16 관련 기존 삭제는 유지한다.

Node selector의 실제 target/필터 출력과 feature-gated 원문·129건 실행 목록을 연결한다.
workflow 구문과 기존 CI 계약 검사를 실행하고, Cargo 공유 작업이 끝난 뒤 실제로
기존 CI와 같은 논리 case 명령을 공용 target에서 순차 실행해 0건/skip을 거부한다.
표준 fresh WASM을 사용하는 실제 브라우저 공용 진입점도 실행한다.
검사 연결은 새 임계값·골든·허용치 변경이 아니며 전쪽 100%를 대신하지 않는다.

CI 원격 실행은 PR 승인·제출 뒤의 단계다. 이번에는 로컬 필수 검증과 PR 준비까지만
진행하며 GitHub 게시·push·PR 생성·upstream 동기화를 하지 않는다.
