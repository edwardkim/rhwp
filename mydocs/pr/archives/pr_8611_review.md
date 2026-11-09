# PR #8611 리뷰 — fix(renderer): Native 경로 글꼴을 매핑으로 공유해 64MiB 넘는 파일도 싣기 (#8597)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8611](https://github.com/edwardkim/rhwp/pull/8611), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `0cbe6a9ae5ee4ef8d994d21c170682f95f4bf236`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(renderer): Native 경로 글꼴을 매핑으로 공유해 64MiB 넘는 파일도 싣기 (#8597)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인

- [tests/cases/issue_8597_invalid_font_name_host_fallback.rs](../../../tests/cases/issue_8597_invalid_font_name_host_fallback.rs)
- [tests/cases/issue_8597_native_font_path_budget.rs](../../../tests/cases/issue_8597_native_font_path_budget.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8611_review_impl.md) · [전체 순서·게이트](../assets/planet6897_20261106/integration-plan.md).

## 실물 글꼴·완결 문서의 주장 대조

R11 source `048f44da99d83e186f3a9e139563d20545c8bdb4`의 같은 원본·독립 Print를
96 DPI·보정 전2px 내용 실루엣으로 전쪽 비교했다. Native/fresh WASM 쪽수가
일치하며, 아래 실제 페이지의 review와 overlay를 두 경로 모두 직접 판독했다.

| 입력 | 쪽수 | Native 전쪽 최저 | fresh WASM 전쪽 최저 | 판독 쪽 |
| --- | ---: | ---: | ---: | --- |
| shortcut | 7 | 97.19% | 97.26% | 7 |
| sungeo | 94 | 99.24% | 99.32% | 1 |
| bookreview | 1 | 100.00% | 100.00% | 1 |

shortcut7쪽은 상단 표 기능/단축키 두 열과 `기타` 밴드, 그림 그리기 기능과
오른쪽 아래 쪽 장식을 확인했다. sungeo1쪽은 漢字成語 제목과 상단 장식, 본문
두 문단 및 차례의 가~하 그림 12개를 대조했다. bookreview1쪽은 제목·지은이·
국적·분류·출간일·책 정보·평점의 일곱 머리항목과 큰 빈 작성 영역이 같은 쪽에
보존된다. 글꼴·자원 준비 실패로 텍스트나 완결 문서 전체가 비는 현상이 없다.
같은 내용의 글꼴 획 두께·이미지 필터 차이를 픽셀 동일이라고 표현하지 않는다.

interview 동일 원본2쪽의 전쪽 측정과 실제1쪽 판독은
[PR #8617의 공유 증거](pr_8617_review.md#실제-interview-nativefresh-wasm-검증)를
참조한다. 해당 문서의 96.01% 최저값과 WordArt/그림 경계의 차이를 누락하거나
다른 원본의 100% 결과로 대체하지 않는다.

shortcut: [Native TSV](../assets/planet6897_20261106/pr8611/r11/shortcut/native.tsv) · [fresh WASM TSV](../assets/planet6897_20261106/pr8611/r11/shortcut/wasm.tsv) · [원본/Print/producer 해시](../assets/planet6897_20261106/pr8611/r11/shortcut/provenance.json).
7쪽 native: [review](../assets/planet6897_20261106/pr8611/r11/shortcut/native-review-007.png) · [overlay](../assets/planet6897_20261106/pr8611/r11/shortcut/native-overlay-007.png).
7쪽 wasm: [review](../assets/planet6897_20261106/pr8611/r11/shortcut/wasm-review-007.png) · [overlay](../assets/planet6897_20261106/pr8611/r11/shortcut/wasm-overlay-007.png).

sungeo: [Native TSV](../assets/planet6897_20261106/pr8611/r11/sungeo/native.tsv) · [fresh WASM TSV](../assets/planet6897_20261106/pr8611/r11/sungeo/wasm.tsv) · [원본/Print/producer 해시](../assets/planet6897_20261106/pr8611/r11/sungeo/provenance.json).
1쪽 native: [review](../assets/planet6897_20261106/pr8611/r11/sungeo/native-review-001.png) · [overlay](../assets/planet6897_20261106/pr8611/r11/sungeo/native-overlay-001.png).
1쪽 wasm: [review](../assets/planet6897_20261106/pr8611/r11/sungeo/wasm-review-001.png) · [overlay](../assets/planet6897_20261106/pr8611/r11/sungeo/wasm-overlay-001.png).

bookreview: [Native TSV](../assets/planet6897_20261106/pr8611/r11/bookreview/native.tsv) · [fresh WASM TSV](../assets/planet6897_20261106/pr8611/r11/bookreview/wasm.tsv) · [원본/Print/producer 해시](../assets/planet6897_20261106/pr8611/r11/bookreview/provenance.json).
1쪽 native: [review](../assets/planet6897_20261106/pr8611/r11/bookreview/native-review-001.png) · [overlay](../assets/planet6897_20261106/pr8611/r11/bookreview/native-overlay-001.png).
1쪽 wasm: [review](../assets/planet6897_20261106/pr8611/r11/bookreview/wasm-review-001.png) · [overlay](../assets/planet6897_20261106/pr8611/r11/bookreview/wasm-overlay-001.png).

후속 제품 보정의 최종 head·필수 회귀·lint·브라우저 검증 완료 전이며 최종 승인으로 판정하지 않는다.
