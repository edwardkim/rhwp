# PR #8604 리뷰 — fix(typeset): #8596 앞 TAC 표 host 줄 안에 저장된 자리차지 표 프레임은 다음 쪽에서 시작한다

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8604](https://github.com/edwardkim/rhwp/pull/8604), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `013459bff4ff5313af53ece5646e79c48647a3aa`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(typeset): #8596 앞 TAC 표 host 줄 안에 저장된 자리차지 표 프레임은 다음 쪽에서 시작한다

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인

- [tests/cases/issue_8596_tac_host_stored_frame_rewind.rs](../../../tests/cases/issue_8596_tac_host_stored_frame_rewind.rs)

통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8604_review_impl.md) · [전체 순서·게이트](../assets/planet6897_20261106/integration-plan.md).

## 실제 주장 페이지와 대조군의 판독

R11 source `048f44da99d83e186f3a9e139563d20545c8bdb4`의 전쪽 측정과 같은
실제 원본의 주장 페이지 review/overlay를 Native/fresh WASM 양쪽에서 직접 대조했다.
96 DPI·색상 경계 보정 전 2px 내용 실루엣이며, 아래 원본별 전쪽 TSV를 보존한다.

| 입력 | 전체 쪽수 | Native 최저 | fresh WASM 최저 | 판독한 쪽 |
| --- | ---: | ---: | ---: | --- |
| certificate | 9 | 99.93% | 99.99% | 2, 3 |
| policy-agenda | 14 | 96.52% | 98.44% | 11 |
| idol-counterexample | 2 | 99.70% | 99.77% | 1 |

자격증 표 2·3쪽의 첫 전자/녹지조경 행과 마지막 생명유전/해양교통시설 행을
각각 확인했다. 저장된 다음 프레임이 앞 TAC host 안으로 끌려오지 않고 각 쪽의
머리행·열 너비·내용 순서·끝 행이 Print와 일치한다. 정책과제 11쪽은 연구자 선정
심의결과서의 제목과 두 표, 비고 및 하단 유의사항의 순서를 대조했다. Native의
연구 방식 체크 기호 주변 간격은 WASM/Print와 잉크 차이가 있어 픽셀 동일로
표현하지 않는다. 대조군의 YG SELECT 안내는 그림 두 개와 관련 법리 상자가
같은 쪽 순서에 남으며 임의의 다음 쪽 이월이 없다. 그림 필터와 외곽선 두께의
차이를 확인했고, 다른 문서의 캡처를 해당 원본 증거로 대체하지 않았다.

certificate: [Native TSV](../assets/planet6897_20261106/pr8604/r11/certificate/native.tsv) · [fresh WASM TSV](../assets/planet6897_20261106/pr8604/r11/certificate/wasm.tsv) · [입력/PDF/producer 해시](../assets/planet6897_20261106/pr8604/r11/certificate/provenance.json).
2쪽 native: [review](../assets/planet6897_20261106/pr8604/r11/certificate/native-review-002.png) · [overlay](../assets/planet6897_20261106/pr8604/r11/certificate/native-overlay-002.png).
2쪽 wasm: [review](../assets/planet6897_20261106/pr8604/r11/certificate/wasm-review-002.png) · [overlay](../assets/planet6897_20261106/pr8604/r11/certificate/wasm-overlay-002.png).
3쪽 native: [review](../assets/planet6897_20261106/pr8604/r11/certificate/native-review-003.png) · [overlay](../assets/planet6897_20261106/pr8604/r11/certificate/native-overlay-003.png).
3쪽 wasm: [review](../assets/planet6897_20261106/pr8604/r11/certificate/wasm-review-003.png) · [overlay](../assets/planet6897_20261106/pr8604/r11/certificate/wasm-overlay-003.png).

policy-agenda: [Native TSV](../assets/planet6897_20261106/pr8604/r11/policy-agenda/native.tsv) · [fresh WASM TSV](../assets/planet6897_20261106/pr8604/r11/policy-agenda/wasm.tsv) · [입력/PDF/producer 해시](../assets/planet6897_20261106/pr8604/r11/policy-agenda/provenance.json).
11쪽 native: [review](../assets/planet6897_20261106/pr8604/r11/policy-agenda/native-review-011.png) · [overlay](../assets/planet6897_20261106/pr8604/r11/policy-agenda/native-overlay-011.png).
11쪽 wasm: [review](../assets/planet6897_20261106/pr8604/r11/policy-agenda/wasm-review-011.png) · [overlay](../assets/planet6897_20261106/pr8604/r11/policy-agenda/wasm-overlay-011.png).

idol-counterexample: [Native TSV](../assets/planet6897_20261106/pr8604/r11/idol-counterexample/native.tsv) · [fresh WASM TSV](../assets/planet6897_20261106/pr8604/r11/idol-counterexample/wasm.tsv) · [입력/PDF/producer 해시](../assets/planet6897_20261106/pr8604/r11/idol-counterexample/provenance.json).
1쪽 native: [review](../assets/planet6897_20261106/pr8604/r11/idol-counterexample/native-review-001.png) · [overlay](../assets/planet6897_20261106/pr8604/r11/idol-counterexample/native-overlay-001.png).
1쪽 wasm: [review](../assets/planet6897_20261106/pr8604/r11/idol-counterexample/wasm-review-001.png) · [overlay](../assets/planet6897_20261106/pr8604/r11/idol-counterexample/wasm-overlay-001.png).

이는 R11 완료 증거이며 후속 메인터너 보정의 최종 후보 검증을 대신하지 않는다.
최종 필수 회귀·lint·fresh WASM·기능 보존 대조를 완료한 뒤 판정을 갱신한다.
