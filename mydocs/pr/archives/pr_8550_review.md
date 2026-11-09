# PR #8550 리뷰 — fix: 글자 없는 host 의 나란한 양수 오프셋 자리차지 표를 자기 오프셋에 배치 (#8516)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8550](https://github.com/edwardkim/rhwp/pull/8550), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `48b1c5c8fed6c8a6dcb7ff0a3a1c28648558d8eb`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix: 글자 없는 host 의 나란한 양수 오프셋 자리차지 표를 자기 오프셋에 배치 (#8516)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인

- [tests/cases/issue_8516_textless_coanchored_floats.rs](../../../tests/cases/issue_8516_textless_coanchored_floats.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8550_review_impl.md) · [전체 순서·게이트](../assets/planet6897_20261106/integration-plan.md).

## 실제 cover 원본과 주장 페이지 대조

R11 source `048f44da99d83e186f3a9e139563d20545c8bdb4`의 같은 원본·독립 Print를
96 DPI·보정 전2px 내용 실루엣으로 전쪽 비교했다. Native/fresh WASM 모두
4쪽으로 쪽수가 일치하며 전쪽 최저는 각각
99.79% / 99.89%다.
아래 주장 페이지의 review와 overlay를 두 경로 모두 직접 판독했다.

상단 왼쪽 `연구서/공개` 두 행 표와 오른쪽 `재정관리총괄과-147(2021.2.23.)` 상자를 각각 원래 양수 오프셋에 놓는다. 가운데 제목·날짜·기획재정부·KDI 순서와 두 표의 분리, 본문 중첩 부재를 직접 확인했다. 같은 host에 있다는 이유로 두 표를 같은 원점에 포개지 않는다.

[Native 전쪽 TSV](../assets/planet6897_20261106/pr8550/r11/cover/native.tsv) · [fresh WASM 전쪽 TSV](../assets/planet6897_20261106/pr8550/r11/cover/wasm.tsv) ·
[원본·Print·producer 해시](../assets/planet6897_20261106/pr8550/r11/cover/provenance.json).

| 쪽 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| 1 | [review](../assets/planet6897_20261106/pr8550/r11/cover/native-review-001.png) · [overlay](../assets/planet6897_20261106/pr8550/r11/cover/native-overlay-001.png) | [review](../assets/planet6897_20261106/pr8550/r11/cover/wasm-review-001.png) · [overlay](../assets/planet6897_20261106/pr8550/r11/cover/wasm-overlay-001.png) |

이는 R11 완료 증거다. 후속 제품 보정의 최종 head·필수 회귀·lint·브라우저 검증 전이며 최종 승인으로 판정하지 않는다.
