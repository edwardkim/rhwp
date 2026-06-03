# Task M100 #1245 최종 보고서

## 작업 개요

- 이슈: [#1245](https://github.com/edwardkim/rhwp/issues/1245)
- 브랜치: `local/task_m100_1245`
- 대상 문서:
  - `samples/3-09월_교육_통합_2022.hwp`
  - `samples/3-09월_교육_통합_2023.hwp`

## 해결 내용

1. `3-09월_교육_통합_2022.hwp` 7쪽 하단 그림 객체가 페이지 밖으로 밀리는 문제를 보정했다.
   - `Square/어울림` 그림 위치 계산에서 raw `LINE_SEG.vertical_pos`를 그대로 더하지 않고, 첫 줄 기준 상대 delta를 사용하도록 수정했다.

2. `3-09월_교육_통합_2022.hwp` 10쪽 `문12)` 우측 단 수식 배치 문제를 보정했다.
   - 본문/미주 수식-only 문단이 불필요한 margin/alignment 보정을 다시 받지 않도록 분리했다.
   - TAC가 없는 선행 guide 줄을 수식 앞 세로 예약으로 쓰지 않도록 처리했다.

3. `3-09월_교육_통합_2023.hwp` 4쪽 `문26)` 미주 선두 번호가 중복 표시되는 문제를 보정했다.
   - 선두 미주 번호가 prefix `TextRun`으로 이미 렌더된 경우 같은 위치의 `FootnoteMarker`를 건너뛰도록 했다.

4. PR #1241 병합 후 `문12)` 첫 수식이 다시 `따라서`와 겹칠 수 있는 회귀를 보정했다.
   - 수식-only TAC 줄 배정에서 TAC가 없는 선행 guide 줄을 후보에서 제외했다.

## 회귀 검증

추가 또는 보강한 테스트:

- `issue_1245_2022_page7_square_pictures_use_relative_line_vpos`
- `issue_1209_2022_sep_page10_question12_uses_safe_vpos_backtrack`
- `issue_1245_2023_page4_question26_endnote_marker_not_duplicated`

Stage별 지정 검증:

```bash
cargo test --test issue_1139_inline_picture_duplicate -- --nocapture
```

최종 PR 전 전체 검증은 PR 생성 직전에 수행한다.

## 충돌 및 회귀 확인

- PR #1240, #1241이 반영된 `upstream/devel` 기준으로 rebase를 완료했다.
- PR #1247이 반영된 `upstream/devel` 기준으로 다시 rebase를 완료했다.
- #1247의 미주 간격 처리(`endnote_between_notes_hu`)와 #1245의 TAC guide 보정 로직이 함께 유지되는 것을 확인했다.

## 남은 사항

- PR 전 `cargo test --tests` 전체 테스트를 수행한다.
- PR 본문에는 #1245 자동 종료 키워드를 포함한다.
