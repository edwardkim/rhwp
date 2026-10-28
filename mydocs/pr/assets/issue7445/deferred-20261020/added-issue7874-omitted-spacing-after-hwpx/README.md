# added-issue7874-omitted-spacing-after-hwpx

추적: #7445 사용자 지정 잔여 이관

관측 runtime base: `efc52afb738bd408ac821c4e156101b8d26e5cbd`.
관측 후보: `a30dacfd037122ceacc65439ca03a1afc2bfe69c` (D61 동결 실행).
후속 D66 및 PR 최종 head의 결과와 구분한다. 이관은 개선 완료가 아니다.

비교: 96dpi, compat 2024, 색상 경계 보정 전 2px `silhouette_raw_match_percent`.
입력 SHA-256: `b1905e369c93f4086047e3c68b77474eab0468491fae7da97bcbd4d944ac0cbf`.
독립 한컴 Print PDF SHA-256: `8a15a05c3f7c7f43e999e4bfa75971255a1892e03643f54f0ffc7cf848da2bce`; 기준 2쪽.

| 관측 | 경로 | 비교 쪽수 | raw 최저 | 90% 미만 쪽 |
| --- | --- | ---: | ---: | --- |
| base | native | 2 | 46.28731% | 1 |
| base | wasm | 2 | 46.72313% | 1 |
| candidate | native | 2 | 46.28731% | 1 |
| candidate | wasm | 2 | 46.72313% | 1 |

## 보존과 복원

- input: `/home/tsjang/rhwp/samples/issue7874/omitted-spacing-after.hwpx` → `mydocs/pr/assets/issue7445/deferred-20261020/added-issue7874-omitted-spacing-after-hwpx/omitted-spacing-after.hwpx` (SHA-256 `b1905e369c93f4086047e3c68b77474eab0468491fae7da97bcbd4d944ac0cbf`).
- pdf: `/home/tsjang/rhwp/pdf/issue7874/omitted-spacing-after-2024.pdf` → `mydocs/pr/assets/issue7445/deferred-20261020/added-issue7874-omitted-spacing-after-hwpx/omitted-spacing-after-2024.pdf` (SHA-256 `8a15a05c3f7c7f43e999e4bfa75971255a1892e03643f54f0ffc7cf848da2bce`).

공유 입력의 정상 parser/serializer·독립 계약 검사는 보존한다. 부적합한 회귀 분리의 정확한 함수 목록은 상위 README에 기록한다. 같은 원본과 Print PDF에서 관련 전쪽 Native/fresh WASM raw 90% 이상·쪽수/내용 소속·확대 판독을 확인한 뒤 관계형 회귀로 복원한다. 글꼴 바이너리는 복사·개명·배포하지 않는다.

## 이관 전 검사 코드

- [전체 검사 모듈과 helper](../tests/issue_7874_stored_ladder_omitted_spacing_after.rs.txt), 이관 전 commit `91ce04bdf0ac285d5e53a88f15a3e690baca2575`, SHA-256 `734726858ce3d0bea4860b2d0e80a29a3c9821bb3d9eb676476e66919028cacb`.
- 함수: `paragraphs_keep_spacing_after_omitted_by_stored_ladder`.
- 처리: 활성 검사 1건 제거; 원문 보존. [복원 원장](../transfer-inventory.json)의 경로·blob·해시를 대조한다.

## 복원 (#7874 재수정)

`fix/7874-omitted-spacing-after-v2`(code `5caaf011e`, base `605ccfd92`)에서 조판의 트림 복원 판정과
배치의 후방 스냅이 같은 술어 `stored_ladder_omits_spacing_after` 를 쓰도록 고쳐 1쪽이 Native/fresh WASM
raw 100% 가 됐다. 입력·PDF 를 원래 경로 `samples/issue7874/omitted-spacing-after.hwpx`·
`pdf/issue7874/omitted-spacing-after-2024.pdf` 로 되돌리고(SHA-256 위와 동일) 이관 전 검사 원문을
수정 없이 `tests/cases/issue_7874_stored_ladder_omitted_spacing_after.rs` 로 복원했다.
증적: `mydocs/report/7874-omitted-spacing-after-v2/`.
