# PR #8649 리뷰 — fix(layout): 칸 안의 쪽 기준 글 앞으로 표를 칸 흐름에 쌓지 않는다 (#8648)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8649](https://github.com/edwardkim/rhwp/pull/8649), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `c546460f55cd9cf94d8106cde271750e0a131c0c`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(layout): 칸 안의 쪽 기준 글 앞으로 표를 칸 흐름에 쌓지 않는다 (#8648)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인

- [tests/cases/issue_8648_cell_page_overlay_no_flow.rs](../../../tests/cases/issue_8648_cell_page_overlay_no_flow.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8649_review_impl.md) · [전체 순서·게이트](../assets/planet6897_20261106/integration-plan.md).

## 실제 발신명 overlay와 셀 흐름 대조

R11 source `048f44da99d83e186f3a9e139563d20545c8bdb4`에서 같은 원본·독립 Print의
전1쪽을 Native/fresh WASM으로 비교했다. 쪽수는 모두 1쪽, raw는 양쪽 **100.00%**다
(96 DPI, 보정 전 2px 내용 실루엣).

원본의 납품 정보·검사 요청 본문·날짜와 하단 `총무부장`을 같은 페이지에서 직접
판독했다. 쪽 기준 글 앞으로 표가 셀 내용 뒤의 흐름 항목으로 중복 적층되지 않고,
본문·날짜의 원점과 별도 발신명의 쪽 기준 위치가 독립 Print와 같다. 글 앞으로 표를
숨겨 통과시킨 결과가 아니며 발신명은 양쪽 출력에 보존된다.

[Native 전쪽 TSV](../assets/planet6897_20261106/pr8649/r11/sender-overlay/native.tsv) ·
[fresh WASM 전쪽 TSV](../assets/planet6897_20261106/pr8649/r11/sender-overlay/wasm.tsv) ·
[입력·PDF·producer SHA](../assets/planet6897_20261106/pr8649/r11/sender-overlay/provenance.json).
Native [review](../assets/planet6897_20261106/pr8649/r11/sender-overlay/native-review-001.png)·
[overlay](../assets/planet6897_20261106/pr8649/r11/sender-overlay/native-overlay-001.png),
fresh WASM [review](../assets/planet6897_20261106/pr8649/r11/sender-overlay/wasm-review-001.png)·
[overlay](../assets/planet6897_20261106/pr8649/r11/sender-overlay/wasm-overlay-001.png).

R11의 실제 주장 검증이며 후속 제품 보정의 최종 head와 필수 gate는 별도로 완료한다.
