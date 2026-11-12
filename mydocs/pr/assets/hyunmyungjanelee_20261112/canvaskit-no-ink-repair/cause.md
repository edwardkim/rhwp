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
이는 원 PR과 #8680이 명시한 범위 밖이다. 요청 범위 원문자·흰 원·체크 상자는 후보가 공급한다.
원본 전체 CanvasKit의 실패 기록은 보존하며 전체 지원으로 표기하지 않는다.

현재는 집중 검사 결과다. 새 소스의 fresh WASM·전체 회귀·실제 Studio 재검증을 최종 증거로 연결한다.
