# PR #8559 리뷰 — fix(layout): 쪽·용지 기준 자리차지 표 띠에 밀린 빈 host 줄은 줄높이와 줄간격을 모두 전진 (#8554)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8559](https://github.com/edwardkim/rhwp/pull/8559), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `efa20ac2eafe1137b370d270f459eac54109b5ca`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(layout): 쪽·용지 기준 자리차지 표 띠에 밀린 빈 host 줄은 줄높이와 줄간격을 모두 전진 (#8554)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인

- [tests/cases/issue_8554_paper_band_host_line.rs](../../../tests/cases/issue_8554_paper_band_host_line.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8559_review_impl.md) · [전체 순서·게이트](../assets/planet6897_20261106/integration-plan.md).

## 원본 2913671 별지 1의 주장 페이지 대조

같은 원본 `samples/issue8554/2913671-appendix1-family-register.hwp`와 기존 한컴 2022
Print의 전체 2쪽을 R11 제품 `048f44da99`에서 비교했다. 96dpi·compat2022·보정 전
2px 내용 실루엣 raw는 Native **99.65% / 100.00%**, fresh WASM **99.66% / 100.00%**다.
각각 PDF와 2=2쪽이며 과거 기여자 수치를 현재 측정으로 바꾸어 적지 않았다.

- Native 전쪽 [TSV](../assets/planet6897_20261106/pr8559/r11/native.tsv), 주장 1쪽 [review](../assets/planet6897_20261106/pr8559/r11/native-review.png) · [overlay](../assets/planet6897_20261106/pr8559/r11/native-overlay.png).
- fresh WASM 전쪽 [TSV](../assets/planet6897_20261106/pr8559/r11/wasm.tsv), 같은 1쪽 [review](../assets/planet6897_20261106/pr8559/r11/wasm-review.png) · [overlay](../assets/planet6897_20261106/pr8559/r11/wasm-overlay.png).
- [입력·PDF·실제 producer와 증거 해시](../assets/planet6897_20261106/pr8559/r11/provenance.json).

두 경로의 같은 1쪽을 직접 판독했다. 첫 자리차지 표 아래의 「2. 입적할 직계비속…」
본문과 다음 표가 독립 Print의 간격·순서로 이어진다. 표 안 한글·한자와 서명 위치가
보존되고 첫 표 뒤 빈 호스트 줄의 높이를 빼서 다음 본문이 올라가는 증상은 없다.
전쪽 90%·주장 영역의 직접 판독은 확인했지만 다른 메인터너 보정 뒤 영향 검증과 통합
최종 필수 검증이 남아 있어 현재는 개별 최종 판정도 보류한다.
