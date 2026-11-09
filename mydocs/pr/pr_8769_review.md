# PR #8769 검토 — 실제 Apple SDK Metal producer 검증

## 최종 판정

**승인 — 명시한 통합 소스에서 로컬 필수 검증 완료.** 전체 회귀 13,005건,
Native 공식 3종 합계 4,077건, 모든 Rust lint와 아래 실제 경로 검증이 통과했다.
작업지시자의 push·Open PR·CI·self-review·merge·후속 처리 승인을 받았다. 최신 제출 head의 필수 CI 성공과 mergeability 재확인 뒤 진행한다. GitHub 승인 review는 아직 게시하지 않았다.

## 고정 소스와 선행 확인

- 원 PR: https://github.com/edwardkim/rhwp/pull/8769
- 작성자 seo-rii; reviewer edwardkim을 지정하고 API로 확인했다.
- 원 head: `40e1301ddabb6290a4ed06c5791e8fd4960403f5`. [소스 재조회](assets/seo-rii-20261109/source-refresh.json)에서도 같았다.
- 기준 devel: `06181802e574dbd91e62375536751dea8c270fcd`.
- 통합 제품 소스: `be778bae9b3803050146dcc47a896940587a612e`.
- 검증 소스: `5bccdb7b90c4f447f523fc7ec4fa0931e761c3a7`.
  두 Rust 회귀 파일의 측정 근거 주석만 보완했으며 제품·검사 본문은 바꾸지 않았다.
- 원 head는 통합 후보의 ancestor로 [보존](assets/seo-rii-20261109/original-head-preservation.json)했다.
  현재 devel의 [미해결 기능 대조](assets/seo-rii-20261109/already-resolved-check.json)와 기존 #8739의
  통합·보정 이력을 선행 확인했다. 원 head의 CI와 새 통합 head의 CI를 혼동하지 않는다.

## 코드 경로와 실제 확인

실제 Apple SDK와 Flutter framework로 producer probe를 compile/link·실행한다. Native supplied input과 consumer source를 구분한다.

실제 P107 exact head SDK 실행·producer 계약과 후속 AppKit 32-frame 원본 artifact를 대조했다. portable 검사를 macOS 실행의 대용으로 쓰지 않았다.

[공통 검증 원장](assets/seo-rii-20261109/README.md)에 실행 명령·source·실제 결과·시각 판독과
플랫폼별 artifact를 연결했다. Native/fresh WASM의 기본 출력 전 7쪽은 각각 raw 최저 100.00%이고
독립 Print PDF와 쪽수가 같다. 기준/후보의 같은 backend PNG 14쌍도 같다. 원문 그림·외곽·간격과
플랫폼의 첫/32번째 실제 화면을 직접 확인했으며 점수만으로 판정하지 않았다.

## 범위와 조판 원칙 검토

제공된 Native 라이브러리를 재사용했다. 초기 영수증의 input-equivalence false를 임의로 true로 바꾸지 않으며 별도 현재 base 대조를 연결한다. 기존 #8726 예외를 새 증거 면제로 옮겨 쓰지 않는다.

이번 변경은 기존 조판·줄 구성·pagination 결과를 소비하는 renderer 또는 Flutter 참조 host다.
원본 저장 LineSeg·측정·페이지 분할/이어받기 생산자를 바꾸지 않아 해당 조판 변경 경로와 편람
383쪽 gate는 비해당이다. 문서별 수치 맞추기나 배치 clamp를 추가하는 변경이 아니다.
Canvas2D의 typed producer/variant 계약과 OS별 consumer 계약 차이를 공통 조판 규칙의 예외로
대신하지 않았다. 실제 실행·명시적 상한·fallback과 정상 대조군을 원장에 구분했다.


## Merge 후 contributor PR comment 계획

실제 Apple SDK compile/link·Metal producer probe 검증이다. 원 P107 source와 제공 Native source·해시, 실제 SDK job을 연결한다. 별도 P110 WindowServer 화면을 P107 producer의 비교 이미지로 대신하지 않는다.

- 실제 integration PR·merge SHA·CI URL·개별 archive 검토를 direct link로 적고 한국어 존댓말로 감사를 전한다. 문서 비교 수치를 이 기능의 새 측정값으로 주장하지 않는다.
- #536·#8701 전체 로드맵은 OPEN을 유지한다. 같은 merge·증거의 기존 댓글이 있으면 중복 게시하지 않는다.

## 다음 원격 절차

[공통 계획](seo_rii_20261109_review_impl.md)과 [원격 처리안](assets/seo-rii-20261109/remote-action-plan.md)을 따른다.
현재 원 기여 branch를 변경하거나 공개 댓글·승인 review·merge·close를 수행하지 않았다.
로컬 필수 검증과 원격 승인 게이트를 완료했다. #536·#8701 전체 로드맵은 OPEN으로 유지한다.
