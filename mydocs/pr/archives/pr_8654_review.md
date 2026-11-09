# PR #8654 리뷰 — fix(layout): 같은 줄 TAC 표 공유 기준선과 rowspan 초과분 — #8645·#8649 위에 쌓음 (#8641)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8654](https://github.com/edwardkim/rhwp/pull/8654), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `d971025c2bb85ebcd2453578800e6130445b81b8`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(layout): 같은 줄 TAC 표 공유 기준선과 rowspan 초과분 — #8645·#8649 위에 쌓음 (#8641)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인

- [tests/cases/issue_8641_shared_baseline_rowspan.rs](../../../tests/cases/issue_8641_shared_baseline_rowspan.rs)
- [tests/cases/issue_8641_tac_prefix_line_owner.rs](../../../tests/cases/issue_8641_tac_prefix_line_owner.rs)
- [tests/cases/issue_8648_cell_page_overlay_no_flow.rs](../../../tests/cases/issue_8648_cell_page_overlay_no_flow.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8654_review_impl.md) · [전체 순서·게이트](../assets/planet6897_20261106/integration-plan.md).

## 같은 발신명 대조군의 기존 증거 재사용

PR #8649와 겹치는 sender-overlay 원본·독립 Print는 같은 입력 SHA의 R11 완료 측정을
재사용한다. 전1쪽 Native/fresh WASM raw는 각각 **100.00%**이고, 실제 발신명·본문·
날짜를 대응 페이지에서 직접 판독했다.
[같은 입력의 TSV·review·overlay 및 판독 기록](pr_8649_review.md#실제-발신명-overlay와-셀-흐름-대조)을
연결한다. 이 한 대조군으로 다른 stamp·노동 보고서·필드 상자 검증을 대체하지 않는다.

PR #8645와 공유하는 field-page-box·tall-prefix 입력도 같은 R11 전쪽 측정과 직접 판독을
재사용한다. 각각 Native/WASM 최저는 99.98/100.00%, 100.00/100.00%이며 1쪽씩이다.
[두 입력의 대응 페이지 PNG·TSV](pr_8645_review.md#실제-선행-줄쪽-기준-상자의-주장-검증)를
연결한다. 이 결과만으로 별도 stamp·노동 보고서 경계의 완료를 선언하지 않는다.

## 결재 표와 rowspan 문서의 개별 주장 페이지

R11 source `048f44da99d83e186f3a9e139563d20545c8bdb4`에서 같은 원본과 독립 Print를
96 DPI·보정 전2px 내용 실루엣으로 전쪽 대조했다. 아래 각 원본은 두 경로 쪽수가
일치하며 모든 실제 주장 페이지의 review와 overlay를 직접 판독했다.

| 입력 | 전체 쪽수 | Native 전쪽 최저 | fresh WASM 전쪽 최저 | 판독 쪽 |
| --- | ---: | ---: | ---: | --- |
| stamp-a | 1 | 100.00% | 100.00% | 1 |
| stamp-b | 1 | 100.00% | 100.00% | 1 |
| labor-report | 53 | 99.95% | 99.98% | 12, 13 |

stamp-a·b1쪽은 왼쪽 시행/수신 표와 오른쪽 결재 표가 나란히 다른 원점에 남고,
결재의 두 줄 이름·날짜가 같은 셀 프레임 안에 배치된다. 문서 제목과 긴 사업명,
본문 항목·날짜·총무부장 발신명이 그 뒤 같은 쪽에 보존된다. labor-report12쪽은
사업장 규모별 적용·배제 표와 각주10·11, 13쪽은 4대 사회보험 표의 병합된 제도 칸,
비고의 화살표·적용확대 표기와 뒤 본문을 확인했다. rowspan의 내용이 앞 행과 겹치거나
뒤 문단을 밀지 않으며 표의 시작·종료 괘선과 후속 내용 소속을 직접 대조했다.

stamp-a: [Native TSV](../assets/planet6897_20261106/pr8654/r11/stamp-a/native.tsv) · [fresh WASM TSV](../assets/planet6897_20261106/pr8654/r11/stamp-a/wasm.tsv) · [입력·Print·producer 해시](../assets/planet6897_20261106/pr8654/r11/stamp-a/provenance.json).
1쪽 Native [review](../assets/planet6897_20261106/pr8654/r11/stamp-a/native-review-001.png) · [overlay](../assets/planet6897_20261106/pr8654/r11/stamp-a/native-overlay-001.png); fresh WASM [review](../assets/planet6897_20261106/pr8654/r11/stamp-a/wasm-review-001.png) · [overlay](../assets/planet6897_20261106/pr8654/r11/stamp-a/wasm-overlay-001.png).

stamp-b: [Native TSV](../assets/planet6897_20261106/pr8654/r11/stamp-b/native.tsv) · [fresh WASM TSV](../assets/planet6897_20261106/pr8654/r11/stamp-b/wasm.tsv) · [입력·Print·producer 해시](../assets/planet6897_20261106/pr8654/r11/stamp-b/provenance.json).
1쪽 Native [review](../assets/planet6897_20261106/pr8654/r11/stamp-b/native-review-001.png) · [overlay](../assets/planet6897_20261106/pr8654/r11/stamp-b/native-overlay-001.png); fresh WASM [review](../assets/planet6897_20261106/pr8654/r11/stamp-b/wasm-review-001.png) · [overlay](../assets/planet6897_20261106/pr8654/r11/stamp-b/wasm-overlay-001.png).

labor-report: [Native TSV](../assets/planet6897_20261106/pr8654/r11/labor-report/native.tsv) · [fresh WASM TSV](../assets/planet6897_20261106/pr8654/r11/labor-report/wasm.tsv) · [입력·Print·producer 해시](../assets/planet6897_20261106/pr8654/r11/labor-report/provenance.json).
12쪽 Native [review](../assets/planet6897_20261106/pr8654/r11/labor-report/native-review-012.png) · [overlay](../assets/planet6897_20261106/pr8654/r11/labor-report/native-overlay-012.png); fresh WASM [review](../assets/planet6897_20261106/pr8654/r11/labor-report/wasm-review-012.png) · [overlay](../assets/planet6897_20261106/pr8654/r11/labor-report/wasm-overlay-012.png).
13쪽 Native [review](../assets/planet6897_20261106/pr8654/r11/labor-report/native-review-013.png) · [overlay](../assets/planet6897_20261106/pr8654/r11/labor-report/native-overlay-013.png); fresh WASM [review](../assets/planet6897_20261106/pr8654/r11/labor-report/wasm-review-013.png) · [overlay](../assets/planet6897_20261106/pr8654/r11/labor-report/wasm-overlay-013.png).

이는 R11 완료 증거다. 후속 제품 보정의 최종 head·필수 회귀·lint·브라우저 검증 전이며 최종 승인으로 판정하지 않는다.
