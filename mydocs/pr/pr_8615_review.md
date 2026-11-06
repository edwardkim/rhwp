# PR #8615 리뷰 — fix(layout): 칸 안 빈 host 의 글 위아래 표 띠가 뒤 문단 앞 간격을 흡수한다 (#8610)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8615](https://github.com/edwardkim/rhwp/pull/8615), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `7d1d9b51f2f2cc9f92aef46eefe101a6d4b433a5`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(layout): 칸 안 빈 host 의 글 위아래 표 띠가 뒤 문단 앞 간격을 흡수한다 (#8610)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인

- [tests/cases/issue_8610_top_bottom_band_absorbs_spacing.rs](../../tests/cases/issue_8610_top_bottom_band_absorbs_spacing.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8615_review_impl.md) · [전체 순서·게이트](assets/planet6897_20261106/integration-plan.md).

## 실제 NARS 문서 판독과 자동 분할 경계선 보정

R11 source의 같은 원본·독립 Print를 두 경로 review/overlay로 직접 확인했다.
2021년 보도자료의 문단 간격은 원 주장대로 보존되지만 1쪽 하단·2쪽 상하·3쪽
상단에 Print에 없는 가로선이 남아 있었다. 98%대 점수만으로 승인하지 않았다.
[원본 바이트·한컴 저장·공식 설명과 소비 경로](pr_8615_review_impl.md#자동-분할-경계선의-메인터너-보정-계획)를
대조해 모델의 별도 경계선 설정 보존과 공통 부분 표 경계 적용을 보정했다.

| 2021 원본 쪽 | R25 보정 전 Native | R26b 보정 후 Native |
| --- | ---: | ---: |
| 1 | 98.47% | 99.85% |
| 2 | 97.89% | 100.00% |
| 3 | 96.50% | 100.00% |

96 DPI·보정 전2px 내용 실루엣이며 전3쪽이 기존 독립 Print와 일치한다.
보정 후 세 쪽 review와 overlay를 직접 대조해 불필요한 가로선 해소, 제목 띠와
뒤 본문, 마지막 실제 종료선·담당자 표기의 보존을 확인했다. 정상 NARS2019
전1쪽과 별도 경계선 설정이 꺼진 #5885 전7쪽도 보정 전후 모두100.00%다.

[보정 전 Native TSV](assets/planet6897_20261106/pr8615/r26/before-native.tsv) ·
[보정 후 Native TSV](assets/planet6897_20261106/pr8615/r26/native.tsv) ·
[실제 producer·소스·PNG 해시](assets/planet6897_20261106/pr8615/r26/native-provenance.json).

1쪽 Native [review](assets/planet6897_20261106/pr8615/r26/native-review-001.png) · [overlay](assets/planet6897_20261106/pr8615/r26/native-overlay-001.png).
2쪽 Native [review](assets/planet6897_20261106/pr8615/r26/native-review-002.png) · [overlay](assets/planet6897_20261106/pr8615/r26/native-overlay-002.png).
3쪽 Native [review](assets/planet6897_20261106/pr8615/r26/native-review-003.png) · [overlay](assets/planet6897_20261106/pr8615/r26/native-overlay-003.png).

새 fresh WASM·저장측 집중 검사·최종 전체 회귀·보호 PR 전쪽·lint·브라우저 검증은 진행 중이다. Native 중간 결과를 최종 승인으로 판정하지 않는다.
