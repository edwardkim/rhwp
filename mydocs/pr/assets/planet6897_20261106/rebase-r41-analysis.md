# R41 리베이스와 수식 표시 경로 보존

## 고정한 source와 실제 리베이스

- 리베이스 전 후보: `651df3b1078b39f229df01eac0a97c8a5f6eb6e3`.
- 이전 base: `b8dd67c5001dfb0410e9e192528676396cdd6a4e`.
- 새 base: `e4ffabea38062e716f6c333cc950fa31b42b4dd7`.
- 리베이스 직후 후보: `c6bf3f74e33fcf13a02ea6a6177ec33242ffc5f4`.
- 129개 브랜치 커밋을 재적용했고, 원 contributor 체리픽 footer 56개는 중복 없이 모두 보존했습니다.
  시각 검증의 실제 입력·독립 Print 141개 경로는 고정한 SHA와 모두 같습니다.
- 직접 겹친 source는 `document_core/queries/rendering.rs`,
  `renderer/skia/equation_conv.rs`, `renderer/skia/image_conv.rs` 세 곳입니다.
  실제 내용 충돌은 `equation_conv.rs` 1개 파일, 2개 hunk입니다.

새 base에 들어온 [PR #8739](https://github.com/edwardkim/rhwp/pull/8739)는
Native·Qt·Flutter P74–P102의 타일·snapshot·presentation·GL·IME 등을 통합했습니다.
추가로 원 수식 프로그램이 없을 때 private code를 원문 문자로 복원하는 실제 producer
결함을 보정했습니다. 이 PR을 새로 체리픽한 것이 아니라 devel의 새 기능으로 보존합니다.

## 충돌 양쪽의 독립 계약

| 처리 | 원 근거 | 보존할 의미 |
| --- | --- | --- |
| 같은 family의 분리 프로그램 선택 | 기존 보정 `70c726a001accc24a82ea24f96553ff89e7f72ce` | 한글·영문 OTF/TTF가 같은 family의 서로 다른 글리프를 담으면 다른 family보다 먼저 coverage를 확인 |
| 원 수식 face 부재의 semantic fallback | PR #8739의 `cda7c8cb33bb7ded970d033ed1543aabb49c63a2` | PUA 숫자·문자를 producer의 원문 의미로 해석하고 대응 face 선택; 원래 positions·baseline·shear 보존 |

`equation_conv`는 새 base의 공통 `draw_hancom_glyphs`를 호출합니다. 공통 helper는
기존 첫 원 face를 유지하고 같은 선언 family의 다른 프로그램을 뒤에 보충합니다.
모든 원 프로그램에 글리프가 없을 때만 semantic decode와 일반 fallback을 사용합니다.
생성 legacy fallback을 원 프로그램으로 승격해 PUA 복원을 건너뛰지 않습니다.
레이아웃 전진이나 줄 프레임을 fallback face의 자연 폭으로 다시 계산하지 않습니다.

## 검사와 아직 남은 검증

새 base의 `real_producer_private_digits_fall_back_to_the_source_number` 등 기존 검사는
그대로 유지했습니다. 공개 글꼴의 같은 family를 분리 공급한 합성 surface 검사를 추가해
원 프로그램의 나머지 coverage를 소비하는지도 확인합니다. 기대값은 같은 공개 프로그램을
독립 Skia paint로 그린 윤곽이며 실물 문서의 절대 px 핀이 아닙니다. 시스템 OTF가 없어도
실행할 수 있습니다.

리베이스 전 전체 Rust 12,851건, Native Skia lib·집중 검사와 기존 시각 측정은 해당
source의 완료 증거로 보존합니다. 새 후보의 실행으로 바꾸어 쓰지 않습니다.
새 base·후보의 실제 Native/fresh WASM 출력과 원 face 부재/공급 경계, 기존 보호 TSV,
공통 호출 경로를 검증한 뒤 최종 판정을 갱신합니다. 현재는 새 후보 검증 완료를 주장하지 않습니다.

리베이스 중 원본 경로를 다시 체크아웃하는 동안 기존 WASM 측정의 요약문 뒤 페이지에서
일시적인 파일 부재가 발생했습니다. 실행을 공유 원본 경로와 겹치게 진행한 작업 순서의
오류입니다. 완료한 25개 문서는 보존했고 실패한 문서와 미실행 문서만 재개했습니다.
수치 미달이나 제품 결함으로 분류하지 않으며, 일부 페이지 완료를 전쪽 완료로 합산하지 않습니다.

## 공개 글꼴 경계의 실제 수정 전후 실행

같은 family의 분리 프로그램 공급 검사는 새 base에서 Screen 윤곽 assertion으로 실제 실패했다(집중 1 FAIL, exit 100). 환경 DNS 실패(exit 102)는 테스트 실행 전 오류로 구분하고 실패 재현으로 세지 않았다. 격리 환경의 resolver 연결 대상을 읽기 전용으로 제공한 뒤 실제 실행했다.

후보 `851e4074514935e4c6a7377f71a97f0cca454191`의 최소 공개 글꼴 환경에서는 같은 신규 검사와 원 숫자 4·3·원 face 완전/부분 공급·부재를 포함한 7건이 모두 통과했다. 기존 숫자 기대값이나 글꼴 공급 조건을 완화하지 않았다.

[새 base/후보 Native 9개 입력·24쪽](rebase-r41-native-pr8739/README.md)의 PNG는 전부 바이트 단위로 같다. 이 범위의 기능 보존은 완료했으며 fresh WASM·통합 전쪽·최종 공통 검증의 완료와 구분한다.
