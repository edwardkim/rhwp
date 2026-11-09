# PR #8563 리뷰 — fix(typeset): 빈 host 자리차지 표 뒤 다음 문단이 빈 자리차지 표 host 여도 저장 host 줄 프레임을 받는다 (#8560)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8563](https://github.com/edwardkim/rhwp/pull/8563), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `bc8a04e056ccfbbe371b21239ecfd13696cea0b2`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(typeset): 빈 host 자리차지 표 뒤 다음 문단이 빈 자리차지 표 host 여도 저장 host 줄 프레임을 받는다 (#8560)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인

- [tests/cases/issue_8560_textless_float_host_successor.rs](../../../tests/cases/issue_8560_textless_float_host_successor.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8563_review_impl.md) · [전체 순서·게이트](../assets/planet6897_20261106/integration-plan.md).

## 같은 안건표지의 R15 보정 결과

원 주장의 `samples/issue8560/21138757-agenda-cover-form.hwp`와 기존 한컴 2020 Print
전1쪽을 R15에서 대조했다. 96dpi·compat2022·색상 경계 보정 전 2px 내용 실루엣 raw는
Native **99.74%**, fresh WASM **99.75%**, 양쪽 모두 PDF와 1=1쪽이다.

- Native 전쪽 [TSV](../assets/planet6897_20261106/pr8563/r15/native.tsv), 주장 1쪽 [review](../assets/planet6897_20261106/pr8563/r15/native-review.png) · [overlay](../assets/planet6897_20261106/pr8563/r15/native-overlay.png).
- fresh WASM 전쪽 [TSV](../assets/planet6897_20261106/pr8563/r15/wasm.tsv), 같은 1쪽 [review](../assets/planet6897_20261106/pr8563/r15/wasm-review.png) · [overlay](../assets/planet6897_20261106/pr8563/r15/wasm-overlay.png).
- [독립 Print·Unicode 분류·공통 원인과 보정](../assets/planet6897_20261106/pr8563/ideographic-zero-plan.md), [실제 producer 해시](../assets/planet6897_20261106/pr8563/r15/provenance.json).

두 경로의 같은 영역을 직접 판독했다. 원 주장의 네 표와 뒤 표의 상하 순서·괘선이
보존된다. Native의 숫자 영 U+3007이 반각으로 좁아지고 조례 제목·제출자 뒤 글자가
당겨지던 추가 차이는 공통 CJK 분류에서 보정했다. WASM의 숫자 영 윤곽에는 PDF와
미세한 차이가 있어 픽셀 동일성을 주장하지 않는다.

새 관계 검사 `pr_8563_ideographic_zero_fallback` 2건은 공개/OFL 글꼴 5개만 노출한
환경에서 통과했다. 세 글자 크기에서 같은 CJK 전진·ASCII 0과의 구별·뒤 글자 원점을
보호하며 특정 설치 글꼴이나 절대 px 좌표를 전제로 하지 않는다. 공개 API 추적도 수정
전 실패·수정 후 통과를 확인했다. 이 집중 결과와 별개로 통합의 최종 검증을 계속한다.
