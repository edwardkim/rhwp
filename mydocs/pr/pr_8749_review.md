# PR #8749 검토 — Canvas2D strict-outline 미리보기

## 판정

**수용 권고 — 로컬 필수 검증 완료, 원격 처리 승인 대기.** 전체 회귀 13,005건,
Native 공식 3종 합계 4,077건, 모든 Rust lint와 아래 실제 경로 검증이 통과했다.
원격 push·공개 review/comment·merge는 별도 승인 전이다. 이 문서는 로컬 검토 판정이며 GitHub 승인 review를 게시한 상태가 아니다.

## 고정 소스와 선행 확인

- 원 PR: https://github.com/edwardkim/rhwp/pull/8749
- 작성자 seo-rii; reviewer edwardkim을 지정하고 API로 확인했다.
- 원 head: `0c8de92939ada977d96e9d3608f13f4fdd707cf3`. [소스 재조회](assets/seo-rii-20261109/source-refresh.json)에서도 같았다.
- 기준 devel: `06181802e574dbd91e62375536751dea8c270fcd`.
- 통합 제품 소스: `be778bae9b3803050146dcc47a896940587a612e`.
- 검증 소스: `5bccdb7b90c4f447f523fc7ec4fa0931e761c3a7`.
  두 Rust 회귀 파일의 측정 근거 주석만 보완했으며 제품·검사 본문은 바꾸지 않았다.
- 원 head는 통합 후보의 ancestor로 [보존](assets/seo-rii-20261109/original-head-preservation.json)했다.
  현재 devel의 [미해결 기능 대조](assets/seo-rii-20261109/already-resolved-check.json)와 기존 #8739의
  통합·보정 이력을 선행 확인했다. 원 head의 CI와 새 통합 head의 CI를 혼동하지 않는다.

## 코드 경로와 실제 확인

`HwpDocument`가 만드는 page-layer tree의 명시적 preview만 공통 variant 선택·전체 part 준비를 거쳐 replay한다. 기본 API, profile/page 검증, patch 경계와 fallback은 보존한다.

Rust 집중 8건, 실제 wasm32 Chromium 12건, 최적화 패키지 공개 API와 기본 출력 전 7쪽 Native/fresh WASM을 확인했다. 두 Rust 회귀 파일의 첫머리에 실제 측정 근거를 보완했으며 검사 본문 해시는 같다.

[공통 검증 원장](assets/seo-rii-20261109/README.md)에 실행 명령·source·실제 결과·시각 판독과
플랫폼별 artifact를 연결했다. Native/fresh WASM의 기본 출력 전 7쪽은 각각 raw 최저 100.00%이고
독립 Print PDF와 쪽수가 같다. 기준/후보의 같은 backend PNG 14쌍도 같다. 원문 그림·외곽·간격과
플랫폼의 첫/32번째 실제 화면을 직접 확인했으며 점수만으로 판정하지 않았다.

## 범위와 조판 원칙 검토

조판 생산자·일반 기본 경로는 변경하지 않는다. preview의 모든 글꼴이 한컴과 일치한다는 주장이나 Studio 기본 renderer 전환으로 수용하지 않는다.

이번 변경은 기존 조판·줄 구성·pagination 결과를 소비하는 renderer 또는 Flutter 참조 host다.
원본 저장 LineSeg·측정·페이지 분할/이어받기 생산자를 바꾸지 않아 해당 조판 변경 경로와 편람
383쪽 gate는 비해당이다. 문서별 수치 맞추기나 배치 clamp를 추가하는 변경이 아니다.
Canvas2D의 typed producer/variant 계약과 OS별 consumer 계약 차이를 공통 조판 규칙의 예외로
대신하지 않았다. 실제 실행·명시적 상한·fallback과 정상 대조군을 원장에 구분했다.

## 다음 원격 절차

[공통 계획](seo_rii_20261109_review_impl.md)과 [원격 처리안](assets/seo-rii-20261109/remote-action-plan.md)을 따른다.
현재 원 기여 branch를 변경하거나 공개 댓글·승인 review·merge·close를 수행하지 않았다.
로컬 필수 검증을 완료했으며 원격 처리 승인을 요청한다. #536·#8701 전체 로드맵은 OPEN으로 유지한다.
