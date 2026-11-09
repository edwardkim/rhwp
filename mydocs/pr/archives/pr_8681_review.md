# PR #8681 리뷰 — fix(layout): 자기 자리차지 표 띠 위끝에 걸친 저장 줄을 띠 끝으로 내린다 (#8679)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8681](https://github.com/edwardkim/rhwp/pull/8681), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `3cc76d84d41b40f47304b62ff2b8b5010828b86b`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(layout): 자기 자리차지 표 띠 위끝에 걸친 저장 줄을 띠 끝으로 내린다 (#8679)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인

- [tests/cases/issue_8679_stored_band_crossing_line.rs](../../../tests/cases/issue_8679_stored_band_crossing_line.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8681_review_impl.md) · [전체 순서·게이트](../assets/planet6897_20261106/integration-plan.md).

## 추가 실문서 대조 기준

[결재문서 36478532 원본과 Print 출처](../assets/planet6897_20261106/pr8681/print-reference.md)를 확보했다. [이슈 #8686](https://github.com/edwardkim/rhwp/issues/8686)이 보고한 칸 안 자리차지 표와 후속 본문의 소속·포함 관계를 검증할 입력이다. 원 PR의 주 입력 36385506과 같은 문서로 취급하지 않는다. MCP 2020의 실제 한컴 11.0.0.9136·1-up 인쇄 실행 및 2쪽 PDF를 확인했다. Native/fresh WASM과의 출력 대조는 진행 전이며 보정 완료를 주장하지 않는다.

[최초 Native 측정과 소비 경로 분석](../assets/planet6897_20261106/pr8681/36478532-before-analysis.md): 1쪽 62.43%, 2쪽 100.00%, 쪽수 일치. 같은 쪽 review·overlay를 직접 판독해 본문과 점검사항 잘림·표 경계 차이를 확인했다. 원본·PDF와 함께 변경 전 증거로 보존하며 보정 완료 판정은 하지 않는다.

## 메인터너 보정 중간 검증

중첩 배제 표의 프레임과 빈 저장 줄의 물리 점유를 측정·배치에서 공유했다.
또한 저장 줄이 없는 문단의 글앞·글뒤 개체 앵커도 폭 없는 줄 내용으로 보존해,
가용 폭을 넘는 뒤 TAC 표가 자기 줄로 넘어가도록 공통 재조판을 보정했다.

실제 producer SHA-256 `901455bd1f38098eb574d2fdefe5830f21f06a673b95197ac7e408a5be80d369`,
Print profile·compat 2022·96 DPI, 원본과 기준 Print PDF를 그대로 사용했다.
[전쪽 Native TSV](../assets/planet6897_20261106/pr8681/36478532-maintainer-native.tsv)는
**1쪽 99.90%, 2쪽 100.00%, 쪽수 2=2**다.
[1쪽 review](../assets/planet6897_20261106/pr8681/36478532-maintainer-native-review.png)와
[overlay](../assets/planet6897_20261106/pr8681/36478532-maintainer-native-overlay.png)를 직접 열어
본문·점검사항·표 하단·후속 지출결의서의 소속과 원점이 복구됐음을 확인했다.
임의 좌표 이동이나 원본 수정으로 맞추지 않았다.

주 입력 stored-band 2쪽은 같은 보정 엔진에서 Native 최저 100.00%다.
나머지 대조군·fresh WASM·전체 회귀·lint가 완료되기 전에는 최종 승인을 선언하지 않는다.
