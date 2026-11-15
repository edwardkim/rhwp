# PR #9070 메인테이너 검토

## 최종 판정

**메인터너 보정 후 수용 가능.** 기존 보호값·영문 스타일·그림 외곽의 차이를 예외 없이 보완했다.
제품 source는 `8ad1c5c08a`, 최신 base는 `2d83fa4c8a`이며 실제 비교용 `e7ee272cfa`와 제품·시각 harness가 같다.
전체 회귀 13,608건·Native 공식 4,073건·모든 lint와 필수 편람 gate를 통과했다.
Native/fresh WASM 보호 각 1,771쪽·3,542행의 공식값 하락은 0건이며 상승 19행을 새 하한에 반영했다.
원 사례·대조군 각 134쪽의 최저는 93.31%·92.35%다. 직접 확대 판독·최신 base 전쪽 비교도 완료했다.
[최종 검증·생산자·전쪽 TSV](assets/jinho-von-choi-20261114/final-source-8ad/README.md).
사용자 승인 후 [수용 PR #9096](https://github.com/edwardkim/rhwp/pull/9096)을 생성했다. 공개 최종 review·merge·issue close는 최신 head CI 확인 뒤 진행한다.

- 작성자: JinHo-von-Choi. 원 head: `8669ec6dc4b91a57d6af8e6b82dd391dff0832af`.
- 원 이슈: #8912. 사용자 지시로 reviewer `edwardkim`을 지정했고 원 commit을 보존했다.
- 기본 경로: maintainer_general. 보조 경로: intake_and_review, local_validation, multi_pr_update_branch, visual_fixture_evidence, post_merge. 정본 router·선택표와 해당 자식 절차를 적용했다. 기존 #9052 수용 이력이 있어 API association만으로 첫 기여로 취급하지 않는다.

## 주장과 메인테이너 보완

일반 기호 7개 복원과 잘린 `ch == 29` 교차 참조 방어는 유효하며 최신 devel에도 문제가 남아 있었다. 기존 #5860·#6380의 실측 PUA override는 보존한다. 원 기여의 괘선 77개 중 65개는 독립 한컴 저장값과 달랐고 점선·굵은 선을 다른 모양으로 매핑했다.

[승인된 보완 계획](pr_9070_review_impl.md)에 따라 77개를 실제 한컴 HWPX의 plane-15 문자로 복원했다. 공통 `native_pua_fonts`의 측정·paint 경로에서 실제 SFNT 프로그램을 함께 소비한다. 기존 지원 11개 순서를 유지하고 중복 없는 75개를 추가했다. 글꼴 bytes·윤곽·문서별 좌표 상수는 추가하지 않았다. 원본 SFNT가 없을 때의 기존 대체 계약도 유지한다.

실제 CLI 대조는 괘선 오인이 65자에서 0자로 줄었다. 정상 specimen의 원 HWP3·HWP 저장본·HWPX 저장본은 각각 전 1쪽이며 세 입력의 최저는 Native 98.46736%·최적화 fresh WASM 98.47794%다. 실제 독립 한컴 HWP5 재열기·HWPX 저장에서도 77개 문자가 같다. [문자 대조](assets/pr9070/linechar-before-after.json) · [한컴 재열기 영수증](assets/pr9070/hancom-hwp5-reopen.json) · [최종 전쪽 TSV](assets/jinho-von-choi-20261114/final-source-8ad/feature-validation.json) · [Print·글꼴 근거](assets/pr9070/README.md).

## 입력·시각 근거와 호스트 장애

공개 원 합성 HWP3의 용지 0·비율 0은 parser-only fixture로 분리했다. 실제 사양의 정상 specimen을 별도로 생성해 사용자가 수동 Print한 PDF를 정식 보존했다. 7개 괘선 집합·일반 기호를 같은 배율로 직접 읽었다. PDF의 실제 Haansoft Batang hmtx·전진을 근거로 명시 font environment를 공급했으며 제품 기본값을 바꾸지 않았다. 사용자가 안내한 로컬 HBATANG을 실제로 사용했다.

호스트 자동 인쇄 실패와 사용자 재부팅은 [중단·복구 기록](jinho_von_choi_20261114_interruption.md)에 남겼다. Windows 호스트의 Hwp.exe·COM·인쇄 자동화는 계속 비활성이다. 사용자 수동 Print와 별도 원격 MCP 정상 재열기를 호스트 자동화 성공으로 바꾸어 기록하지 않는다.

## 최종 공통 검증과 다음 조건

[최종 원장](assets/jinho-von-choi-20261114/final-source-8ad/README.md)의 17개 필수 단계가 모두 통과했다.
정본 보호의 하락·누락은 0행이며 수용 예외를 적용하지 않았다. 최신 devel의 추가 문서 389쪽과
RowBreak 14쪽씩도 실제 source로 대조했다. 원 사례와 추가 문서의 2%p 초과 감소는 모두 0쪽이다.
감소 영역은 원본 Print·base·후보를 같은 배율로 직접 읽고 실제 export의 bbox·내용·순서를 확인했다.

실행 입력·독립 PDF 118개는 실제 SHA-256과 Git blob을 대조했고 미커밋 입력은 0개다.
초기 잘못된 필터 호출·보안 입력 전달·중간 source의 미달은 [조사 이력](assets/jinho-von-choi-20261114/protected-environment-review.md)에
보존했다. 과거 실패를 지우거나 과거 통과 수를 최종 검증에 합산하지 않는다.
현재 다음 조건은 준비한 upstream 수용 PR 경로의 원격 승인, 해당 최신 head CI·self-review·merge다.
원 기여 SHA와 fork branch를 보존하고 연결 이슈는 각 검증 범위에 맞게 후속 처리한다.

## 제출 중 최신 devel 재검증

#9093을 포함한 `1805d8f08c`를 로컬 후보에 병합했다. [실제 Studio 기능 보존](assets/jinho-von-choi-20261114/final-source-8ad/incoming9093/README.md)에서 양쪽 12개 조합이 같고 후보 전체 2,798건·브라우저 26건이 통과했다. Rust·독립 시각 검증 경로는 바뀌지 않았다.
