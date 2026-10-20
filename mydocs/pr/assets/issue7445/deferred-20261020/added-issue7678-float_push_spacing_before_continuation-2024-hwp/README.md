# added-issue7678-float_push_spacing_before_continuation-2024-hwp

추적: #7445 사용자 지정 잔여 이관

관측 runtime base: `efc52afb738bd408ac821c4e156101b8d26e5cbd`.
관측 후보: `a30dacfd037122ceacc65439ca03a1afc2bfe69c` (D61 동결 실행).
후속 D66 및 PR 최종 head의 결과와 구분한다. 이관은 개선 완료가 아니다.

비교: 96dpi, compat 2024, 색상 경계 보정 전 2px `silhouette_raw_match_percent`.
입력 SHA-256: `5f67a3bb5efda84987d45b21c0c90baf0b11994fad87e34a2864f2d05be237f5`.
독립 한컴 Print PDF SHA-256: `c7607495671467daaad828c75183ef5c927bda19cef8b69f1b81b2e2e6d49a07`; 기준 8쪽.

| 관측 | 경로 | 비교 쪽수 | raw 최저 | 90% 미만 쪽 |
| --- | --- | ---: | ---: | --- |
| base | native | 8 | 22.81828% | 8 |
| base | wasm | 8 | 23.31081% | 8 |
| candidate | native | 8 | 87.54666% | 6 |
| candidate | wasm | 8 | 87.46720% | 6 |

## 보존과 복원

- input: `/home/tsjang/rhwp/samples/issue7678/float_push_spacing_before_continuation-2024.hwp` → `mydocs/pr/assets/issue7445/deferred-20261020/added-issue7678-float_push_spacing_before_continuation-2024-hwp/float_push_spacing_before_continuation-2024.hwp` (SHA-256 `5f67a3bb5efda84987d45b21c0c90baf0b11994fad87e34a2864f2d05be237f5`).
- pdf: `/home/tsjang/rhwp/pdf/issue7678/float_push_spacing_before_continuation-2024.pdf` → `mydocs/pr/assets/issue7445/deferred-20261020/added-issue7678-float_push_spacing_before_continuation-2024-hwp/float_push_spacing_before_continuation-2024.pdf` (SHA-256 `c7607495671467daaad828c75183ef5c927bda19cef8b69f1b81b2e2e6d49a07`).

공유 입력의 정상 parser/serializer·독립 계약 검사는 보존한다. 부적합한 회귀 분리의 정확한 함수 목록은 상위 README에 기록한다. 같은 원본과 Print PDF에서 관련 전쪽 Native/fresh WASM raw 90% 이상·쪽수/내용 소속·확대 판독을 확인한 뒤 관계형 회귀로 복원한다. 글꼴 바이너리는 복사·개명·배포하지 않는다.
