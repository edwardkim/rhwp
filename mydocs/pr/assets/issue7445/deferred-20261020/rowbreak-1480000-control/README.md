# rowbreak-1480000-control

추적: #7718 및 #7445

관측 runtime base: `efc52afb738bd408ac821c4e156101b8d26e5cbd`.
관측 후보: `a30dacfd037122ceacc65439ca03a1afc2bfe69c` (D61 동결 실행).
후속 D66 및 PR 최종 head의 결과와 구분한다. 이관은 개선 완료가 아니다.

비교: 96dpi, compat 2022, 색상 경계 보정 전 2px `silhouette_raw_match_percent`.
입력 SHA-256: `18765ac22270a2cb4ece6a1e6539c2dea6ca6ccde2573b42fb9acce7c34fa83a`.
독립 한컴 Print PDF SHA-256: `fe01f1ddffc3153d015adab93178f0fa8a5e9624763ff529362d4410d50955dc`; 기준 205쪽.

| 관측 | 경로 | 비교 쪽수 | raw 최저 | 90% 미만 쪽 |
| --- | --- | ---: | ---: | --- |
| base | native | 미산출 | 측정 불가 | 쪽수 {'rhwp': 206, 'pdf': 205}; 전쪽 raw 통과로 보고하지 않음 |
| base | wasm | 미산출 | 측정 불가 | 쪽수 {'rhwp': 206, 'pdf': 205}; 전쪽 raw 통과로 보고하지 않음 |
| candidate | native | 205 | 16.38154% | 11, 14, 16, 41, 79, 80, 81, 82, 83, 130, 136, 137, 140, 141, 143, 144, 145, 146, 147, 148, 155, 158 |
| candidate | wasm | 205 | 16.41011% | 11, 14, 16, 41, 79, 80, 81, 82, 83, 130, 136, 137, 140, 141, 143, 144, 145, 146, 147, 148, 155, 158 |

## 보존과 복원

- input: `/home/tsjang/rhwp/samples/issue5941/1480000-201900698-native-neartop-reset.hwp` → `samples/issue5941/1480000-201900698-native-neartop-reset.hwp` (SHA-256 `18765ac22270a2cb4ece6a1e6539c2dea6ca6ccde2573b42fb9acce7c34fa83a`).
- pdf: `/home/tsjang/rhwp/pdf/issue5941-1480000-201900698-2020.pdf` → `pdf/issue5941-1480000-201900698-2020.pdf` (SHA-256 `fe01f1ddffc3153d015adab93178f0fa8a5e9624763ff529362d4410d50955dc`).

공유 입력의 정상 parser/serializer·독립 계약 검사는 보존한다. 부적합한 회귀 분리의 정확한 함수 목록은 상위 README에 기록한다. 같은 원본과 Print PDF에서 관련 전쪽 Native/fresh WASM raw 90% 이상·쪽수/내용 소속·확대 판독을 확인한 뒤 관계형 회귀로 복원한다. 글꼴 바이너리는 복사·개명·배포하지 않는다.
