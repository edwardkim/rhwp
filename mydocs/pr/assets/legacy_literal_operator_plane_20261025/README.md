# 구형 문자·연산자 평면과 현대 COLOR의 독립 증거

제품 소스는 `b7fca6dbe92a2f357b401e3b9ce4ce93dec22687`, 정책 base는
`9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`다. Native211과 최적화 fresh WASM058을
같은 원본·Print·96dpi·글꼴 공급으로 실행했다. 이 단계 이후 변경은 검사·증거 기록이며 제품 소스는 같다.
입력·PDF SHA, 실제 명령, 각 TSV·PNG SHA와 producer는 [evidence.json](evidence.json)에 있다.

## 전체 페이지 대조

16개 조건의 각 239쪽을 양쪽에서 비교했다. 독립 PDF 쪽수가 모두 같고 누락·90% 미만 쪽은 없다.
신규 원점 회귀에 쓰는 네 grid는 전 23쪽이 양쪽 모두 98% 이상이다. 평균으로 미달 쪽을 상쇄하지 않았다.

| 입력 | 독립 Print | 쪽수 | Native 최저 raw | fresh 최저 raw | Native TSV | fresh TSV |
| --- | --- | ---: | ---: | ---: | --- | --- |
| [equation-cc9-legacy-color](inputs/equation-cc9-legacy-color.hwpx) | [PDF](../../../../pdf/legacy-literal-operator-plane/equation-cc9-legacy-color-2020.pdf) | 1 | 100.00000% | 100.00000% | [TSV](scores/native-equation-cc9-legacy-color.tsv) | [TSV](scores/wasm-equation-cc9-legacy-color.tsv) |
| [equation-cc9-legacy-percent](inputs/equation-cc9-legacy-percent.hwpx) | [PDF](../../../../pdf/legacy-literal-operator-plane/equation-cc9-legacy-percent-2020.pdf) | 1 | 100.00000% | 100.00000% | [TSV](scores/native-equation-cc9-legacy-percent.tsv) | [TSV](scores/wasm-equation-cc9-legacy-percent.tsv) |
| [equation-cc9-legacy-triangle-dot](inputs/equation-cc9-legacy-triangle-dot.hwpx) | [PDF](../../../../pdf/legacy-literal-operator-plane/equation-cc9-legacy-triangle-dot-2020.pdf) | 1 | 100.00000% | 100.00000% | [TSV](scores/native-equation-cc9-legacy-triangle-dot.tsv) | [TSV](scores/wasm-equation-cc9-legacy-triangle-dot.tsv) |
| [equation-cc9-legacy-plain](inputs/equation-cc9-legacy-plain.hwpx) | [PDF](../../../../pdf/legacy-literal-operator-plane/equation-cc9-legacy-plain-2020.pdf) | 1 | 100.00000% | 100.00000% | [TSV](scores/native-equation-cc9-legacy-plain.tsv) | [TSV](scores/wasm-equation-cc9-legacy-plain.tsv) |
| [equation-cc9-modern-color](inputs/equation-cc9-modern-color.hwpx) | [PDF](../../../../pdf/legacy-literal-operator-plane/equation-cc9-modern-color-2020.pdf) | 1 | 100.00000% | 100.00000% | [TSV](scores/native-equation-cc9-modern-color.tsv) | [TSV](scores/wasm-equation-cc9-modern-color.tsv) |
| [equation-cc9-modern-percent](inputs/equation-cc9-modern-percent.hwpx) | [PDF](../../../../pdf/legacy-literal-operator-plane/equation-cc9-modern-percent-2020.pdf) | 1 | 100.00000% | 100.00000% | [TSV](scores/native-equation-cc9-modern-percent.tsv) | [TSV](scores/wasm-equation-cc9-modern-percent.tsv) |
| [legacy-cjk-operator-grid](inputs/legacy-cjk-operator-grid.hwpx) | [PDF](../../../../pdf/legacy-literal-operator-plane/legacy-cjk-operator-grid-2020.pdf) | 7 | 99.40476% | 99.46460% | [TSV](scores/native-legacy-cjk-operator-grid.tsv) | [TSV](scores/wasm-legacy-cjk-operator-grid.tsv) |
| [legacy-literal-advance-grid](inputs/legacy-literal-advance-grid.hwpx) | [PDF](../../../../pdf/legacy-literal-operator-plane/legacy-literal-advance-grid-2020.pdf) | 10 | 99.40622% | 99.42098% | [TSV](scores/native-legacy-literal-advance-grid.tsv) | [TSV](scores/wasm-legacy-literal-advance-grid.tsv) |
| [legacy-group-operator-boundaries](inputs/legacy-group-operator-boundaries.hwpx) | [PDF](../../../../pdf/legacy-literal-operator-plane/legacy-group-operator-boundaries-2020.pdf) | 1 | 100.00000% | 100.00000% | [TSV](scores/native-legacy-group-operator-boundaries.tsv) | [TSV](scores/wasm-legacy-group-operator-boundaries.tsv) |
| [legacy-mixed-operator-grid](inputs/legacy-mixed-operator-grid.hwpx) | [PDF](../../../../pdf/legacy-literal-operator-plane/legacy-mixed-operator-grid-2020.pdf) | 5 | 99.69619% | 99.70065% | [TSV](scores/native-legacy-mixed-operator-grid.tsv) | [TSV](scores/wasm-legacy-mixed-operator-grid.tsv) |
| [modern-group-plain](inputs/modern-group-plain.hwpx) | [PDF](../../../../pdf/legacy-literal-operator-plane/modern-group-plain-2020.pdf) | 1 | 100.00000% | 100.00000% | [TSV](scores/native-modern-group-plain.tsv) | [TSV](scores/wasm-modern-group-plain.tsv) |
| [modern-group-grouped](inputs/modern-group-grouped.hwpx) | [PDF](../../../../pdf/legacy-literal-operator-plane/modern-group-grouped-2020.pdf) | 1 | 100.00000% | 100.00000% | [TSV](scores/native-modern-group-grouped.tsv) | [TSV](scores/wasm-modern-group-grouped.tsv) |
| [modern-group-color-grouped](inputs/modern-group-color-grouped.hwpx) | [PDF](../../../../pdf/legacy-literal-operator-plane/modern-group-color-grouped-2020.pdf) | 1 | 100.00000% | 100.00000% | [TSV](scores/native-modern-group-color-grouped.tsv) | [TSV](scores/wasm-modern-group-color-grouped.tsv) |
| [modern-group-color-ungrouped](inputs/modern-group-color-ungrouped.hwpx) | [PDF](../../../../pdf/legacy-literal-operator-plane/modern-group-color-ungrouped-2020.pdf) | 1 | 100.00000% | 100.00000% | [TSV](scores/native-modern-group-color-ungrouped.tsv) | [TSV](scores/wasm-modern-group-color-ungrouped.tsv) |
| [legacy-character-plane-control](../../../../tests/fixtures/issue1949/legacy-character-plane.hwpx) | [PDF](../../../../pdf/issue1949-legacy-character-plane/legacy-character-plane-2020.pdf) | 9 | 99.98654% | 100.00000% | [TSV](scores/native-legacy-character-plane-control.tsv) | [TSV](scores/wasm-legacy-character-plane-control.tsv) |
| [giant4138-tab-period](../../../../tests/fixtures/issue4138/tab-period-split.hwp) | [PDF](../../../../pdf/issue4138-tab-period-split-20261015-2024.pdf) | 197 | 94.53675% | 94.84172% | [TSV](scores/native-giant4138-tab-period.tsv) | [TSV](scores/wasm-giant4138-tab-period.tsv) |

14개 합성 입력과 14개 기준 PDF는 이 단계의 정식 경로에 동일 바이트로 포함했다.
기존 문자 평면과 실제 원문·PDF는 이미 추적하는 파일을 재사용했다. 실행 로그·중간 JSON·글꼴 바이트는 포함하지 않는다.

## 기준 출력과 입력 생성의 구분

새 grid와 현대 대조군은 한컴 2020 `11.0.0.9136`, PrintMethod 0·1-up·전처리 none으로 출력했다.
실제 `tab-period-split.hwp`는 기존 독립 한컴 2024 Print를 재사용했다. engine·job·PDF의 대응은
[독립 좌표](independent_coordinates.json) 및 기존 원문 증거와 연결한다.

기존 helper의 `char_count=2`는 8단위 컨트롤과 문단 끝을 담기에 맞지 않는다. 새 `equation-cc9-*`는
다른 모델 속성을 유지하고 `char_count=9`로 재생성한 입력이다. 원 helper와 그 실패 진단은 보존했으며,
새 입력의 통과를 원래 잘못된 직렬화의 통과로 보고하지 않는다. 기존 baseline 편집 helper 전체를 바꾸지 않았다.

빈 버전의 `COLOR`는 한컴에서 유효한 현대 색상 wrapper로 해석되지 않는다. `legacy-color`의 빈 출력
100%를 COLOR 구현 성공으로 세지 않는다. 현대 색상 검사는 명시적인 `Equation Version 60` 입력을 쓴다.

## 독립 기대값과 회귀 변경

- 기존 잘못된 구형 퍼센트 PUA 기대값을 실제 구형 Type3 `%`와 현대 HyhwpEQ U+E042로 구분했다.
- COLOR를 투명한 연산자 경계로 가정하던 검사를 바꿨다. 독립 Print에서 앞 여백 차이는 약 0.21em,
  뒤 문자 원점 차이는 약 0.42em이다. 실제 빨간 기호, 한글 원점, 내용·순서·누락·중복도 검사한다.
- 실제 130쪽의 원문 `△․`와 Type3 공급을 보존한다. 잘못된 `△⋅`·HyhwpEQ 기대값을 바로잡고,
  원본 d1 1em 전진 계약을 유지했다. 원문 130쪽 raw는 Native 98.43072%, fresh 98.45609%다.
- 새 [문서별 회귀 4개](../../../../tests/cases/verified_legacy_literal_operator_plane.rs)는 literal 108개,
  CJK·연산자 72개, 혼재 54개, 그룹 9개 행을 각각 보호한다. 원본·줄·쪽·컨트롤 소유와 내용,
  자연 크기, 선언 프레임을 검사하고 완료한 각 페이지의 독립 원점을 ±2 CSS px로 함께 확인한다.
  고정 좌표는 엔진 계산식을 재인용하지 않고 PDF 원점에서 가져왔다. root 여백은 원문 표식과의
  상대 위치로, 혼재 baseline은 첫 글리프와의 관계로 검사한다.

쪽 하단 표식과 다음 쪽 수식이 갈리는 19개 행은 독립 PDF raw span에서 실제 글리프 쪽을 다시 읽었다.
표식의 쪽 번호를 수식 소속으로 복사하지 않았다. 잘못된 초기 진단 원장은 output에 보존했다.

최종 정식 검사 소스와 같은 입력 바이트로 수정 전 Native210의 **7 FAIL**, 보정 후 Native211의
**30 PASS / 0 FAIL / 0 ignored**를 확인했다. 이전 실패는 대상 표시·간격·공통 원점 계약이며,
초기 임시 include 경로 오류나 표식 쪽을 쓴 진단 오류를 제품 결함 검출 증거로 세지 않는다.

## 직접 판독과 대표 이미지

각 표의 review와 standalone overlay를 함께 열어 동일 영역의 문자·자연 크기·간격·그룹 경계와
수정한 실제 기호를 대조했다. 얇은 획의 농도와 서브픽셀 색상 테두리 차이는 남는다.
raw는 2px 관용 실루엣이며 RGB 완전 일치나 다른 수식 전체의 일치로 확대해 해석하지 않는다.

| 대표 범위 | Native review / overlay | fresh review / overlay |
| --- | --- | --- |
| literal p5 | [review](images/native-literal-review-005.png) / [overlay](images/native-literal-overlay-005.png) | [review](images/wasm-literal-review-005.png) / [overlay](images/wasm-literal-overlay-005.png) |
| cjk p7 | [review](images/native-cjk-review-007.png) / [overlay](images/native-cjk-overlay-007.png) | [review](images/wasm-cjk-review-007.png) / [overlay](images/wasm-cjk-overlay-007.png) |
| mixed p5 | [review](images/native-mixed-review-005.png) / [overlay](images/native-mixed-overlay-005.png) | [review](images/wasm-mixed-review-005.png) / [overlay](images/wasm-mixed-overlay-005.png) |
| groups p1 | [review](images/native-groups-review-001.png) / [overlay](images/native-groups-overlay-001.png) | [review](images/wasm-groups-review-001.png) / [overlay](images/wasm-groups-overlay-001.png) |
| modern-color p1 | [review](images/native-modern-color-review-001.png) / [overlay](images/native-modern-color-overlay-001.png) | [review](images/wasm-modern-color-review-001.png) / [overlay](images/wasm-modern-color-overlay-001.png) |
| real130 p130 | [review](images/native-real130-review-130.png) / [overlay](images/native-real130-overlay-130.png) | [review](images/wasm-real130-review-130.png) / [overlay](images/wasm-real130-overlay-130.png) |

## 고정 devel 대조와 실제 쪽수 보정

`6097253468`에서 빌드한 base producer와 정책 base `9418ed0c7c` 사이의 제품 코드·Cargo·
자원·wrapper가 같은 것을 확인하고 동일 환경으로 비교했다. 13개 조건은 양쪽 전쪽 raw를 측정했다.
나머지 세 조건은 다음과 같이 devel의 실제 쪽수가 Print와 달라 전쪽 비교가 exit 1로 중단됐다.
이 결과를 성공으로 바꾸거나 없는 쪽의 점수를 만들지 않았다.

| 범위 | 실제 base 쪽수 | 후보 = Print 쪽수 | 원본 수식 수 | base의 잘못된 쪽 소속 | 후보의 잘못된 쪽 소속 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 기존 문자 평면 | 8 | 9 | 186 | 131 | 0 |
| literal grid | 9 | 10 | 108 | 72 | 0 |
| CJK grid | 6 | 7 | 72 | 36 | 0 |

[실제 쪽 소유 대조](base_page_ownership.json)에서 두 backend 모두 원본 수식이 누락·중복 없이
한 번씩 보존됐고 후보의 쪽 소속은 모두 Print와 일치했다. 실제로 있는 공통 8·9·6쪽은 기존
PNG를 같은 raw 방식으로 측정했다. 다른 13개 조건과 합친 backend별 236개 비교 가능 페이지에서
2%p 초과 하락은 없다. base에 없는 쪽은 가짜 흰 페이지나 점수를 만들지 않고 쪽 소속 보정으로
기록한다. 세 후보 문서는 양쪽 전쪽 실측과 대표 영역 직접 판독을 마쳤다.
이 구조 개선 증거와 정식 전쪽 raw 비교의 쪽수 불일치를 구분하며 전체 PR 판정을 자동 승인하지 않는다.
base TSV 32개와 비교 범위·명령은 [evidence.json](evidence.json)의 `base_comparison`에 있다.

## 단계 완료와 전체 PR의 구분

이 기록은 해당 수식 보정과 정식 회귀의 증거다. 남은 필수 실패, 최종 전체 Rust·lint·Skia·Studio,
원래 45개 PR 및 #8064 검증 범위와 최신 base 대비 2%p 검증은 전체 PR 준비에서 계속 완료한다.
실행하지 않은 전체 gate를 이 단계의 30개 통과로 대체하지 않는다. 현재 PR 준비 완료나 승인을 뜻하지 않는다.
