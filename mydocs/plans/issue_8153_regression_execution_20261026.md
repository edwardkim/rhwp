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

전체 서식과 `--workspace --all-targets` Clippy는 자동 수집 suite를 준비한
독립 worktree에서 실행했다. sample16 제거로 반복 대상이 하나가 된 기존 검사
두 곳의 단일 반복문을 직접 호출로 정리한 뒤 모두 통과했다. 해당 검사의 대상과
기대값은 바꾸지 않았다. Native 라이브러리 및 wasm32 라이브러리 Clippy도
통과했다. 빌드·검사의 공용 Cargo target은 `target/pr-review`, 병렬도는 8이다.
이 lint 결과가 위 두 회귀 실패를 해결한 것은 아니다.

## 회귀 파일 첫머리 측정값 대조

새 `issue_8153_*.rs` 46개 파일의 첫 줄 검사 유형, Native·WASM·최저값 표기와
정식 근거 경로를 확인했다. 첫머리에 명시된 편람 원본·쪽별 수치 중 정규 형식의
122건은 `ffae46e8a6`에서 산출한 반올림 전 Native·fresh WASM 측정값과 모두
일치한다. 이 자동 확인은 다른 표기 형식과 해당 문장이 실제 검사 대상인지까지
판정하지 않는다. 최종 소스의 전쪽 재검증 후 같은 수치를 다시 확인해야 한다.

합성 단위 입력을 사용하는 8개 파일은 편람에서 관찰한 별도 결과를 첫머리에
구분해 적고 있지만, 합성 입력 자체의 독립 PDF와 Visual Sweep 근거로 바꾸어
읽어서는 안 된다. 새 검사 추가 조건이 실제 해당 입력에 적용되는지, 해당하면
같은 입력의 독립 출력이 확보됐는지 별도로 확인해야 한다. 이 확인 전에는 모든
새 회귀 검사의 시각 선행 조건이 충족됐다고 판정하지 않는다.
46개 파일에 각각 적힌 두 원본의 전쪽 최저값 92건도 같은 측정 결과와 일치한다.

## 2026-10-29 최종 제품 소스의 Native Skia 재검증

제품 소스 `a1e511dbebcc27e187ff847d5510358fa9fbada4`에서 `release-test`,
`native-skia`, `target/pr-review`, 빌드 잡 8개로 실행했다. 별도 검증 worktree에
`template/`이 누락된 첫 실행은 파일 읽기 오류 3건으로 실패했다. 기존 입력을
연결한 뒤 전체 라이브러리를 재실행해 본 라이브러리 3,865건 통과·12건 무시,
같은 명령의 나머지 라이브러리 182건 통과를 확인했다.

이어 기존 CI 경로의 다섯 논리 case를 같은 제품 소스에서 순차 실행했다.
`issue_2225_missing_picture_placeholder` 2건, `render_p37_direct_pdf_export`
14건, `issue_8153_opaque_mixed_axis_image_sampling` 2건,
`issue_8153_print_glyph_geometry` 3건, `issue_8153_symbol_fallback` 1건이
모두 통과했다. 위 결과는 기존 전체 integration 회귀의 미해결 2건이나
후속 fresh WASM 브라우저 검사 결과를 대신하지 않는다.

동일 제품 소스의 최적화 fresh WASM
`3dbc71ea881554b86eeae35fcc124779946906c1e0d5b6145747a7d5fa040c77`을
실제 Chromium의 `visual-sweep-canvas-fonts.browser.mjs`에 전달한 실행은
8건 통과, 실패·skip 각 0건이다. 이 결과는 브라우저 회귀 실행에 한정하며
Studio CanvasKit의 128쪽 상한에 따른 대체 동작과 별도로 기록한다.

## source-side 검사 이전 진행

최종 소스의 unit-tier 검사에서 신규 source-side 검사와 테스트 전용 import가
정책에 걸렸다. 테스트 전용 import는 사용 지점으로 옮겼다. 셀 clip, 알파 정수,
별도 마스크 표본, 빈 글자 겹침 검사를 기존 `tests/cases/`로 이전하고 있다.
셀 clip은 원본 배경 ID 추출과 공통 분할 함수의 본문이 분리 전과 같으며,
마스크 함수는 주석 표기를 제외한 본문이 같다. 알파 정수 함수 본문도 분리 전과 같다.
이전한 검사는 준비된 검증 worktree의 실제 suite에서 글자 겹침 4건, 알파 마스크
5건, 셀 clip 7건이 모두 통과했다. 최종 제품 소스의 Native/fresh WASM 전쪽
시각 검증은 아직 확인 중이다.

검증 worktree에서 기존 suite를 다시 준비한 뒤 48/48 target 예산과 고정 base
manifest 검사는 통과했다. manifest 자체 검사도 23건 통과했다. 이는 이전한
회귀 검사의 실제 실행이나 전체 회귀 실패의 해결을 대신하지 않는다.

검사 이전 소스 `412426515e`에서 `cargo fmt --all -- --check`, workspace
all-target Clippy, Native 라이브러리 Clippy, wasm32 라이브러리 Clippy가 모두
통과했다. 종료된 Cargo·Rust 프로세스가 없음을 확인한 뒤 검토 전용
`target/pr-review/debug/incremental`의 종료된 대형 캐시 7개를 정리해 빌드
여유 공간을 확보했다. 최종 소스의 전쪽 재검증은 별도로 실행한다.
