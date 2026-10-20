# rowbreak-1613000-control

추적: #7727 및 #7445

관측 runtime base: `efc52afb738bd408ac821c4e156101b8d26e5cbd`.
관측 후보: `a30dacfd037122ceacc65439ca03a1afc2bfe69c` (D61 동결 실행).
후속 D66 및 PR 최종 head의 결과와 구분한다. 이관은 개선 완료가 아니다.

비교: 96dpi, compat 2022, 색상 경계 보정 전 2px `silhouette_raw_match_percent`.
입력 SHA-256: `8ef9de3f35690bf9d7994527f77cb02d4a4fcff447c219a78fbc2855d64be6e7`.
독립 한컴 Print PDF SHA-256: `71145684dae73b459f088a8e10acc0335a7585c5345d7207401a3d988696fe6b`; 기준 204쪽.

| 관측 | 경로 | 비교 쪽수 | raw 최저 | 90% 미만 쪽 |
| --- | --- | ---: | ---: | --- |
| base | native | 204 | 22.50591% | 1, 6, 23, 32, 58, 59, 75, 85, 124, 125, 127, 128, 185, 186, 187, 188, 189, 190, 191, 192, 193, 196, 197, 198, 201, 202 |
| base | wasm | 204 | 22.53012% | 1, 6, 23, 32, 58, 59, 75, 85, 124, 125, 127, 128, 185, 186, 187, 188, 189, 190, 191, 192, 193, 196, 197, 198, 201, 202 |
| candidate | native | 204 | 37.99162% | 1, 6, 32, 59, 75, 142, 144, 146, 148, 156, 160, 164, 168, 181, 189, 190 |
| candidate | wasm | 204 | 38.25314% | 1, 6, 32, 59, 75, 142, 144, 146, 148, 156, 160, 164, 168, 181, 189, 190 |

## 보존과 복원

- input: `/home/tsjang/rhwp/samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp` → `samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp` (SHA-256 `8ef9de3f35690bf9d7994527f77cb02d4a4fcff447c219a78fbc2855d64be6e7`).
- pdf: `/home/tsjang/rhwp/pdf/1613000-202200037-air-traffic-controller-cbta-2020.pdf` → `pdf/1613000-202200037-air-traffic-controller-cbta-2020.pdf` (SHA-256 `71145684dae73b459f088a8e10acc0335a7585c5345d7207401a3d988696fe6b`).

공유 입력의 정상 parser/serializer·독립 계약 검사는 보존한다. 부적합한 회귀 분리의 정확한 함수 목록은 상위 README에 기록한다. 같은 원본과 Print PDF에서 관련 전쪽 Native/fresh WASM raw 90% 이상·쪽수/내용 소속·확대 판독을 확인한 뒤 관계형 회귀로 복원한다. 글꼴 바이너리는 복사·개명·배포하지 않는다.
