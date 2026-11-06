# PR #8650 리뷰 — fix(layout): 한 줄의 TAC 표 여러 개는 선언 폭만큼 전진한다 (#8647)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8650](https://github.com/edwardkim/rhwp/pull/8650), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `068250c0e8bc9b07b8a25911ab8002d5504f63eb`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(layout): 한 줄의 TAC 표 여러 개는 선언 폭만큼 전진한다 (#8647)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인

- [tests/cases/issue_8647_inline_tac_table_declared_advance.rs](../../tests/cases/issue_8647_inline_tac_table_declared_advance.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8650_review_impl.md) · [전체 순서·게이트](assets/planet6897_20261106/integration-plan.md).

## 실제 네 개 TAC 표의 같은 줄 전진

R11 source `048f44da99d83e186f3a9e139563d20545c8bdb4`의 같은 과오납금 반환통지서와
독립 한컴 Print를 전체 1쪽 비교했다. Native/fresh WASM은 각각 같은 1쪽이며
전쪽 raw **100.00%**다(96 DPI, 보정 전 2px 내용 실루엣).

주장 영역인 가로 네 표의 왼쪽·오른쪽 테두리와 표 사이 간격, 각 표 안의 `입금` 상자,
날짜·하단 결재 칸을 Native/fresh WASM에서 직접 대조했다. 표를 내부 그리드 폭만큼
전진시켜 오른쪽 표들이 연쇄 이동하는 현상이 없고 원부·통지서·반환명령·보고서 순서와
각 표의 내용 소속이 보존된다. 표 아래 빈 칸이나 다른 페이지로 증거를 대체하지 않았다.

[Native 전쪽 TSV](assets/planet6897_20261106/pr8650/r11/refund-notice/native.tsv) ·
[fresh WASM 전쪽 TSV](assets/planet6897_20261106/pr8650/r11/refund-notice/wasm.tsv) ·
[입력·PDF·producer SHA](assets/planet6897_20261106/pr8650/r11/refund-notice/provenance.json).
Native [review](assets/planet6897_20261106/pr8650/r11/refund-notice/native-review-001.png)·
[overlay](assets/planet6897_20261106/pr8650/r11/refund-notice/native-overlay-001.png),
fresh WASM [review](assets/planet6897_20261106/pr8650/r11/refund-notice/wasm-review-001.png)·
[overlay](assets/planet6897_20261106/pr8650/r11/refund-notice/wasm-overlay-001.png).

후속 제품 보정의 최종 head·필수 회귀·gate는 별도로 완료해야 한다.
