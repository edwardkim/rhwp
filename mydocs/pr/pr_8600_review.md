# PR #8600 리뷰 — fix(renderer): 도형 그림 채우기 크기와 한컴 윤고딕 250 연결 (#8575)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8600](https://github.com/edwardkim/rhwp/pull/8600), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `983933bc448e5ed4f7c8d15be97d894e129a5b9b`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(renderer): 도형 그림 채우기 크기와 한컴 윤고딕 250 연결 (#8575)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인

- [tests/cases/issue_4966_font_rule_projection.rs](../../tests/cases/issue_4966_font_rule_projection.rs)
- [tests/cases/issue_8575_shape_image_fill_size.rs](../../tests/cases/issue_8575_shape_image_fill_size.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8600_review_impl.md) · [전체 순서·게이트](assets/planet6897_20261106/integration-plan.md).

## 실제 주장 영역의 Native/fresh WASM 대조

R11 source `048f44da99d83e186f3a9e139563d20545c8bdb4`의 완료 산출물을 재사용했다.
같은 그림 채우기 입력 HWP와 HWPX는 각각 독립 Print와 쪽수가 같은 2쪽이며,
전쪽 Native/fresh WASM raw는 모두 **100.00%**다. 측정은 96 DPI, 보정 전 2px
내용 실루엣이며 실제 입력·PDF SHA, renderer 및 WASM producer SHA는 아래 provenance에 있다.

1쪽은 크기와 종횡비가 다른 여섯 도형의 네 색 영역·채움 경계·바깥선이 Print와 같은지
직접 판독했다. 전체 원본을 확대해 놓는 크기와 가로·세로를 다르게 바꾼 크기를 구분했고,
여섯 번째 도형의 오른쪽·아래쪽 채움도 빠지지 않는다. 2쪽의 `2021. 9. . 윤고딕 250 한글`은
숫자·한글 모양과 사이 간격을 양쪽 backend에서 같은 Print 영역으로 대조했다.
실루엣 100%를 색상 픽셀의 완전 동일성으로 해석하지 않는다.

| 같은 입력·기준 | 전쪽 Native TSV | 전쪽 fresh WASM TSV | 판독 페이지 |
| --- | --- | --- | --- |
| HWP | [2쪽](assets/planet6897_20261106/pr8600/r11/image-fill-hwp/native.tsv) | [2쪽](assets/planet6897_20261106/pr8600/r11/image-fill-hwp/wasm.tsv) | 1·2쪽 |
| HWPX | [2쪽](assets/planet6897_20261106/pr8600/r11/image-fill-hwpx/native.tsv) | [2쪽](assets/planet6897_20261106/pr8600/r11/image-fill-hwpx/wasm.tsv) | 1·2쪽 |

대표 HWPX 1쪽 Native [review](assets/planet6897_20261106/pr8600/r11/image-fill-hwpx/native-review-001.png)·
[overlay](assets/planet6897_20261106/pr8600/r11/image-fill-hwpx/native-overlay-001.png),
fresh WASM [review](assets/planet6897_20261106/pr8600/r11/image-fill-hwpx/wasm-review-001.png)·
[overlay](assets/planet6897_20261106/pr8600/r11/image-fill-hwpx/wasm-overlay-001.png).
HWP·HWPX 각각의 같은 위치 증거와 2쪽도 같은 디렉터리에 보존했다.
[HWP provenance](assets/planet6897_20261106/pr8600/r11/image-fill-hwp/provenance.json)·
[HWPX provenance](assets/planet6897_20261106/pr8600/r11/image-fill-hwpx/provenance.json).

이 절은 R11의 실제 주장 검증 기록이다. 이후 통합 보정의 최종 head·회귀·필수 gate가
아직 완료되지 않았으므로 통합 PR 전체의 최종 머지 승인으로 대신하지 않는다.
