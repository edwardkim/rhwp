# 전쪽 시각 선행 조건을 충족하지 못한 새 검사 후보

이 자료는 활성 검사에서 수집하지 않는 원문 보존본이다. 등록 기준은
[시각 fixture 증거](../../../../manual/pr_review/visual_fixture_evidence.md)를 따른다.

- 검토 branch: `review/planet6897-followup-20261011`
- 고정 upstream base: `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`
- 보존 당시 head: `4cebc8d095b64ce2786bc5071d31ad5a76a09bb1`
- 아래3개 source/7개 함수는 고정 base에 등록되어 있지 않았다.
- 이는 **새 검사 후보 등록 보류**다. 기존 devel 회귀의 이관 또는 구현 회귀의 해결로
  분류하지 않는다. 원 contributor 기능 commit의 provenance는 개별 검토 기록에 남긴다.
- 현재 source/정상 대조군의 시각 수용은 별도 게이트다. 전쪽이 미달인 원본의 검사
  기대값을 고치거나 새로운 정상 golden으로 고정하지 않는다.
- 실물 배치를 절대px 위치로 고정하지 않으며, 같은 원본·독립Print의 전쪽 Native/fresh
  90% 이상 및 확대 판독을 확인한 뒤 관계형 검사로 복원한다.
- 원본·Print는 다른 검사와 공유하므로 참조를 유지한다. 이 기록은 byte input/기준을
  정상 생성본으로 바꾸거나 공유 parser/serializer/model 계약을 제거하지 않는다.

## 후보와 보류 근거

| 원 source | 함수 수 | 독립 증거 / 후속 범위 |
| --- | ---: | --- |
| `tests/cases/issue_7665_endnote_title_by_ownership.rs` | 4 | #7672 / #7665. 첫 편집본20쪽 Native/fresh 최저29.12455%,미달12쪽. 다른 편집 대조도 검증 후 등록한다. |
| `tests/cases/cell_row_line_box_matches_hangul_pitch.rs` | 2 | #7692 / #7783. 80168 HWPX156쪽 Native/fresh 최저1.49475%,미달110쪽. 그림 칸 원본4쪽의 현재Native는1쪽71.11796%,2쪽39.21772%로 미달이며 fresh는 미검증이다. 18쪽RowBreak 대조군과 다른 입력이다. |
| `tests/cases/rowspan_solver_declared_overflow_before_free_row.rs` | 1 | #7692 / #7743. 1342000 교육과정418쪽 대415쪽Print의 전쪽 대응 미충족. 저장 선언을 새 배치 정답으로 고정하지 않는다. |

각 본문은 같은 basename의 `.rs.txt`로 바이트를 보존했다. 실제 source/원본 조건과 함수명,
SHA-256은 아래에 고정한다. 전체 검사 실행에서 이7건을 먼저 실행했던 결과는 등록 자격을
입증하지 않는다. 최종 정식 검사 수는 파생 suite 재준비 뒤 다시 기록한다.

## 보존 원문 해시

- `issue_7665_endnote_title_by_ownership.rs.txt`: `87aa377d8c7605dd85dc4a28a713abc5857812cde6b52bf832b0edd62507f641`
  - `same_note_mun_body_2023_note_348`
  - `same_note_mun_body_gap20_note_134`
  - `same_note_mun_body_gap20_note_254`
  - `same_note_mun_body_separator_note_229`
- `cell_row_line_box_matches_hangul_pitch.rs.txt`: `02bdea516fbd84ceace5d23b626e7a1cba71470563d50db25430b8f25089489e`
  - `picture_only_cell_without_stored_lines_is_padding_offset_and_picture`
  - `line_break_only_line_keeps_the_break_char_line_box`
- `rowspan_solver_declared_overflow_before_free_row.rs.txt`: `b756d4aa7f834e0f73426763320faa3807e8a96369a636f604896938950e5755`
  - `two_row_cells_keep_their_stored_height_when_a_free_row_is_solved`
