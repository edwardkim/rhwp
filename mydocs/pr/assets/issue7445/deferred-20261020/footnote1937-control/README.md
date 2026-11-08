# footnote1937-control

추적: #7732 및 #7445

관측 runtime base: `efc52afb738bd408ac821c4e156101b8d26e5cbd`.
관측 후보: `a30dacfd037122ceacc65439ca03a1afc2bfe69c` (D61 동결 실행).
후속 D66 및 PR 최종 head의 결과와 구분한다. 이관은 개선 완료가 아니다.

비교: 96dpi, compat 2022, 색상 경계 보정 전 2px `silhouette_raw_match_percent`.
입력 SHA-256: `a3a075594994e4741a7fbe973bc230d95ae75fdee6578ac6017cec0d52844990`.
독립 한컴 Print PDF SHA-256: `7f14bc85eb389a68cc88bd40fcafeb7a638d8454293a30bb2c8fe884a16978eb`; 기준 50쪽.

| 관측 | 경로 | 비교 쪽수 | raw 최저 | 90% 미만 쪽 |
| --- | --- | ---: | ---: | --- |
| base | native | 미산출 | 측정 불가 | 쪽수 {'rhwp': 51, 'pdf': 50}; 전쪽 raw 통과로 보고하지 않음 |
| base | wasm | 미산출 | 측정 불가 | 쪽수 {'rhwp': 51, 'pdf': 50}; 전쪽 raw 통과로 보고하지 않음 |
| candidate | native | 50 | 45.74537% | 19, 20, 21, 23, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 49 |
| candidate | wasm | 50 | 45.69349% | 19, 20, 21, 23, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43 |

## 보존과 복원

- input: `/home/tsjang/rhwp/samples/issue1937_rowbreak_footnote_overpagination.hwp` → `samples/issue1937_rowbreak_footnote_overpagination.hwp` (SHA-256 `a3a075594994e4741a7fbe973bc230d95ae75fdee6578ac6017cec0d52844990`).
- pdf: `/home/tsjang/rhwp/pdf/issue7382-regression-review/issue1937_rowbreak_footnote_overpagination-2020.pdf` → `pdf/issue7382-regression-review/issue1937_rowbreak_footnote_overpagination-2020.pdf` (SHA-256 `7f14bc85eb389a68cc88bd40fcafeb7a638d8454293a30bb2c8fe884a16978eb`).

공유 입력의 정상 parser/serializer·독립 계약 검사는 보존한다. 부적합한 회귀 분리의 정확한 함수 목록은 상위 README에 기록한다. 같은 원본과 Print PDF에서 관련 전쪽 Native/fresh WASM raw 90% 이상·쪽수/내용 소속·확대 판독을 확인한 뒤 관계형 회귀로 복원한다. 글꼴 바이너리는 복사·개명·배포하지 않는다.
