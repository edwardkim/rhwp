# 리베이스 후 고정 범위의 전체 페이지 TSV

- 고정 devel: `238de6de41a208648f4ade6b494fe433a81c1fce`.
- 렌더 소스: `bbf3aed18dd82bbf344072b77b5ac590e7a71f3a`; 코드·회귀 후보: `61528013511c87c732524f91c6f224286a055337`.
- Native255는 실제 Skia PNG, fresh WASM073은 표준 release 및 실제 `wasm-opt -O`를 마친 Canvas PNG다.
- 현재 보정 대상과 리베이스 기능 보존의 고정 17조건·213쪽/backend만 담는다. 기존의 다른 문서나 삭제한 검사·파생 참조를 되살리지 않는다.
- 각 후보의 전체 쪽수는 독립 Print PDF와 같다. 전체 페이지 순서·중복·누락·측정값을 확인했고 모든 쪽이 raw 90% 이상이다. 정확히 90%는 통과한다.
- 비교 가능한 16조건·211쪽/backend에는 Decimal로 계산한 0.1%p 초과 하락이 없다. #6133의 구조 개선 2쪽은 아래에서 별도로 구분한다.
- 원본·PDF·실제 producer·TSV SHA-256과 출력 backend·인쇄 profile·호환 세대는 [provenance.json](provenance.json)에 있다. 입력과 기준 PDF의 실제 바이트가 검토 commit에 포함되어 있음을 확인했다.
- 원문 TSV 66개는 byte 단위로 그대로 복사했다. `base_candidate_comparison.tsv`만 두 TSV의 원래 소수값으로 산출한 비교표다. 로그·중간 분석 JSON은 커밋하지 않는다.
- 아래 값은 색상 경계 보정 전 2px 내용 실루엣이다. RGB 완전 일치나 미세 모양의 직접 판독을 대신하지 않는다. 대표 review·overlay와 판독은 원 PR별 리뷰 및 기존 주제별 증거에 연결한다.

| 경로 | 후보 전체 쪽수 | 후보 최저 raw | 수치 비교 쪽수 | 0.1%p 초과 하락 |
| --- | ---: | ---: | ---: | ---: |
| Native | 213 | 92.04708% | 211 | 0 |
| fresh WASM | 213 | 91.53306% | 211 | 0 |

## 입력별 전체 TSV

| 조건 | 후보 쪽수 | Native 후보 | Native base | fresh WASM 후보 | fresh WASM base |
| --- | ---: | --- | --- | --- | --- |
| 1510-hwp | 1 | [TSV](native/candidate/1510-hwp_silhouette.tsv) | [TSV](native/base/1510-hwp_silhouette.tsv) | [TSV](fresh-wasm/candidate/1510-hwp_silhouette.tsv) | [TSV](fresh-wasm/base/1510-hwp_silhouette.tsv) |
| 1510 | 2 | [TSV](native/candidate/1510_silhouette.tsv) | [TSV](native/base/1510_silhouette.tsv) | [TSV](fresh-wasm/candidate/1510_silhouette.tsv) | [TSV](fresh-wasm/base/1510_silhouette.tsv) |
| 1535 | 2 | [TSV](native/candidate/1535_silhouette.tsv) | [TSV](native/base/1535_silhouette.tsv) | [TSV](fresh-wasm/candidate/1535_silhouette.tsv) | [TSV](fresh-wasm/base/1535_silhouette.tsv) |
| 6133 | 2 | [TSV](native/candidate/6133_silhouette.tsv) | 쪽수 미달; 수치 대응 불가 | [TSV](fresh-wasm/candidate/6133_silhouette.tsv) | 쪽수 미달; 수치 대응 불가 |
| 6970 | 3 | [TSV](native/candidate/6970_silhouette.tsv) | [TSV](native/base/6970_silhouette.tsv) | [TSV](fresh-wasm/candidate/6970_silhouette.tsv) | [TSV](fresh-wasm/base/6970_silhouette.tsv) |
| field-reset | 2 | [TSV](native/candidate/field-reset_silhouette.tsv) | [TSV](native/base/field-reset_silhouette.tsv) | [TSV](fresh-wasm/candidate/field-reset_silhouette.tsv) | [TSV](fresh-wasm/base/field-reset_silhouette.tsv) |
| field-plain | 2 | [TSV](native/candidate/field-plain_silhouette.tsv) | [TSV](native/base/field-plain_silhouette.tsv) | [TSV](fresh-wasm/candidate/field-plain_silhouette.tsv) | [TSV](fresh-wasm/base/field-plain_silhouette.tsv) |
| titled-host | 4 | [TSV](native/candidate/titled-host_silhouette.tsv) | [TSV](native/base/titled-host_silhouette.tsv) | [TSV](fresh-wasm/candidate/titled-host_silhouette.tsv) | [TSV](fresh-wasm/base/titled-host_silhouette.tsv) |
| saved-host | 4 | [TSV](native/candidate/saved-host_silhouette.tsv) | [TSV](native/base/saved-host_silhouette.tsv) | [TSV](fresh-wasm/candidate/saved-host_silhouette.tsv) | [TSV](fresh-wasm/base/saved-host_silhouette.tsv) |
| omitted-after | 2 | [TSV](native/candidate/omitted-after_silhouette.tsv) | [TSV](native/base/omitted-after_silhouette.tsv) | [TSV](fresh-wasm/candidate/omitted-after_silhouette.tsv) | [TSV](fresh-wasm/base/omitted-after_silhouette.tsv) |
| symbol-slot-2020 | 1 | [TSV](native/candidate/symbol-slot-2020_silhouette.tsv) | [TSV](native/base/symbol-slot-2020_silhouette.tsv) | [TSV](fresh-wasm/candidate/symbol-slot-2020_silhouette.tsv) | [TSV](fresh-wasm/base/symbol-slot-2020_silhouette.tsv) |
| symbol-slot-2024 | 1 | [TSV](native/candidate/symbol-slot-2024_silhouette.tsv) | [TSV](native/base/symbol-slot-2024_silhouette.tsv) | [TSV](fresh-wasm/candidate/symbol-slot-2024_silhouette.tsv) | [TSV](fresh-wasm/base/symbol-slot-2024_silhouette.tsv) |
| 4599 | 64 | [TSV](native/candidate/4599_silhouette.tsv) | [TSV](native/base/4599_silhouette.tsv) | [TSV](fresh-wasm/candidate/4599_silhouette.tsv) | [TSV](fresh-wasm/base/4599_silhouette.tsv) |
| 5846 | 66 | [TSV](native/candidate/5846_silhouette.tsv) | [TSV](native/base/5846_silhouette.tsv) | [TSV](fresh-wasm/candidate/5846_silhouette.tsv) | [TSV](fresh-wasm/base/5846_silhouette.tsv) |
| 7518 | 6 | [TSV](native/candidate/7518_silhouette.tsv) | [TSV](native/base/7518_silhouette.tsv) | [TSV](fresh-wasm/candidate/7518_silhouette.tsv) | [TSV](fresh-wasm/base/7518_silhouette.tsv) |
| 8060 | 48 | [TSV](native/candidate/8060_silhouette.tsv) | [TSV](native/base/8060_silhouette.tsv) | [TSV](fresh-wasm/candidate/8060_silhouette.tsv) | [TSV](fresh-wasm/base/8060_silhouette.tsv) |
| 6448 | 3 | [TSV](native/candidate/6448_silhouette.tsv) | [TSV](native/base/6448_silhouette.tsv) | [TSV](fresh-wasm/candidate/6448_silhouette.tsv) | [TSV](fresh-wasm/base/6448_silhouette.tsv) |

- [Native 페이지별 base/후보 비교표](native/base_candidate_comparison.tsv)
- [fresh WASM 페이지별 base/후보 비교표](fresh-wasm/base_candidate_comparison.tsv)

## #6133의 쪽수 복원

같은 원본·독립 Print는 2쪽이며 base 출력은 1쪽이었다. 후보는 두 출력 경로 모두 2쪽이고
각 쪽 raw는 100%다. 대응하지 않는 base의 두 번째 쪽 점수를 만들거나 구조 변경을
동일 페이지의 수치 비교로 주장하지 않았다. 이 입력의 base에는 TSV가 없으며, 확인한
base 1쪽·Print 2쪽과 후보의 내용·쪽 소속 복원을 `base_structural_diagnostic`에 기록한다.

## 검증 경계

새로 등록한 양식·정상 한컴 저장 Square의 3조건·3쪽/backend 증거는
[별도 동일 후보 자료](../inline_wrap_qualified_20261026/README.md)에 있다. 두 묶음을 합하면
현재 20조건·216쪽/backend이며, 이 숫자를 원 PR 45개 각각의 모든 필수 검증이 끝났다는
의미로 사용하지 않는다. 원 PR별 검토는 해당 review 문서에서 소비 경로·반례·입력에 연결한다.

전체 Rust·lint·Native Skia와 원격 CI·merge는 별도 게이트다. TSV 성공만으로 PR 준비 완료를
선언하지 않으며, 실제 배치·미세 모양의 확인된 결함을 점수나 평균 개선으로 면제하지 않는다.
