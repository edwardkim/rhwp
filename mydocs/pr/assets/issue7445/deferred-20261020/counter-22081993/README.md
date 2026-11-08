# counter-22081993

추적: #7445 사용자 지정 잔여 이관

관측 runtime base: `efc52afb738bd408ac821c4e156101b8d26e5cbd`.
관측 후보: `a30dacfd037122ceacc65439ca03a1afc2bfe69c` (D61 동결 실행).
후속 D66 및 PR 최종 head의 결과와 구분한다. 이관은 개선 완료가 아니다.

비교: 96dpi, compat 2022, 색상 경계 보정 전 2px `silhouette_raw_match_percent`.
입력 SHA-256: `5ca5e5256dc960690474f741879b9d2f19055a09ecfca8a810da6133e2600b1f`.
독립 한컴 Print PDF SHA-256: `fce705c647b247beed54c0029bc9e4eccb8496e5a00ba65b9abcf0531a202060`; 기준 121쪽.

| 관측 | 경로 | 비교 쪽수 | raw 최저 | 90% 미만 쪽 |
| --- | --- | ---: | ---: | --- |
| base | native | 121 | 15.30344% | 10, 11, 12, 13, 34, 40, 90, 91, 92, 107, 110, 118 |
| base | wasm | 121 | 15.38214% | 10, 11, 12, 13, 34, 40, 90, 91, 92, 107, 110, 118 |
| candidate | native | 121 | 15.30344% | 90, 91 |
| candidate | wasm | 121 | 15.38214% | 90, 91 |

## 보존과 복원

- input: `/home/tsjang/Downloads/korea_downloads/ordin_downloads/전남광주통합특별시교육청/22081993_[별표 1] 사무분장표(제77조제1항¸ 제79조¸ 제82조 관련)(전남광주통합특별시교육청 행정기구 설치 조례 시행규칙).hwp` → `mydocs/pr/assets/issue7445/deferred-20261020/counter-22081993/22081993_[별표 1] 사무분장표(제77조제1항¸ 제79조¸ 제82조 관련)(전남광주통합특별시교육청 행정기구 설치 조례 시행규칙).hwp` (SHA-256 `5ca5e5256dc960690474f741879b9d2f19055a09ecfca8a810da6133e2600b1f`).
- pdf: `/home/tsjang/rhwp/pdf/pr-review-planet6897-20261018/22081993-counter-print/22081993-office-duties-counter-2020.pdf` → `mydocs/pr/assets/issue7445/deferred-20261020/counter-22081993/22081993-office-duties-counter-2020.pdf` (SHA-256 `fce705c647b247beed54c0029bc9e4eccb8496e5a00ba65b9abcf0531a202060`).

공유 입력의 정상 parser/serializer·독립 계약 검사는 보존한다. 부적합한 회귀 분리의 정확한 함수 목록은 상위 README에 기록한다. 같은 원본과 Print PDF에서 관련 전쪽 Native/fresh WASM raw 90% 이상·쪽수/내용 소속·확대 판독을 확인한 뒤 관계형 회귀로 복원한다. 글꼴 바이너리는 복사·개명·배포하지 않는다.
