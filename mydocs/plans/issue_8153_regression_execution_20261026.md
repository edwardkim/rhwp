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

## 2026-10-29 전체 회귀 실패 재검토

기존 전체 nextest는 11,549건 중 11,543건 통과, 6건 실패였다. 이전 실패 중
#6143 1건과 #7998 1건은 분리된 제품 소스에서 개별 재검증해 통과했고, 팔레트
합성 검사 2건은 독립 Print·전쪽 Native/fresh WASM 대조 후 테스트의 출력 표본
전제를 수정해 통과했다. 그 뒤 문단 분할 Clippy 보정과 함께 팔레트 2건·편람
묶음 빈칸 6건도 다시 실행해 모두 통과했다.

남은 2건은 현재 제품 소스의 분리 검증에서 다시 실패했다. `issue_501`은 기존
SVG 좌표를 소수 문자열 전체로 찾다가 직렬화 표현 차이로 대상을 찾지 못한다.
`issue_7445_reference_font_slot_ownership`은 SVG paint 순서와 논리 문서 순서를
완전히 같다고 요구하는 부분에서 실패했다. 이것만으로 원본 내용·그리기 결과의
정상 여부나 기대값 변경의 타당성이 증명되는 것은 아니다. 두 입력은 편람이
아니므로 사용자의 편람 외 HWP 불관여 지시를 존중해 검사와 구현을 임의로
변경하지 않았다. 두 실패를 전체 회귀 통과로 간주하지 않으며, 해결 또는 권위
있는 예외 근거를 확보하기 전에는 PR 준비 완료로 판정하지 않는다.
