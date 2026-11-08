# stored-empty-paragraph-56345

추적: #7806 및 #7445

관측 runtime base: `efc52afb738bd408ac821c4e156101b8d26e5cbd`.
관측 후보: `a30dacfd037122ceacc65439ca03a1afc2bfe69c` (D61 동결 실행).
후속 D66 및 PR 최종 head의 결과와 구분한다. 이관은 개선 완료가 아니다.

비교: 96dpi, compat 2022, 색상 경계 보정 전 2px `silhouette_raw_match_percent`.
입력 SHA-256: `58013017c3a3dc7e2d278b99c5b4fa1c61de0aa861f913a2c41a49145baadafc`.
독립 한컴 Print PDF SHA-256: `c66ea20c3b8d6b31af73752e170372bc7af839c38812991125bdde4dfba35abb`; 기준 21쪽.

| 관측 | 경로 | 비교 쪽수 | raw 최저 | 90% 미만 쪽 |
| --- | --- | ---: | ---: | --- |
| base | native | 21 | 19.47205% | 7, 8, 9, 17 |
| base | wasm | 21 | 19.60130% | 7, 8, 9, 17 |
| candidate | native | 21 | 19.47205% | 7, 8, 9, 17, 21 |
| candidate | wasm | 21 | 19.60130% | 7, 8, 9, 17, 21 |

## 보존과 복원

- input: `/home/tsjang/rhwp/samples/issue6111/56345_regulatory_impact_analysis.hwp` → `samples/issue6111/56345_regulatory_impact_analysis.hwp` (SHA-256 `58013017c3a3dc7e2d278b99c5b4fa1c61de0aa861f913a2c41a49145baadafc`).
- pdf: `/home/tsjang/rhwp/pdf/issue6111/56345_regulatory_impact_analysis-hwp-2020.pdf` → `pdf/issue6111/56345_regulatory_impact_analysis-hwp-2020.pdf` (SHA-256 `c66ea20c3b8d6b31af73752e170372bc7af839c38812991125bdde4dfba35abb`).

공유 입력의 정상 parser/serializer·독립 계약 검사는 보존한다. 부적합한 회귀 분리의 정확한 함수 목록은 상위 README에 기록한다. 같은 원본과 Print PDF에서 관련 전쪽 Native/fresh WASM raw 90% 이상·쪽수/내용 소속·확대 판독을 확인한 뒤 관계형 회귀로 복원한다. 글꼴 바이너리는 복사·개명·배포하지 않는다.

## 이관 전 검사 코드

- [전체 검사 모듈과 helper](../tests/issue_7644_stored_block_empty_paragraph.rs.txt), 이관 전 commit `91ce04bdf0ac285d5e53a88f15a3e690baca2575`, SHA-256 `7b7a15d756a709bed59c98645a0584ea1243a85496735ced68f120103b579bca`.
- 함수: `stored_block_tail_preserves_child_cut_padding_and_the_empty_line_owner`.
- 처리: 정상 검사 유지; 이관 전 전체 원문 보존. [복원 원장](../transfer-inventory.json)의 경로·blob·해시를 대조한다.
