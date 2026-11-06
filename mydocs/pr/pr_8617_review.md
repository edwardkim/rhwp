# PR #8617 리뷰 — fix(visual-sweep): 글꼴 catalog family 이름의 앞뒤 공백을 Native 와 같이 정규화 (#8612)

## 최종 판정

**머지 보류 — 누적 적용·메인터너 보정 및 필수 실제 검증 진행 중.**

원 PR의 보고 수치를 현재 통합 head의 측정으로 인정하지 않는다. 전쪽 Native/fresh WASM·확대 판독·필수 회귀와 lint 완료 후 최종 판정을 갱신한다.

## 접수 정보

- [원 PR #8617](https://github.com/edwardkim/rhwp/pull/8617), 작성자 `planet6897`, reviewer `jangster77` API 배정 확인.
- 원 head `0fd746a87d28cd70fe7297149882cacd84074475`, base `devel`, 통합 base `08963f6825956984d38b0d9392f44777db7107ab`.
- 통합 브랜치 `review/integration-20261106-planet6897-open`.
- Open·Non-Draft 및 head는 접수 시점 참고값이며 최종 원격 조치 전 재조회한다.

## 원 주장과 현재 검증 경계

fix(visual-sweep): 글꼴 catalog family 이름의 앞뒤 공백을 Native 와 같이 정규화 (#8612)

원본 저장/편집 계약·측정과 배치의 공통 결과·backend 실제 소비를 소스와 독립 한컴 Print로 대조할 예정이다.
개별 주장 페이지에 대응하는 입력·PDF를 output의 PR별 폴더에 준비했다. 약칭과 코드 예시는 실제 입력으로 세지 않는다.

## 회귀·입력 확인


통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 기본 feature 전체 Rust 회귀는 **12,796건 통과, 실패 0건, 기존 ignored 45건**이다. 공개 글꼴 5개만 노출한 최소 글꼴 환경에서 수행했다. 이는 최초 통합 source의 결과다. 이후 메인터너 보정으로 제품·테스트 코드가 바뀌었으므로 보정 후 head의 회귀 통과로 재사용하지 않는다. 새 보정본의 검증 결과와 원본·Print 추가를 구분한다.

Native Skia 공식 3종(라이브러리 4,040건 및 통합 24건)과 이번 PR의 추가 5개 모듈 14건이 모두 통과했다. 라이브러리의 기존 ignored는 12건이며, focused 실행에서 제외된 다른 테스트를 실패나 ignored로 합산하지 않는다. 전쪽 Native/fresh WASM·쪽수·확대 판독·lint는 별도 완료 조건이며, 회귀 통과로 대신하지 않는다.

## 적용과 보정 계획

[이 PR 구현 계획](pr_8617_review_impl.md) · [전체 순서·게이트](assets/planet6897_20261106/integration-plan.md).

## 실제 집중 검증 — 글꼴 catalog 공백

- 고정 base `08963f6825956984d38b0d9392f44777db7107ab`의 `visual_sweep_png.py`와 후보 source `46aca162bcf7fbcbc114cc42a251e6047598b371`을 같은 새 테스트 클래스에 공급했다.
- `FontFamilyTrailingSpaceTests` 2개는 base에서 각각 assertion FAIL이고 오류(errors)는 0이었다. 후보에서는 2개 모두 PASS했다.
- 실제 검사: 처음/캐시 공급의 primaryFamily·별칭 앞뒤 공백 제거, 가운데 공백 보존, PostScript 이름 무변경, 예전 공백 포함 cache의 무효화·재생성.
- 테스트 글꼴은 저장소의 `tests/fixtures/native_inline_placeholder/placeholder-contract.ttf`를 사용했고 name table family만 앞뒤 공백을 가진 값으로 바꿨다. 설치 글꼴이 없어도 실행되는 관계 검사다.
- 실행 로그·중간 JSON은 ignored output에만 보존했다. 이 결과를 `interview.hwp`의 실제 Native/fresh WASM 2쪽 시각 검증이나 최종 승인으로 확대하지 않는다.

PNG 공급·catalogue 검사 `scripts.tests.test_visual_sweep_png`는 40건 통과했다. 앞서 기록한 base 2건 assertion 실패 → 후보 2건 통과의 음성 대조와 별개로 전체 모듈을 실행했다. 실제 interview fresh WASM 검증은 남아 있다.

## 실제 interview Native/fresh WASM 검증

R11 source `048f44da99d83e186f3a9e139563d20545c8bdb4`에서 같은 interview 원본·독립
Print를 전2쪽 측정했다. Native/fresh WASM 모두 쪽수가 2쪽이고 raw는 각각
1쪽 **96.01%**, 2쪽 **99.99%**다. catalog 공백 때문에 WASM 글꼴 공급이 중단되던
경로가 실제 입력에서도 끝까지 완료됐다. 측정법은 96 DPI·보정 전 2px 내용 실루엣이다.

주장 페이지인 1쪽의 제목 `면접과 취업`, `10가지 계명과 10가지 원칙!` 및 본문의 좌우
1~10 항목을 review·overlay로 직접 판독했다. 제목·본문 글꼴이 공급되고 항목의 줄 구성과
순서가 보존된다. 큰 그림의 경계 필터링과 제목 그림자에는 Print 대비 잔차가 보여
실루엣 수치를 픽셀 완전 동일로 기록하지 않는다. catalog 정규화의 음성 대조 2건·
전체 모듈 40건 통과와 이 실제 문서의 완료 측정을 구분한다.

[Native 전쪽 TSV](assets/planet6897_20261106/pr8617/r11/interview/native.tsv) ·
[fresh WASM 전쪽 TSV](assets/planet6897_20261106/pr8617/r11/interview/wasm.tsv) ·
[원본·PDF·producer SHA](assets/planet6897_20261106/pr8617/r11/interview/provenance.json).
Native [review](assets/planet6897_20261106/pr8617/r11/interview/native-review-001.png)·
[overlay](assets/planet6897_20261106/pr8617/r11/interview/native-overlay-001.png),
fresh WASM [review](assets/planet6897_20261106/pr8617/r11/interview/wasm-review-001.png)·
[overlay](assets/planet6897_20261106/pr8617/r11/interview/wasm-overlay-001.png).

R11 결과를 후속 수정의 최종 head 통과로 대신하지 않는다.
