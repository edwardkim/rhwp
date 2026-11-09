# PR #8638 리뷰 — fix(renderer): 돋움 ․(U+2024)은 Haansoft Dotum 이 측정·그리기 공급 (#8619)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8638](https://github.com/edwardkim/rhwp/pull/8638), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `71bd8abf48dbad4afa09d66803f6011546add734`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(renderer): 돋움 ․(U+2024)은 Haansoft Dotum 이 측정·그리기 공급 (#8619)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인

- [tests/cases/issue_8619_dotum_one_dot_leader_supplier.rs](../../../tests/cases/issue_8619_dotum_one_dot_leader_supplier.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8638_review_impl.md) · [전체 순서·게이트](../assets/planet6897_20261106/integration-plan.md).

## 실제 확보한 원본의 두 경로 확인

원 주장의 평가결과서 원본과 실제 한컴 2020 Print를 대조했다. 원 기여자의 2022
보고값을 이번 2020 PDF의 측정값으로 재표기하지 않았다. 전체 2쪽의 R15 결과는
Native 93.90·100.00%, fresh WASM 96.56·100.00%이며 쪽수도 2=2다.

- 전쪽 TSV: [Native](../assets/planet6897_20261106/pr8638/r15/native.tsv) · [fresh WASM](../assets/planet6897_20261106/pr8638/r15/wasm.tsv).
- 주장한 1쪽 Native: [review](../assets/planet6897_20261106/pr8638/r15/native-review.png) · [overlay](../assets/planet6897_20261106/pr8638/r15/native-overlay.png).
- 같은 1쪽 fresh WASM: [review](../assets/planet6897_20261106/pr8638/r15/wasm-review.png) · [overlay](../assets/planet6897_20261106/pr8638/r15/wasm-overlay.png).

두 경로를 직접 판독했다. 연구결과의 `감독․검사` 문단이 셀 안에서 같은 5개 항목·같은
줄 구성으로 유지되고 평가결과·평가자 행이 뒤에 보존된다. 원 주장과 다른 문서의
대표 PNG로 대신하지 않았다. 표 위 제목의 세로 위치와 Native 평가결과 항목의 일부
글줄 원점에는 차이가 남아 있다. raw 90% 통과만으로 이 차이의 해결을 선언하지 않는다.
현재 최종 판정은 다른 필수 검증과 함께 계속 보류하며 남은 차이의 원인을 확인한다.
