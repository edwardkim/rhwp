# PR #8645 리뷰 — fix(layout): TAC 표 앞 저장 줄 — 쪽 기준 상자에 밀려 누락되고 그리기가 줄 밖 개체 높이로 되감던 것 (#8641)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8645](https://github.com/edwardkim/rhwp/pull/8645), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `2aedb2a8580da9bb1b2d97643d3a97defa4f1660`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(layout): TAC 표 앞 저장 줄 — 쪽 기준 상자에 밀려 누락되고 그리기가 줄 밖 개체 높이로 되감던 것 (#8641)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인

- [tests/cases/issue_8641_tac_prefix_line_owner.rs](../../../tests/cases/issue_8641_tac_prefix_line_owner.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8645_review_impl.md) · [전체 순서·게이트](../assets/planet6897_20261106/integration-plan.md).

## 실제 선행 줄·쪽 기준 상자의 주장 검증

R11 source `048f44da99d83e186f3a9e139563d20545c8bdb4`에서 각 원본과 독립 Print의
전쪽을 Native/fresh WASM으로 비교했다. 측정법은 96 DPI·보정 전 2px 내용 실루엣이며
두 입력 모두 각각 1쪽으로 쪽수가 일치한다.

| 입력 | Native 전쪽 최저 | fresh WASM 전쪽 최저 | 직접 판독 |
| --- | ---: | ---: | --- |
| field-page-box | 99.98% | 100.00% | 제목·수신자·접수번호·담당자·본문·날짜·발신명 |
| tall-prefix | 100.00% | 100.00% | 중간보고서 제목·단위·연도 머리행·마지막 대졸 행 |

첫 입력의 `검사원지정요구서`와 `접수번호` 등 표 앞 저장 줄이 빠지지 않는다.
쪽 기준 상자에 밀려 제목·담당자·계약 정보가 누락되지 않고, 결재 표와 뒤 본문의
소속·순서를 Print와 같은 페이지에서 확인했다. 두 번째 입력은 제목과 단위, 표 첫 행과
끝 행을 함께 판독해 앞 줄 점유가 큰 TAC 표 높이로 되감기지 않는지 대조했다.

같은 입력의 [field-page-box Native TSV](../assets/planet6897_20261106/pr8645/r11/field-page-box/native.tsv)·
[fresh WASM TSV](../assets/planet6897_20261106/pr8645/r11/field-page-box/wasm.tsv),
[tall-prefix Native TSV](../assets/planet6897_20261106/pr8645/r11/tall-prefix/native.tsv)·
[fresh WASM TSV](../assets/planet6897_20261106/pr8645/r11/tall-prefix/wasm.tsv)를 제출한다.
실제 입력·PDF·producer SHA는 각 디렉터리의 provenance.json에 있다.

field-page-box Native [review](../assets/planet6897_20261106/pr8645/r11/field-page-box/native-review-001.png)·
[overlay](../assets/planet6897_20261106/pr8645/r11/field-page-box/native-overlay-001.png),
fresh WASM [review](../assets/planet6897_20261106/pr8645/r11/field-page-box/wasm-review-001.png)·
[overlay](../assets/planet6897_20261106/pr8645/r11/field-page-box/wasm-overlay-001.png).
tall-prefix Native [review](../assets/planet6897_20261106/pr8645/r11/tall-prefix/native-review-001.png)·
[overlay](../assets/planet6897_20261106/pr8645/r11/tall-prefix/native-overlay-001.png),
fresh WASM [review](../assets/planet6897_20261106/pr8645/r11/tall-prefix/wasm-review-001.png)·
[overlay](../assets/planet6897_20261106/pr8645/r11/tall-prefix/wasm-overlay-001.png).

후속 보정의 최종 head·회귀·gate를 완료한 뒤 전체 통합 판정을 갱신한다.
