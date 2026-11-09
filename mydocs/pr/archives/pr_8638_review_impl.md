# PR #8638 통합 적용·메인터너 보정 기록

## 통합 PR의 exact code head CI

- [통합 PR #8857](https://github.com/edwardkim/rhwp/pull/8857), code head `29c74b69851bc6578f10a2c78e23592a05b69a5d`.
- [CI 실행](https://github.com/edwardkim/rhwp/actions/runs/42621490405)과 같은 code head의 CodeQL·Render Diff·Adapter·Proptest·정책 검사를 실제 확인했습니다. 기록 확인 시각: `2026-11-09T22:58:24.689803+09:00`.
- 기본 기능 worker A–E의 실제 nextest는 **12,934 PASS / 0 FAIL / 45 skip**입니다. 기존 로컬 13,105건과 실행 범위·HEAD가 다르므로 합산하지 않습니다.
- source·test·workflow를 추가로 변경하지 않고 이 검토·오늘할일만 단일 부모 trailing commit으로 보완합니다. trailing head의 실제 CI/fast-pass와 mergeability는 push 뒤 별도로 확인하며, 문서만 바뀌었다는 이유로 통과를 미리 가정하지 않습니다.
- [잔여 이슈 #8856](https://github.com/edwardkim/rhwp/issues/8856)은 9문서군의 글리프·간격·도형 및 clip5587 Native 4쪽의 엄격 0.1%p 미충족을 계속 추적합니다. 부분 해결 #8635·#8636과 부모 #7445는 일괄 종료하지 않습니다.


## 최종 source와 재사용 범위

제출 code head `2e262d0d4b2885b130e2db917b86e894710425e4`, base `e00f9b7e26e581896fa17eaff3c8eb3f4889f9d4`입니다. 아래 전체 회귀·전쪽 수치는 원 실측 HEAD의 증거를 재사용한 기록입니다. 최신 집중 검사와 페이지별 producer, 엄격 0.1%p 잔여 한 건은 [재사용·최종 판정](../assets/planet6897_20261106/final-b768bee1-20261109/validation-reuse.md)을 따릅니다. 보호 TSV 표시값 하락은 0입니다.

## 최종 구현·검증 결과

검증 code head `eb876d1416fb558c874f9efec65c5722a6769751`, base `b768bee1c4c0023087687f79a0a95f12108743e9`에서 원 기여와 메인터너 보정을 함께 검증했습니다. 최소 글꼴 전체 회귀 13,105 PASS / 0 FAIL / 45 skip, Native 전용 73 PASS이며, Native/fresh WASM 각각 94조건·2,995쪽의 미달·누락·보호 표시값 하락은 없습니다. 원 PR의 개별 입력·페이지 판독과 보정 내용은 [최종 review](pr_8638_review.md)에 연결합니다. 원격 통합 PR의 최종 CI와 병합은 별도 단계입니다.

## 단계별 기록의 시점

아래 R 단계와 과거 실행의 진행 중·실패·추가 검증 표현은 해당 시점의 원인 추적 기록입니다. 과거 결과를 현재 code head의 통과로 합산하지 않으며, 현재 상태는 위 최종 결과와 연결된 review를 기준으로 판단합니다.

## 대상과 단계

[원 PR #8638](https://github.com/edwardkim/rhwp/pull/8638)의 원 head는 `71bd8abf48dbad4afa09d66803f6011546add734`다.
[공통 계획](../assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `5554a192b7a62836fc7f9a460623b5da205cf151` | `8a9a9cc5329a1206989c2e55cdc05207e4bfe59b` | fix(renderer): Dotum U+2024 은 Haansoft Dotum 이 공급 (#8619) |
| `71bd8abf48dbad4afa09d66803f6011546add734` | `718d23cae8d6c9b354e7b747459695283370c34f` | docs: #8619 rebase 뒤 Native/fresh WASM sweep 증거 갱신 |

## 최종 리베이스 후 적용 상태

검증 base `b768bee1c4c0023087687f79a0a95f12108743e9`, 검증 제품 head
`eb876d1416fb558c874f9efec65c5722a6769751`의 실제 Git 이력을 대조해 위 적용 SHA를 갱신했다.
[고유 56개 최종 적용표](../assets/planet6897_20261106/contributor-commit-map-final.tsv)는 원 기여 commit의
cherry-picked footer와 현재 적용 commit을 일대일로 확인한 결과다. 중복 patch·bridge의 기존 처리는 유지한다.
이번 b768bee1 리베이스에는 실제 파일 충돌이 없었습니다. index.lock 경합이 끝나고 lock이 없는 상태에서 계속했으며 lock을 삭제하거나 todo를 바꾸지 않았습니다. 제품 src/crates/tests/scripts와 지침의 바이트는 이전 후보와 같고 의존성 변경에 대한 새 검증을 수행합니다.
이력 확인과 현재 제품의 기능·회귀·시각 검증 판정은 구분하며, 필수 검증 완료 전에는 승인하지 않는다.

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.

## 평가결과서 첫 줄 원점의 후속 분석

2026-11-07, 최종 R26 Native 전2쪽은97.02/100%다. 표 제목과 연구결과 영역은 개선됐지만
평가결과 셀에서 항목 제목의 첫 줄만 Print보다 왼쪽에 놓이는 차이가 직접 판독에 남았다.
90% 통과로 면제하지 않고 원본의 들여쓰기·저장 LineSeg 가로 원점과 실제 paint 소비를 추적한다.
현재 Rust 전체 검증과 전쪽 비교의 source는 고정해 두고 먼저 원인을 분석한다.
기존 Print는 재출력하거나 교체하지 않는다. 필요 시 한컴2020의 HWPX 저장 진단본을 사용하되
이 저장본을 새 원본·독립 Print로 취급하지 않는다. 기준 위치는 기존 PDF의 bbox와 원시 속성에서 정한다.

### 첫 줄 원점이 아닌 글머리표의 공통 전진폭으로 원인 특정

원시 CFB·PARA_TEXT에서 항목 제목은 `∙정책연구용역…`이며 점 뒤 공백이 없고 저장
LineSeg도 없다. 한컴2020의 HWPX 진단 저장은 같은 문단 속성(들여쓰기−3001/−3626HU)과
첫 줄 horzpos0을 유지했다. Native 첫 점 x240.72px와 독립 Print의180.38224pt는 거의 같다.
따라서 제목 문단을 옆으로 옮기거나 음수 들여쓰기를 clamp하는 보정은 부적절하다.

기존 Print의 다음 한글 원점은193.33522pt, 글자 크기는12.949415pt다. 점에서 다음 글자까지
약1em을 점유하지만 Native는240.72→249.38667px, 17.3333px 크기의0.5em이다. 뒤 한글의
너비 자체는 Print와 일치한다. [원본·Print·실제 Native 관측](../assets/planet6897_20261106/pr8638/marker-advance/before-observation.json)을 보존했다.

기존 HFT 한양신명조 U+2219의 독립 Type3 폭은 이미 `font_metrics_overlays.rs`에1em으로
등록돼 있다. 공통 CharWidthDecision은 이를 읽은 뒤 마지막 native OTF 전진 선택을 거친다.
검증된 전체 원본 HFT 표와 개별 원본 글리프 증거의 신뢰도 차이를 확인해, 후단 소비가 이미
확인된 이 원본 글리프 전진을 표시 TTF/OTF의 반각으로 덮지 않도록 해야 한다. 전체 family를
검증된 글꼴로 승격하거나 다른 미측정 문자의 OTF 우선을 없애지 않는다. 명시 TrueType의
자연 폭·정상 기호와 보호 문서 대조군도 유지한다. 현재는 분석이며 추가 제품 수정 전이다.


### R27 공통 전진 보정과 설치 글꼴 없는 검사

원본 HFT로 확인한 한양신명조의 일반 U+2218/U+2219만 개별 원본 글리프 전진 경로에 연결했다.
가족 전체를 검증된 HFT 표로 승격하거나 명시 TrueType의 폭을 바꾸지 않는다.
공개 Noto Sans Symbols 2 프로그램을 공급하는 반례는 설치 글꼴 없이 통과했다.
이 프로그램의 실제 두 글자 hmtx는0.276/0.270em으로 원본의1em과 다르며,
표시 프로그램 공급 전후 원본 전진 보존과 명시 TrueType의 비승격을 검사한다.
기존 공급 계약과 함께 R27 집중 결과는25통과/2실패이며 남은 실패는 #7872다.
실물 평가결과서의 새 Native/fresh WASM 전쪽 비교·직접 판독 전에는 시각 완료로 판정하지 않는다.
