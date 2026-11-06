# PR #8642 리뷰 — fix(typeset): 앞 조각만 들어가는 각주는 앞 조각만 비용으로 잡고, 블록 표 각주 조각은 paint 측정으로 잰다 (#8468 후속)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8642](https://github.com/edwardkim/rhwp/pull/8642), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `7d40345bc014f3cb4eca4235783afcfb670899f9`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(typeset): 앞 조각만 들어가는 각주는 앞 조각만 비용으로 잡고, 블록 표 각주 조각은 paint 측정으로 잰다 (#8468 후속)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인

- [tests/cases/issue_8468_footnote_line_reserve.rs](../../tests/cases/issue_8468_footnote_line_reserve.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 검증 뒤 제품·테스트 코드는 변경하지 않았다. 검증 원본·Print 기준의 추가와 문서 기록을 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8642_review_impl.md) · [전체 순서·게이트](assets/planet6897_20261106/integration-plan.md).
