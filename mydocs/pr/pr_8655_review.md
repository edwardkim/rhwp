# PR #8655 리뷰 — fix(layout): TAC 표만 담은 문단의 동반 표를 줄 소유자 기준선에 앉힌다 (#8653)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8655](https://github.com/edwardkim/rhwp/pull/8655), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `cd4cc992f30fdc50478217d3b954c9fd9e1d34e7`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(layout): TAC 표만 담은 문단의 동반 표를 줄 소유자 기준선에 앉힌다 (#8653)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인

- [tests/cases/issue_8653_tac_companion_box_on_owner_baseline.rs](../../tests/cases/issue_8653_tac_companion_box_on_owner_baseline.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8655_review_impl.md) · [전체 순서·게이트](assets/planet6897_20261106/integration-plan.md).

## 실제 같은 줄 동반 표 대조

R11 source `048f44da99d83e186f3a9e139563d20545c8bdb4`에서 같은 입력·독립 Print를
Native/fresh WASM으로 전체 1쪽 비교했다. 쪽수는 모두 1쪽이고 raw는 양쪽
**100.00%**다(96 DPI, 보정 전 2px 내용 실루엣).

왼쪽 시행·수신자 표와 오른쪽 결재 표의 서로 다른 윗변·아랫변을 직접 대조했다.
동반 표를 같은 상단으로 강제 정렬하지 않고 소유 줄의 기준선 관계를 유지한다.
오른쪽 표의 두 줄 날짜·이름, `결재` 세로 칸, 왼쪽 마지막 `수신자 건축부장`까지
Print와 대응하며 겹침·누락·임의 이월이 없다.

[Native 전쪽 TSV](assets/planet6897_20261106/pr8655/r11/companion-baseline/native.tsv) ·
[fresh WASM 전쪽 TSV](assets/planet6897_20261106/pr8655/r11/companion-baseline/wasm.tsv) ·
[입력·PDF·producer SHA](assets/planet6897_20261106/pr8655/r11/companion-baseline/provenance.json).
Native [review](assets/planet6897_20261106/pr8655/r11/companion-baseline/native-review-001.png)·
[overlay](assets/planet6897_20261106/pr8655/r11/companion-baseline/native-overlay-001.png),
fresh WASM [review](assets/planet6897_20261106/pr8655/r11/companion-baseline/wasm-review-001.png)·
[overlay](assets/planet6897_20261106/pr8655/r11/companion-baseline/wasm-overlay-001.png).

이 절은 R11의 실제 주장 검증이다. 통합 최종 head와 필수 회귀·gate 완료 여부는
후속 보정 검증 결과와 함께 판정한다.
