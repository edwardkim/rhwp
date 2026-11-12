# 크기 0 원문의 CanvasKit 명령 계약 보정

원 PR #8916의 동일 입력과 사용자 확인 한컴 2024 Print는 Native 99.75%, fresh WASM 99.72%였다.
전쪽 점수 통과 후 실제 Studio CanvasKit 경로를 검사하자 크기 0 원문을 `textRun:invalidGeometry`로 거절했다.
이는 실제 소스의 137개 Z, fontSize 0, ratio 0, 폭 0, 높이 12의 논리 상자와 전진 0 배열이다.
원문·caret 정보가 남는 것은 정상이며 그 상자가 그릴 잉크를 의미하지는 않는다.

Rust replay 정책과 Studio replay는 유한한 비음수 기하를 먼저 검증한다. 그 안에서 명시적 크기 0,
폭 0과 모든 전진 0인 run을 잉크 없는 직접 명령으로 수용한다. 원문과 소유 정보는 삭제하지 않는다.
음수·NaN·무한대와 크기 있는 장평 0은 계속 거절하고, 실제 glyph·paint를 만들지 않는다.
숨은 Canvas2D overlay를 허용하거나 readiness 문턱을 낮추지 않았다.

실제 최소 글꼴 Rust 관계 검사는 수정 전 3건 중 2 PASS / 1 FAIL, 보정 후 3 PASS / 0 FAIL이다.
Studio 실행형 replay 계약도 `unsupportedOps.size` 1로 실패한 뒤 통과했고, 음수·비유한 값 거절을 함께 확인했다.
최초 테스트 helper의 Box 타입 오류는 제품 결함 증거로 쓰지 않았고 수정 후 실제 assertion 실패를 기록했다.

별도로 #8918의 실제 glyph 질의를 관찰했다. 후보별 누락 교집합은 U+F0FA뿐이며,
원 PR의 원문자·흰 원·체크 상자는 후보가 공급하지만 이 기호는 표시 매핑에서 빠져 있었다.
이후 사용자 요청으로 U+F0FA도 보정 범위에 포함했다.
[기호 보정 근거](../wingdings-arrow-shaft-repair/cause.md)를 따르며,
보정 전 전체 CanvasKit의 실패 기록은 보존한다.

현재는 집중 검사 결과다. 새 소스의 fresh WASM·전체 회귀·실제 Studio 재검증을 최종 증거로 연결한다.

## 최종 실행과 현재 리베이스의 재사용 범위

위 단계별 실패·중간 수치는 원인 분리 이력이다. 검증 source `30fa7739f0340f2a248b22542c7dd36d9354f056`에서 다음 결과를 확인했다. #8916의 Native/fresh WASM은 99.75% / 99.72%이고 관련 관계 검사 3건이 통과했다. 실제 CanvasKit compat/default screen/print에서 잉크 없는 원문을 거부하지 않고 TOP·표·AFTER를 보존했다.

최소 글꼴 전체 13,365 PASS / 0 FAIL / 45 skipped와 Native 집중 141 PASS 및 필수 16단계 게이트를 완료했다. [전체 게이트](../final-rebase-30fa7739/gates-validation.json), [집중 검사](../final-rebase-30fa7739/focused-validation.json), [Native 전쪽](../final-rebase-30fa7739/native-validation.json), [fresh WASM 전쪽](../final-rebase-30fa7739/wasm-validation.json), [실제 backend 지원 범위](../final-rebase-30fa7739/actual-backends/summary.json)를 구분한다. RawSvg 차트의 강제 CanvasKit 진단은 기존 미지원이므로 지원 통과로 세지 않는다.

현재 `73d06ed0`은 새 base `a85d3265`의 #9017 HTML 붙여넣기를 반영했다. 기존 조판·입력·PDF·캡처 경로가 동일해 이 시각 증적을 원 producer와 함께 재사용한다. [동일성 감사와 추가 검증 경계](../final-rebase-30fa7739/rebase-a85d3265/README.md)를 따르며, 이전 source의 전체 실행 결과를 새 후보의 실행 결과로 고쳐 쓰지 않는다.

현재 제품 source `73d06ed0`에서도 최소 글꼴 Native 집중 164 PASS, 전체 13,380 PASS / 0 FAIL / 45 skipped(실행 404.945초), 최신 WASM·실제 브라우저 6사례/24회 저장/API11건 및 필수 21단계를 모두 통과했다. [현재 실행 원장](../final-rebase-30fa7739/rebase-a85d3265/gates-validation.json)과 [현재 기능 보존](../final-rebase-30fa7739/rebase-a85d3265/incoming-base-validation.json)을 기존 시각 producer의 자료와 구분한다.
