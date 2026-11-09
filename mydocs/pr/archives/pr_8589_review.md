# PR #8589 리뷰 — fix(layout): 어울림 표가 뒤 문단을 띠로 흡수하면 흐름상 다음 문단으로 host 뒤 사다리 일치를 판정 (#8567)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8589](https://github.com/edwardkim/rhwp/pull/8589), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `a68704a019cf56f4dca8db0ee5ff8ae5103febb8`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(layout): 어울림 표가 뒤 문단을 띠로 흡수하면 흐름상 다음 문단으로 host 뒤 사다리 일치를 판정 (#8567)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인

- [tests/cases/issue_8567_square_wrap_successor_ladder.rs](../../../tests/cases/issue_8567_square_wrap_successor_ladder.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8589_review_impl.md) · [전체 순서·게이트](../assets/planet6897_20261106/integration-plan.md).

## 부산 재의요구안의 실제 주장 페이지

R11 제품 `048f44da99`에서 원 주장의 부산 재의요구안 원본과 기존 독립 Print 전체 1쪽을
대조했다. Native raw **99.97%**, fresh WASM **99.99%**, 쪽수는 각각 PDF와 1=1이다.
96dpi·compat2022·색상 경계 보정 전 2px 내용 실루엣이며 다른 문서의 수치를 인용하지 않았다.

- Native [전쪽 TSV](../assets/planet6897_20261106/pr8589/r11/busan-form/native.tsv), 주장 1쪽 [review](../assets/planet6897_20261106/pr8589/r11/busan-form/native-review-001.png) · [overlay](../assets/planet6897_20261106/pr8589/r11/busan-form/native-overlay-001.png).
- fresh WASM [전쪽 TSV](../assets/planet6897_20261106/pr8589/r11/busan-form/wasm.tsv), 같은 1쪽 [review](../assets/planet6897_20261106/pr8589/r11/busan-form/wasm-review-001.png) · [overlay](../assets/planet6897_20261106/pr8589/r11/busan-form/wasm-overlay-001.png).
- [입력·PDF·실제 producer·증거 해시](../assets/planet6897_20261106/pr8589/r11/busan-form/provenance.json).

두 경로의 같은 페이지를 직접 판독했다. 의안번호 표 옆 제출일·제출자와 표 아래의
재의요구 본문이 Print와 같은 순서·간격으로 배치된다. 「이유」와 두 번호의 점선,
마지막 「조례안 첨부」까지 보존되어 host 뒤 기준을 소스 배열의 다음 문단으로 잘못
잡아 뒤 내용을 이동시키는 증상은 없다. WASM의 점선 획은 안티앨리어싱 차이가 있어
픽셀 바이트 동일성을 주장하지 않는다. 메인터너 보정 이후 영향 확인과 통합 최종
필수 검증은 별도 완료 조건으로 유지한다.
