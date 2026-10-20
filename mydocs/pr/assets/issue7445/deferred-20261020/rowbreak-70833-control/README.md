# rowbreak-70833-control

추적: #7728 및 #7445

관측 runtime base: `efc52afb738bd408ac821c4e156101b8d26e5cbd`.
관측 후보: `a30dacfd037122ceacc65439ca03a1afc2bfe69c` (D61 동결 실행).
후속 D66 및 PR 최종 head의 결과와 구분한다. 이관은 개선 완료가 아니다.

비교: 96dpi, compat 2022, 색상 경계 보정 전 2px `silhouette_raw_match_percent`.
입력 SHA-256: `a80b09754ce4f8e73398af9a9467f570c02be237815fec203ee8ff2db1bb8cbb`.
독립 한컴 Print PDF SHA-256: `ae8b6bd5a07fa543dd9f69be750690851846a3e4e09668e3158bb05fa71bc126`; 기준 18쪽.

| 관측 | 경로 | 비교 쪽수 | raw 최저 | 90% 미만 쪽 |
| --- | --- | ---: | ---: | --- |
| base | native | 18 | 48.25863% | 5, 6, 7, 15, 18 |
| base | wasm | 18 | 48.57902% | 5, 6, 7, 15, 18 |
| candidate | native | 18 | 16.71402% | 5, 6, 7, 9, 10, 12, 15, 18 |
| candidate | wasm | 18 | 16.67226% | 5, 6, 7, 9, 10, 12, 15, 18 |

## 보존과 복원

- input: `/home/tsjang/rhwp/samples/issue6854/70833-electrical-safety-rule-regulatory-analysis.hwp` → `samples/issue6854/70833-electrical-safety-rule-regulatory-analysis.hwp` (SHA-256 `a80b09754ce4f8e73398af9a9467f570c02be237815fec203ee8ff2db1bb8cbb`).
- pdf: `/home/tsjang/rhwp/pdf/70833-electrical-safety-rule-regulatory-analysis-2020.pdf` → `pdf/70833-electrical-safety-rule-regulatory-analysis-2020.pdf` (SHA-256 `ae8b6bd5a07fa543dd9f69be750690851846a3e4e09668e3158bb05fa71bc126`).

공유 입력의 정상 parser/serializer·독립 계약 검사는 보존한다. 부적합한 회귀 분리의 정확한 함수 목록은 상위 README에 기록한다. 같은 원본과 Print PDF에서 관련 전쪽 Native/fresh WASM raw 90% 이상·쪽수/내용 소속·확대 판독을 확인한 뒤 관계형 회귀로 복원한다. 글꼴 바이너리는 복사·개명·배포하지 않는다.
