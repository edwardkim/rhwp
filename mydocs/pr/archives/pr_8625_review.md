# PR #8625 리뷰 — fix(typeset): #8620 글자처럼 취급 표 host 의 컨트롤 전용 꼬리 줄은 쪽을 넘으면 다음 쪽에 놓는다

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8625](https://github.com/edwardkim/rhwp/pull/8625), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `39ceb92ace1a6ad2ca9578c5b72b6940bb096281`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(typeset): #8620 글자처럼 취급 표 host 의 컨트롤 전용 꼬리 줄은 쪽을 넘으면 다음 쪽에 놓는다

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인

- [tests/cases/issue_8620_tac_control_only_tail_next_page.rs](../../../tests/cases/issue_8620_tac_control_only_tail_next_page.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8625_review_impl.md) · [전체 순서·게이트](../assets/planet6897_20261106/integration-plan.md).

## 요약문 표·컨트롤 전용 꼬리 줄의 실제 쪽 대조

R11 source `048f44da99d83e186f3a9e139563d20545c8bdb4`에서 원 PR이 사용하는
`summary_tail_trimmed.hwpx`와 기존 독립 한컴 Print PDF를 96 DPI·보정 전2px
내용 실루엣으로 비교했다. Native/fresh WASM 모두 **3쪽, 전쪽 100.00%**다.
두 경로의 1·2·3쪽 review와 overlay를 직접 판독했다.

1쪽의 `중간결과보고서 요약문` 제목, 연구과제명~연구진구성 다섯 머리행,
`최종보고서에 수록` 내용 및 표의 큰 빈 영역과 끝선이 같은 쪽에 있다.
2쪽은 독립 Print와 마찬가지로 보이는 표·글자가 없는 쪽이다. 3쪽에는
`중간결과보고서` 머리 상자와 `차 례`, 하단 `- 1 -`이 보존된다. 따라서
단순히 3쪽이라는 수치만 확인한 것이 아니라, 원 PR의 컨트롤 전용 꼬리 줄이
만드는 중간 쪽과 그 뒤 명시적 쪽 나누기의 내용 소속을 함께 확인했다.
100%는 해당 실루엣 판정값이며 글꼴 획의 픽셀 동일성을 뜻하지 않는다.

[Native 전쪽 TSV](../assets/planet6897_20261106/pr8625/r11/summary-tail/native.tsv) ·
[fresh WASM 전쪽 TSV](../assets/planet6897_20261106/pr8625/r11/summary-tail/wasm.tsv) ·
[원본·Print·producer 해시](../assets/planet6897_20261106/pr8625/r11/summary-tail/provenance.json).

| 쪽 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| 1 | [review](../assets/planet6897_20261106/pr8625/r11/summary-tail/native-review-001.png) · [overlay](../assets/planet6897_20261106/pr8625/r11/summary-tail/native-overlay-001.png) | [review](../assets/planet6897_20261106/pr8625/r11/summary-tail/wasm-review-001.png) · [overlay](../assets/planet6897_20261106/pr8625/r11/summary-tail/wasm-overlay-001.png) |
| 2 | [review](../assets/planet6897_20261106/pr8625/r11/summary-tail/native-review-002.png) · [overlay](../assets/planet6897_20261106/pr8625/r11/summary-tail/native-overlay-002.png) | [review](../assets/planet6897_20261106/pr8625/r11/summary-tail/wasm-review-002.png) · [overlay](../assets/planet6897_20261106/pr8625/r11/summary-tail/wasm-overlay-002.png) |
| 3 | [review](../assets/planet6897_20261106/pr8625/r11/summary-tail/native-review-003.png) · [overlay](../assets/planet6897_20261106/pr8625/r11/summary-tail/native-overlay-003.png) | [review](../assets/planet6897_20261106/pr8625/r11/summary-tail/wasm-review-003.png) · [overlay](../assets/planet6897_20261106/pr8625/r11/summary-tail/wasm-overlay-003.png) |

후속 제품 보정의 최종 head·필수 회귀·lint·브라우저 검증 완료 전이며 최종 승인으로 판정하지 않는다.
