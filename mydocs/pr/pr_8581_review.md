# PR #8581 리뷰 — fix(renderer): 줄 머리 TAC 표 뒤 문단 끝 공백에 줄을 주지 않는다 (#8574)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8581](https://github.com/edwardkim/rhwp/pull/8581), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `f25efcf2c9dba5793f78a5e5d9c845944c562dfb`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(renderer): 줄 머리 TAC 표 뒤 문단 끝 공백에 줄을 주지 않는다 (#8574)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인

- [tests/cases/issue_8574_tac_host_trailing_space_line.rs](../../tests/cases/issue_8574_tac_host_trailing_space_line.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8581_review_impl.md) · [전체 순서·게이트](assets/planet6897_20261106/integration-plan.md).

## 원 주장 페이지의 실제 두 세대 Print 대조

R11 제품 `048f44da99`의 같은 원본 `samples/issue8574/tac_trailing_space_nols.hwp`를
기존 한컴 2020·2024 Print와 각각 비교했다. 입력은 한컴 저장본에서 호스트의 저장
LineSeg만 제거한 재현물이며 정상 저장본 자체와 구분한다. 임의 다른 문서의 PNG가 아니다.
96dpi·compat2022·색상 경계 보정 전 2px raw로 각 문서 전체 1쪽, Native/fresh WASM
모두 **100.00%**, PDF 쪽수도 각각 1=1이다.

| 독립 Print | Native 전쪽 TSV·주장 1쪽 | fresh WASM 전쪽 TSV·같은 1쪽 |
| --- | --- | --- |
| 한컴 2020 | [TSV](assets/planet6897_20261106/pr8581/r11/2020-native.tsv) · [review](assets/planet6897_20261106/pr8581/r11/2020-native-review.png) · [overlay](assets/planet6897_20261106/pr8581/r11/2020-native-overlay.png) | [TSV](assets/planet6897_20261106/pr8581/r11/2020-wasm.tsv) · [review](assets/planet6897_20261106/pr8581/r11/2020-wasm-review.png) · [overlay](assets/planet6897_20261106/pr8581/r11/2020-wasm-overlay.png) |
| 한컴 2024 | [TSV](assets/planet6897_20261106/pr8581/r11/2024-native.tsv) · [review](assets/planet6897_20261106/pr8581/r11/2024-native-review.png) · [overlay](assets/planet6897_20261106/pr8581/r11/2024-native-overlay.png) | [TSV](assets/planet6897_20261106/pr8581/r11/2024-wasm.tsv) · [review](assets/planet6897_20261106/pr8581/r11/2024-wasm-review.png) · [overlay](assets/planet6897_20261106/pr8581/r11/2024-wasm-overlay.png) |

네 조합의 review·overlay를 직접 판독했다. 표의 시작·끝과 칸 내부 글자가 보존되고,
「표 뒤 첫 줄」과 「끝」이 Print와 같은 줄 간격·순서로 이어진다. 말미 공백을 위한 추가
본문 줄이 표 뒤를 밀어내지 않는다. 100%는 원시 픽셀 바이트 동일을 뜻하지 않는다.
[입력·PDF·실제 PNG·TSV 해시](assets/planet6897_20261106/pr8581/r11/provenance.json)를 보존했다.
이는 R11의 검증 완료 기록이다. 이후 보정 후보의 영향 검증·최종 필수 회귀가 남아 있어
통합 PR 전체의 최종 승인을 이 한 사례로 대신하지 않는다.
