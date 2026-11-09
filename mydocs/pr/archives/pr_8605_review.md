# PR #8605 리뷰 — fix(render): WASM Canvas 합성 굵기를 실제 face 윤곽의 fill+stroke로 그린다 (#8430)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8605](https://github.com/edwardkim/rhwp/pull/8605), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `5d37286267136c9978d7ac2f804913730ab6becf`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(render): WASM Canvas 합성 굵기를 실제 face 윤곽의 fill+stroke로 그린다 (#8430)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인


통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8605_review_impl.md) · [전체 순서·게이트](../assets/planet6897_20261106/integration-plan.md).

## 실제 첫 페이지와 그림 페이지의 주장 검증

R11 source `048f44da99d83e186f3a9e139563d20545c8bdb4`에서 같은 childcare 원본과
독립 Print의 전체 25쪽을 96 DPI·보정 전2px 내용 실루엣으로 비교했다.
Native 전쪽 최저는 98.66%, fresh WASM 전쪽 최저는 98.68%이며
양쪽 쪽수가 PDF와 일치한다.

1쪽 review/overlay에서 `국민권익위원회 의결`, 의안번호2015-458호, 제목·대상 기관·
의결일·주문·이유를 직접 대조했다. 내용 순서와 줄바꿈, 첫 제목부터 주문까지의
간격을 확인했다. 18쪽은 `일자리 정보 연계를 통한 구직자 접근성 제고` 제목과
워크넷 본문, `일자리 정보 연계 개선` 그림을 같은 영역에서 판독했다. 제목이 그림
아래로 내려오거나 그림과 중복되지 않고 알리오·개별기관·워크넷·지방공기업클린아이
노드와 화살표가 원본의 그림 영역 안에 유지된다.

[Native 전쪽 TSV](../assets/planet6897_20261106/pr8605/r11/childcare/native.tsv) ·
[fresh WASM 전쪽 TSV](../assets/planet6897_20261106/pr8605/r11/childcare/wasm.tsv) ·
[원본·Print·producer SHA](../assets/planet6897_20261106/pr8605/r11/childcare/provenance.json).

1쪽 native: [review](../assets/planet6897_20261106/pr8605/r11/childcare/native-review-001.png) · [overlay](../assets/planet6897_20261106/pr8605/r11/childcare/native-overlay-001.png).
1쪽 wasm: [review](../assets/planet6897_20261106/pr8605/r11/childcare/wasm-review-001.png) · [overlay](../assets/planet6897_20261106/pr8605/r11/childcare/wasm-overlay-001.png).
18쪽 native: [review](../assets/planet6897_20261106/pr8605/r11/childcare/native-review-018.png) · [overlay](../assets/planet6897_20261106/pr8605/r11/childcare/native-overlay-018.png).
18쪽 wasm: [review](../assets/planet6897_20261106/pr8605/r11/childcare/wasm-review-018.png) · [overlay](../assets/planet6897_20261106/pr8605/r11/childcare/wasm-overlay-018.png).

이는 R11 완료 측정의 증거이며 후속 보정의 최종 head·회귀·lint·gate 판정은 별도로 완료한다.
