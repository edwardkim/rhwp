# PR #8638 누적 검토 구현 계획

## 대상과 단계

[원 PR #8638](https://github.com/edwardkim/rhwp/pull/8638)의 원 head는 `71bd8abf48dbad4afa09d66803f6011546add734`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `5554a192b7a62836fc7f9a460623b5da205cf151` | `f6963c4e2cee2db6f67eafceaee747164f43122b` | fix(renderer): Dotum U+2024 은 Haansoft Dotum 이 공급 (#8619) |
| `71bd8abf48dbad4afa09d66803f6011546add734` | `0e1b172de8bd90a5ffeae905e4f7788b56a76364` | docs: #8619 rebase 뒤 Native/fresh WASM sweep 증거 갱신 |

## R42 리베이스 후 적용 상태

새 base `1e16a19aede5ccb2657d35974940eb596286cdf1`, 리베이스 직후 head
`896708a4a02c4138064ab3d426a487bbce52fc02` 기준으로 위 실제 적용 SHA를 갱신했다.
중복 patch 행의 원 기여 commit 참조는 유지한다. [고유 56개 적용표](assets/planet6897_20261106/contributor-commit-map-r42.tsv)와
[리베이스·충돌 처리 보고서](assets/planet6897_20261106/rebase-r42-analysis.md)를 따른다.
원 PR head와 입력은 보존됐으며, 이력 갱신을 최종 검증 통과나 승인으로 표시하지 않는다.

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
너비 자체는 Print와 일치한다. [원본·Print·실제 Native 관측](assets/planet6897_20261106/pr8638/marker-advance/before-observation.json)을 보존했다.

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
