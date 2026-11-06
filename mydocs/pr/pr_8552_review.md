# PR #8552 리뷰 — feat(edit): 글자 서식 props 에 글꼴 이름 fontName 을 받는다

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8552](https://github.com/edwardkim/rhwp/pull/8552), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `423b369e9d81cbfd6f31ec527d53cbea5d43d5e1`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

feat(edit): 글자 서식 props 에 글꼴 이름 fontName 을 받는다

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 실제 공개 API 대조와 메인터너 보정

제품 `048f44da99`의 최적화 WASM에서 본문·평면 셀·중첩 셀·머리말·꼬리말의
이름 적용과 HWP/HWPX 저장·재열기는 성공했다. 대상 run이 7개 언어 슬롯에서 요청한
이름을 참조함을 저장 XML로 확인했다.

반면 평면 셀 주소 오류·잘못된 중첩 경로·빈 경로는 오류를 반환한 뒤에도 7개 슬롯에
새 글꼴을 남겼다. 본문·없는 머리말·fontIds 충돌은 문서를 보존했다. 셀 주소 검증이
등록 helper 뒤에 있는 공통 원인이므로 주소 검증과 원본 모양 ID 수집을 등록 앞으로
옮기는 메인터너 보정을 진행한다. 이 결함이 남아 있는 제품은 수용하지 않는다.

[원인·보정·검증 기록](assets/planet6897_20261106/pr8552/fontname-error-atomicity.md).
이는 합성 상태 API 계약의 실제 실행이며, 설치 글꼴의 시각 일치나 통합 최종 검증 완료를
의미하지 않는다.

## 회귀·입력 확인

- [tests/cases/apply_char_format_contract.rs](../../tests/cases/apply_char_format_contract.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8552_review_impl.md) · [전체 순서·게이트](assets/planet6897_20261106/integration-plan.md).

## 메인터너 보정 후 실제 API 확인

앞 절의 오류 뒤 이름 등록은 R11의 결함 재현이다. root `47679ca172`에서 셀 주소·
원본 글자 모양 확인을 등록 앞으로 옮기고, 등록 helper는 7개 언어 슬롯을 모두 사전 검사한다.
공개/OFL 글꼴 5개만 노출한 최소 환경의 집중 Rust는 **36/36 통과**, 그중 새 문서 불변
상태 관계 검사는 5건이다. 제외된 다른 테스트를 이 결과의 통과로 합산하지 않았다.

R15 최적화 WASM의 실제 공개 API 11종과 영문 슬롯 등록 상한 두 경계를 확인했다.
오류 주소·없는 머리말·슬롯 충돌·중첩 경로 오류에서 저장 HWPX 바이트가 호출 전후 같고,
상한 슬롯에 이름이 이미 있으면 정상 재사용된다. 본문·평면 셀·중첩 셀·머리말·꼬리말은
HWPX 저장과 HWP 저장 후 재열기 양쪽에서 실제 대상 run의 글자 모양→7개 언어별
fontRef가 요청한 이름을 참조한다. 목록에 이름이 추가된 사실만으로 적용을 인정하지 않았다.

[실제 API·저장 참조·package 해시](assets/planet6897_20261106/pr8552/api-verification-r15.json),
[보정 전후 원인과 검사 범위](assets/planet6897_20261106/pr8552/fontname-error-atomicity.md).
이 증거는 API 상태와 저장 참조 계약이며 설치 글꼴의 모양을 입증하는 PNG로 대신하지 않는다.
통합의 최종 필수 검증은 계속 진행 중이다.
