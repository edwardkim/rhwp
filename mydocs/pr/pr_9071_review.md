# PR #9071 메인테이너 검토

## 최종 판정

**메인터너 보정 후 수용 가능.** 기존 보호값·영문 스타일·그림 외곽의 차이를 예외 없이 보완했다.
제품 source는 `8ad1c5c08a`, 최신 base는 `2d83fa4c8a`이며 실제 비교용 `e7ee272cfa`와 제품·시각 harness가 같다.
전체 회귀 13,608건·Native 공식 4,073건·모든 lint와 필수 편람 gate를 통과했다.
Native/fresh WASM 보호 각 1,771쪽·3,542행의 공식값 하락은 0건이며 상승 19행을 새 하한에 반영했다.
원 사례·대조군 각 134쪽의 최저는 93.31%·92.35%다. 직접 확대 판독·최신 base 전쪽 비교도 완료했다.
[최종 검증·생산자·전쪽 TSV](assets/jinho-von-choi-20261114/final-source-8ad/README.md).
사용자 승인 후 [수용 PR #9096](https://github.com/edwardkim/rhwp/pull/9096)을 생성했다. 공개 최종 review·merge·issue close는 최신 head CI 확인 뒤 진행한다.

작성자 JinHo-von-Choi, 원 head `a1bd3fa4edb8bb79c0ae312166358b6be299e5d0`. reviewer `edwardkim`이며 원 commit을 보존했다. 원 이슈는 #7631·#8318·#9046·#9049이고 추가 보완의 같은 원 신고는 #8307이다. 라우팅은 [검토 계획](jinho_von_choi_20261114_review_plan.md)과 공통 원장에 기록한다.

## 원 주장과 기존 해결 추적

| 범위 | 실제 판정 |
| --- | --- |
| #7631 | flowWithText·allowOverlap을 공통 속성과 HWP attr bit 13·14에 같이 보존한다. 글자 취급 bit 0은 유지하며 조합별 HWP/HWPX 왕복을 확인했다. |
| #8318 | 색 없음은 구형 소비자의 `#FFFFFFFF` 저장 계약으로 통일하고 기존 `none`·대소문자 입력도 읽는다. 실제 2018 원본/보정본과 원 신고 자료를 확인했다. 현재 검증의 2020 Print를 새 2018 앱 실행이라고 주장하지 않는다. |
| #9046 | #3930의 Even→Both 호환 복제가 #8906의 실제 적용 종류 bit 저장 이후 슬롯을 잃게 했다. 안정 종류 정렬과 내용을 보존하고 중복 복제를 제거한다. 정상 한컴 11조합과 실제 HWP 저장본의 종류·개수·내용·geometry를 대조했다. |
| #9049 | i32 읽기·쓰기는 #7862의 `1407917803a4d393d7d4e549b218815011bc2d54` → #7867 → #7923에서 이미 통합됐다. 신고한 v0.8.7은 그보다 이전 source다. 이번 PR은 음수·0·큰 값의 저장 회귀를 보강하며 신규 제품 수리로 계산하지 않는다. |

기존 `issue_3500`·`issue_5598`의 색 없음 및 `issue_8262`의 표 attr 검사를 같은 저장 계약에서 재검토했다. 정확한 보호를 제거하거나 ZIP의 무관한 차이를 일반 허용하지 않는다.

## 승인된 공통 배치 보완

`0fe9e02170`은 PageAreas의 원래 본문 원점과 현재 맞쪽 본문 원점으로 공통 용지 프레임을 만든다. Both 기본 바탕쪽의 그림·사각형·표가 같은 프레임을 소비하고 뒤의 본문에는 원래 물리 용지 원점을 복원한다. 종류별 적용 범위와 최종 소비 위치는 [바탕쪽 원인·구현 계획](pr_9071_review_impl.md)에 남겼다. clamp·너비 축소·backend별 보정은 없다.

`5e61789786`의 TAC 경계 보완은 `eeaecff59e`에서 원점 0 재시작과 양수 좌표 되감김을 구분하도록 정정했다. 기존 `stored_owned_tac_line`으로 바깥 여백까지 혼자 소유한 원본 줄을 확인하고, fit·실제 후행 간격이 같은 결과를 소비한다. 편집/합성 정보·일반 줄·공유 줄을 해당 원본 계약의 근거로 삼지 않는다. 기존 #8191/#7445의 양수 되감김 기대값은 유지했고 전체 회귀에서 PASS했다. [TAC 근거·소비 경로·독립 Print](pr_9071_color_rendering_finding.md).

원 바탕쪽 11조합의 HWPX·실제 HWP 저장본 66쪽, 색상 원본·실제 저장본 6쪽, 대칭/반전 여백과 정상 사각형/표 12쪽은 Native/fresh WASM 모두 100.00%다. #8307의 원 신고와 같은 HWP 및 정상 HWPX를 당시 실제 한컴 2018 Print와 대조한 6쪽도 양쪽 모두 100.00%다. [전쪽 근거·확대 판독](assets/pr9071/README.md) · [#8307](assets/pr9071/issue8307/README.md).

33쪽씩의 집중 시각 확인 뒤 등록한 관계 검사는 바탕쪽 개체·본문 이동, Even 비적용 쪽, 표와 후속 문단의 쪽 소속·순서·누락·중복·빈 문단을 보호한다. HWP 저장은 실제 CLI의 `export_hwp_with_adapter`를 호출한다. 절대 px 핀은 없다. [실제 CLI 본문 편집·두 형식 2회 저장 재열기](assets/jinho-von-choi-20261114/final-source-8ad/cli-repeated-save.json)에서도 바탕쪽·그림 bytes를 보존했다.

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
