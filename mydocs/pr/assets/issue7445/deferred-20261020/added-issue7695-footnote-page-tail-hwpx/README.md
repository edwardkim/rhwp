# added-issue7695-footnote-page-tail-hwpx

추적: #7445 사용자 지정 잔여 이관

관측 runtime base: `efc52afb738bd408ac821c4e156101b8d26e5cbd`.
관측 후보: `a30dacfd037122ceacc65439ca03a1afc2bfe69c` (D61 동결 실행).
후속 D66 및 PR 최종 head의 결과와 구분한다. 이관은 개선 완료가 아니다.

비교: 96dpi, compat 2024, 색상 경계 보정 전 2px `silhouette_raw_match_percent`.
입력 SHA-256: `032ac821200737606d7b1d6b4dd5b53af0199a3ccfb2bb81cff6907f4f788bf3`.
독립 한컴 Print PDF SHA-256: `d82042b149d3df8aae8a693a14c6c2bb89945f2cab72a4b58b786e6a4928b89b`; 기준 32쪽.

| 관측 | 경로 | 비교 쪽수 | raw 최저 | 90% 미만 쪽 |
| --- | --- | ---: | ---: | --- |
| base | native | 32 | 69.98080% | 2, 4, 6, 8, 10, 12, 14, 16, 18 |
| base | wasm | 32 | 71.62443% | 2, 4, 6, 8, 10, 12, 14, 16, 18 |
| candidate | native | 32 | 69.98080% | 2, 4, 6, 8, 10, 12, 14, 16, 18 |
| candidate | wasm | 32 | 71.62443% | 2, 4, 6, 8, 10, 12, 14, 16, 18 |

## 보존과 복원

- input: `/home/tsjang/rhwp/samples/issue7695/footnote-page-tail.hwpx` → `mydocs/pr/assets/issue7445/deferred-20261020/added-issue7695-footnote-page-tail-hwpx/footnote-page-tail.hwpx` (SHA-256 `032ac821200737606d7b1d6b4dd5b53af0199a3ccfb2bb81cff6907f4f788bf3`).
- pdf: `/home/tsjang/rhwp/pdf/issue7695/footnote-page-tail-2024.pdf` → `mydocs/pr/assets/issue7445/deferred-20261020/added-issue7695-footnote-page-tail-hwpx/footnote-page-tail-2024.pdf` (SHA-256 `d82042b149d3df8aae8a693a14c6c2bb89945f2cab72a4b58b786e6a4928b89b`).

공유 입력의 정상 parser/serializer·독립 계약 검사는 보존한다. 부적합한 회귀 분리의 정확한 함수 목록은 상위 README에 기록한다. 같은 원본과 Print PDF에서 관련 전쪽 Native/fresh WASM raw 90% 이상·쪽수/내용 소속·확대 판독을 확인한 뒤 관계형 회귀로 복원한다. 글꼴 바이너리는 복사·개명·배포하지 않는다.
