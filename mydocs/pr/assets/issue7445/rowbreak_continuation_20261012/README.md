# #7701의 미달 문서 회귀 재등록 보류

고정 base `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`와 통합 head
`c9323fd567228ed110302b0e0e0ffe5459866298`를 동일 입력·독립 Print PDF·글꼴로 비교했다.
새 incoming `tests/cases/rowbreak_continuation_reopens_outer_top.rs`는 이 base에 없었다.
따라서 기존 정상 회귀를 삭제한 것이 아니라 시각 선행 조건을 충족하지 못한 신규 등록을 철회한다.
정확한 415쪽 기대값을 418쪽으로 완화하지 않는다.

| 범위 | 확인 결과 | 처리 |
| --- | --- | --- |
| 1342000 교육과정 HWP | base/current 모두 418쪽, 독립 한컴2020 Print 415쪽. 같은 물리 121·124·271쪽 Native 점수는 base 38.59023·74.99324·40.51075%, current 38.58858·75.02186·40.53571%. 이 세 쪽의 표·글줄 좌표와 내용·소유 순서 동일 | 기존 [#7743](https://github.com/edwardkim/rhwp/issues/7743)에서 개선. 신규 124쪽 절대 px 검사와 415쪽 등록 보류 |
| rowbreak-problem-pages HWPX | current Native/fresh WASM 전18쪽 최저 55.00512%, 미달 4·6·7·8·15·16쪽 | 기존 [#7741](https://github.com/edwardkim/rhwp/issues/7741)에 연결. 신규 18쪽 검사 등록 보류. 이 실행은 exact base의 전18쪽 실행으로 보고하지 않음 |
| issue7336 저장 frame HWPX 정상 대조군 | current Native/fresh WASM 전7쪽 최저 96.93933%. 4쪽 표 경계·내용 순서 일치. 7쪽 확대 판독에서 체크박스 glyph 모양 차이가 남음 | 기존 정확한 검사 유지. mixed 신규 함수에서 분리해 새로 등록하는 작업도 시각 판독 완료까지 보류 |

원 검사 전체는 [rowbreak_continuation_reopens_outer_top.rs.txt](rowbreak_continuation_reopens_outer_top.rs.txt)에
보존한다. 활성 신규 검사 두 함수가 줄며 `#[ignore]`나 조용한 return으로 숨기지 않는다.
원 HWP/HWPX/PDF는 기존 공유 IR·소유 관계 검사가 사용하므로 기존 사용자 지시에 따라 이동하거나
삭제하지 않는다. 기존 이관 문서의 정상 출력 검사 복원·결함 해결·이슈 종료로 보고하지 않는다.

대표 같은 물리 124쪽: [base review](base_education_p124_review.png),
[current review](current_education_p124_review.png). 표 안 내용과 세로 괘선의 독립 Print 차이는
양쪽에 남아 있다. 전체 415쪽 또는 fresh WASM 교육과정 통과로 확대 해석하지 않는다.

재현 명령·실행 binary/source·입력/PDF/font SHA와 TSV는 ignored
`output/pr-review/planet6897-followup-20261011/`의
`exact-base-current-comparison-results.json`,
`education-exact-base-current-owner-geometry-comparison.json`,
`education-exact-base-current-delta-by-type.json`,
`rowbreak-counter-current-results.json` 및 각 실행 로그에 보존한다.
TSV·중간 JSON·비공개 글꼴은 Git에 넣지 않는다.

복원 조건은 동일 입력·독립 Print와 Native/fresh WASM 전쪽 최저90% 이상, 전체 쪽수 일치,
수식·간격·known glyph 차이의 직접 확대 판독이다. 이후 px 핀 대신 소유·내용 순서·포함·겹침
관계로 검사를 복원한다. 제거 후 파생 manifest 재생성과 관련 회귀·필수 lint 결과는 PR 검토 기록에 연결한다.

## 입력 및 독립 Print SHA-256

| 기존 경로 | SHA-256 |
| --- | --- |
| `samples/task2287/1342000_edu_curriculum_map.hwp` | `623b00d56beffc45d27c5bf23911bdc49d3a541ded8aecbb323d0716a2bc9f4e` |
| `pdf/task2287/1342000_edu_curriculum_map-hwp-2020.pdf` | `bc16ee92640644307fd1ce92ab8ffe663088210caa7740bef620daaa0187e197` |
| `samples/rowbreak-problem-pages.hwpx` | `4ccc321e7eb146f28b768654502a128ba1ae408238fd1ed37d51e963732eb5b0` |
| `pdf/rowbreak-problem-pages-hwpx-2020.pdf` | `e3dc7bb21707f92e42e83808023bf155f62475fc4eb0e0498709d03986c5afe5` |
| `samples/issue7336/stored_frame_page_larger_rowbreak.hwpx` | `042633046ab469b971aa155417ecd568c8825d586d37f87a76194145edb903dc` |
| `pdf/issue7336/stored_frame_page_larger_rowbreak-2020.pdf` | `5e19bd4c4d0cc3e7b177834e939a8e75df57e4387d43d1a9a1bd8f74840b993e` |

## #7701의 추가 원 커밋 검토 — 2026-10-13

최신 원 head `8a3e192d81fbdf925a825c09c50dd77809dae9be`의 rowspan 시작 행 보정을 추가로 받았다.
일반 행과 같은 `RowScanQuery::whole_row_height`를 이어받는 rowspan 시작 행도 소비하고,
`required_height → consumed → fragment emit`으로 예약한다. 앞 조각이 이미 그린 물리 띠를
행 전체 높이로 다시 세지 않는 조건이며 문서 ID를 분기 조건으로 사용하지 않는다.
기존 C14의 crossing rowspan 빈 물리 꼬리 보존은 함께 검증한다.

새 [원 검사](rowspan_continuation_start_row_carried_tail-8a3.rs.txt)는 실제
수정 전 C15 라이브러리에서 compile exit0 / test exit101로 재현됐다. 30쪽에 독립 Print의
`■ 경기도교육청 성교육 진흥 조례`가 없고 안전교육 조항에서 끝나는 원인으로 실패했다.
이는 독립 진단 실행이며 활성 suite 등록이나 전쪽 피델리티 통과가 아니다.
기존 #7743의 동일 원본·전체415쪽/Print 조건을 충족하기 전에는 신규 등록하지 않는다.
1342000의 공유 입력/Print와 범위 밖 정상 검사는 보존한다.

원 커밋의 기존 검사 변경본도 [별도 원문](rowbreak_continuation_reopens_outer_top-8a3.rs.txt)에
보존하며 이전 보류 원문은 그대로 유지한다. 수정 후 경계·정상 대조군·필수 lint·시각 결과는
`output/pr-review/planet6897-followup-20261011/`의 `pr7701-c15-carried-tail-before-*` 및
후속 C16 검증 기록에 연결한다. 현재 전체 시각 통과나 #7743 해결로 판정하지 않는다.
