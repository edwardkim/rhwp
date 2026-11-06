# PR #8662 리뷰 — fix: 저장 줄 없는 HWP5 TAC 표 행 높이·넘친 공백 줄 끝·변경 추적 0x08 저장 줄 유지 (#8639)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8662](https://github.com/edwardkim/rhwp/pull/8662), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `f84e0f3ffe3b154d286bc878fc90facdb7ced153`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix: 저장 줄 없는 HWP5 TAC 표 행 높이·넘친 공백 줄 끝·변경 추적 0x08 저장 줄 유지 (#8639)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 표지판 관리대장의 실제 Print 조건 대조

기존 회귀가 명시한 Haansoft Batang 조건과 2022 PDF의 실제 내장 프로그램을 맞춰
같은 원본 전1쪽을 비교했다. 제품 `048f44da99`에서 Native·fresh WASM 모두 raw
**100.00%**다. 「의무설치기한」 머리 칸의 3행 보존·셀 내부 포함·표 겹침 없음과
뒤 관리카드·설치 표의 경계를 직접 대조했다.

2020 PDF는 서울한강·서울남산을 사용하므로 같은 이름의 별도 Print를 동일 조건으로
취급하지 않는다. 초기 2022 측정의 글꼴 조건 누락을 보정했으며 원본·PDF·임계값은
바꾸지 않았다. [원 주장 페이지의 두 TSV·review·overlay와 조건](assets/planet6897_20261106/pr8662/print2022/README.md).
나머지 주장과 통합 최종 검증은 아직 완료되지 않아 최종 판정은 보류한다.

## 회귀·입력 확인

- [tests/cases/issue_8619_dotum_one_dot_leader_supplier.rs](../../tests/cases/issue_8619_dotum_one_dot_leader_supplier.rs)
- [tests/cases/issue_8639_hwp5_no_lineseg_tac_row_growth.rs](../../tests/cases/issue_8639_hwp5_no_lineseg_tac_row_growth.rs)
- [tests/cases/issue_8639_trackchange_stored_rows.rs](../../tests/cases/issue_8639_trackchange_stored_rows.rs)
- [tests/cases/issue_8639_trailing_space_overflow.rs](../../tests/cases/issue_8639_trailing_space_overflow.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8662_review_impl.md) · [전체 순서·게이트](assets/planet6897_20261106/integration-plan.md).

## 메인터너 보정 중간 검증

표지판 관리대장의 원본 서울서체 프로그램을 실제로 선택하도록 공통 글꼴 이름 조회와
미분류 원본 TrueType의 메트릭 공급을 보정했다.
[원인·실제 producer·조건](assets/planet6897_20261106/pr8662/sign-board-analysis.md)을 따른다.

| 독립 기준과 범위 | 보정 전 Native | 보정 후 Native | 증거 |
| --- | ---: | ---: | --- |
| 한컴 2020 Print, 전체 1쪽 | 33.10% | 100.00% | [전쪽 TSV](assets/planet6897_20261106/pr8662/sign-board-2020-native.tsv), [같은 쪽 review](assets/planet6897_20261106/pr8662/sign-board-2020-native-review.png), [overlay](assets/planet6897_20261106/pr8662/sign-board-2020-native-overlay.png) |

전체 시스템 글꼴을 사용하며 특정 글꼴 경로나 대체 환경을 지정하지 않은 자동 선택 결과다.
실제 crate의 Cargo 라이브러리 경로에서 `font_paths.rs` 검사 6건을 최소 글꼴 환경으로
실행해 모두 통과·경고 0건·exit 0을 확인했다. 단독 파일 컴파일의 미사용 항목 경고를
제품 코드에서 억제하지 않고 실제 소비 모듈을 포함해 검증했다.
실제 가족 이름은 같은 프로그램을 가리키고 Full Name·PostScript 이름을 가족으로
승격하지 않는 반례를 함께 확인했다. 원본 언어 슬롯의 표시 family 공유 반례 1건도 정식 Cargo 경로에서 통과했다.
이 7건의 집중 검사는 전체 Cargo 회귀·lint·fresh WASM을 대신하지 않는다.
2022 Print 대조군은 그 출력의 실제 Haansoft Batang 대체 환경과 구분해 검사한다.
