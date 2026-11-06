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

실행 전이므로 PASS로 기록하지 않는다. HWP/HWPX/PDF 커밋 포함·실행 파일 SHA, 전쪽 TSV 및 확대 판독은 아직 완료하지 않았다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8650_review_impl.md) · [전체 순서·게이트](assets/planet6897_20261106/integration-plan.md).
