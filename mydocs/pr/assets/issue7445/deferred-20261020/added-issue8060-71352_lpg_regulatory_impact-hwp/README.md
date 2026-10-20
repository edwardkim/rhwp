# added-issue8060-71352_lpg_regulatory_impact-hwp

추적: #8060 및 #7445

관측 runtime base: `efc52afb738bd408ac821c4e156101b8d26e5cbd`.
관측 후보: `a30dacfd037122ceacc65439ca03a1afc2bfe69c` (D61 동결 실행).
후속 D66 및 PR 최종 head의 결과와 구분한다. 이관은 개선 완료가 아니다.

비교: 96dpi, compat 2024, 색상 경계 보정 전 2px `silhouette_raw_match_percent`.
입력 SHA-256: `4e9b240082ba91b99669325292cae2b62e88765854d1e7bbbc6d44146dc14d55`.
독립 한컴 Print PDF SHA-256: `c2e1b4beea546d02436e85f9b491ab64093c6327bf6ef227f02f4f075608f501`; 기준 48쪽.

| 관측 | 경로 | 비교 쪽수 | raw 최저 | 90% 미만 쪽 |
| --- | --- | ---: | ---: | --- |
| base | native | 48 | 31.18975% | 8, 9, 11, 12, 16, 20, 21, 22, 28, 29, 31, 37, 42, 43, 47, 48 |
| base | wasm | 48 | 31.28414% | 8, 9, 11, 12, 16, 20, 21, 22, 28, 29, 31, 37, 42, 43, 47, 48 |
| candidate | native | 48 | 31.18975% | 8, 11, 12, 20, 21, 22, 28, 29, 31, 37, 47, 48 |
| candidate | wasm | 48 | 31.28414% | 8, 11, 12, 20, 21, 22, 28, 29, 31, 37, 47, 48 |

## 보존과 복원

- input: `/home/tsjang/rhwp/samples/issue8060/71352_lpg_regulatory_impact.hwp` → `samples/issue8060/71352_lpg_regulatory_impact.hwp` (SHA-256 `4e9b240082ba91b99669325292cae2b62e88765854d1e7bbbc6d44146dc14d55`).
- pdf: `/home/tsjang/rhwp/pdf/issue8060/71352_lpg_regulatory_impact-2024.pdf` → `pdf/issue8060/71352_lpg_regulatory_impact-2024.pdf` (SHA-256 `c2e1b4beea546d02436e85f9b491ab64093c6327bf6ef227f02f4f075608f501`).

공유 입력의 정상 parser/serializer·독립 계약 검사는 보존한다. 부적합한 회귀 분리의 정확한 함수 목록은 상위 README에 기록한다. 같은 원본과 Print PDF에서 관련 전쪽 Native/fresh WASM raw 90% 이상·쪽수/내용 소속·확대 판독을 확인한 뒤 관계형 회귀로 복원한다. 글꼴 바이너리는 복사·개명·배포하지 않는다.
