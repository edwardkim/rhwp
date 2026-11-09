# PR #8777 검토 — Linux 실제 consumer·연속 GL 프레임

## 최종 판정

**승인 — 명시한 통합 소스에서 로컬 필수 검증 완료.** 전체 회귀 13,005건,
Native 공식 3종 합계 4,077건, 모든 Rust lint와 아래 실제 경로 검증이 통과했다.
작업지시자의 push·Open PR·CI·self-review·merge·후속 처리 승인을 받았다. 최신 제출 head의 필수 CI 성공과 mergeability 재확인 뒤 진행한다. GitHub 승인 review는 아직 게시하지 않았다.

## 고정 소스와 선행 확인

- 원 PR: https://github.com/edwardkim/rhwp/pull/8777
- 작성자 seo-rii; reviewer edwardkim을 지정하고 API로 확인했다.
- 원 head: `def466097d790671247e30c1ec412931ad606e6a`. [소스 재조회](assets/seo-rii-20261109/source-refresh.json)에서도 같았다.
- 기준 devel: `06181802e574dbd91e62375536751dea8c270fcd`.
- 통합 제품 소스: `be778bae9b3803050146dcc47a896940587a612e`.
- 검증 소스: `5bccdb7b90c4f447f523fc7ec4fa0931e761c3a7`.
  두 Rust 회귀 파일의 측정 근거 주석만 보완했으며 제품·검사 본문은 바꾸지 않았다.
- 원 head는 통합 후보의 ancestor로 [보존](assets/seo-rii-20261109/original-head-preservation.json)했다.
  현재 devel의 [미해결 기능 대조](assets/seo-rii-20261109/already-resolved-check.json)와 기존 #8739의
  통합·보정 이력을 선행 확인했다. 원 head의 CI와 새 통합 head의 CI를 혼동하지 않는다.

## 코드 경로와 실제 확인

실제 Flutter public headers compile/link와 기존 Native GL producer를 연결한다. source/revision/view stamp와 bounded target 계약을 유지한다.

메인터너가 새 Native GL FFI와 owner·GTK runner를 빌드했다. Xvfb+Mesa llvmpipe에서 실제 32회 게시·변경·resize·종료와 Native 계수 9개 0을 확인했다.

[공통 검증 원장](assets/seo-rii-20261109/README.md)에 실행 명령·source·실제 결과·시각 판독과
플랫폼별 artifact를 연결했다. Native/fresh WASM의 기본 출력 전 7쪽은 각각 raw 최저 100.00%이고
독립 Print PDF와 쪽수가 같다. 기준/후보의 같은 backend PNG 14쌍도 같다. 원문 그림·외곽·간격과
플랫폼의 첫/32번째 실제 화면을 직접 확인했으며 점수만으로 판정하지 않았다.

## 범위와 조판 원칙 검토

CPU/GPU AA 전체 바이트 동일성·실물 GPU 성능을 주장하지 않는다. 실제 shader/consumer 경로와 source palette 경계 비교 계약을 구분해 기록했다.

이번 변경은 기존 조판·줄 구성·pagination 결과를 소비하는 renderer 또는 Flutter 참조 host다.
원본 저장 LineSeg·측정·페이지 분할/이어받기 생산자를 바꾸지 않아 해당 조판 변경 경로와 편람
383쪽 gate는 비해당이다. 문서별 수치 맞추기나 배치 clamp를 추가하는 변경이 아니다.
Canvas2D의 typed producer/variant 계약과 OS별 consumer 계약 차이를 공통 조판 규칙의 예외로
대신하지 않았다. 실제 실행·명시적 상한·fallback과 정상 대조군을 원장에 구분했다.


## Merge 후 contributor PR comment 계획

desktop-consumer.hwpx p1의 실제 메인테이너 Linux GTK consumer 단계 1·2·3·32다. 각 CPU·실제 display와 32회 전체 결과, 종료 계수 0과 실제 비교 계약을 설명한다.

- 실제 integration PR·merge SHA·CI URL·개별 archive 검토를 direct link로 적고 한국어 존댓말로 감사를 전한다.
- 아래 원본과 대응하는 이미지 전부를 merge SHA 고정 raw URL로 첫 comment부터 표시한다. 게시 전 응답 형식·해시, 게시 후 API 본문·이미지 표시를 확인한다.
- 이미지: `mydocs/pr/assets/seo-rii-20261109/linux-actual/cpu-01.png`.
- 이미지: `mydocs/pr/assets/seo-rii-20261109/linux-actual/display-01.png`.
- 이미지: `mydocs/pr/assets/seo-rii-20261109/linux-actual/cpu-02.png`.
- 이미지: `mydocs/pr/assets/seo-rii-20261109/linux-actual/display-02.png`.
- 이미지: `mydocs/pr/assets/seo-rii-20261109/linux-actual/cpu-03.png`.
- 이미지: `mydocs/pr/assets/seo-rii-20261109/linux-actual/display-03.png`.
- 이미지: `mydocs/pr/assets/seo-rii-20261109/linux-actual/cpu-32.png`.
- 이미지: `mydocs/pr/assets/seo-rii-20261109/linux-actual/display-32.png`.
- #536·#8701 전체 로드맵은 OPEN을 유지한다. 같은 merge·증거의 기존 댓글이 있으면 중복 게시하지 않는다.

## 다음 원격 절차

[공통 계획](seo_rii_20261109_review_impl.md)과 [원격 처리안](assets/seo-rii-20261109/remote-action-plan.md)을 따른다.
현재 원 기여 branch를 변경하거나 공개 댓글·승인 review·merge·close를 수행하지 않았다.
로컬 필수 검증과 원격 승인 게이트를 완료했다. #536·#8701 전체 로드맵은 OPEN으로 유지한다.
