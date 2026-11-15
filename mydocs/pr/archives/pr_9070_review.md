# PR #9070 메인테이너 검토

## 최종 판정

**승인·통합 완료.** 기존 보호값·영문 스타일·그림 외곽의 차이를 예외 없이 보완했다.
Rust 제품 source는 `8ad1c5c08a`, Rust 검증 기준 base는 `2d83fa4c8a`이며 실제 비교용 `e7ee272cfa`와 제품·시각 harness가 같다. 최종 제출은 새 devel `1805d8f08c`를 병합하고 frontend 기능 보존을 재검증한 `b6a9116676`이다.
전체 회귀 13,608건·Native 공식 4,073건·모든 lint와 필수 편람 gate를 통과했다.
Native/fresh WASM 보호 각 1,771쪽·3,542행의 공식값 하락은 0건이며 상승 19행을 새 하한에 반영했다.
원 사례·대조군 각 134쪽의 최저는 93.31%·92.35%다. 직접 확대 판독·최신 base 전쪽 비교도 완료했다.
[최종 검증·생산자·전쪽 TSV](../assets/jinho-von-choi-20261114/final-source-8ad/README.md).
사용자 승인 후 [수용 PR #9096](https://github.com/edwardkim/rhwp/pull/9096)의 모든 CI·공개 self-review를 완료하고 `e4f8ff2d637d5b0a313211455ae3ccc53444d969`로 병합했다. 원 PR의 MERGED 상태·연결 이슈의 completed 종료와 [결과 댓글](https://github.com/edwardkim/rhwp/pull/9070#issuecomment-6895400152)을 확인했다.

- 작성자: JinHo-von-Choi. 원 head: `8669ec6dc4b91a57d6af8e6b82dd391dff0832af`.
- 원 이슈: #8912. 사용자 지시로 reviewer `edwardkim`을 지정했고 원 commit을 보존했다.
- 기본 경로: maintainer_general. 보조 경로: intake_and_review, local_validation, multi_pr_update_branch, visual_fixture_evidence, post_merge. 정본 router·선택표와 해당 자식 절차를 적용했다. 기존 #9052 수용 이력이 있어 API association만으로 첫 기여로 취급하지 않는다.

## 주장과 메인테이너 보완

일반 기호 7개 복원과 잘린 `ch == 29` 교차 참조 방어는 유효하며 최신 devel에도 문제가 남아 있었다. 기존 #5860·#6380의 실측 PUA override는 보존한다. 원 기여의 괘선 77개 중 65개는 독립 한컴 저장값과 달랐고 점선·굵은 선을 다른 모양으로 매핑했다.

[승인된 보완 계획](pr_9070_review_impl.md)에 따라 77개를 실제 한컴 HWPX의 plane-15 문자로 복원했다. 공통 `native_pua_fonts`의 측정·paint 경로에서 실제 SFNT 프로그램을 함께 소비한다. 기존 지원 11개 순서를 유지하고 중복 없는 75개를 추가했다. 글꼴 bytes·윤곽·문서별 좌표 상수는 추가하지 않았다. 원본 SFNT가 없을 때의 기존 대체 계약도 유지한다.

실제 CLI 대조는 괘선 오인이 65자에서 0자로 줄었다. 정상 specimen의 원 HWP3·HWP 저장본·HWPX 저장본은 각각 전 1쪽이며 세 입력의 최저는 Native 98.46736%·최적화 fresh WASM 98.47794%다. 실제 독립 한컴 HWP5 재열기·HWPX 저장에서도 77개 문자가 같다. [문자 대조](../assets/pr9070/linechar-before-after.json) · [한컴 재열기 영수증](../assets/pr9070/hancom-hwp5-reopen.json) · [최종 전쪽 TSV](../assets/jinho-von-choi-20261114/final-source-8ad/feature-validation.json) · [Print·글꼴 근거](../assets/pr9070/README.md).

## 입력·시각 근거와 호스트 장애

공개 원 합성 HWP3의 용지 0·비율 0은 parser-only fixture로 분리했다. 실제 사양의 정상 specimen을 별도로 생성해 사용자가 수동 Print한 PDF를 정식 보존했다. 7개 괘선 집합·일반 기호를 같은 배율로 직접 읽었다. PDF의 실제 Haansoft Batang hmtx·전진을 근거로 명시 font environment를 공급했으며 제품 기본값을 바꾸지 않았다. 사용자가 안내한 로컬 HBATANG을 실제로 사용했다.

호스트 자동 인쇄 실패와 사용자 재부팅은 [중단·복구 기록](jinho_von_choi_20261114_interruption.md)에 남겼다. Windows 호스트의 Hwp.exe·COM·인쇄 자동화는 계속 비활성이다. 사용자 수동 Print와 별도 원격 MCP 정상 재열기를 호스트 자동화 성공으로 바꾸어 기록하지 않는다.

## 최종 공통 검증과 다음 조건

[최종 원장](../assets/jinho-von-choi-20261114/final-source-8ad/README.md)의 17개 필수 단계가 모두 통과했다.
정본 보호의 하락·누락은 0행이며 수용 예외를 적용하지 않았다. 최신 devel의 추가 문서 389쪽과
RowBreak 14쪽씩도 실제 source로 대조했다. 원 사례와 추가 문서의 2%p 초과 감소는 모두 0쪽이다.
감소 영역은 원본 Print·base·후보를 같은 배율로 직접 읽고 실제 export의 bbox·내용·순서를 확인했다.

실행 입력·독립 PDF 118개는 실제 SHA-256과 Git blob을 대조했고 미커밋 입력은 0개다.
초기 잘못된 필터 호출·보안 입력 전달·중간 source의 미달은 [조사 이력](../assets/jinho-von-choi-20261114/protected-environment-review.md)에
보존했다. 과거 실패를 지우거나 과거 통과 수를 최종 검증에 합산하지 않는다.
현재 다음 조건은 준비한 upstream 수용 PR 경로의 원격 승인, 해당 최신 head CI·self-review·merge다.
원 기여 SHA와 fork branch를 보존하고 연결 이슈는 각 검증 범위에 맞게 후속 처리한다.

## 제출 중 최신 devel 재검증

#9093을 포함한 `1805d8f08c`를 로컬 후보에 병합했다. [실제 Studio 기능 보존](../assets/jinho-von-choi-20261114/final-source-8ad/incoming9093/README.md)에서 양쪽 12개 조합이 같고 후보 전체 2,798건·브라우저 26건이 통과했다. Rust·독립 시각 검증 경로는 바뀌지 않았다.

## Merge 후 contributor PR comment 계획

- 수용 PR 실제 번호·merge SHA·성공 CI URL·self-review 결과를 기록하고 원 commit 보존을 확인한 뒤 게시한다. 대상은 [PR #9070](https://github.com/edwardkim/rhwp/pull/9070)다.
- 운영 기록·archive review·증적의 devel 반영과 로컬 devel 동기화를 먼저 완료한다. 개별 review 경로는 `mydocs/pr/archives/pr_9070_review.md`다.
- 원 PR 본문을 API로 대조했으며 메인테이너 보완과 원 기여를 구분한다. 원 fork branch를 보존한다. 부모 #7445·#7620은 OPEN을 유지한다.
- 문서 비교 절차는 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)이다. 각 대표의 review·standalone overlay를 모두 첫 댓글에 Markdown 이미지로 표시한다.

| 입력·쪽 | 경로 | raw | pixel match | visual_accuracy_proxy_percent | review / overlay 경로 |
| --- | --- | ---: | ---: | ---: | --- |
| hwp3-specimen p1 | native | 98.46736% | 99.21167% | 35.15834% | `mydocs/pr/assets/jinho-von-choi-20261114/final-source-8ad/representatives/linechar-native-review.png` / `mydocs/pr/assets/jinho-von-choi-20261114/final-source-8ad/representatives/linechar-native-overlay.png` |
| hwp3-specimen p1 | wasm | 98.47794% | 99.17990% | 33.41232% | `mydocs/pr/assets/jinho-von-choi-20261114/final-source-8ad/representatives/linechar-wasm-review.png` / `mydocs/pr/assets/jinho-von-choi-20261114/final-source-8ad/representatives/linechar-wasm-overlay.png` |

이 값들은 자동 보조 지표이며 사람 판정 정확도가 아니다. 실제 같은 페이지의 모양·위치·내용을 직접 판독한 최종 검증을 연결한다. 괘선 full 실행은 경로별 flagged 0/1이다.
- 게시 전에 merge SHA의 raw PNG 응답·형식·해시를 현재 `representatives/receipts.json`과 대조하고, 게시 후 API 본문·실제 이미지 표시를 확인한다. 대응 증거가 없거나 CI·merge가 미완료이면 게시하지 않는다.

## 원격 수용 완료

[통합 PR #9096](https://github.com/edwardkim/rhwp/pull/9096)을 `e4f8ff2d637d5b0a313211455ae3ccc53444d969`로 병합했습니다. 정확한 검토 head는 `b6a91166761b212d8501faf70b4b13e19b26474e`이며, 원 PR도 2026-11-15에 간접 MERGED 상태를 확인했습니다. 원 기여 SHA와 fork branch를 보존했습니다. [통합 종료 기록](pr_9096_review.md)에 실제 CI·연결 이슈·후속 처리 근거를 기록했습니다.
