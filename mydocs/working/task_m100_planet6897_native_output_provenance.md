# Native 출력의 문서 파생 글꼴 이름 출처 보정

## 범위와 독립 근거

2026-11-04, 통합 제품 `40778b67d6`의 최소 글꼴 환경 전체 회귀에서
`provenance_contract::every_text_bearing_command_declares_untrusted_fields`가 실패했다.
검사는 실제 문서에 삽입한 문자열이 JSON 응답으로 나왔는지 확인한다. 실패를 없애기 위해
검출 대상이나 기대값을 바꾸지 않는다.

- 원 계약은 이슈 #3787의 문서 파생 값 출처 표지다. 현재 선언은
  `crates/rhwp-contracts/src/provenance.rs`로 이동했으며, 원 해결 이후 이력을 확인했다.
- 새 base의 PR #8565(`5551091085`)는 Native 글꼴 공급 보고서를 SVG·PDF·PNG JSON에
  추가했다. `NativeWebfontSupply::unavailable`에는 문서가 지정한 글꼴 이름이나 그 이름을
  포함한 실패 메시지가 들어간다. 글꼴 치환 지도와 글리프 대체 요청도 문서에서 유래한다.
- 실제 실패는 SVG·PDF의 `nativeFontSupply.unavailable[]` 누락 4건이다. 이는 픽셀·조판
  기대값 문제가 아니며 시각 점수를 근거로 보안 계약을 완화할 수 없다.

## 생산·소비 경로와 보정

`native_webfonts::prepare` → `cli::outputs::apply_font_environment`의 보고서 →
SVG·PDF·PNG 응답 → `provenance::marked` → 기계 소비자의 `untrustedFields` 경계다.
응답은 이미 공통 `marked`를 호출하지만, 출처 지도는 세 출력을 문서 문자열 없는 응답으로
선언하고 있었다. 해당 공통 선언에서 실제 문서 파생 글꼴 필드를 지정한다.

고정 공급 URL·바이너리 해시·산출 경로·쪽수·바이트 수를 문서 문자열로 새로 분류하지 않는다.
공급 보고서의 문서 글꼴 이름, 문서 선언에서 만든 환경과 관측 대체 기록을 표시한다.
같은 필드 목록을 세 명령이 공유하며, 없는 필드·빈 목록·null은 기존 `present_fields`가 제외한다.

## 검증 상태

수정 전 전체 회귀의 실제 실패를 확인했다. 원시 로그는 ignored
`output/pr-review/planet6897-20261103/logs/owned-frame-full-nextest-minfont.log`에 있다.
수정 후 기존 출처 계약 전체와 Native PNG 경로를 최소 글꼴 환경에서 재실행하고,
실제 종료 코드와 결과를 이 절에 추가한다. 현재는 수정 후 통과를 선언하지 않는다.

수정 전 전체 회귀가 최종 종료됐다. 12,511건 실행, 12,510건 통과, 1건 실패, 45건 skip,
exit 100이다. 실패는 위 출처 계약 1건뿐이며 남은 긴 실행도 종료를 확인했다.
현재 준비한 소스의 rustfmt 검사와 `git diff --check`는 통과했다.

## 첫 보정의 실제 재검증

제품 `874fac36f2`의 Native Skia 출처 계약 묶음은 11건 중 10건 통과, 1건 실패,
exit 101이다. 최초 누락된 문서 글꼴 필드는 이제 표시된다. 남은 실패는 생성된
`untrustedFields[]`의 정적 경로 문자열 자체를 문서 문자열로 탐지한 3건이다.
출처 지도와 생성 표지의 의미·기존 오라클 제외 규칙을 먼저 대조한 뒤 필요한 보정을
판정한다. 아직 이 묶음을 통과로 기록하지 않는다. 원시 로그는
`logs/batch-874fac36f2-provenance-native-skia-minfont.log`다.

## 정적 경로 메타데이터의 오탐 보정

남은 세 실패는 도구가 생성한 `untrustedFields[]`의 경로 이름 자체와 문서 토큰의
부분 문자열 충돌이었다. 실제 `nativeFontSupply.unavailable[]`는 이제 올바르게 표시된다.
기존 `untrusted_flag_matches_map`은 표지 경로가 정적 지도에 있는지 별도로 검사한다.

탐지기는 **최상위** `untrustedFields` 배열이고 그 원소가 전부 해당 명령의 정적 선언 경로와
정확히 같을 때만 메타데이터로 구분한다. 원시 문서 값 검출에 지도를 사용하지 않는다.
위조한 경로, 중첩 문서 객체의 같은 키, 실제 공급 실패 메시지는 계속 독립 오라클로 검사한다.
새 반례는 일부러 정적 `unavailable` 경로와 같은 토큰을 문서 글꼴 이름에 넣어 구분을 확인한다.
검사 목록을 지우거나 누락된 문서 필드를 면제하지 않는다.

현재 리베이스 기준은 `96827b3accd98ad0b80f76260b42fba24eded504`다. 이 보정과 리베이스된
출력 명령을 함께 최소 글꼴 환경에서 재실행한 결과를 이후 기록한다.

## 리베이스 후 실제 통과

제품 `2071603998219defdd50cfca796f0fe8207c8397`에서 Native Skia를 포함한 출처 계약
12건이 모두 통과했다(exit 0). 실제 문서 문자열 누락 검사와 새 위조 경로 반례도 실행했다.
최소 글꼴 환경은 저장소의 공개 글꼴 5개만 노출했다. 원시 로그는 ignored output의
`logs/rebase-96827-provenance-native-skia-minfont.log`다. 이 집중 결과를 전체 Rust 검증
완료로 기록하지 않으며, 최종 전체 회귀는 뒤에서 별도로 수행한다.
